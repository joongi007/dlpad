//! CPU painter host for the same accessible Candidate; no GL context or helper app.
use crate::accessible;
use eframe::egui;
use egui_software_backend::{BufferMutRef, ColorFieldOrder, EguiSoftwareRender};
use egui_winit::accesskit_winit;
use std::{
    collections::HashMap,
    error::Error,
    num::NonZeroU32,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Instant,
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    window::{Window, WindowId},
};

const MAX_FRAME_PIXELS: usize = 16 * 1024 * 1024;
const MAX_TEXTURE_PIXELS: usize = 4 * 1024 * 1024;
const MAX_TEXTURE_SIDE: usize = 4096;
const MAX_TEXTURES: usize = 64;
const MAX_VERTICES: usize = 1_000_000;
const MAX_INDICES: usize = 3_000_000;

type Textures = HashMap<egui::TextureId, [usize; 2]>;
type HostError = Box<dyn Error>;

#[derive(Debug)]
enum UserEvent {
    Repaint,
    AccessKit(accesskit_winit::Event),
    #[cfg(all(windows, feature = "first-paint-fixture"))]
    PointerFixture(WindowId, pointer_render_fixture::Command),
    #[cfg(all(windows, feature = "first-paint-fixture"))]
    PointerFixtureFailed(String),
}
impl From<accesskit_winit::Event> for UserEvent {
    fn from(event: accesskit_winit::Event) -> Self {
        Self::AccessKit(event)
    }
}

#[derive(Clone, Copy)]
struct RepaintRequest {
    deadline: Instant,
    pass: u64,
}
#[derive(Default)]
struct Repaint {
    // Only the current and preceding completed-pass generations can be valid.
    // Keep deadlines distinct: a fresh request must not revive an old deadline.
    requests: [Option<RepaintRequest>; 2],
    queued: bool,
}
impl Repaint {
    fn request(&mut self, info: egui::RequestRepaintInfo, now: Instant) -> bool {
        if info.viewport_id != egui::ViewportId::ROOT {
            return false;
        }
        let Some(when) = now.checked_add(info.delay) else {
            return false;
        };
        self.merge(when, info.current_cumulative_pass_nr)
    }
    fn merge(&mut self, when: Instant, pass: u64) -> bool {
        if let Some(request) = self
            .requests
            .iter_mut()
            .flatten()
            .find(|request| request.pass == pass)
        {
            request.deadline = request.deadline.min(when);
        } else if let Some(slot) = self.requests.iter_mut().find(|slot| slot.is_none()) {
            *slot = Some(RepaintRequest {
                deadline: when,
                pass,
            });
        } else {
            let oldest = if self.requests[0].unwrap().pass <= self.requests[1].unwrap().pass {
                0
            } else {
                1
            };
            if pass > self.requests[oldest].unwrap().pass {
                self.requests[oldest] = Some(RepaintRequest {
                    deadline: when,
                    pass,
                });
            }
        }
        if self.queued {
            false
        } else {
            self.queued = true;
            true
        }
    }
    fn take(&mut self, current_pass: u64) -> Option<Instant> {
        self.queued = false;
        std::mem::take(&mut self.requests)
            .into_iter()
            .flatten()
            .filter(|request| {
                current_pass == request.pass || request.pass.checked_add(1) == Some(current_pass)
            })
            .map(|request| request.deadline)
            .min()
    }
}

fn frame_pixels(width: u32, height: u32) -> Result<Option<usize>, HostError> {
    if width == 0 || height == 0 {
        return Ok(None);
    }
    let pixels = (width as usize)
        .checked_mul(height as usize)
        .filter(|pixels| *pixels <= MAX_FRAME_PIXELS)
        .ok_or("CPU frame pixel budget exceeded")?;
    Ok(Some(pixels))
}

// Validate into a copy, so a malformed delta never partially changes our ledger.
fn validate_textures(
    current: &Textures,
    delta: &egui::TexturesDelta,
) -> Result<Textures, HostError> {
    if delta.set.len() > MAX_TEXTURES || delta.free.len() > MAX_TEXTURES {
        return Err("CPU texture delta count exceeded".into());
    }
    let mut next = current.clone();
    for (id, update) in &delta.set {
        let size = update.image.size();
        if size.contains(&0) || size.iter().any(|side| *side > MAX_TEXTURE_SIDE) {
            return Err("CPU texture dimension exceeded".into());
        }
        let pixels = size[0]
            .checked_mul(size[1])
            .ok_or("texture size overflow")?;
        let egui::ImageData::Color(image) = &update.image;
        if pixels != image.pixels.len() {
            return Err("CPU texture pixels do not match dimensions".into());
        }
        if let Some(position) = update.pos {
            let old = next.get(id).ok_or("partial update without texture")?;
            for axis in 0..2 {
                if position[axis]
                    .checked_add(size[axis])
                    .is_none_or(|end| end > old[axis])
                {
                    return Err("CPU partial texture update escaped dimensions".into());
                }
            }
        } else {
            next.insert(*id, size);
        }
        if next.len() > MAX_TEXTURES {
            return Err("CPU texture count exceeded".into());
        }
        let total = next
            .values()
            .try_fold(0usize, |total, size| {
                total.checked_add(size[0].checked_mul(size[1])?)
            })
            .ok_or("CPU texture aggregate overflow")?;
        if total > MAX_TEXTURE_PIXELS {
            return Err("CPU texture aggregate pixel budget exceeded".into());
        }
    }
    // Free applies after this frame's paint, matching renderer semantics.
    for id in &delta.free {
        next.remove(id);
    }
    Ok(next)
}

fn validate_primitives(primitives: &[egui::ClippedPrimitive]) -> Result<(), HostError> {
    let mut vertices = 0usize;
    let mut indices = 0usize;
    for primitive in primitives {
        let egui::epaint::Primitive::Mesh(mesh) = &primitive.primitive else {
            return Err("CPU painter cannot render a GPU callback".into());
        };
        vertices = vertices
            .checked_add(mesh.vertices.len())
            .ok_or("vertex count overflow")?;
        indices = indices
            .checked_add(mesh.indices.len())
            .ok_or("index count overflow")?;
        if vertices > MAX_VERTICES || indices > MAX_INDICES || mesh.indices.len() % 3 != 0 {
            return Err("CPU mesh budget/triangle contract violated".into());
        }
        if !primitive.clip_rect.is_finite()
            || mesh
                .indices
                .iter()
                .any(|index| *index as usize >= mesh.vertices.len())
            || mesh
                .vertices
                .iter()
                .any(|vertex| !vertex.pos.is_finite() || !vertex.uv.is_finite())
        {
            return Err("CPU mesh contains invalid coordinates or indices".into());
        }
    }
    Ok(())
}

// Apply validated texture changes without creating/presenting a zero-size surface.
// The painter requires nonzero dimensions; an owned single pixel and no primitives
// also discard stale primitive caches before the next complete visible repaint.
fn update_hidden_textures(
    renderer: &mut EguiSoftwareRender,
    delta: &egui::TexturesDelta,
    scale: f32,
) {
    let mut scratch = [[0u8; 4]; 1];
    renderer.render(
        &mut BufferMutRef::new(&mut scratch, 1, 1),
        &[],
        delta,
        scale,
    );
}

// Window-local Windows DWM opt-in. No system theme changes or retry timer.
#[cfg(windows)]
mod titlebar_theme {
    use windows_sys::Win32::{
        Graphics::Dwm::{
            DWMWA_CAPTION_COLOR, DWMWA_COLOR_DEFAULT, DWMWA_TEXT_COLOR,
            DWMWA_USE_IMMERSIVE_DARK_MODE, DwmSetWindowAttribute,
        },
        System::Threading::{GetCurrentProcessId, GetCurrentThreadId},
        UI::WindowsAndMessaging::GetWindowThreadProcessId,
    };
    use winit::{
        raw_window_handle::{HasWindowHandle, RawWindowHandle},
        window::Window,
    };

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub(super) struct Wanted {
        dark: bool,
        caption: u32,
        text: u32,
    }
    impl Wanted {
        pub(super) fn from_context(ctx: &eframe::egui::Context) -> Self {
            let style = ctx.style();
            let visuals = &style.visuals;
            let system = ctx.options(|options| options.theme_preference)
                == eframe::egui::ThemePreference::System;
            Self {
                dark: visuals.dark_mode,
                caption: if system {
                    DWMWA_COLOR_DEFAULT
                } else {
                    colorref(visuals.panel_fill)
                },
                text: if system {
                    DWMWA_COLOR_DEFAULT
                } else {
                    colorref(visuals.text_color())
                },
            }
        }
    }
    fn colorref(color: eframe::egui::Color32) -> u32 {
        u32::from(color.r()) | (u32::from(color.g()) << 8) | (u32::from(color.b()) << 16)
    }
    #[derive(Default)]
    pub(super) struct Cache {
        attempted: Option<Wanted>,
    }
    impl Cache {
        // Cache attempts, including unavailable/unsupported results. Do not retry
        // a failed attribute every frame; a genuine theme change gets one attempt.
        pub(super) fn sync(
            &mut self,
            wanted: Wanted,
            apply: impl FnOnce(Wanted) -> Option<[i32; 3]>,
        ) -> Option<[i32; 3]> {
            if self.attempted == Some(wanted) {
                return None;
            }
            self.attempted = Some(wanted);
            apply(wanted)
        }
    }
    pub(super) fn apply(window: &Window, wanted: Wanted) -> Option<[i32; 3]> {
        let RawWindowHandle::Win32(raw) = window.window_handle().ok()?.as_raw() else {
            return None;
        };
        let hwnd = raw.hwnd.get() as windows_sys::Win32::Foundation::HWND;
        let mut pid = 0;
        // SAFETY: live retained window handle; identity queries do not mutate it.
        let thread = unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
        // SAFETY: caller-local identity queries. Reject a foreign HWND/UI thread.
        if thread == 0
            || thread != unsafe { GetCurrentThreadId() }
            || pid != unsafe { GetCurrentProcessId() }
        {
            return None;
        }
        let value: windows_sys::core::BOOL = i32::from(wanted.dark);
        // SAFETY: this UI thread owns the retained HWND; documented scalar BOOL
        // attributes, correctly sized BOOL/COLORREF values live through each call.
        // All three calls execute independently; unsupported HRESULTs are cached,
        // with no warning/retry loop. Partial support is not atomic rollback.
        Some(unsafe {
            [
                DwmSetWindowAttribute(
                    hwnd,
                    DWMWA_USE_IMMERSIVE_DARK_MODE as u32,
                    (&value as *const windows_sys::core::BOOL).cast(),
                    std::mem::size_of_val(&value) as u32,
                ),
                DwmSetWindowAttribute(
                    hwnd,
                    DWMWA_CAPTION_COLOR as u32,
                    (&wanted.caption as *const u32).cast(),
                    std::mem::size_of_val(&wanted.caption) as u32,
                ),
                DwmSetWindowAttribute(
                    hwnd,
                    DWMWA_TEXT_COLOR as u32,
                    (&wanted.text as *const u32).cast(),
                    std::mem::size_of_val(&wanted.text) as u32,
                ),
            ]
        })
    }

    #[cfg(test)]
    mod tests {
        use super::{Cache, DWMWA_COLOR_DEFAULT, Wanted};
        fn manual(dark: bool) -> Wanted {
            Wanted {
                dark,
                caption: if dark { 0x1e1e1e } else { 0xffffff },
                text: if dark { 0xf5f5f5 } else { 0 },
            }
        }
        #[test]
        fn unchanged_frames_do_not_repeat_native_work() {
            let mut cache = Cache::default();
            let mut calls = 0;
            for _ in 0..10_000 {
                let _ = cache.sync(manual(true), |_| {
                    calls += 1;
                    Some([0; 3])
                });
            }
            assert_eq!(calls, 1);
            assert_eq!(
                cache.sync(manual(false), |_| {
                    calls += 1;
                    Some([0; 3])
                }),
                Some([0; 3])
            );
            assert_eq!(calls, 2);
        }
        #[test]
        fn independent_failures_are_cached_without_disabling_changed_request() {
            let mut cache = Cache::default();
            assert_eq!(
                cache.sync(manual(true), |_| Some([-2147024809, 0, -2147467259])),
                Some([-2147024809, 0, -2147467259])
            );
            for _ in 0..10_000 {
                assert_eq!(
                    cache.sync(manual(true), |_| panic!("no failure retry")),
                    None
                );
            }
            assert_eq!(cache.sync(manual(false), |_| Some([0; 3])), Some([0; 3]));
        }
        #[test]
        fn system_resets_custom_colors_even_with_unchanged_resolved_dark() {
            let mut cache = Cache::default();
            let _ = cache.sync(manual(true), |_| Some([0; 3]));
            let system = Wanted {
                dark: true,
                caption: DWMWA_COLOR_DEFAULT,
                text: DWMWA_COLOR_DEFAULT,
            };
            assert_eq!(
                cache.sync(system, |request| {
                    assert_eq!(request.caption, DWMWA_COLOR_DEFAULT);
                    assert_eq!(request.text, DWMWA_COLOR_DEFAULT);
                    Some([0; 3])
                }),
                Some([0; 3])
            );
            assert_eq!(cache.sync(system, |_| panic!("no reset retry")), None);
        }
        #[test]
        fn system_reset_is_derived_from_actual_context_preference() {
            let ctx = eframe::egui::Context::default();
            ctx.set_theme(eframe::egui::ThemePreference::Dark);
            let custom = Wanted::from_context(&ctx);
            assert_ne!(custom.caption, DWMWA_COLOR_DEFAULT);
            assert_ne!(custom.text, DWMWA_COLOR_DEFAULT);
            ctx.set_theme(eframe::egui::ThemePreference::System);
            let system = Wanted::from_context(&ctx);
            assert_eq!(system.caption, DWMWA_COLOR_DEFAULT);
            assert_eq!(system.text, DWMWA_COLOR_DEFAULT);
        }
        #[test]
        fn palette_change_updates_without_dark_toggle() {
            let mut cache = Cache::default();
            let mut wanted = manual(true);
            let _ = cache.sync(wanted, |_| Some([0; 3]));
            wanted.text = 0xe0e0e0;
            assert_eq!(cache.sync(wanted, |_| Some([0; 3])), Some([0; 3]));
        }
        #[test]
        fn unavailable_owner_is_cached_and_replacement_view_starts_fresh() {
            let mut cache = Cache::default();
            assert_eq!(cache.sync(manual(false), |_| None), None);
            assert_eq!(
                cache.sync(manual(false), |_| panic!("no unavailable retry")),
                None
            );
            let mut replacement = Cache::default();
            assert_eq!(
                replacement.sync(manual(false), |_| Some([0; 3])),
                Some([0; 3])
            );
        }
    }
}

struct View {
    #[cfg(all(windows, feature = "first-paint-fixture"))]
    pointer_fixture: Option<pointer_render_fixture::State>,
    scroll_trace_remaining: u16,
    input: egui_winit::State,
    surface: softbuffer::Surface<Arc<Window>, Arc<Window>>,
    window: Arc<Window>,
    _context: softbuffer::Context<Arc<Window>>,
    renderer: EguiSoftwareRender,
    textures: Textures,
    frame_reuse: frame_reuse::FrameReuse,
    size: Option<[u32; 2]>,
    close_requested: bool,
    redraw_pending: bool,
    initial_show_pending: bool,
    #[cfg(windows)]
    titlebar_theme: titlebar_theme::Cache,
    #[cfg(all(windows, feature = "first-paint-fixture"))]
    frame_phase: Option<crate::frame_phase_fixture::Probe>,
}
struct Host {
    candidate: accessible::Candidate,
    ctx: egui::Context,
    proxy: EventLoopProxy<UserEvent>,
    repaint: Arc<Mutex<Repaint>>,
    next_repaint: Option<Instant>,
    metrics_deadline: Option<Instant>,
    view: Option<View>,
    error: Option<HostError>,
}
impl Host {
    #[cfg(all(windows, feature = "first-paint-fixture"))]
    fn diagnostic_event(
        &self,
        event: crate::frame_phase_fixture::WakeEvent,
    ) -> Result<(), HostError> {
        if let Some(probe) = self.view.as_ref().and_then(|v| v.frame_phase.as_ref()) {
            probe.event(event)?;
        }
        Ok(())
    }
    #[cfg(all(windows, feature = "first-paint-fixture"))]
    fn diagnostic_schedule(
        &self,
        when: Instant,
        source: crate::frame_phase_fixture::TimerSource,
    ) -> Result<(), HostError> {
        if let Some(probe) = self.view.as_ref().and_then(|v| v.frame_phase.as_ref()) {
            probe.schedule(when, source)?;
        }
        Ok(())
    }
    fn request_redraw(&mut self) {
        if let Some(view) = &mut self.view
            && !view.redraw_pending
        {
            view.window.request_redraw();
            view.redraw_pending = true;
        }
    }
    fn fail(&mut self, event_loop: &ActiveEventLoop, error: HostError) {
        #[cfg(all(windows, feature = "ime-compose-fixture"))]
        if let Some(view) = &self.view {
            let _ = crate::ime_compose_fixture::restore_on_close(&view.window);
        }
        self.error = Some(error);
        if self.candidate.folder_picker_busy() {
            self.candidate.cancel_folder_picker();
            self.request_redraw();
        } else {
            event_loop.exit();
        }
    }
    fn paint(&mut self, event_loop: &ActiveEventLoop) -> Result<(), HostError> {
        let Some(view) = self.view.as_mut() else {
            return Ok(());
        };
        #[cfg(all(windows, feature = "first-paint-fixture"))]
        let mut frame_phase = view
            .frame_phase
            .as_ref()
            .map(|probe| probe.begin())
            .transpose()?;
        view.redraw_pending = false;
        let size = view.window.inner_size();
        let drawable = frame_pixels(size.width, size.height)?.is_some();
        // Even zero-size/minimized windows must acknowledge worker/close/accessibility.
        let mut input = view.input.take_egui_input(&view.window);
        if view.scroll_trace_remaining > 0 {
            let mut moves = 0usize;
            let mut wheels = 0usize;
            let mut gone = 0usize;
            for event in &input.events {
                match event {
                    egui::Event::PointerMoved(_) => moves += 1,
                    egui::Event::MouseWheel { .. } => wheels += 1,
                    egui::Event::PointerGone => gone += 1,
                    _ => {}
                }
            }
            if moves + wheels + gone > 0 {
                view.scroll_trace_remaining -= 1;
                eprintln!(
                    "dlpad-scroll-trace stage=raw-before-frame remaining={} moves={} wheels={} gone={} focused={}",
                    view.scroll_trace_remaining, moves, wheels, gone, input.focused
                );
            }
        }
        let viewport = input.viewports.entry(egui::ViewportId::ROOT).or_default();
        egui_winit::update_viewport_info(viewport, &self.ctx, &view.window, false);
        let requested_close = std::mem::take(&mut view.close_requested);
        if requested_close {
            viewport.events.push(egui::ViewportEvent::Close);
        }
        #[cfg(all(windows, feature = "first-paint-fixture"))]
        if let Some(fixture) = &mut view.pointer_fixture {
            fixture.append(&mut input)?;
        }
        let output = self.ctx.run(input, |ctx| self.candidate.show_ui(ctx));
        #[cfg(windows)]
        let _ = view
            .titlebar_theme
            .sync(titlebar_theme::Wanted::from_context(&self.ctx), |wanted| {
                titlebar_theme::apply(&view.window, wanted)
            });
        view.input
            .handle_platform_output(&view.window, output.platform_output);
        #[cfg(all(windows, feature = "ime-owner-fixture"))]
        crate::ime_owner_fixture::on_frame(&view.window)?;
        #[cfg(all(windows, feature = "ime-compose-fixture"))]
        crate::ime_compose_fixture::on_frame(&view.window)?;
        #[cfg(all(windows, feature = "first-paint-fixture"))]
        crate::first_paint_fixture::on_frame()?;
        let mut cancelled_close = false;
        let mut explicit_close = false;
        let mut commands = Vec::new();
        if let Some(root) = output.viewport_output.get(&egui::ViewportId::ROOT) {
            if root.commands.len() > 128 {
                return Err("CPU viewport command budget exceeded".into());
            }
            for command in &root.commands {
                match command {
                    egui::ViewportCommand::CancelClose => cancelled_close = true,
                    egui::ViewportCommand::Close => explicit_close = true,
                    other => commands.push(other.clone()),
                }
            }
            if let Some(when) = Instant::now().checked_add(root.repaint_delay) {
                #[cfg(all(windows, feature = "first-paint-fixture"))]
                if let Some(probe) = &view.frame_phase {
                    probe.schedule(when, crate::frame_phase_fixture::TimerSource::Root)?;
                }
                self.next_repaint = Some(self.next_repaint.map_or(when, |old| old.min(when)));
            }
        }
        if output.viewport_output.len() > 1 {
            return Err("CPU candidate supports one native viewport".into());
        }
        let mut info = egui::ViewportInfo::default();
        let mut actions = Vec::new();
        egui_winit::process_viewport_commands(
            &self.ctx,
            &mut info,
            commands,
            &view.window,
            &mut actions,
        );
        let has_actions = !actions.is_empty();
        for action in actions {
            match action {
                egui_winit::ActionRequested::Cut => {
                    view.input.egui_input_mut().events.push(egui::Event::Cut)
                }
                egui_winit::ActionRequested::Copy => {
                    view.input.egui_input_mut().events.push(egui::Event::Copy)
                }
                egui_winit::ActionRequested::Paste => {
                    if let Some(text) = view.input.clipboard_text() {
                        let text = text.replace("\r\n", "\n");
                        if !text.is_empty() {
                            view.input
                                .egui_input_mut()
                                .events
                                .push(egui::Event::Paste(text));
                        }
                    }
                }
                egui_winit::ActionRequested::Screenshot(_) => {
                    return Err("CPU screenshot command is not supported".into());
                }
            }
        }
        if has_actions {
            let when = Instant::now();
            #[cfg(all(windows, feature = "first-paint-fixture"))]
            if let Some(probe) = &view.frame_phase {
                probe.schedule(when, crate::frame_phase_fixture::TimerSource::Action)?;
            }
            self.next_repaint = Some(when);
        }
        if explicit_close || (requested_close && !cancelled_close) {
            #[cfg(all(windows, feature = "ime-compose-fixture"))]
            crate::ime_compose_fixture::restore_on_close(&view.window)?;
            event_loop.exit();
        }
        #[cfg(all(windows, feature = "first-paint-fixture"))]
        if let Some(frame) = &mut frame_phase {
            frame.phase(0)?;
        }
        let next_textures = validate_textures(&view.textures, &output.textures_delta)?;
        if !output.pixels_per_point.is_finite() || output.pixels_per_point <= 0.0 {
            return Err("CPU invalid scale".into());
        }
        if !drawable || self.candidate.hidden() {
            view.frame_reuse.invalidate();
            update_hidden_textures(
                &mut view.renderer,
                &output.textures_delta,
                output.pixels_per_point,
            );
            view.textures = next_textures;
            // Hidden/non-drawable paints do not postpone an already armed retry.
            arm_if_absent(&mut self.metrics_deadline, || {
                self.candidate.metrics_freshness_deadline(Instant::now())
            });
            return Ok(());
        }
        let primitives = self.ctx.tessellate(output.shapes, output.pixels_per_point);
        validate_primitives(&primitives)?;
        if view.size != Some([size.width, size.height]) {
            view.frame_reuse.invalidate();
            view.surface.resize(
                NonZeroU32::new(size.width).unwrap(),
                NonZeroU32::new(size.height).unwrap(),
            )?;
            view.size = Some([size.width, size.height]);
        }
        let mut buffer = view.surface.buffer_mut()?;
        let reused = view.frame_reuse.can_reuse(
            &primitives,
            [size.width, size.height],
            output.pixels_per_point,
            &output.textures_delta,
            buffer.age(),
        );
        #[cfg(all(windows, feature = "first-paint-fixture"))]
        if let Some(frame) = &mut frame_phase {
            frame.phase(1)?;
        }
        if !reused {
            buffer.fill(0);
            let bytes: &mut [[u8; 4]] = bytemuck::cast_slice_mut(&mut buffer);
            view.renderer.render(
                &mut BufferMutRef::new(bytes, size.width as usize, size.height as usize),
                &primitives,
                &output.textures_delta,
                output.pixels_per_point,
            );
            for pixel in buffer.iter_mut() {
                *pixel &= 0x00ff_ffff; // softbuffer requires XRGB, not BGRA alpha.
            }
        }
        #[cfg(all(windows, feature = "first-paint-fixture"))]
        if let Some(frame) = &mut frame_phase {
            frame.phase(2)?;
        }
        #[cfg(all(windows, feature = "first-paint-fixture"))]
        let first_observation = if view.initial_show_pending {
            let pixel = buffer[buffer.len() - 1];
            Some(crate::first_paint_fixture::FirstPresentObservation {
                before_present_visible: view.window.is_visible().unwrap_or(true),
                after_present_visible: true,
                initial_show_performed: false,
                dark_mode: self.ctx.style().visuals.dark_mode,
                system_theme: view.window.theme().map(|theme| match theme {
                    winit::window::Theme::Light => "Light",
                    winit::window::Theme::Dark => "Dark",
                }),
                background_rgb: self.ctx.style().visuals.panel_fill.to_array()[..3]
                    .try_into()
                    .unwrap(),
                actual_pixel_rgb: [
                    ((pixel >> 16) & 255) as u8,
                    ((pixel >> 8) & 255) as u8,
                    (pixel & 255) as u8,
                ],
                width: size.width,
                height: size.height,
            })
        } else {
            None
        };
        if let Err(error) = buffer.present() {
            view.frame_reuse.invalidate();
            return Err(error.into());
        }
        #[cfg(all(windows, feature = "first-paint-fixture"))]
        if let Some(fixture) = &view.pointer_fixture {
            fixture.presented();
        }
        view.frame_reuse.presented(
            primitives,
            [size.width, size.height],
            output.pixels_per_point,
        );
        #[cfg(all(windows, feature = "first-paint-fixture"))]
        let after_present_visible = view.window.is_visible().unwrap_or(true);
        if view.initial_show_pending {
            // The first native show follows a complete successful themed frame.
            view.initial_show_pending = false;
            view.window.set_visible(true);
        }
        #[cfg(all(windows, feature = "first-paint-fixture"))]
        if let Some(mut observation) = first_observation {
            observation.after_present_visible = after_present_visible;
            observation.initial_show_performed = true;
            crate::first_paint_fixture::after_first_present(&view.window, observation)?;
        }
        view.textures = next_textures;
        #[cfg(all(windows, feature = "first-paint-fixture"))]
        if let Some(mut frame) = frame_phase {
            frame.phase(3)?;
            frame.finish(reused)?;
        }
        #[cfg(windows)]
        crate::app_update_entry::frame_presented(self.candidate.update_frame_ready());
        self.candidate.metrics_frame_presented();
        self.metrics_deadline = self.candidate.metrics_freshness_deadline(Instant::now());
        Ok(())
    }
}
// One actual production dispatch path, also exercised with real metrics in tests.
trait DueTarget {
    fn hidden(&self) -> bool;
    fn pump_hidden(&mut self) -> Result<(), HostError>;
    fn redraw(&mut self);
}
fn dispatch_due(
    target: &mut impl DueTarget,
    generic_due: bool,
    metrics_changed: bool,
) -> Result<(), HostError> {
    // Never consume generic/Close/ACK work or cancel a queued native redraw.
    if generic_due || (metrics_changed && !target.hidden()) {
        if target.hidden() {
            target.pump_hidden()?;
            if !target.hidden() {
                target.redraw();
            }
        } else {
            target.redraw();
        }
    }
    Ok(())
}
fn arm_if_absent(deadline: &mut Option<Instant>, supply: impl FnOnce() -> Option<Instant>) {
    if deadline.is_none() {
        *deadline = supply();
    }
}
struct HostDue<'a> {
    host: &'a mut Host,
    event_loop: &'a ActiveEventLoop,
}
impl DueTarget for HostDue<'_> {
    fn hidden(&self) -> bool {
        self.host.candidate.hidden()
    }
    fn pump_hidden(&mut self) -> Result<(), HostError> {
        self.host.paint(self.event_loop)
    }
    fn redraw(&mut self) {
        self.host.request_redraw();
    }
}

impl ApplicationHandler<UserEvent> for Host {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.view.is_some() {
            return;
        }
        let result = (|| -> Result<View, HostError> {
            let icon = winit::window::Icon::from_rgba(crate::app_icon::rgba(32), 32, 32)?;
            let attributes = Window::default_attributes()
                .with_title("dlpad")
                .with_window_icon(Some(icon.clone()))
                .with_visible(false)
                .with_inner_size(LogicalSize::new(960.0, 640.0));
            #[cfg(windows)]
            let attributes = {
                use winit::platform::windows::WindowAttributesExtWindows;
                attributes.with_taskbar_icon(Some(icon))
            };
            let window = Arc::new(event_loop.create_window(attributes)?);
            #[cfg(windows)]
            {
                use winit::platform::windows::WindowExtWindows;
                self.candidate
                    .set_native_drop_registration(window.file_drop_registration());
                self.candidate.set_folder_picker_owner(window.clone());
            }
            let context = softbuffer::Context::new(window.clone())?;
            let surface = softbuffer::Surface::new(&context, window.clone())?;
            let mut input = egui_winit::State::new(
                self.ctx.clone(),
                egui::ViewportId::ROOT,
                &*window,
                Some(window.scale_factor() as f32),
                window.theme(),
                Some(MAX_TEXTURE_SIDE),
            );
            input.init_accesskit(event_loop, &window, self.proxy.clone());
            #[cfg(all(windows, feature = "first-paint-fixture"))]
            let frame_phase = crate::frame_phase_fixture::Probe::new(window.clone())?;
            #[cfg(all(windows, feature = "first-paint-fixture"))]
            let pointer_fixture =
                pointer_render_fixture::State::start(window.id(), self.proxy.clone())?;
            Ok(View {
                #[cfg(all(windows, feature = "first-paint-fixture"))]
                pointer_fixture,
                scroll_trace_remaining: if cfg!(all(windows, feature = "first-paint-fixture"))
                    && std::env::var_os("DLPAD_TEST_SCROLL_TRACE").is_some_and(|v| v == "1")
                {
                    128
                } else {
                    0
                },
                #[cfg(all(windows, feature = "first-paint-fixture"))]
                frame_phase,
                window,
                _context: context,
                surface,
                input,
                renderer: EguiSoftwareRender::new(ColorFieldOrder::Bgra).with_caching(false),
                textures: HashMap::new(),
                frame_reuse: frame_reuse::FrameReuse::default(),
                size: None,
                close_requested: false,
                redraw_pending: false,
                initial_show_pending: true,
                #[cfg(windows)]
                titlebar_theme: titlebar_theme::Cache::default(),
            })
        })();
        match result {
            Ok(view) => {
                // AccessKit is ready while hidden. Present the actual first frame
                // before showing; a failed or zero-size first paint stays hidden.
                self.view = Some(view);
                if let Err(error) = self.paint(event_loop) {
                    self.fail(event_loop, error);
                    return;
                }
                if self
                    .view
                    .as_ref()
                    .is_some_and(|view| view.initial_show_pending)
                {
                    self.fail(event_loop, "CPU initial themed frame unavailable".into());
                    return;
                }
                self.request_redraw();
            }
            Err(error) => self.fail(event_loop, error),
        }
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(view) = &mut self.view else {
            return;
        };
        if view.window.id() != window_id {
            return;
        }
        #[cfg(all(windows, feature = "ime-compose-fixture"))]
        if matches!(event, WindowEvent::CloseRequested)
            && let Err(error) = crate::ime_compose_fixture::restore_on_close(&view.window)
        {
            self.fail(event_loop, error.into());
            return;
        }
        #[cfg(all(windows, feature = "ime-compose-fixture"))]
        if let WindowEvent::Ime(ref ime) = event {
            crate::ime_compose_fixture::on_native_ime(ime);
        }
        if view.scroll_trace_remaining > 0 {
            match &event {
                WindowEvent::MouseWheel { delta, .. } => {
                    view.scroll_trace_remaining -= 1;
                    eprintln!(
                        "dlpad-scroll-trace stage=native-wheel remaining={} delta={:?}",
                        view.scroll_trace_remaining, delta
                    );
                }
                WindowEvent::CursorMoved { position, .. } => {
                    view.scroll_trace_remaining -= 1;
                    eprintln!(
                        "dlpad-scroll-trace stage=native-motion remaining={} position={:?}",
                        view.scroll_trace_remaining, position
                    );
                }
                WindowEvent::CursorLeft { .. } => {
                    view.scroll_trace_remaining -= 1;
                    eprintln!(
                        "dlpad-scroll-trace stage=native-cursor-left remaining={}",
                        view.scroll_trace_remaining
                    );
                }
                _ => {}
            }
        }
        let response = view.input.on_window_event(&view.window, &event);
        #[cfg(all(windows, feature = "first-paint-fixture"))]
        if let Some(probe) = &view.frame_phase {
            let reason = match &event {
                WindowEvent::RedrawRequested => Some(crate::frame_phase_fixture::WakeEvent::Redraw),
                WindowEvent::CloseRequested => Some(crate::frame_phase_fixture::WakeEvent::Close),
                WindowEvent::Destroyed => None,
                _ if response.repaint => Some(crate::frame_phase_fixture::WakeEvent::WindowRepaint),
                _ => None,
            };
            if let Some(reason) = reason
                && let Err(error) = probe.event(reason)
            {
                self.fail(event_loop, error.into());
                return;
            }
        }
        match event {
            WindowEvent::RedrawRequested => {
                if let Err(error) = self.paint(event_loop) {
                    self.fail(event_loop, error);
                }
            }
            WindowEvent::CloseRequested => {
                view.close_requested = true;
                self.request_redraw();
            }
            WindowEvent::Destroyed => event_loop.exit(),
            _ if response.repaint => self.request_redraw(),
            _ => {}
        }
    }
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            #[cfg(all(windows, feature = "first-paint-fixture"))]
            UserEvent::PointerFixture(window, command) => {
                let received = self
                    .view
                    .as_mut()
                    .filter(|v| v.window.id() == window)
                    .and_then(|v| v.pointer_fixture.as_mut())
                    .ok_or("pointer fixture original window unavailable")
                    .and_then(|fixture| {
                        fixture
                            .receive(command)
                            .map_err(|_| "pointer fixture command rejected")
                    });
                if let Err(error) = received {
                    self.fail(_event_loop, error.into());
                    return;
                }
                self.request_redraw();
            }
            #[cfg(all(windows, feature = "first-paint-fixture"))]
            UserEvent::PointerFixtureFailed(error) => {
                self.fail(_event_loop, error.into());
            }

            UserEvent::Repaint => {
                #[cfg(all(windows, feature = "first-paint-fixture"))]
                if let Err(error) =
                    self.diagnostic_event(crate::frame_phase_fixture::WakeEvent::Repaint)
                {
                    self.fail(_event_loop, error);
                    return;
                }
                let current_pass = self.ctx.cumulative_pass_nr_for(egui::ViewportId::ROOT);
                let when = self.repaint.lock().unwrap().take(current_pass);
                if let Some(when) = when {
                    #[cfg(all(windows, feature = "first-paint-fixture"))]
                    if let Err(error) = self.diagnostic_schedule(
                        when,
                        crate::frame_phase_fixture::TimerSource::Callback,
                    ) {
                        self.fail(_event_loop, error);
                        return;
                    }
                    self.next_repaint = Some(self.next_repaint.map_or(when, |old| old.min(when)));
                }
            }
            UserEvent::AccessKit(event) => {
                let Some(view) = &mut self.view else {
                    return;
                };
                if event.window_id != view.window.id() {
                    return;
                }
                match event.window_event {
                    accesskit_winit::WindowEvent::InitialTreeRequested => {
                        #[cfg(all(windows, feature = "first-paint-fixture"))]
                        if let Some(probe) = &view.frame_phase
                            && let Err(error) =
                                probe.event(crate::frame_phase_fixture::WakeEvent::AccessInitial)
                        {
                            self.fail(_event_loop, error.into());
                            return;
                        }
                        self.ctx.enable_accesskit();
                        self.request_redraw();
                    }
                    accesskit_winit::WindowEvent::ActionRequested(request) => {
                        #[cfg(all(windows, feature = "first-paint-fixture"))]
                        if let Some(probe) = &view.frame_phase
                            && let Err(error) =
                                probe.event(crate::frame_phase_fixture::WakeEvent::AccessAction)
                        {
                            self.fail(_event_loop, error.into());
                            return;
                        }
                        view.input.on_accesskit_action_request(request);
                        self.request_redraw();
                    }
                    accesskit_winit::WindowEvent::AccessibilityDeactivated => {
                        self.ctx.disable_accesskit()
                    }
                }
            }
        }
    }
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.error.is_some() {
            self.candidate.cancel_folder_picker();
            self.candidate.poll_folder_picker(&self.ctx);
            if !self.candidate.folder_picker_busy() {
                event_loop.exit();
            } else {
                event_loop.set_control_flow(ControlFlow::WaitUntil(
                    Instant::now() + std::time::Duration::from_millis(100),
                ));
            }
            return;
        }
        let now = Instant::now();
        let mut metrics_changed = false;
        if self.metrics_deadline.is_some_and(|when| when <= now) {
            metrics_changed = self.candidate.check_metrics_freshness(now);
            // Rearm only after a check; unrelated events must not postpone Unknown retries.
            self.metrics_deadline = self.candidate.metrics_freshness_deadline(now);
            #[cfg(all(windows, feature = "first-paint-fixture"))]
            if let Some(probe) = self.view.as_ref().and_then(|v| v.frame_phase.as_ref())
                && let Err(error) = probe.record_metrics_check(metrics_changed)
            {
                self.fail(event_loop, error.into());
                return;
            }
        }
        let generic_due = self.next_repaint.is_some_and(|when| when <= Instant::now());
        if generic_due {
            #[cfg(all(windows, feature = "first-paint-fixture"))]
            if let Some(probe) = self.view.as_ref().and_then(|v| v.frame_phase.as_ref())
                && let Err(error) = probe.timer_due()
            {
                self.fail(event_loop, error.into());
                return;
            }
            self.next_repaint = None;
        }
        if let Err(error) = dispatch_due(
            &mut HostDue {
                host: self,
                event_loop,
            },
            generic_due,
            metrics_changed,
        ) {
            self.fail(event_loop, error);
        }
        let deadline = match (self.next_repaint, self.metrics_deadline) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
        event_loop.set_control_flow(deadline.map_or(ControlFlow::Wait, ControlFlow::WaitUntil));
    }
}

pub fn run(data: PathBuf) -> Result<(), HostError> {
    if cfg!(target_endian = "big") {
        return Err("CPU painter requires little-endian XRGB adapter".into());
    }
    let font = accessible::font_bytes();
    let ctx = egui::Context::default();
    accessible::configure_context(&ctx, font);
    let mut candidate = accessible::Candidate::new(data);
    candidate.use_host_metrics_freshness_timer();
    candidate.apply_appearance(&ctx);
    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    let repaint = Arc::new(Mutex::new(Repaint::default()));
    let proxy = event_loop.create_proxy();
    let callback_queue = repaint.clone();
    let callback_proxy = proxy.clone();
    ctx.set_request_repaint_callback(move |info| {
        // Never read Context while this callback holds its internal write lock.
        let send = callback_queue.lock().unwrap().request(info, Instant::now());
        if send && callback_proxy.send_event(UserEvent::Repaint).is_err() {
            callback_queue.lock().unwrap().queued = false;
        }
    });
    let mut host = Host {
        candidate,
        ctx,
        proxy,
        repaint,
        next_repaint: None,
        metrics_deadline: None,
        view: None,
        error: None,
    };
    event_loop.run_app(&mut host)?;
    if let Some(error) = host.error {
        return Err(error);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn full_texture(id: u64, size: [usize; 2]) -> egui::TexturesDelta {
        egui::TexturesDelta {
            set: vec![(
                egui::TextureId::Managed(id),
                egui::epaint::ImageDelta::full(
                    egui::ColorImage::new(size, vec![egui::Color32::WHITE; size[0] * size[1]]),
                    egui::TextureOptions::LINEAR,
                ),
            )],
            free: Vec::new(),
        }
    }
    #[test]
    fn zero_size_and_frame_budget_are_checked() {
        assert_eq!(frame_pixels(0, 640).unwrap(), None);
        assert_eq!(frame_pixels(960, 0).unwrap(), None);
        assert_eq!(frame_pixels(960, 640).unwrap(), Some(614400));
        assert!(frame_pixels(u32::MAX, u32::MAX).is_err());
        assert!(frame_pixels(4097, 4096).is_err());
    }
    #[test]
    fn repaint_wake_is_coalesced_to_earliest_deadline() {
        let now = Instant::now();
        let mut queue = Repaint::default();
        assert!(queue.merge(now + std::time::Duration::from_secs(10), 0));
        for _ in 0..1000 {
            assert!(!queue.merge(now, 0));
        }
        assert_eq!(queue.take(0), Some(now));
        assert!(queue.merge(now, 0));
    }
    #[test]
    fn malformed_partial_texture_does_not_change_ledger() {
        let current = validate_textures(&Textures::new(), &full_texture(1, [2, 2])).unwrap();
        let mut delta = full_texture(1, [2, 2]);
        delta.set[0].1.pos = Some([1, 0]);
        assert!(validate_textures(&current, &delta).is_err());
        assert_eq!(current[&egui::TextureId::Managed(1)], [2, 2]);
        delta.set[0].1.pos = Some([usize::MAX, 0]);
        assert!(validate_textures(&current, &delta).is_err());
    }
    #[test]
    fn hidden_frame_applies_font_texture_before_visible_partial_update() {
        let mut painter = EguiSoftwareRender::new(ColorFieldOrder::Bgra).with_caching(false);
        let full = full_texture(0, [2, 2]);
        let ledger = validate_textures(&Textures::new(), &full).unwrap();
        update_hidden_textures(&mut painter, &full, 1.0);
        let mut partial = full_texture(0, [1, 1]);
        partial.set[0].1.pos = Some([1, 1]);
        validate_textures(&ledger, &partial).unwrap();
        let mut pixel = [[0u8; 4]; 1];
        painter.render(&mut BufferMutRef::new(&mut pixel, 1, 1), &[], &partial, 1.0);
        let free = egui::TexturesDelta {
            set: Vec::new(),
            free: vec![egui::TextureId::Managed(0)],
        };
        assert!(validate_textures(&ledger, &free).unwrap().is_empty());
        update_hidden_textures(&mut painter, &free, 1.0);
    }
    #[test]
    fn invalid_mesh_indices_are_rejected_before_painter() {
        let mesh = egui::epaint::Mesh {
            indices: vec![0, 1, 2],
            ..Default::default()
        };
        let primitive = egui::ClippedPrimitive {
            clip_rect: egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
            primitive: egui::epaint::Primitive::Mesh(mesh),
        };
        assert!(validate_primitives(&[primitive]).is_err());
    }
}

mod frame_reuse {
    //! Bounded exact-frame reuse without an extra pixel buffer.
    //! Host must call invalidate BEFORE hidden/zero-size, resize/recreation and
    //! on render/present failure. Only commit after successful visible present.
    use eframe::egui::{
        self,
        epaint::{ClippedPrimitive, Primitive},
    };
    use std::mem::size_of;

    const RETAINED_LIMIT: usize = 2 * 1024 * 1024;
    #[derive(Default)]
    pub(crate) struct FrameReuse {
        previous: Option<Frame>,
    }
    struct Frame {
        size: [u32; 2],
        scale: f32,
        primitives: Vec<ClippedPrimitive>,
    }
    fn retained_bytes(primitives: &Vec<ClippedPrimitive>) -> Option<usize> {
        // Account actual allocated capacities, not only live lengths. Reject callbacks
        // even if a caller bypasses the host's normal primitive validation.
        let mut total = size_of::<Frame>().checked_add(
            primitives
                .capacity()
                .checked_mul(size_of::<ClippedPrimitive>())?,
        )?;
        for item in primitives {
            let Primitive::Mesh(mesh) = &item.primitive else {
                return None;
            };
            total = total.checked_add(
                mesh.vertices
                    .capacity()
                    .checked_mul(size_of::<egui::epaint::Vertex>())?,
            )?;
            total = total.checked_add(mesh.indices.capacity().checked_mul(size_of::<u32>())?)?;
            if total > RETAINED_LIMIT {
                return None;
            }
        }
        (total <= RETAINED_LIMIT).then_some(total)
    }
    fn same(a: &[ClippedPrimitive], b: &[ClippedPrimitive]) -> bool {
        a.len() == b.len()
            && a.iter().zip(b).all(|(a, b)| {
                a.clip_rect == b.clip_rect
                    && match (&a.primitive, &b.primitive) {
                        (Primitive::Mesh(a), Primitive::Mesh(b)) => a == b,
                        _ => false,
                    }
            })
    }
    impl FrameReuse {
        pub(crate) fn invalidate(&mut self) {
            self.previous = None;
        }
        /// Call after normal egui/platform output, texture/primitive validation and
        /// surface resize, before fill/render/mask. Always perform full present.
        pub(crate) fn can_reuse(
            &mut self,
            primitives: &Vec<ClippedPrimitive>,
            size: [u32; 2],
            scale: f32,
            delta: &egui::TexturesDelta,
            age: u8,
        ) -> bool {
            if size.contains(&0)
                || !scale.is_finite()
                || scale <= 0.0
                || !delta.set.is_empty()
                || !delta.free.is_empty()
                || age != 1
                || retained_bytes(primitives).is_none()
            {
                self.invalidate();
                return false;
            }
            self.previous.as_ref().is_some_and(|prior| {
                prior.size == size && prior.scale == scale && same(&prior.primitives, primitives)
            })
        }
        /// Move ownership after successful present; no per-frame primitive clone.
        /// Pending texture changes have been applied by normal full render first.
        pub(crate) fn presented(
            &mut self,
            primitives: Vec<ClippedPrimitive>,
            size: [u32; 2],
            scale: f32,
        ) {
            self.previous = if size.contains(&0)
                || !scale.is_finite()
                || scale <= 0.0
                || retained_bytes(&primitives).is_none()
            {
                None
            } else {
                Some(Frame {
                    size,
                    scale,
                    primitives,
                })
            };
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        fn frame() -> Vec<ClippedPrimitive> {
            let mut mesh = egui::epaint::Mesh::default();
            mesh.add_colored_rect(
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(10.0, 10.0)),
                egui::Color32::WHITE,
            );
            vec![ClippedPrimitive {
                clip_rect: egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(32.0, 32.0)),
                primitive: Primitive::Mesh(mesh),
            }]
        }
        #[test]
        fn unchanged_reuses_only_successful_prior_frame() {
            let mut cache = FrameReuse::default();
            let delta = egui::TexturesDelta::default();
            assert!(!cache.can_reuse(&frame(), [32, 32], 1.0, &delta, 1));
            cache.presented(frame(), [32, 32], 1.0);
            assert!(cache.can_reuse(&frame(), [32, 32], 1.0, &delta, 1));
            cache.invalidate();
            assert!(!cache.can_reuse(&frame(), [32, 32], 1.0, &delta, 1));
        }
        #[test]
        fn shape_clip_size_scale_and_age_changes_do_not_reuse() {
            let delta = egui::TexturesDelta::default();
            for (size, scale, age) in [
                ([64, 32], 1.0, 1),
                ([32, 32], 2.0, 1),
                ([32, 32], 1.0, 0),
                ([32, 32], 1.0, 2),
                ([0, 32], 1.0, 1),
            ] {
                let mut cache = FrameReuse::default();
                cache.presented(frame(), [32, 32], 1.0);
                assert!(!cache.can_reuse(&frame(), size, scale, &delta, age));
            }
            let mut changed = frame();
            changed[0].clip_rect = egui::Rect::NOTHING;
            let mut cache = FrameReuse::default();
            cache.presented(frame(), [32, 32], 1.0);
            assert!(!cache.can_reuse(&changed, [32, 32], 1.0, &delta, 1));
            let mut changed = frame();
            let Primitive::Mesh(mesh) = &mut changed[0].primitive else {
                unreachable!()
            };
            mesh.vertices[0].color = egui::Color32::BLACK;
            assert!(!cache.can_reuse(&changed, [32, 32], 1.0, &delta, 1));
        }
        #[test]
        fn texture_set_or_free_invalidates_and_capacity_budget_is_real() {
            for delta in [
                egui::TexturesDelta {
                    set: vec![],
                    free: vec![egui::TextureId::Managed(0)],
                },
                egui::TexturesDelta {
                    set: vec![(
                        egui::TextureId::Managed(0),
                        egui::epaint::ImageDelta::full(
                            egui::ColorImage::new([1, 1], vec![egui::Color32::WHITE]),
                            egui::TextureOptions::LINEAR,
                        ),
                    )],
                    free: vec![],
                },
            ] {
                let mut cache = FrameReuse::default();
                cache.presented(frame(), [32, 32], 1.0);
                assert!(!cache.can_reuse(&frame(), [32, 32], 1.0, &delta, 1));
                assert!(cache.previous.is_none());
            }
            let mut huge = frame();
            let Primitive::Mesh(mesh) = &mut huge[0].primitive else {
                unreachable!()
            };
            mesh.indices.reserve(RETAINED_LIMIT / size_of::<u32>() + 1);
            assert!(retained_bytes(&huge).is_none());
            let mut cache = FrameReuse::default();
            cache.presented(huge, [32, 32], 1.0);
            assert!(cache.previous.is_none());
        }
    }
}

#[cfg(test)]
mod freshness_due_tests {
    use super::*;
    use crate::accessible_metrics::due_test_fixture::Fixture;
    use std::time::Duration;
    struct Target {
        metrics: Fixture,
        hidden: bool,
        paints: usize,
        redraws: usize,
        restore_during_pump: bool,
        fail_pump: bool,
        queued_redraw: bool,
    }
    impl Target {
        fn new(hidden: bool) -> Self {
            Self {
                metrics: Fixture::new(),
                hidden,
                paints: 0,
                redraws: 0,
                restore_during_pump: false,
                fail_pump: false,
                queued_redraw: false,
            }
        }
        fn fresh_presented(&mut self, now: Instant) {
            self.metrics.publish(now, 1_073_741_824);
            assert!(self.metrics.panel.check_freshness(now));
            self.metrics.render();
            self.metrics.panel.frame_presented();
            assert!(!self.metrics.panel.check_freshness(now));
        }
    }
    impl DueTarget for Target {
        fn hidden(&self) -> bool {
            self.hidden
        }
        fn pump_hidden(&mut self) -> Result<(), HostError> {
            self.paints += 1;
            if self.fail_pump {
                return Err("synthetic hidden paint failure".into());
            }
            self.metrics.pump();
            if self.restore_during_pump {
                self.hidden = false;
            }
            Ok(())
        }
        fn redraw(&mut self) {
            self.redraws += 1;
            self.queued_redraw = true;
        }
    }
    #[test]
    fn generic_coincidence_redraws_when_real_mailbox_freshness_is_unchanged() {
        let now = Instant::now();
        let mut target = Target::new(false);
        target.fresh_presented(now);
        target
            .metrics
            .publish(now + Duration::from_secs(1), 1_073_741_824);
        let changed = target
            .metrics
            .panel
            .check_freshness(now + Duration::from_secs(1));
        assert!(!changed);
        dispatch_due(&mut target, true, changed).unwrap();
        assert_eq!((target.paints, target.redraws), (0, 1));
        assert!(target.queued_redraw);
        dispatch_due(&mut target, false, changed).unwrap();
        assert!(target.queued_redraw);
        assert_eq!(target.redraws, 1);
    }
    #[test]
    fn generic_and_dirty_coincidence_issues_one_effect_and_does_not_commit() {
        let now = Instant::now();
        let mut target = Target::new(false);
        target.fresh_presented(now);
        target.metrics.publish(now, 2_147_483_648);
        let changed = target.metrics.panel.check_freshness(now);
        assert!(changed);
        dispatch_due(&mut target, true, changed).unwrap();
        assert_eq!((target.paints, target.redraws), (0, 1));
        assert!(target.metrics.panel.check_freshness(now));
        target.metrics.render();
        target.metrics.panel.frame_presented();
        assert!(!target.metrics.panel.check_freshness(now));
    }
    #[test]
    fn hidden_metrics_only_keeps_real_dirty_key_until_restored_present() {
        let now = Instant::now();
        let mut target = Target::new(true);
        target.fresh_presented(now);
        let later = now + Duration::from_secs(4);
        let changed = target.metrics.panel.check_freshness(later);
        assert!(changed);
        dispatch_due(&mut target, false, changed).unwrap();
        assert_eq!((target.paints, target.redraws), (0, 0));
        assert!(target.metrics.panel.check_freshness(later));
        target.hidden = false;
        dispatch_due(&mut target, false, true).unwrap();
        assert_eq!((target.paints, target.redraws), (0, 1));
        target.metrics.render();
        target.metrics.panel.frame_presented();
        assert!(target.metrics.unknown_was_presented());
    }
    #[test]
    fn hidden_generic_pump_preserves_real_stop_and_ack_wake_path() {
        let now = Instant::now();
        let mut target = Target::new(true);
        target.metrics.panel.request_stop();
        let changed = target.metrics.panel.check_freshness(now);
        assert!(changed);
        dispatch_due(&mut target, true, changed).unwrap();
        assert_eq!((target.paints, target.redraws), (1, 0));
        assert!(!target.metrics.panel.stopped());
        let before = target.metrics.wake_count();
        // Original worker completion guard, not a fabricated ACK bool.
        target.metrics.acknowledge_worker();
        assert!(target.metrics.panel.stopped());
        assert!(target.metrics.wake_count() > before);
        assert_eq!(target.metrics.panel.freshness_deadline(now), None);
    }
    #[test]
    fn generic_hidden_pump_followed_by_restore_requests_visible_redraw() {
        let now = Instant::now();
        let mut target = Target::new(true);
        target.fresh_presented(now);
        target.restore_during_pump = true;
        let changed = target.metrics.panel.check_freshness(now);
        assert!(!changed);
        dispatch_due(&mut target, true, changed).unwrap();
        assert!(!target.hidden);
        assert_eq!((target.paints, target.redraws), (1, 1));
    }
    #[test]
    fn armed_hidden_retry_never_calls_new_deadline_supplier() {
        let now = Instant::now();
        let mut metrics = Fixture::new();
        assert!(metrics.panel.check_freshness(now));
        let original = metrics.panel.freshness_deadline(now).unwrap();
        let mut deadline = Some(original);
        let mut calls = 0;
        for step in 1..10 {
            arm_if_absent(&mut deadline, || {
                calls += 1;
                metrics
                    .panel
                    .freshness_deadline(now + Duration::from_millis(step * 100))
            });
        }
        assert_eq!(calls, 0);
        assert_eq!(deadline, Some(original));
        deadline = None;
        arm_if_absent(&mut deadline, || {
            calls += 1;
            metrics.panel.freshness_deadline(now)
        });
        assert_eq!(calls, 1);
        assert_eq!(deadline, Some(original));
        metrics.panel.request_stop();
        deadline = None;
        arm_if_absent(&mut deadline, || metrics.panel.freshness_deadline(now));
        assert_eq!(deadline, None);
    }
    #[test]
    fn failed_hidden_pump_returns_original_error_without_redraw_or_commit() {
        let now = Instant::now();
        let mut target = Target::new(true);
        target.fresh_presented(now);
        target.metrics.publish(now, 2_147_483_648);
        let changed = target.metrics.panel.check_freshness(now);
        assert!(changed);
        target.fail_pump = true;
        let error = dispatch_due(&mut target, true, changed).unwrap_err();
        assert_eq!(error.to_string(), "synthetic hidden paint failure");
        assert_eq!((target.paints, target.redraws), (1, 0));
        assert!(target.metrics.panel.check_freshness(now));
    }
}

#[cfg(all(windows, feature = "first-paint-fixture"))]
mod pointer_render_fixture {
    use super::{EventLoopProxy, UserEvent, WindowId, egui};
    use serde::{Deserialize, Serialize};
    use std::{
        fs::{File, OpenOptions},
        io::{Read, Write},
        os::windows::{
            fs::{MetadataExt, OpenOptionsExt},
            io::AsRawHandle,
        },
        path::{Path, PathBuf},
        sync::{
            Arc,
            atomic::{AtomicBool, AtomicU8, Ordering},
        },
        time::{Duration, Instant},
    };
    use windows_sys::Win32::{
        Storage::FileSystem::{
            BY_HANDLE_FILE_INFORMATION, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
            FILE_SHARE_READ, FILE_SHARE_WRITE, GetFileInformationByHandle,
        },
        System::Threading::GetCurrentThreadId,
    };
    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetThreadDesktop(thread: u32) -> *mut std::ffi::c_void;
        fn GetUserObjectInformationW(
            handle: *mut std::ffi::c_void,
            index: i32,
            buffer: *mut std::ffi::c_void,
            length: u32,
            needed: *mut u32,
        ) -> i32;
    }
    #[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "snake_case")]
    pub(super) enum Action {
        Idle,
        Move,
        Press,
        Release,
    }
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(deny_unknown_fields)]
    pub(super) struct Command {
        pub sequence: u8,
        pub target: String,
        pub action: Action,
        pub x: f32,
        pub y: f32,
    }
    impl Command {
        fn validate(&self) -> Result<(), String> {
            if !(1..=16).contains(&self.sequence)
                || !self.x.is_finite()
                || !self.y.is_finite()
                || !(0.0..=4096.0).contains(&self.x)
                || !(0.0..=4096.0).contains(&self.y)
                || ![
                    "nav-settings",
                    "nav-download",
                    "url",
                    "network-interface",
                    "primary-disabled",
                    "secondary-refresh",
                ]
                .contains(&self.target.as_str())
            {
                return Err("pointer fixture bounded command rejected".into());
            }
            Ok(())
        }
    }
    #[derive(Debug, Serialize)]
    struct Ack {
        sequence: u8,
        target: String,
        action: Action,
        input_source: &'static str,
        os_pointer_verified: bool,
        physical_pointer_verified: bool,
        presented: bool,
    }
    #[derive(Debug)]
    pub(super) struct State {
        pending: Option<Command>,
        current: Option<Command>,
        down: bool,
        applied: Arc<AtomicU8>,
        stop: Arc<AtomicBool>,
    }
    impl Drop for State {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::Release);
        }
    }
    fn regular(file: &File) -> Result<(u32, u32, u32, u64), String> {
        let meta = file.metadata().map_err(|_| "pointer fixture metadata")?;
        if !meta.is_file() || meta.file_attributes() & 0x400 != 0 || meta.len() > 1024 {
            return Err("pointer fixture original file bound/reparse".into());
        }
        let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
        if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0 {
            return Err("pointer fixture file identity".into());
        }
        Ok((
            info.dwVolumeSerialNumber,
            info.nFileIndexHigh,
            info.nFileIndexLow,
            meta.len(),
        ))
    }
    fn load(path: &Path) -> Result<Command, String> {
        let mut file = OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)
            .map_err(|_| "pointer fixture command open")?;
        let before = regular(&file)?;
        let mut raw = Vec::new();
        Read::by_ref(&mut file)
            .take(1025)
            .read_to_end(&mut raw)
            .map_err(|_| "pointer fixture command read")?;
        if raw.len() > 1024 || regular(&file)? != before {
            return Err("pointer fixture command changed/bounded".into());
        }
        let again = OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)
            .map_err(|_| "pointer fixture original pathname")?;
        if regular(&again)? != before {
            return Err("pointer fixture pathname identity".into());
        }
        let command: Command =
            serde_json::from_slice(&raw).map_err(|_| "pointer fixture command schema")?;
        command.validate()?;
        Ok(command)
    }
    fn directory_identity(file: &File) -> Result<(u32, u32, u32), String> {
        let meta = file
            .metadata()
            .map_err(|_| "pointer fixture opened directory metadata")?;
        if !meta.is_dir() || meta.file_attributes() & 0x400 != 0 {
            return Err("pointer fixture opened directory/reparse rejected".into());
        }
        let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
        if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0
            || info.dwFileAttributes & 0x400 != 0
            || info.dwFileAttributes & 0x10 == 0
        {
            return Err("pointer fixture original native directory identity".into());
        }
        Ok((
            info.dwVolumeSerialNumber,
            info.nFileIndexHigh,
            info.nFileIndexLow,
        ))
    }
    fn open_directory(path: &Path) -> Result<File, String> {
        OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)
            .map_err(|_| "pointer fixture original directory open".into())
    }
    fn pin_directories(root: &Path) -> Result<Vec<File>, String> {
        if !root.is_absolute() {
            return Err("pointer fixture absolute owned root".into());
        }
        let paths: Vec<_> = root.ancestors().collect();
        if paths.len() > 32 {
            return Err("pointer fixture ancestor bound".into());
        }
        let mut pins = Vec::new();
        // Anchor the outermost parent first; every subsequent original ancestor
        // stays open without FILE_SHARE_DELETE throughout child resolution.
        for path in paths.into_iter().rev() {
            let meta =
                std::fs::symlink_metadata(path).map_err(|_| "pointer fixture ancestor metadata")?;
            if !meta.is_dir() || meta.file_attributes() & 0x400 != 0 {
                return Err("pointer fixture ancestor reparse".into());
            }
            let original = open_directory(path)?;
            let identity = directory_identity(&original)?;
            let pathname = open_directory(path)?;
            if directory_identity(&pathname)? != identity
                || directory_identity(&original)? != identity
            {
                return Err("pointer fixture directory pathname changed".into());
            }
            pins.push(original);
        }
        Ok(pins)
    }
    fn private_desktop() -> Result<(), String> {
        let expected = std::env::var("DLPAD_POINTER_FIXTURE_DESKTOP")
            .map_err(|_| "pointer fixture desktop binding")?;
        let nonce = expected
            .strip_prefix("dlpad-cookie-")
            .ok_or("pointer fixture desktop namespace")?;
        if nonce.len() != 32 || !nonce.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("pointer fixture desktop nonce".into());
        }
        let mut name = [0u16; 128];
        let mut needed = 0;
        if unsafe {
            GetUserObjectInformationW(
                GetThreadDesktop(GetCurrentThreadId()),
                2,
                name.as_mut_ptr().cast(),
                256,
                &mut needed,
            )
        } == 0
            || needed > 256
        {
            return Err("pointer fixture native desktop query".into());
        }
        let end = name
            .iter()
            .position(|v| *v == 0)
            .ok_or("pointer fixture desktop name bound")?;
        if String::from_utf16(&name[..end]).map_err(|_| "pointer fixture desktop encoding")?
            != expected
        {
            return Err("pointer fixture exact private desktop".into());
        }
        Ok(())
    }
    impl State {
        pub(super) fn start(
            window: WindowId,
            proxy: EventLoopProxy<UserEvent>,
        ) -> Result<Option<Self>, String> {
            if std::env::var_os("DLPAD_POINTER_RENDER_FIXTURE").is_none() {
                return Ok(None);
            }
            if std::env::var("DLPAD_POINTER_RENDER_FIXTURE").as_deref() != Ok("private-owned") {
                return Err("pointer fixture explicit marker".into());
            }
            private_desktop()?;
            let data = PathBuf::from(
                std::env::var_os("DLPAD_TEST_DATA").ok_or("pointer fixture owned data")?,
            );
            let parent = data
                .parent()
                .and_then(Path::file_name)
                .and_then(|v| v.to_str())
                .ok_or("pointer fixture owned parent")?;
            let suffix = parent
                .strip_prefix("dlpad-settings-owned-")
                .ok_or("pointer fixture owned namespace")?;
            if data.file_name().and_then(|v| v.to_str()) != Some("data")
                || suffix.len() != 32
                || !suffix.bytes().all(|b| b.is_ascii_hexdigit())
            {
                return Err("pointer fixture owned root shape".into());
            }
            let root = data.join("pointer-fixture");
            let applied = Arc::new(AtomicU8::new(0));
            let stop = Arc::new(AtomicBool::new(false));
            let (worker_applied, worker_stop) = (applied.clone(), stop.clone());
            std::thread::Builder::new()
                .name("dlpad-pointer-render-fixture".into())
                .spawn(move || {
                    let run = || -> Result<(), String> {
                        let _pins = pin_directories(&root)?;
                        let deadline = Instant::now() + Duration::from_secs(120);
                        for sequence in 1..=16u8 {
                            let path = root.join(format!("command-{sequence:02}.json"));
                            while !path.exists() {
                                if worker_stop.load(Ordering::Acquire) {
                                    return Ok(());
                                }
                                if Instant::now() >= deadline {
                                    return Err("pointer fixture command deadline".into());
                                }
                                std::thread::sleep(Duration::from_millis(20));
                            }
                            let command = load(&path)?;
                            if command.sequence != sequence {
                                return Err("pointer fixture exact sequence".into());
                            }
                            proxy
                                .send_event(UserEvent::PointerFixture(window, command.clone()))
                                .map_err(|_| "pointer fixture owned event loop closed")?;
                            while worker_applied.load(Ordering::Acquire) != sequence {
                                if worker_stop.load(Ordering::Acquire) {
                                    return Ok(());
                                }
                                if Instant::now() >= deadline {
                                    return Err("pointer fixture presentation deadline".into());
                                }
                                std::thread::sleep(Duration::from_millis(20));
                            }
                            let raw = serde_json::to_vec(&Ack {
                                sequence,
                                target: command.target,
                                action: command.action,
                                input_source: "synthetic-egui-fixture",
                                os_pointer_verified: false,
                                physical_pointer_verified: false,
                                presented: true,
                            })
                            .map_err(|_| "pointer fixture ack schema")?;
                            let mut ack = OpenOptions::new()
                                .write(true)
                                .create_new(true)
                                .share_mode(0)
                                .open(root.join(format!("ack-{sequence:02}.json")))
                                .map_err(|_| "pointer fixture create-only ack")?;
                            ack.write_all(&raw)
                                .map_err(|_| "pointer fixture ack write")?;
                        }
                        Ok(())
                    };
                    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(run)) {
                        Ok(Ok(())) => {}
                        Ok(Err(error)) => {
                            let _ = proxy.send_event(UserEvent::PointerFixtureFailed(error));
                        }
                        Err(_) => {
                            let _ = proxy.send_event(UserEvent::PointerFixtureFailed(
                                "pointer fixture worker panicked".into(),
                            ));
                        }
                    }
                })
                .map_err(|_| "pointer fixture worker spawn")?;
            Ok(Some(Self {
                pending: None,
                current: None,
                down: false,
                applied,
                stop,
            }))
        }
        pub(super) fn receive(&mut self, command: Command) -> Result<(), String> {
            command.validate()?;
            if self.pending.is_some()
                || command.sequence != self.applied.load(Ordering::Acquire) + 1
            {
                return Err("pointer fixture capacity/sequence".into());
            }
            self.pending = Some(command);
            Ok(())
        }
        pub(super) fn append(&mut self, input: &mut egui::RawInput) -> Result<(), String> {
            if let Some(command) = self.pending.take() {
                let pos = egui::pos2(command.x, command.y);
                if !input.screen_rect.is_some_and(|r| r.contains(pos)) {
                    return Err("pointer fixture original client coordinate".into());
                }
                match command.action {
                    Action::Idle if !self.down => input.events.push(egui::Event::PointerGone),
                    Action::Move if !self.down => input.events.push(egui::Event::PointerMoved(pos)),
                    Action::Press if !self.down => {
                        input.events.push(egui::Event::PointerMoved(pos));
                        input.events.push(egui::Event::PointerButton {
                            pos,
                            button: egui::PointerButton::Primary,
                            pressed: true,
                            modifiers: egui::Modifiers::NONE,
                        });
                        self.down = true;
                    }
                    Action::Release
                        if self.down
                            && self.current.as_ref().is_some_and(|old| {
                                old.target == command.target
                                    && old.x == command.x
                                    && old.y == command.y
                            }) =>
                    {
                        input.events.push(egui::Event::PointerMoved(pos));
                        input.events.push(egui::Event::PointerButton {
                            pos,
                            button: egui::PointerButton::Primary,
                            pressed: false,
                            modifiers: egui::Modifiers::NONE,
                        });
                        self.down = false;
                    }
                    _ => return Err("pointer fixture transition".into()),
                }
                self.current = Some(command);
            } else if let Some(command) = &self.current
                && command.action != Action::Idle
            {
                input
                    .events
                    .push(egui::Event::PointerMoved(egui::pos2(command.x, command.y)));
            }
            Ok(())
        }
        pub(super) fn presented(&self) {
            if let Some(command) = &self.current {
                self.applied.store(command.sequence, Ordering::Release);
            }
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        fn state() -> State {
            State {
                pending: None,
                current: None,
                down: false,
                applied: Arc::new(AtomicU8::new(0)),
                stop: Arc::new(AtomicBool::new(false)),
            }
        }
        fn command(sequence: u8, action: Action, x: f32, y: f32) -> Command {
            Command {
                sequence,
                target: "nav-settings".into(),
                action,
                x,
                y,
            }
        }
        #[test]
        fn actual_egui_pointer_pipeline_hover_press_release_and_capacity() {
            let mut fixture = state();
            let ctx = egui::Context::default();
            let mut button = egui::Rect::NOTHING;
            let mut draw = |fixture: &mut State, time: f64| {
                let mut raw = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(400.0, 240.0),
                    )),
                    focused: true,
                    time: Some(time),
                    ..Default::default()
                };
                fixture.append(&mut raw).unwrap();
                let mut flags = (false, false, false);
                let out = ctx.run(raw, |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let response = ui.add(
                            egui::Button::new("owned renderer fixture target")
                                .min_size(egui::vec2(160.0, 32.0)),
                        );
                        button = response.rect;
                        flags = (
                            response.hovered(),
                            response.is_pointer_button_down_on(),
                            response.clicked(),
                        );
                    });
                });
                (out, flags, button)
            };
            let (_, _, rect) = draw(&mut fixture, 0.0);
            let pos = rect.center();
            fixture
                .receive(command(1, Action::Move, pos.x, pos.y))
                .unwrap();
            let (_, hover, _) = draw(&mut fixture, 0.05);
            assert!(hover.0);
            fixture.presented();
            fixture
                .receive(command(2, Action::Press, pos.x, pos.y))
                .unwrap();
            let (_, press, _) = draw(&mut fixture, 0.10);
            assert!(press.1);
            fixture.presented();
            fixture
                .receive(command(3, Action::Release, pos.x, pos.y))
                .unwrap();
            assert!(
                fixture
                    .receive(command(4, Action::Move, pos.x, pos.y))
                    .is_err(),
                "capacity1 rejects second command before frame"
            );
            let (_, release, _) = draw(&mut fixture, 0.15);
            assert!(release.2, "actual egui dispatch clicks on release");
            fixture.presented();
            fixture
                .receive(command(4, Action::Press, 800.0, 900.0))
                .unwrap();
            assert!(
                fixture
                    .append(&mut egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(400.0, 240.0)
                        )),
                        ..Default::default()
                    })
                    .is_err(),
                "outside original client fail closed"
            );
        }
        #[test]
        fn append_preserves_original_os_events_and_rejects_release_without_press() {
            let mut fixture = state();
            fixture
                .receive(command(1, Action::Move, 20.0, 20.0))
                .unwrap();
            let mut raw = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(400.0, 240.0),
                )),
                events: vec![egui::Event::PointerGone],
                ..Default::default()
            };
            fixture.append(&mut raw).unwrap();
            assert!(matches!(raw.events[0], egui::Event::PointerGone));
            assert!(matches!(raw.events[1], egui::Event::PointerMoved(_)));
            fixture.presented();
            fixture
                .receive(command(2, Action::Release, 20.0, 20.0))
                .unwrap();
            assert!(fixture.append(&mut raw).is_err());
        }
    }
}

#[cfg(test)]
mod repaint_generation_tests {
    use super::*;
    use std::time::Duration;

    struct Fixture {
        ctx: egui::Context,
        queue: Arc<Mutex<Repaint>>,
        requests: Arc<Mutex<Vec<egui::RequestRepaintInfo>>>,
        anchor: Instant,
    }
    impl Fixture {
        fn new() -> Self {
            let ctx = egui::Context::default();
            for _ in 0..3 {
                let _ = ctx.run(Self::input(), |_| {});
            }
            let queue = Arc::new(Mutex::new(Repaint::default()));
            let requests = Arc::new(Mutex::new(Vec::new()));
            let anchor = Instant::now();
            let queued = queue.clone();
            let recorded = requests.clone();
            ctx.set_request_repaint_callback(move |info| {
                recorded.lock().unwrap().push(info);
                queued.lock().unwrap().request(info, anchor);
            });
            Self {
                ctx,
                queue,
                requests,
                anchor,
            }
        }
        fn input() -> egui::RawInput {
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(320.0, 240.0),
                )),
                focused: true,
                ..Default::default()
            }
        }
        fn pass(&self) -> u64 {
            self.ctx.cumulative_pass_nr_for(egui::ViewportId::ROOT)
        }
        fn draw(&self) -> egui::FullOutput {
            self.ctx.run(Self::input(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.painter().rect_filled(
                        egui::Rect::from_min_size(egui::pos2(16.0, 16.0), egui::vec2(32.0, 32.0)),
                        0,
                        egui::Color32::WHITE,
                    );
                });
            })
        }
        fn take(&self) -> Option<Instant> {
            {
                let pass = self.pass();
                self.queue.lock().unwrap().take(pass)
            }
        }
    }

    #[test]
    fn actual_context_callback_preserves_the_one_pass_settling_repaint() {
        let fixture = Fixture::new();
        let requested = fixture.pass();
        fixture.ctx.request_repaint();
        assert_eq!(
            fixture
                .requests
                .lock()
                .unwrap()
                .last()
                .unwrap()
                .current_cumulative_pass_nr,
            requested
        );
        assert_eq!(fixture.take(), Some(fixture.anchor));
        let _ = fixture.draw();
        assert_eq!(fixture.pass(), requested + 1);
        assert_eq!(
            fixture.take(),
            Some(fixture.anchor),
            "actual begin-pass callback must retain egui's settling repaint"
        );
        let _ = fixture.draw();
        assert_eq!(fixture.pass(), requested + 2);
        assert_eq!(fixture.take(), None);
    }

    #[test]
    fn callback_already_surpassed_by_two_actual_passes_is_stale_even_on_cache_hit() {
        let fixture = Fixture::new();
        // Warm this real primitive set before testing reuse or callback generation.
        let _ = fixture.draw();
        let requested = fixture.pass();
        fixture.ctx.request_repaint();
        let first = fixture.draw();
        let mut cache = frame_reuse::FrameReuse::default();
        let primitives = fixture.ctx.tessellate(first.shapes, first.pixels_per_point);
        cache.presented(primitives, [320, 240], first.pixels_per_point);
        let second = fixture.draw();
        let primitives = fixture
            .ctx
            .tessellate(second.shapes, second.pixels_per_point);
        assert!(
            cache.can_reuse(
                &primitives,
                [320, 240],
                second.pixels_per_point,
                &second.textures_delta,
                1
            ),
            "actual unchanged egui shapes must exercise the cache-hit seam"
        );
        assert_eq!(
            fixture.pass(),
            requested + 2,
            "cache reuse must not erase completed egui passes"
        );
        assert_eq!(
            fixture.take(),
            None,
            "old pending callback must not schedule a third paint"
        );
    }

    #[test]
    fn fresh_worker_delayed_request_is_not_retagged_with_an_old_stale_deadline() {
        let fixture = Fixture::new();
        fixture.ctx.request_repaint();
        let _ = fixture.draw();
        let _ = fixture.draw();
        // The old immediate callback is still pending/coalesced. Its deadline
        // must not be relabelled as the fresh worker's future request.
        let ctx = fixture.ctx.clone();
        std::thread::spawn(move || ctx.request_repaint_after(Duration::from_secs(3)))
            .join()
            .unwrap();
        let fresh = *fixture.requests.lock().unwrap().last().unwrap();
        assert_eq!(fresh.current_cumulative_pass_nr, fixture.pass());
        let actual_future = fixture.anchor.checked_add(fresh.delay).unwrap();
        assert!(actual_future > fixture.anchor);
        assert_eq!(fixture.take(), Some(actual_future));
        assert_eq!(fixture.take(), None);
    }

    #[test]
    fn actual_worker_immediate_request_survives_an_old_pending_callback() {
        let fixture = Fixture::new();
        fixture.ctx.request_repaint();
        let _ = fixture.draw();
        let _ = fixture.draw();
        let ctx = fixture.ctx.clone();
        std::thread::spawn(move || ctx.request_repaint())
            .join()
            .unwrap();
        assert_eq!(
            fixture
                .requests
                .lock()
                .unwrap()
                .last()
                .unwrap()
                .current_cumulative_pass_nr,
            fixture.pass()
        );
        assert_eq!(fixture.take(), Some(fixture.anchor));
        let _ = fixture.draw();
        assert_eq!(
            fixture.take(),
            Some(fixture.anchor),
            "fresh worker's legitimate settling callback must still run"
        );
    }

    #[test]
    fn real_other_viewport_request_does_not_queue_a_root_window_wake() {
        let fixture = Fixture::new();
        let other = egui::ViewportId::from_hash_of("owned-test-other-viewport");
        fixture.ctx.request_repaint_of(other);
        assert_eq!(
            fixture.requests.lock().unwrap().last().unwrap().viewport_id,
            other
        );
        assert!(!fixture.queue.lock().unwrap().queued);
        assert_eq!(fixture.take(), None);
        fixture.ctx.request_repaint();
        assert!(fixture.queue.lock().unwrap().queued);
        assert_eq!(fixture.take(), Some(fixture.anchor));
    }

    #[test]
    fn current_and_previous_generation_keep_the_earliest_actual_delayed_request() {
        let fixture = Fixture::new();
        fixture.ctx.request_repaint_after(Duration::from_secs(9));
        let _ = fixture.draw();
        fixture.ctx.request_repaint_after(Duration::from_secs(3));
        let current = fixture.pass();
        let expected = fixture
            .requests
            .lock()
            .unwrap()
            .iter()
            .filter(|request| {
                request.viewport_id == egui::ViewportId::ROOT
                    && (request.current_cumulative_pass_nr == current
                        || request.current_cumulative_pass_nr.checked_add(1) == Some(current))
            })
            .map(|request| fixture.anchor.checked_add(request.delay).unwrap())
            .min();
        assert!(expected.is_some());
        assert_eq!(fixture.take(), expected);
    }
    #[test]
    fn many_actual_pass_generations_remain_bounded_without_losing_valid_deadlines() {
        let fixture = Fixture::new();
        for _ in 0..8 {
            fixture.ctx.request_repaint_after(Duration::from_secs(9));
            let _ = fixture.draw();
            assert!(
                fixture
                    .queue
                    .lock()
                    .unwrap()
                    .requests
                    .iter()
                    .flatten()
                    .count()
                    <= 2
            );
        }
        fixture.ctx.request_repaint_after(Duration::from_secs(3));
        let current = fixture.pass();
        let expected = fixture
            .requests
            .lock()
            .unwrap()
            .iter()
            .filter(|request| {
                request.viewport_id == egui::ViewportId::ROOT
                    && (request.current_cumulative_pass_nr == current
                        || request.current_cumulative_pass_nr.checked_add(1) == Some(current))
            })
            .map(|request| fixture.anchor.checked_add(request.delay).unwrap())
            .min();
        assert!(expected.is_some());
        assert_eq!(fixture.take(), expected);
    }
}

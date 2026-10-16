//! CPU managed-tool UI. One owned worker performs all I/O.
use crate::accessible_tool_preferences::ManagedAutoTicket;
use crate::tool_update_ui as update_ui;
use crate::{
    catalog as c,
    managed::{ToolId, candidate, tool_error},
};
use dlpad_engine::tools::tool_update_flow::{
    dynamic_ytdlp::{self, DynamicError},
    release_discovery::{self, CapturedPin, DiscoveryError, Observation, Tool as ReleaseTool},
    version_actions::{self, Catalog, FlowError, Supply},
};
use dlpad_engine::tools::{InstallManifest, InstalledTool, StagingRecovery, ToolManager};
use eframe::egui;
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    thread,
    time::Duration,
};

struct Inventory {
    installed: Vec<InstalledTool>,
    active: Option<InstalledTool>,
    recovery: StagingRecovery,
    inspection: Option<ManagedInspection>,
    auto_selected: Option<(ToolId, PathBuf, ManagedAutoTicket)>,
    explicit_selected: Option<(ToolId, PathBuf)>,
    rows: std::collections::BTreeMap<RowKey, RowState>,
}
#[derive(Clone, Copy)]
enum BackgroundWork {
    Lookup,
    VersionCheck,
    Install,
    Verify,
    Remove,
    Select,
}
impl BackgroundWork {
    fn label(self) -> &'static str {
        match self {
            Self::Lookup => c::TOOL_BACKGROUND_LOOKUP,
            Self::VersionCheck => c::TOOL_BACKGROUND_VERSION_CHECK,
            Self::Install => c::TOOL_BACKGROUND_INSTALL,
            Self::Verify => c::TOOL_BACKGROUND_VERIFY,
            Self::Remove => c::TOOL_BACKGROUND_REMOVE,
            Self::Select => c::TOOL_BACKGROUND_SELECT,
        }
    }
}
enum Action {
    Update(UpdateRequest),
    Refresh,
    Install,
    Select(InstallManifest),
    Remove(InstallManifest),
    Use,
}
struct Command {
    tool: ToolId,
    action: Action,
    context: egui::Context,
    automatic_tickets: [Option<ManagedAutoTicket>; 3],
    request_seq: u64,
}
enum Outcome {
    Update(Box<UpdateCompletion>),
    Tagged {
        tool: ToolId,
        request_seq: u64,
        result: Box<Result<Outcome, String>>,
    },
    Inventory(Box<Inventory>),
    Selected {
        tool: ToolId,
        path: PathBuf,
    },
}
#[derive(Default)]
struct Signals {
    stop: AtomicBool,
    stopped: AtomicBool,
    failed: AtomicBool,
    cancel: Arc<AtomicBool>,
}
struct Ack {
    signals: Arc<Signals>,
    context: egui::Context,
}
impl Drop for Ack {
    fn drop(&mut self) {
        self.signals.stopped.store(true, Ordering::Release);
        self.context.request_repaint();
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum DisplayState {
    #[default]
    Empty,
    Fresh,
    Stale,
    Refreshing,
    Failed,
}
pub(crate) struct Panel {
    updates: UpdateState,
    commands: SyncSender<Command>,
    results: Receiver<Result<Outcome, String>>,
    signals: Arc<Signals>,
    inventory: Option<Inventory>,
    cached: [Option<Inventory>; 3],
    display_state: [DisplayState; 3],
    errors: [Option<String>; 3],
    request_seq: u64,
    pending_dispatch: Option<(ToolId, u64)>,
    background_work: Option<BackgroundWork>,
    initial_refresh_pending: bool,
    error: Option<String>,
    selected: Option<(ToolId, PathBuf)>,
    auto_selected: Option<(ToolId, PathBuf, ManagedAutoTicket)>,
    inspection: Option<ManagedInspection>,
    automatic_tickets: [Option<ManagedAutoTicket>; 3],
    pending_row: Option<RowKey>,
    tool: ToolId,
    busy: bool,
    context: egui::Context,
}
/// UI-only availability; slots are always [yt-dlp, FFmpeg, Deno].
#[derive(Clone, Debug)]
pub(crate) enum Availability {
    Checking,
    Available(PathBuf),
    Unavailable(&'static str),
}
#[derive(Clone, Debug)]
pub(crate) struct ManagedInspection {
    pub slots: [Availability; 3],
}
fn tool_slot(tool: ToolId) -> usize {
    match tool {
        ToolId::YtDlp => 0,
        ToolId::Ffmpeg => 1,
        ToolId::Deno => 2,
    }
}
fn inspection_reason(error: &dlpad_engine::tools::ToolError) -> &'static str {
    use dlpad_engine::tools::ToolError;
    match error {
        ToolError::HashMismatch | ToolError::UnexpectedSize => c::TOOL_HASH_FAILED,
        ToolError::VersionMismatch => c::TOOL_VERSION_FAILED,
        ToolError::TimedOut => c::TOOL_TIMEOUT,
        _ => c::TOOL_AUTO_UNAVAILABLE,
    }
}
fn inspect_checkpoint(cancel: &Arc<AtomicBool>) -> Result<(), dlpad_engine::tools::ToolError> {
    if cancel.load(Ordering::Acquire) {
        Err(dlpad_engine::tools::ToolError::Cancelled)
    } else {
        Ok(())
    }
}
fn inspect_terminal(error: &dlpad_engine::tools::ToolError) -> bool {
    matches!(
        error,
        dlpad_engine::tools::ToolError::Cancelled
            | dlpad_engine::tools::ToolError::ExecutionPending
    )
}
fn existing_managed_manager(data: &std::path::Path) -> Result<Option<ToolManager>, String> {
    let root = data.join("managed-tools");
    match std::fs::symlink_metadata(&root) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => (),
        _ => return Err(c::TOOL_FAILED.to_owned()),
    }
    // Never adopt an unmarked directory or create a missing activation database.
    for name in [".dlpad-owner", "activation.sqlite"] {
        let metadata =
            std::fs::symlink_metadata(root.join(name)).map_err(|_| c::TOOL_FAILED.to_owned())?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(c::TOOL_FAILED.to_owned());
        }
    }
    ToolManager::new(&root).map(Some).map_err(tool_error)
}
fn inspect_recorded_tool(
    manager: &mut ToolManager,
    tool: ToolId,
    manifest: &InstallManifest,
    cancel: &Arc<AtomicBool>,
) -> Result<InstalledTool, dlpad_engine::tools::ToolError> {
    use dlpad_engine::tools::ToolError;
    inspect_checkpoint(cancel)?;
    if manifest.tool != tool.id() {
        return Err(ToolError::InvalidManifest);
    }
    match tool {
        ToolId::YtDlp => manager.inspect_existing(manifest, cancel),
        ToolId::Deno | ToolId::Ffmpeg => {
            let recorded = manager
                .recorded_package_for_manifest(manifest)?
                .ok_or(ToolError::InvalidManifest)?;
            let verified = match tool {
                ToolId::Deno => manager.inspect_verified_deno(&recorded.package, cancel),
                ToolId::Ffmpeg => manager.inspect_verified_ffmpeg(&recorded.package, cancel),
                ToolId::YtDlp => unreachable!(),
            }?;
            Ok(verified.main)
        }
    }
}
fn inspect_automatic_tool(
    manager: &mut ToolManager,
    tool: ToolId,
    cancel: &Arc<AtomicBool>,
) -> Result<Option<InstalledTool>, dlpad_engine::tools::ToolError> {
    inspect_checkpoint(cancel)?;
    let mut rejected_active = None;
    // Existing active leaf first, independent of malformed unrelated inactive entries.
    match manager.active_installed(tool.id()) {
        Ok(Some(active)) => match inspect_recorded_tool(manager, tool, &active.manifest, cancel) {
            Ok(verified) => {
                inspect_checkpoint(cancel)?;
                return Ok(Some(verified));
            }
            Err(error) if inspect_terminal(&error) => return Err(error),
            Err(error) => rejected_active = Some(error),
        },
        Ok(None) => (),
        Err(error) if inspect_terminal(&error) => return Err(error),
        Err(error) => rejected_active = Some(error),
    }
    inspect_checkpoint(cancel)?;
    // Exact current pinned candidate only: no installed()/arbitrary version scan.
    let fallback = match tool {
        ToolId::YtDlp => match candidate() {
            Ok((manifest, _)) => Some(manager.inspect_existing(&manifest, cancel)),
            Err(_) => None,
        },
        ToolId::Deno | ToolId::Ffmpeg => {
            crate::managed_candidates::archive_candidate(tool).map(|package| match tool {
                ToolId::Deno => manager
                    .inspect_verified_deno(&package, cancel)
                    .map(|verified| verified.main),
                ToolId::Ffmpeg => manager
                    .inspect_verified_ffmpeg(&package, cancel)
                    .map(|verified| verified.main),
                ToolId::YtDlp => unreachable!(),
            })
        }
    };
    match fallback {
        Some(Ok(verified)) => {
            inspect_checkpoint(cancel)?;
            Ok(Some(verified))
        }
        Some(Err(error)) if inspect_terminal(&error) => Err(error),
        Some(Err(dlpad_engine::tools::ToolError::Io(error)))
            if error.kind() == std::io::ErrorKind::NotFound =>
        {
            if let Some(active_error) = rejected_active {
                Err(active_error)
            } else {
                Ok(None)
            }
        }
        Some(Err(error)) => Err(error),
        None => {
            if let Some(error) = rejected_active {
                Err(error)
            } else {
                Ok(None)
            }
        }
    }
}
/// Worker-only, readonly all3 slots even when current UI mode is Manual.
pub(crate) fn inspect_managed(
    data: &std::path::Path,
    cancel: &Arc<AtomicBool>,
) -> Result<ManagedInspection, String> {
    inspect_managed_with(data, cancel, inspect_automatic_tool)
}
fn inspect_managed_with(
    data: &std::path::Path,
    cancel: &Arc<AtomicBool>,
    mut inspect: impl FnMut(
        &mut ToolManager,
        ToolId,
        &Arc<AtomicBool>,
    ) -> Result<Option<InstalledTool>, dlpad_engine::tools::ToolError>,
) -> Result<ManagedInspection, String> {
    inspect_checkpoint(cancel).map_err(tool_error)?;
    let mut slots = [
        Availability::Unavailable(c::TOOL_AUTO_UNAVAILABLE),
        Availability::Unavailable(c::TOOL_AUTO_UNAVAILABLE),
        Availability::Unavailable(c::TOOL_AUTO_UNAVAILABLE),
    ];
    let Some(mut manager) = existing_managed_manager(data)? else {
        return Ok(ManagedInspection { slots });
    };
    for tool in [ToolId::YtDlp, ToolId::Ffmpeg, ToolId::Deno] {
        inspect_checkpoint(cancel).map_err(tool_error)?;
        slots[tool_slot(tool)] = match inspect(&mut manager, tool, cancel) {
            Ok(Some(verified)) => Availability::Available(verified.executable),
            Ok(None) => Availability::Unavailable(c::TOOL_AUTO_UNAVAILABLE),
            Err(error) if inspect_terminal(&error) => return Err(tool_error(error)),
            Err(error) => Availability::Unavailable(inspection_reason(&error)),
        };
        // A root ownership/database failure is fatal, not a per-tool fallback reason.
        if existing_managed_manager(data)?.is_none() {
            return Err(c::TOOL_FAILED.to_owned());
        }
    }
    inspect_checkpoint(cancel).map_err(tool_error)?;
    Ok(ManagedInspection { slots })
}

type RowKey = (String, String, String);
#[derive(Clone, Default)]
enum RowState {
    #[default]
    Unknown,
    Checking,
    Verified,
    Rejected(&'static str),
}
fn row_key(manifest: &InstallManifest) -> RowKey {
    (
        manifest.tool.clone(),
        manifest.version.clone(),
        manifest.target.clone(),
    )
}
fn reason_for_message(message: &str) -> &'static str {
    if message == c::TOOL_HASH_FAILED {
        c::TOOL_HASH_FAILED
    } else if message == c::TOOL_VERSION_FAILED {
        c::TOOL_VERSION_FAILED
    } else if message == c::TOOL_TIMEOUT {
        c::TOOL_TIMEOUT
    } else if message == c::TOOL_CANCELLED {
        c::TOOL_CANCELLED
    } else if message == c::TOOL_EXECUTION_PENDING {
        c::TOOL_EXECUTION_PENDING
    } else {
        c::TOOL_AUTO_UNAVAILABLE
    }
}
fn current_pinned_key(tool: ToolId) -> Option<RowKey> {
    match tool {
        ToolId::YtDlp => candidate().ok().map(|(manifest, _)| row_key(&manifest)),
        ToolId::Deno | ToolId::Ffmpeg => crate::managed_candidates::archive_candidate(tool)
            .map(|p| (p.tool, p.tool_version, p.target)),
    }
}
/// A metadata display/gating comparison, never runtime or mutation authority.
fn pinned_record_present(tool: ToolId, inventory: Option<&Inventory>) -> bool {
    let expected = match tool {
        ToolId::YtDlp => candidate().ok().map(|(manifest, _)| manifest),
        ToolId::Deno | ToolId::Ffmpeg => crate::managed_candidates::archive_candidate(tool)
            .and_then(|package| {
                let main = package.components.iter().find(|component| {
                    component.role == dlpad_engine::tools::ArchiveComponentRole::Executable
                })?;
                Some(InstallManifest {
                    tool: package.tool,
                    version: package.tool_version,
                    target: package.target,
                    source_url: package.source_url,
                    sha256: main.sha256.clone(),
                    license: package.license,
                    ownership: package.ownership,
                    executable: main.name.clone(),
                })
            }),
    };
    expected.is_some_and(|expected| {
        inventory.is_some_and(|inventory| {
            inventory.installed.iter().any(|installed| {
                let manifest = &installed.manifest;
                manifest.ownership == dlpad_engine::tools::Ownership::Managed
                    && manifest.tool == expected.tool
                    && manifest.version == expected.version
                    && manifest.target == expected.target
                    && manifest.source_url == expected.source_url
                    && manifest.sha256 == expected.sha256
                    && manifest.license == expected.license
                    && manifest.executable == expected.executable
            })
        })
    })
}
fn activate_inspected(
    manager: &mut ToolManager,
    tool: ToolId,
    verified: InstalledTool,
    cancel: &Arc<AtomicBool>,
) -> Result<InstalledTool, dlpad_engine::tools::ToolError> {
    inspect_checkpoint(cancel)?;
    if manager.active(tool.id())?
        == Some((
            verified.manifest.version.clone(),
            verified.manifest.target.clone(),
        ))
    {
        // Fresh readonly runtime/hash proof for unchanged active selection: no DB rewrite.
        return Ok(verified);
    }
    match tool {
        ToolId::YtDlp => manager.select_verified_existing(&verified.manifest, cancel),
        ToolId::Deno | ToolId::Ffmpeg => {
            let package = manager
                .recorded_package_for_manifest(&verified.manifest)?
                .ok_or(dlpad_engine::tools::ToolError::InvalidManifest)?;
            let selected = match tool {
                ToolId::Deno => manager.select_verified_deno(&package.package, cancel),
                ToolId::Ffmpeg => manager.select_verified_ffmpeg(&package.package, cancel),
                ToolId::YtDlp => unreachable!(),
            }?;
            Ok(selected.main)
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CurrentSource {
    Unknown,
    Managed,
    External,
}
struct UpdateState {
    renderer: update_ui::Panel,
    render_snapshots: Option<Box<[update_ui::ToolSnapshot; 3]>>,
    render_dirty: bool,
    serial: u64,
    snapshot_generation: [u64; 3],
    observed_generation: [Option<u64>; 3],
    observed: [Option<Observation>; 3],
    captured: [bool; 3],
    evidence: [Option<RetainedEvidence>; 3],
    status: [update_ui::Status; 3],
    current_sources: [CurrentSource; 3],
    pending: Option<(ToolId, u64)>,
}
impl Default for UpdateState {
    fn default() -> Self {
        Self {
            renderer: Default::default(),
            render_snapshots: None,
            render_dirty: true,
            serial: 0,
            snapshot_generation: [0; 3],
            observed_generation: [None; 3],
            observed: std::array::from_fn(|_| None),
            captured: [false; 3],
            evidence: std::array::from_fn(|_| None),
            status: std::array::from_fn(|_| update_ui::Status::Idle),
            current_sources: [CurrentSource::Unknown; 3],
            pending: None,
        }
    }
}
#[derive(Clone, Copy)]
enum UpdateKind {
    Check,
    Install,
    Use,
}
enum UpdateRequest {
    Check {
        generation: u64,
    },
    Install {
        generation: u64,
        observation_generation: u64,
        version: update_ui::VersionKey,
    },
    Use {
        generation: u64,
        manifest: InstallManifest,
    },
}
impl UpdateRequest {
    fn receipt(&self) -> (u64, UpdateKind) {
        match self {
            Self::Check { generation } => (*generation, UpdateKind::Check),
            Self::Install { generation, .. } => (*generation, UpdateKind::Install),
            Self::Use { generation, .. } => (*generation, UpdateKind::Use),
        }
    }
}
enum UpdateValue {
    Observed(Observation, bool),
    Inventory(Box<Inventory>),
    Selected {
        inventory: Box<Inventory>,
        path: PathBuf,
    },
}
#[derive(Clone)]
struct RetainedEvidence {
    directory: PathBuf,
    state: EvidenceState,
    // Preserve a backend-reported publication even if the completion receipt failed.
    installed: Option<InstalledTool>,
}
#[derive(Clone, Copy)]
enum EvidenceState {
    Prepared,
    InstalledInactive,
    InstallFailed,
    InstalledIncomplete,
}
struct UpdateCompletion {
    tool: ToolId,
    generation: u64,
    kind: UpdateKind,
    evidence: Option<RetainedEvidence>,
    result: Result<UpdateValue, &'static str>,
}
type UpdateCache = [Option<(u64, Observation, Option<CapturedPin>)>; 3];
fn release_tool(tool: ToolId) -> ReleaseTool {
    match tool {
        ToolId::YtDlp => ReleaseTool::YtDlp,
        ToolId::Deno => ReleaseTool::Deno,
        ToolId::Ffmpeg => ReleaseTool::Ffmpeg,
    }
}
fn compiled_supply(tool: ToolId) -> Option<Supply> {
    match tool {
        ToolId::YtDlp => candidate()
            .ok()
            .map(|(manifest, bytes)| Supply::Direct { manifest, bytes }),
        ToolId::Deno | ToolId::Ffmpeg => {
            crate::managed_candidates::archive_candidate(tool).map(Supply::Archive)
        }
    }
}
fn exact_approved_observation(
    tool: ToolId,
    observation: &Observation,
    version: &update_ui::VersionKey,
) -> bool {
    observation.tool == release_tool(tool)
        && observation.tag == version.version
        && current_pinned_key(tool).is_some_and(|(id, v, t)| {
            id == tool.id() && v == version.version && t == version.target
        })
}
fn captured_observation_matches(
    tool: ToolId,
    observation: &Observation,
    version: &update_ui::VersionKey,
) -> bool {
    cfg!(all(windows, target_arch = "x86_64", feature = "cpu"))
        && tool == ToolId::YtDlp
        && observation.tool == ReleaseTool::YtDlp
        && observation.tag == version.version
        && version.target == "x86_64-pc-windows-msvc"
}
fn dynamic_reason(error: &DynamicError) -> &'static str {
    match error {
        DynamicError::Discovery(error) => discovery_reason(error),
        DynamicError::Tool(error)
        | DynamicError::InstallFailed { error, .. }
        | DynamicError::InstalledEvidenceFailed { error, .. } => {
            if matches!(error, dlpad_engine::tools::ToolError::Cancelled) {
                c::TOOL_CANCELLED
            } else {
                inspection_reason(error)
            }
        }
        _ => c::TOOL_UPDATE_UNVERIFIED,
    }
}

fn discovery_reason(error: &DiscoveryError) -> &'static str {
    match error {
        DiscoveryError::Cancelled => c::TOOL_CANCELLED,
        DiscoveryError::Transport | DiscoveryError::Http(_) | DiscoveryError::TimedOut => {
            c::TOOL_UPDATE_OFFLINE
        }
        _ => c::TOOL_UPDATE_UNVERIFIED,
    }
}
fn flow_reason(error: FlowError) -> &'static str {
    match error {
        FlowError::Cancelled => c::TOOL_CANCELLED,
        FlowError::Tool(e) => inspection_reason(&e),
        _ => c::TOOL_UPDATE_UNVERIFIED,
    }
}
fn perform_update(
    data: &std::path::Path,
    tool: ToolId,
    request: UpdateRequest,
    cancel: &Arc<AtomicBool>,
    rows: &mut std::collections::BTreeMap<RowKey, RowState>,
    cache: &mut UpdateCache,
) -> UpdateCompletion {
    let (generation, kind) = request.receipt();
    let mut evidence = None;
    let result = (|| {
        inspect_checkpoint(cancel).map_err(|_| c::TOOL_CANCELLED)?;
        match request {
            UpdateRequest::Check { .. } => {
                cache[tool_slot(tool)] = None;
                if compiled_supply(tool).is_none() {
                    return Err(c::TOOL_UPDATE_UNSUPPORTED);
                }
                let (observed, pin) = if tool == ToolId::YtDlp {
                    let pin = release_discovery::capture_latest(release_tool(tool), cancel)
                        .map_err(|e| discovery_reason(&e))?;
                    (pin.observation().clone(), Some(pin))
                } else {
                    (
                        release_discovery::discover_latest(release_tool(tool), cancel)
                            .map_err(|e| discovery_reason(&e))?,
                        None,
                    )
                };
                if observed.tool != release_tool(tool) {
                    return Err(c::TOOL_UPDATE_UNVERIFIED);
                }
                inspect_checkpoint(cancel).map_err(|_| c::TOOL_CANCELLED)?;
                let captured = pin.is_some();
                cache[tool_slot(tool)] = Some((generation, observed.clone(), pin));
                Ok(UpdateValue::Observed(observed, captured))
            }
            UpdateRequest::Install {
                observation_generation,
                version,
                ..
            } => {
                let Some((cached_generation, observed, pin)) = &cache[tool_slot(tool)] else {
                    return Err(c::TOOL_UPDATE_UNVERIFIED);
                };
                let allowed = if tool == ToolId::YtDlp {
                    pin.is_some() && captured_observation_matches(tool, observed, &version)
                } else {
                    exact_approved_observation(tool, observed, &version)
                };
                if *cached_generation != observation_generation || !allowed {
                    return Err(c::TOOL_UPDATE_UNVERIFIED);
                }
                let mut manager = ToolManager::new(&data.join("managed-tools"))
                    .map_err(|e| inspection_reason(&e))?;
                if tool == ToolId::YtDlp {
                    let reservation = crate::tool_update_receipts::reserve(data, generation)
                        .map_err(|error| match error {
                            crate::tool_update_receipts::Error::Full => c::TOOL_UPDATE_RECEIPT_FULL,
                            _ => c::TOOL_UPDATE_RECEIPT_UNAVAILABLE,
                        })?;
                    let (_, _, pin) = cache[tool_slot(tool)]
                        .take()
                        .ok_or(c::TOOL_UPDATE_UNVERIFIED)?;
                    let pin = pin.ok_or(c::TOOL_UPDATE_UNVERIFIED)?;
                    let prepared =
                        dynamic_ytdlp::prepare_ytdlp(pin, reservation.leaf(), &manager, cancel)
                            .map_err(|e| dynamic_reason(&e))?;
                    if prepared.receipt_directory() != reservation.leaf()
                        || reservation.check().is_err()
                    {
                        return Err(c::TOOL_UPDATE_RECEIPT_UNAVAILABLE);
                    }
                    evidence = Some(RetainedEvidence {
                        directory: reservation.leaf().to_owned(),
                        state: EvidenceState::Prepared,
                        installed: None,
                    });
                    match dynamic_ytdlp::install_prepared_ytdlp(prepared, &mut manager, cancel) {
                        Ok(done) => {
                            let retained = reservation.retained(&done.receipt_directory);
                            // Keep the already-published inactive artifact even if path validation fails.
                            evidence = Some(RetainedEvidence {
                                directory: reservation.leaf().to_owned(),
                                state: if retained {
                                    EvidenceState::InstalledInactive
                                } else {
                                    EvidenceState::InstalledIncomplete
                                },
                                installed: Some(done.installed),
                            });
                            if !retained {
                                return Err(c::TOOL_UPDATE_RECEIPT_UNAVAILABLE);
                            }
                        }
                        Err(error) => {
                            match &error {
                                DynamicError::InstallFailed {
                                    receipt_directory, ..
                                } if reservation.retained(receipt_directory) => {
                                    evidence = Some(RetainedEvidence {
                                        directory: receipt_directory.clone(),
                                        state: EvidenceState::InstallFailed,
                                        installed: None,
                                    });
                                }
                                DynamicError::InstalledEvidenceFailed { installed, .. } => {
                                    // Display only the original owned leaf; never a foreign error path.
                                    evidence = Some(RetainedEvidence {
                                        directory: reservation.leaf().to_owned(),
                                        state: EvidenceState::InstalledIncomplete,
                                        installed: Some((**installed).clone()),
                                    });
                                }
                                _ => (),
                            }
                            return Err(dynamic_reason(&error));
                        }
                    }
                } else {
                    let supply = compiled_supply(tool).ok_or(c::TOOL_UPDATE_UNSUPPORTED)?;
                    let catalog = Catalog::from_compiled(vec![supply]).map_err(flow_reason)?;
                    version_actions::install_approved(
                        &mut manager,
                        &catalog,
                        release_tool(tool),
                        &version.version,
                        cancel,
                    )
                    .map_err(flow_reason)?;
                }
                // Fresh inventory with NO automatic ticket: explicit update installation is inactive.
                let outcome =
                    perform_worker(data, tool, Action::Refresh, cancel, [None; 3], rows, cache)
                        .map_err(|e| reason_for_message(&e))?;
                let Outcome::Inventory(inventory) = outcome else {
                    return Err(c::TOOL_UPDATE_UNVERIFIED);
                };
                Ok(UpdateValue::Inventory(inventory))
            }
            UpdateRequest::Use { manifest, .. } => {
                if manifest.tool != tool.id() {
                    return Err(c::TOOL_UPDATE_UNVERIFIED);
                }
                let mut manager = existing_managed_manager(data)
                    .map_err(|e| reason_for_message(&e))?
                    .ok_or(c::TOOL_EMPTY)?;
                let selected = version_actions::rollback_verified(&mut manager, &manifest, cancel)
                    .map_err(flow_reason)?;
                inspect_checkpoint(cancel).map_err(|_| c::TOOL_CANCELLED)?;
                let outcome =
                    perform_worker(data, tool, Action::Refresh, cancel, [None; 3], rows, cache)
                        .map_err(|e| reason_for_message(&e))?;
                let Outcome::Inventory(inventory) = outcome else {
                    return Err(c::TOOL_UPDATE_UNVERIFIED);
                };
                Ok(UpdateValue::Selected {
                    inventory,
                    path: selected.executable,
                })
            }
        }
    })();
    UpdateCompletion {
        tool,
        generation,
        kind,
        evidence,
        result,
    }
}

fn perform_worker(
    data: &std::path::Path,
    tool: ToolId,
    action: Action,
    cancel: &Arc<AtomicBool>,
    automatic_tickets: [Option<ManagedAutoTicket>; 3],
    rows: &mut std::collections::BTreeMap<RowKey, RowState>,
    update_cache: &mut UpdateCache,
) -> Result<Outcome, String> {
    let action = match action {
        Action::Update(request) => {
            return Ok(Outcome::Update(Box::new(perform_update(
                data,
                tool,
                request,
                cancel,
                rows,
                update_cache,
            ))));
        }
        other => other,
    };
    inspect_checkpoint(cancel).map_err(tool_error)?;
    let install_finished = matches!(&action, Action::Install);
    if matches!(&action, Action::Refresh) {
        rows.clear();
    }
    if matches!(&action, Action::Use) {
        let mut manager =
            existing_managed_manager(data)?.ok_or_else(|| c::TOOL_EMPTY.to_owned())?;
        let verified = inspect_automatic_tool(&mut manager, tool, cancel)
            .map_err(tool_error)?
            .ok_or_else(|| c::TOOL_EMPTY.to_owned())?;
        let selected =
            activate_inspected(&mut manager, tool, verified, cancel).map_err(tool_error)?;
        inspect_checkpoint(cancel).map_err(tool_error)?;
        return Ok(Outcome::Selected {
            tool,
            path: selected.executable,
        });
    }
    let requested = if let Action::Select(manifest) = &action {
        Some(manifest.clone())
    } else {
        None
    };
    let mut explicit_selected = None;
    let result = if let Some(manifest) = &requested {
        let key = row_key(manifest);
        let mut manager =
            existing_managed_manager(data)?.ok_or_else(|| c::TOOL_FAILED.to_owned())?;
        match inspect_recorded_tool(&mut manager, tool, manifest, cancel)
            .and_then(|verified| activate_inspected(&mut manager, tool, verified, cancel))
        {
            Ok(activated) => {
                rows.insert(key, RowState::Verified);
                explicit_selected = Some((tool, activated.executable));
            }
            Err(error) if inspect_terminal(&error) => return Err(tool_error(error)),
            Err(error) => {
                rows.insert(key, RowState::Rejected(inspection_reason(&error)));
            }
        }
        perform(data, tool, Action::Refresh, cancel)?
    } else {
        perform(data, tool, action, cancel)?
    };
    let Outcome::Inventory(mut inventory) = result else {
        return Ok(result);
    };
    let mut inspection = inspect_managed(data, cancel)?;
    // Cache only currently displayed tool entities; never128 version probes on startup.
    rows.retain(|key, _| {
        inventory
            .installed
            .iter()
            .any(|item| row_key(&item.manifest) == *key)
    });
    if inventory.installed.len() > 128 {
        return Err(c::TOOL_INVENTORY_LIMIT.to_owned());
    }
    let pinned = current_pinned_key(tool);
    for item in &inventory.installed {
        let key = row_key(&item.manifest);
        rows.entry(key.clone()).or_default();
        let available = matches!(&inspection.slots[tool_slot(tool)], Availability::Available(path) if *path == item.executable);
        if available && !matches!(rows.get(&key), Some(RowState::Rejected(_))) {
            rows.insert(key, RowState::Verified);
        } else if !available
            && (pinned.as_ref() == Some(&key)
                || inventory
                    .active
                    .as_ref()
                    .is_some_and(|active| row_key(&active.manifest) == key))
        {
            let reason =
                if let Availability::Unavailable(reason) = &inspection.slots[tool_slot(tool)] {
                    *reason
                } else {
                    c::TOOL_AUTO_UNAVAILABLE
                };
            rows.insert(key, RowState::Rejected(reason));
        }
    }
    if let Some(ticket) = automatic_tickets[tool_slot(tool)].filter(|_| install_finished) {
        let stored = crate::accessible_tool_preferences::automatic_modes_for_managed_restore(data)
            .map_err(|_| c::TOOL_PREF_LOAD_FAILED.to_owned())?;
        let environment_key = match tool {
            ToolId::YtDlp => "DLPAD_YTDLP",
            ToolId::Ffmpeg => "DLPAD_FFMPEG",
            ToolId::Deno => "DLPAD_DENO",
        };
        if stored[tool_slot(tool)] && std::env::var_os(environment_key).is_none() {
            // Installation selects the newly installed current pinned entity, not
            // the previous active-first startup candidate.
            let pinned = current_pinned_key(tool).ok_or_else(|| c::TOOL_UNSUPPORTED.to_owned())?;
            let installed = inventory
                .installed
                .iter()
                .find(|item| row_key(&item.manifest) == pinned)
                .ok_or_else(|| c::TOOL_FAILED.to_owned())?;
            let mut manager =
                existing_managed_manager(data)?.ok_or_else(|| c::TOOL_FAILED.to_owned())?;
            let verified = inspect_recorded_tool(&mut manager, tool, &installed.manifest, cancel)
                .map_err(tool_error)?;
            let activated =
                activate_inspected(&mut manager, tool, verified, cancel).map_err(tool_error)?;
            inspect_checkpoint(cancel).map_err(tool_error)?;
            rows.insert(row_key(&activated.manifest), RowState::Verified);
            inspection.slots[tool_slot(tool)] =
                Availability::Available(activated.executable.clone());
            inventory.active = Some(activated.clone());
            inventory.auto_selected = Some((tool, activated.executable, ticket));
        }
    }
    inventory.explicit_selected = explicit_selected;
    inventory.rows = rows.clone();
    inventory.inspection = Some(inspection);
    Ok(Outcome::Inventory(inventory))
}

fn verified_selection(
    manager: &mut ToolManager,
    tool: ToolId,
    manifest: &InstallManifest,
    cancel: &Arc<AtomicBool>,
) -> Result<InstalledTool, String> {
    if manifest.tool != tool.id() {
        return Err(c::TOOL_FAILED.into());
    }
    match tool {
        ToolId::YtDlp => manager
            .select_verified_existing(manifest, cancel)
            .map_err(tool_error),
        ToolId::Deno | ToolId::Ffmpeg => {
            let package = manager
                .recorded_package_for_manifest(manifest)
                .map_err(tool_error)?
                .ok_or_else(|| c::TOOL_FAILED.to_owned())?;
            let verified = match tool {
                ToolId::Deno => manager.select_verified_deno(&package.package, cancel),
                ToolId::Ffmpeg => manager.select_verified_ffmpeg(&package.package, cancel),
                ToolId::YtDlp => unreachable!(),
            }
            .map_err(tool_error)?;
            Ok(verified.main)
        }
    }
}
/// Recovery-worker compatibility wrapper; never overwrite unrequested custom/env fields.
pub(crate) fn restore_active(
    data: &std::path::Path,
    requested: &[(ToolId, bool); 3],
    cancel: &Arc<AtomicBool>,
) -> Result<Vec<(ToolId, PathBuf)>, String> {
    inspect_checkpoint(cancel).map_err(tool_error)?;
    if !requested.iter().any(|(_, restore)| *restore) {
        return Ok(Vec::new());
    }
    let Some(mut manager) = existing_managed_manager(data)? else {
        return Ok(Vec::new());
    };
    let mut restored = Vec::with_capacity(3);
    for &(tool, restore) in requested {
        inspect_checkpoint(cancel).map_err(tool_error)?;
        if !restore {
            continue;
        }
        match inspect_automatic_tool(&mut manager, tool, cancel).and_then(|item| {
            item.map(|verified| activate_inspected(&mut manager, tool, verified, cancel))
                .transpose()
        }) {
            Ok(Some(verified)) => restored.push((tool, verified.executable)),
            Ok(None) => (),
            Err(error) if inspect_terminal(&error) => return Err(tool_error(error)),
            Err(_) => (), // Caller receives all3 fixed unavailable reasons via inspect_managed.
        }
        if existing_managed_manager(data)?.is_none() {
            return Err(c::TOOL_FAILED.to_owned());
        }
    }
    inspect_checkpoint(cancel).map_err(tool_error)?;
    Ok(restored)
}

fn perform(
    data: &std::path::Path,
    tool: ToolId,
    action: Action,
    cancel: &Arc<AtomicBool>,
) -> Result<Outcome, String> {
    let mut manager = ToolManager::new(&data.join("managed-tools")).map_err(tool_error)?;
    if matches!(action, Action::Use) {
        let mut active = if tool == ToolId::YtDlp {
            manager.active_installed(tool.id()).map_err(tool_error)?
        } else {
            let selected = manager.active(tool.id()).map_err(tool_error)?;
            manager
                .installed(tool.id())
                .map_err(tool_error)?
                .into_iter()
                .find(|item| {
                    selected.as_ref().is_some_and(|(version, target)| {
                        item.manifest.version == *version && item.manifest.target == *target
                    })
                })
        }
        .ok_or_else(|| c::TOOL_EMPTY.to_owned())?;
        if tool != ToolId::YtDlp {
            active = verified_selection(&mut manager, tool, &active.manifest, cancel)?;
        }
        return Ok(Outcome::Selected {
            tool,
            path: active.executable,
        });
    }
    let recovery = manager.recover_staging().map_err(tool_error)?;
    match action {
        Action::Update(_) => return Err(c::TOOL_UPDATE_UNVERIFIED.into()),
        Action::Refresh => {}
        Action::Install => {
            if tool == ToolId::YtDlp {
                let (manifest, bytes) = candidate()?;
                manager
                    .download_and_install_inactive(manifest, bytes, cancel)
                    .map_err(tool_error)?;
            } else {
                let package = crate::managed_candidates::archive_candidate(tool)
                    .ok_or_else(|| c::TOOL_UNSUPPORTED.to_owned())?;
                manager
                    .download_archive_and_install(package, cancel)
                    .map_err(tool_error)?;
            }
        }
        Action::Select(manifest) => {
            verified_selection(&mut manager, tool, &manifest, cancel)?;
        }
        Action::Remove(manifest) => {
            if manifest.tool != tool.id() {
                return Err(c::TOOL_FAILED.into());
            }
            manager.remove(&manifest).map_err(tool_error)?;
        }
        Action::Use => unreachable!(),
    }
    let installed = manager.installed(tool.id()).map_err(tool_error)?;
    let selected = manager.active(tool.id()).map_err(tool_error)?;
    let active = installed
        .iter()
        .find(|item| {
            selected.as_ref().is_some_and(|(version, target)| {
                item.manifest.version == *version && item.manifest.target == *target
            })
        })
        .cloned();
    Ok(Outcome::Inventory(Box::new(Inventory {
        installed,
        active,
        recovery,
        inspection: None,
        auto_selected: None,
        explicit_selected: None,
        rows: Default::default(),
    })))
}
fn supervise(signals: Arc<Signals>, context: egui::Context, run: impl FnOnce()) {
    let _ack = Ack {
        signals: signals.clone(),
        context,
    };
    if std::panic::catch_unwind(std::panic::AssertUnwindSafe(run)).is_err() {
        signals.failed.store(true, Ordering::Release);
    }
}
impl Panel {
    pub(crate) fn new(data: PathBuf) -> Self {
        let (commands, requests) = mpsc::sync_channel::<Command>(1);
        let (responses, results) = mpsc::sync_channel(1);
        let signals = Arc::new(Signals::default());
        let worker = signals.clone();
        let context = egui::Context::default();
        let wake = context.clone();
        let spawn = thread::Builder::new()
            .name("dlpad-tools".into())
            .spawn(move || {
                supervise(worker.clone(), wake, || {
                    let mut rows = std::collections::BTreeMap::new();
                    let mut update_cache: UpdateCache = std::array::from_fn(|_| None);
                    loop {
                        if worker.stop.load(Ordering::Acquire) {
                            break;
                        }
                        let command = match requests.recv_timeout(Duration::from_millis(50)) {
                            Ok(command) => command,
                            Err(mpsc::RecvTimeoutError::Timeout) => continue,
                            Err(_) => break,
                        };
                        if worker.stop.load(Ordering::Acquire) {
                            break;
                        }
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            perform_worker(
                                &data,
                                command.tool,
                                command.action,
                                &worker.cancel,
                                command.automatic_tickets,
                                &mut rows,
                                &mut update_cache,
                            )
                        }))
                        .unwrap_or_else(|_| Err(c::TOOL_FAILED.into()));
                        if responses
                            .try_send(Ok(Outcome::Tagged {
                                tool: command.tool,
                                request_seq: command.request_seq,
                                result: Box::new(result),
                            }))
                            .is_err()
                        {
                            worker.failed.store(true, Ordering::Release);
                            break;
                        }
                        command.context.request_repaint();
                    }
                });
            });
        if spawn.is_err() {
            signals.failed.store(true, Ordering::Release);
            signals.stopped.store(true, Ordering::Release);
        }
        Self {
            updates: Default::default(),
            commands,
            results,
            signals,
            inventory: None,
            cached: std::array::from_fn(|_| None),
            display_state: [DisplayState::Empty; 3],
            errors: std::array::from_fn(|_| None),
            request_seq: 0,
            pending_dispatch: None,
            background_work: None,
            initial_refresh_pending: true,
            error: None,
            selected: None,
            auto_selected: None,
            inspection: None,
            automatic_tickets: [None; 3],
            pending_row: None,
            tool: ToolId::default(),
            busy: false,
            context,
        }
    }

    fn inventory_for(&self, tool: ToolId) -> Option<&Inventory> {
        if tool == self.tool {
            self.inventory.as_ref()
        } else {
            self.cached[tool_slot(tool)].as_ref()
        }
    }
    fn update_matches(&self, outer: ToolId, completion: &UpdateCompletion) -> bool {
        if completion.tool != outer || self.updates.pending != Some((outer, completion.generation))
        {
            return false;
        }
        match (&completion.kind, &completion.result) {
            (UpdateKind::Check, Ok(UpdateValue::Observed(o, _))) => o.tool == release_tool(outer),
            (UpdateKind::Install, Ok(UpdateValue::Inventory(i)))
            | (UpdateKind::Use, Ok(UpdateValue::Selected { inventory: i, .. })) => {
                i.installed.len() <= 128
                    && i.installed.iter().all(|v| v.manifest.tool == outer.id())
                    && i.active
                        .as_ref()
                        .is_none_or(|v| v.manifest.tool == outer.id())
            }
            (_, Err(_)) => true,
            _ => false,
        }
    }
    fn replace_update_inventory(&mut self, tool: ToolId, mut inventory: Inventory) {
        self.updates.render_dirty = true;
        // Dynamic install/use never turns a display cache into automatic selection authority.
        inventory.auto_selected = None;
        inventory.explicit_selected = None;
        self.display_state[tool_slot(tool)] = DisplayState::Fresh;
        self.errors[tool_slot(tool)] = None;
        if tool == self.tool {
            self.inventory = Some(inventory);
            self.error = None;
        } else {
            self.cached[tool_slot(tool)] = Some(inventory);
        }
        self.pending_row = None;
    }
    /// Display classification only; never substitutes for controller tickets/provenance.
    /// Slot order is existing preference order [yt-dlp, FFmpeg, Deno].
    pub(crate) fn set_update_current_sources(&mut self, sources: [CurrentSource; 3]) {
        if self.updates.current_sources != sources {
            self.updates.current_sources = sources;
            self.updates.render_dirty = true;
        }
    }
    fn submit_update(&mut self, intent: update_ui::Intent, enabled: bool) {
        if !enabled || self.busy || self.signals.stop.load(Ordering::Acquire) || self.stopped() {
            return;
        }
        let (tool, snapshot_generation) = match &intent {
            update_ui::Intent::Check { tool, generation }
            | update_ui::Intent::InstallApprovedObserved {
                tool, generation, ..
            }
            | update_ui::Intent::UseInstalled {
                tool, generation, ..
            } => (*tool, *generation),
        };
        if tool != self.tool
            || snapshot_generation != self.updates.snapshot_generation[tool_slot(tool)]
        {
            return;
        }
        let Some(generation) = self.updates.serial.checked_add(1) else {
            self.error = Some(c::TOOL_FAILED.into());
            return;
        };
        let Some(request_seq) = self.request_seq.checked_add(1) else {
            self.error = Some(c::TOOL_FAILED.into());
            return;
        };
        let request = match intent {
            update_ui::Intent::Check { .. } => UpdateRequest::Check { generation },
            update_ui::Intent::InstallApprovedObserved { version, .. } => {
                let Some(observation_generation) =
                    self.updates.observed_generation[tool_slot(tool)]
                else {
                    return;
                };
                if !self.updates.observed[tool_slot(tool)]
                    .as_ref()
                    .is_some_and(|o| {
                        if tool == ToolId::YtDlp {
                            self.updates.captured[tool_slot(tool)]
                                && captured_observation_matches(tool, o, &version)
                        } else {
                            exact_approved_observation(tool, o, &version)
                        }
                    })
                {
                    return;
                }
                UpdateRequest::Install {
                    generation,
                    observation_generation,
                    version,
                }
            }
            update_ui::Intent::UseInstalled { manifest, .. } => {
                if !self.inventory.as_ref().is_some_and(|i| {
                    i.installed.iter().any(|row| {
                        serde_json::to_vec(&row.manifest)
                            .is_ok_and(|a| serde_json::to_vec(&manifest).is_ok_and(|b| a == b))
                    })
                }) {
                    return;
                }
                if self
                    .inventory
                    .as_ref()
                    .and_then(|i| i.rows.get(&row_key(&manifest)))
                    .is_some_and(|r| matches!(r, RowState::Rejected(_) | RowState::Checking))
                {
                    return;
                }
                UpdateRequest::Use {
                    generation,
                    manifest,
                }
            }
        };
        let (_, kind) = request.receipt();
        self.signals.cancel.store(false, Ordering::Release);
        if self
            .commands
            .try_send(Command {
                tool,
                action: Action::Update(request),
                context: self.context.clone(),
                automatic_tickets: [None; 3],
                request_seq,
            })
            .is_err()
        {
            self.error = Some(c::TOOL_FAILED.into());
            return;
        }
        self.updates.render_dirty = true;
        self.request_seq = request_seq;
        self.pending_dispatch = Some((tool, request_seq));
        self.display_state[tool_slot(tool)] = DisplayState::Refreshing;
        self.background_work = Some(match kind {
            UpdateKind::Check => BackgroundWork::VersionCheck,
            UpdateKind::Install => BackgroundWork::Install,
            UpdateKind::Use => BackgroundWork::Verify,
        });
        self.updates.serial = generation;
        self.updates.pending = Some((tool, generation));
        self.updates.snapshot_generation[tool_slot(tool)] = generation;
        self.updates.status[tool_slot(tool)] = match kind {
            UpdateKind::Check => update_ui::Status::Checking,
            UpdateKind::Install => update_ui::Status::Installing,
            UpdateKind::Use => update_ui::Status::Selecting,
        };
        if matches!(kind, UpdateKind::Install) && tool == ToolId::YtDlp {
            // A captured pin is one-shot; a new check is required after any attempted install.
            self.updates.captured[tool_slot(tool)] = false;
        }
        if matches!(kind, UpdateKind::Check) {
            self.updates.captured[tool_slot(tool)] = false;
            self.updates.observed[tool_slot(tool)] = None;
            self.updates.observed_generation[tool_slot(tool)] = None;
        }
        self.busy = true;
        self.error = None;
    }
    fn accept_update(&mut self, completion: UpdateCompletion) {
        if self.signals.stop.load(Ordering::Acquire)
            || self.stopped()
            || !self.update_matches(completion.tool, &completion)
        {
            return;
        }
        self.updates.render_dirty = true;
        self.busy = false;
        self.background_work = None;
        self.updates.pending = None;
        let index = tool_slot(completion.tool);
        self.updates.snapshot_generation[index] = completion.generation;
        if let Some(evidence) = completion.evidence {
            self.updates.evidence[index] = Some(evidence);
        }
        match completion.result {
            Ok(UpdateValue::Observed(observed, captured)) => {
                // Release discovery is not an inventory refresh or fresh runtime proof.
                self.display_state[index] = if self.inventory_for(completion.tool).is_some() {
                    DisplayState::Stale
                } else {
                    DisplayState::Empty
                };
                self.updates.captured[index] = captured;
                self.updates.observed_generation[index] = Some(completion.generation);
                self.updates.status[index] = if (completion.tool == ToolId::YtDlp && captured)
                    || current_pinned_key(completion.tool)
                        .is_some_and(|(_, v, _)| v == observed.tag)
                {
                    update_ui::Status::Idle
                } else {
                    update_ui::Status::Unverified
                };
                self.updates.observed[index] = Some(observed);
                self.errors[index] = None;
                if completion.tool == self.tool {
                    self.error = None;
                }
            }
            Ok(UpdateValue::Inventory(inventory)) => {
                // Installation is inactive: never publish automatic or explicit field selection.
                self.inspection = inventory.inspection.clone();
                self.replace_update_inventory(completion.tool, *inventory);
                self.updates.status[index] = update_ui::Status::Idle;
                self.errors[index] = None;
                if completion.tool == self.tool {
                    self.error = None;
                }
            }
            Ok(UpdateValue::Selected { inventory, path }) => {
                self.inspection = inventory.inspection.clone();
                self.replace_update_inventory(completion.tool, *inventory);
                // Existing explicit Use semantics; Main owns busy/CloseWait/manual-field routing.
                self.selected = Some((completion.tool, path));
                self.updates.status[index] = update_ui::Status::Idle;
                self.errors[index] = None;
                if completion.tool == self.tool {
                    self.error = None;
                }
            }
            Err(reason) => {
                self.updates.status[index] = if reason == c::TOOL_UPDATE_OFFLINE {
                    update_ui::Status::Offline
                } else if reason == c::TOOL_UPDATE_UNSUPPORTED {
                    update_ui::Status::Unsupported
                } else if reason == c::TOOL_UPDATE_UNVERIFIED {
                    update_ui::Status::Unverified
                } else {
                    update_ui::Status::Failed(reason)
                };
                self.failure_for(completion.tool, reason.into());
            }
        }
    }
    fn update_snapshots(&self) -> [update_ui::ToolSnapshot; 3] {
        [ToolId::YtDlp, ToolId::Deno, ToolId::Ffmpeg].map(|tool| {
            let index = tool_slot(tool);
            let current_inventory = self.inventory_for(tool);
            let inventory = current_inventory.filter(|_| self.tool == tool);
            let current = match self.updates.current_sources[index] {
                CurrentSource::Unknown => update_ui::CurrentVersion::Unknown,
                CurrentSource::External => update_ui::CurrentVersion::External,
                CurrentSource::Managed => current_inventory.and_then(|i| i.active.as_ref()).map_or(
                    update_ui::CurrentVersion::Unknown,
                    |v| {
                        update_ui::CurrentVersion::Managed(update_ui::VersionKey {
                            version: v.manifest.version.clone(),
                            target: v.manifest.target.clone(),
                        })
                    },
                ),
            };
            let observed = self.updates.observed[index].as_ref().map(|o| {
                let target = if tool == ToolId::YtDlp {
                    "x86_64-pc-windows-msvc".to_owned()
                } else {
                    current_pinned_key(tool)
                        .map_or_else(|| "unavailable".to_owned(), |(_, _, target)| target)
                };
                let key = update_ui::VersionKey {
                    version: o.tag.clone(),
                    target,
                };
                let allowed = if tool == ToolId::YtDlp {
                    self.updates.captured[index] && captured_observation_matches(tool, o, &key)
                } else {
                    exact_approved_observation(tool, o, &key)
                };
                let install = if allowed {
                    update_ui::Eligibility::Ready
                } else {
                    update_ui::Eligibility::Unavailable(c::TOOL_UPDATE_UNVERIFIED)
                };
                update_ui::ObservedVersion { key, install }
            });
            let installed = inventory.map_or_else(Vec::new, |i| {
                i.installed
                    .iter()
                    .map(|v| {
                        let select = match i.rows.get(&row_key(&v.manifest)) {
                            Some(RowState::Rejected(reason)) => {
                                update_ui::Eligibility::Unavailable(reason)
                            }
                            Some(RowState::Checking) => update_ui::Eligibility::Checking,
                            _ => update_ui::Eligibility::Ready,
                        };
                        update_ui::InstalledVersion {
                            manifest: v.manifest.clone(),
                            select,
                        }
                    })
                    .collect()
            });
            let status = if compiled_supply(tool).is_none() {
                update_ui::Status::Unsupported
            } else {
                self.updates.status[index].clone()
            };
            update_ui::ToolSnapshot {
                tool,
                generation: self.updates.snapshot_generation[index],
                current,
                observed,
                installed,
                status,
            }
        })
    }
    fn submit(&mut self, action: Action, enabled: bool) {
        if matches!(&action, Action::Update(_))
            || (matches!(&action, Action::Install)
                && pinned_record_present(self.tool, self.inventory.as_ref()))
        {
            return;
        }
        let background_work = match &action {
            Action::Refresh => BackgroundWork::Lookup,
            Action::Install => BackgroundWork::Install,
            Action::Select(_) => BackgroundWork::Verify,
            Action::Remove(_) => BackgroundWork::Remove,
            Action::Use => BackgroundWork::Select,
            Action::Update(_) => return,
        };
        let checking = if let Action::Select(manifest) = &action {
            Some(row_key(manifest))
        } else {
            None
        };
        if !enabled || self.busy || self.signals.stop.load(Ordering::Acquire) || self.stopped() {
            return;
        }
        let Some(request_seq) = self.request_seq.checked_add(1) else {
            self.error = Some(c::TOOL_FAILED.into());
            return;
        };
        self.signals.cancel.store(false, Ordering::Release);
        if self
            .commands
            .try_send(Command {
                action,
                tool: self.tool,
                context: self.context.clone(),
                automatic_tickets: self.automatic_tickets,
                request_seq,
            })
            .is_ok()
        {
            self.updates.render_dirty = true;
            self.request_seq = request_seq;
            self.pending_dispatch = Some((self.tool, request_seq));
            self.display_state[tool_slot(self.tool)] = DisplayState::Refreshing;
            self.busy = true;
            self.background_work = Some(background_work);
            self.error = None;
            self.pending_row = checking.clone();
            if let (Some(key), Some(inventory)) = (checking, self.inventory.as_mut()) {
                inventory.rows.insert(key, RowState::Checking);
            }
        } else {
            self.error = Some(c::TOOL_FAILED.into());
        }
    }
    fn inventory_mut_for(&mut self, tool: ToolId) -> Option<&mut Inventory> {
        if tool == self.tool {
            self.inventory.as_mut()
        } else {
            self.cached[tool_slot(tool)].as_mut()
        }
    }
    fn failure_for(&mut self, tool: ToolId, error: String) {
        if self
            .updates
            .pending
            .is_some_and(|(origin, _)| origin == tool)
        {
            self.updates.pending = None;
            self.updates.status[tool_slot(tool)] =
                update_ui::Status::Failed(reason_for_message(&error));
        }
        if let Some(key) = self.pending_row.take()
            && let Some(inventory) = self.inventory_mut_for(tool)
        {
            inventory
                .rows
                .insert(key, RowState::Rejected(reason_for_message(&error)));
        }
        self.display_state[tool_slot(tool)] = DisplayState::Failed;
        self.errors[tool_slot(tool)] = Some(error.clone());
        if tool == self.tool {
            self.error = Some(error);
        }
    }
    /// Navigate only; existing show/worker trust gates still own all I/O and actions.
    pub(crate) fn select_tool(&mut self, tool: ToolId) {
        if !self.signals.stop.load(Ordering::Acquire) && !self.stopped() {
            self.choose_tool(tool);
        }
    }
    pub(crate) fn selected_tool(&self) -> ToolId {
        self.tool
    }
    fn choose_tool(&mut self, tool: ToolId) {
        if tool == self.tool {
            return;
        }
        self.updates.render_dirty = true;
        let previous = tool_slot(self.tool);
        self.cached[previous] = self.inventory.take();
        self.errors[previous] = self.error.take();
        if self.display_state[previous] == DisplayState::Fresh {
            self.display_state[previous] = DisplayState::Stale;
        }
        self.tool = tool;
        let current = tool_slot(tool);
        self.inventory = self.cached[current].take();
        self.error = self.errors[current].clone();
        if self.display_state[current] == DisplayState::Fresh {
            self.display_state[current] = DisplayState::Stale;
        }
        // Returning to a session cache does no I/O. Explicit actions still obtain fresh worker proof.
        // First visits wait for the existing origin command before requesting their inventory.
        self.initial_refresh_pending = self.inventory.is_none();
    }
    pub(crate) fn poll(&mut self) {
        let received = match self.results.try_recv() {
            Ok(result) => Some(result),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => {
                if let Some((tool, _)) = self.pending_dispatch.take() {
                    self.busy = false;
                    self.background_work = None;
                    self.failure_for(tool, c::TOOL_FAILED.into());
                }
                None
            }
        };
        if let Some(received) = received {
            let completion = match received {
                Ok(Outcome::Tagged {
                    tool,
                    request_seq,
                    result,
                }) => {
                    if self.pending_dispatch == Some((tool, request_seq)) {
                        if matches!(&*result,Ok(Outcome::Update(completion)) if !self.update_matches(tool,completion))
                        {
                            None
                        } else {
                            self.pending_dispatch = None;
                            Some((tool, *result))
                        }
                    } else {
                        None
                    } // stale outcome never clears another pending request
                }
                untagged => {
                    // Old unit fixtures inject raw acknowledgements. Production worker
                    // always sends the exact origin/request envelope above.
                    #[cfg(test)]
                    {
                        let origin = self.pending_dispatch.map_or(self.tool, |(tool, _)| tool);
                        if matches!(&untagged,Ok(Outcome::Update(completion)) if !self.update_matches(origin,completion))
                        {
                            None
                        } else {
                            Some((
                                self.pending_dispatch
                                    .take()
                                    .map_or(self.tool, |(tool, _)| tool),
                                untagged,
                            ))
                        }
                    }
                    #[cfg(not(test))]
                    {
                        let _ = untagged;
                        None
                    }
                }
            };
            if let Some((tool, result)) = completion {
                self.updates.render_dirty = true;
                self.busy = false;
                self.background_work = None;
                let publish = !self.signals.stop.load(Ordering::Acquire) && !self.stopped();
                match result {
                    Ok(Outcome::Update(completion)) => self.accept_update(*completion),
                    Ok(Outcome::Inventory(mut inventory)) => {
                        if inventory.installed.len() > 128
                            || inventory
                                .installed
                                .iter()
                                .any(|item| item.manifest.tool != tool.id())
                            || inventory
                                .active
                                .as_ref()
                                .is_some_and(|item| item.manifest.tool != tool.id())
                        {
                            self.failure_for(tool, c::TOOL_INVENTORY_LIMIT.into());
                        } else {
                            if publish {
                                self.inspection = inventory.inspection.clone();
                                if let Some((origin, path, ticket)) = inventory.auto_selected.take()
                                    && origin == tool
                                {
                                    self.auto_selected = Some((origin, path, ticket));
                                }
                                if let Some((origin, path)) = inventory.explicit_selected.take()
                                    && origin == tool
                                {
                                    self.selected = Some((origin, path));
                                }
                            }
                            self.display_state[tool_slot(tool)] = DisplayState::Fresh;
                            self.errors[tool_slot(tool)] = None;
                            if tool == self.tool {
                                self.inventory = Some(*inventory);
                                self.error = None;
                            } else {
                                self.cached[tool_slot(tool)] = Some(*inventory);
                            }
                            self.pending_row = None;
                        }
                    }
                    Ok(Outcome::Selected { tool: origin, path }) if origin == tool => {
                        if publish {
                            self.selected = Some((origin, path));
                        }
                        self.errors[tool_slot(tool)] = None;
                        if tool == self.tool {
                            self.error = None;
                        }
                        if self.inventory_mut_for(tool).is_some() {
                            self.display_state[tool_slot(tool)] = DisplayState::Stale;
                        }
                        self.pending_row = None;
                    }
                    Ok(_) => self.failure_for(tool, c::TOOL_FAILED.into()),
                    Err(error) => self.failure_for(tool, error),
                }
            }
        }
        if self.signals.failed.load(Ordering::Acquire) {
            if let Some((tool, _)) = self.pending_dispatch.take() {
                self.failure_for(tool, c::TOOL_FAILED.into());
            }
            self.busy = false;
            self.background_work = None;
            self.error = Some(c::TOOL_FAILED.into());
        }
        if self.stopped() {
            self.busy = false;
            self.background_work = None;
        }
    }
    pub(crate) fn busy(&self) -> bool {
        self.busy
    }
    /// Presentation only: operation labels never alter worker lifetime or eligibility.
    pub(crate) fn background_status(&self) -> Option<&'static str> {
        self.busy.then(|| {
            self.background_work
                .map_or(c::TOOL_BACKGROUND_WORK, BackgroundWork::label)
        })
    }
    pub(crate) fn request_stop(&mut self) {
        if self.updates.pending.take().is_some() {
            self.updates.render_dirty = true;
        }
        self.signals.cancel.store(true, Ordering::Release);
        self.signals.stop.store(true, Ordering::Release);
    }
    pub(crate) fn stopped(&self) -> bool {
        self.signals.stopped.load(Ordering::Acquire)
    }
    pub(crate) fn set_automatic_tickets(&mut self, tickets: [Option<ManagedAutoTicket>; 3]) {
        self.automatic_tickets = tickets;
    }
    pub(crate) fn take_inspection(&mut self) -> Option<ManagedInspection> {
        self.inspection.take()
    }
    pub(crate) fn take_auto_selected_path(
        &mut self,
    ) -> Option<(ToolId, PathBuf, ManagedAutoTicket)> {
        self.auto_selected.take()
    }
    pub(crate) fn take_selected_path(&mut self) -> Option<(ToolId, PathBuf)> {
        self.selected.take()
    }
    pub(crate) fn show(&mut self, ui: &mut egui::Ui, enabled: bool) {
        self.context = ui.ctx().clone();
        self.poll();
        let navigation_enabled =
            enabled && !self.signals.stop.load(Ordering::Acquire) && !self.stopped();
        let enabled = navigation_enabled && !self.busy;
        if enabled && self.initial_refresh_pending {
            // A first visible, enabled page loads inventory using the existing worker.
            // Consume the attempt even on submission failure: only explicit Refresh retries.
            self.initial_refresh_pending = false;
            self.submit(Action::Refresh, enabled);
        }
        let enabled = enabled && !self.busy;
        let mut selected_tool = self.tool;
        ui.horizontal_wrapped(|ui| {
            // Disable the existing row without introducing a child ID seed per tab.
            if !navigation_enabled {
                ui.disable();
            }
            for tool in [ToolId::YtDlp, ToolId::Deno, ToolId::Ffmpeg] {
                if crate::ui_components::choice(ui, self.tool == tool, tool.label()).clicked() {
                    selected_tool = tool;
                }
            }
        });
        let tool_changed = selected_tool != self.tool;
        if tool_changed {
            self.choose_tool(selected_tool);
            if enabled && self.initial_refresh_pending {
                self.initial_refresh_pending = false;
                self.submit(Action::Refresh, true);
            }
        }

        let enabled = enabled && !self.busy;
        let (role, origin) = match self.tool {
            ToolId::YtDlp => (c::TOOL_YTDLP_ROLE, c::TOOL_YTDLP_ORIGIN_MANAGED),
            ToolId::Deno => (c::TOOL_DENO_ROLE, c::TOOL_DENO_ORIGIN_MANAGED),
            ToolId::Ffmpeg => (c::TOOL_FFMPEG_ROLE, c::TOOL_FFMPEG_ORIGIN_MANAGED),
        };
        let cache_caption = if self.inventory.is_some() {
            if self.initial_refresh_pending
                && self.busy
                && self
                    .pending_dispatch
                    .is_some_and(|(origin, _)| origin != self.tool)
            {
                Some(c::TOOL_CACHE_WAITING)
            } else {
                match self.display_state[tool_slot(self.tool)] {
                    DisplayState::Refreshing => Some(c::TOOL_CACHE_REFRESHING),
                    DisplayState::Failed => Some(c::TOOL_CACHE_FAILED),
                    DisplayState::Stale => Some(c::TOOL_CACHE_STALE),
                    _ => None,
                }
            }
        } else {
            None
        };
        let guide = format!(
            "{role}\n{origin}\n{}{}",
            c::TOOL_STATUS_RECORD_HELP,
            cache_caption.map_or_else(String::new, |caption| format!("\n{caption}"))
        );
        // Session cache is displayed immediately; all file actions still revalidate on the worker.
        ui.push_id("tool-status-summary", |ui| {
            let display = self.display_state[tool_slot(self.tool)];
            let status =
                if display == DisplayState::Failed || self.signals.failed.load(Ordering::Acquire) {
                    c::TOOL_STATUS_FAILED
                } else if let Some(inventory) = &self.inventory {
                    if inventory.installed.is_empty() {
                        c::TOOL_STATUS_NOT_INSTALLED
                    } else {
                        c::TOOL_STATUS_INSTALLED
                    }
                } else {
                    c::TOOL_STATUS_CHECKING
                };
            ui.horizontal_wrapped(|ui| {
                ui.label(status);
                if cache_caption.is_some() {
                    ui.weak(c::TOOL_CACHE_STATUS_SHORT);
                }
                crate::ui_components::help(
                    ui,
                    ("tool-state-guide", self.tool.id()),
                    c::TOOL_DETAILS_HELP_TITLE,
                    &guide,
                );
                if self.inventory.is_some() && display == DisplayState::Refreshing {
                    ui.weak(c::TOOL_STATUS_CHECKING);
                }
                if let Some(inventory) = &self.inventory
                    && let Some(installed) = inventory.installed.first()
                {
                    let versions = inventory
                        .installed
                        .iter()
                        .take(3)
                        .map(|row| row.manifest.version.as_str())
                        .collect::<Vec<_>>()
                        .join(" · ");
                    let label = format!("{}: {}", c::TOOL_VERSION_LABEL, versions);
                    ui.add(egui::Label::new(label).truncate())
                        .on_hover_text(&installed.manifest.version);
                }
            });
            // This is a text-only selection summary; retain the 32px help/action controls.
            ui.spacing_mut().item_spacing.y = 4.0;
            ui.spacing_mut().interact_size.y = ui.text_style_height(&egui::TextStyle::Body);
            ui.horizontal_wrapped(|ui| {
                ui.label(c::TOOL_UPDATE_CURRENT);
                match self.updates.current_sources[tool_slot(self.tool)] {
                    CurrentSource::Unknown => {
                        ui.label(c::TOOL_UPDATE_CURRENT_UNKNOWN);
                    }
                    CurrentSource::External => {
                        ui.label(c::TOOL_UPDATE_EXTERNAL);
                    }
                    CurrentSource::Managed => {
                        ui.label(
                            self.inventory
                                .as_ref()
                                .and_then(|i| i.active.as_ref())
                                .map_or(c::TOOL_UPDATE_CURRENT_UNKNOWN, |active| {
                                    active.manifest.version.as_str()
                                }),
                        );
                    }
                }
            });
        });
        let pinned_installed = pinned_record_present(self.tool, self.inventory.as_ref());
        let mut action = None;
        ui.horizontal_wrapped(|ui| {
            if crate::ui_components::secondary(ui, c::TOOL_REFRESH, enabled).clicked() {
                action = Some(Action::Refresh);
            }
            let install_label = match self.tool {
                ToolId::YtDlp => c::TOOL_INSTALL_YTDLP,
                ToolId::Deno => c::TOOL_INSTALL_DENO,
                ToolId::Ffmpeg => c::TOOL_INSTALL_FFMPEG,
            };
            let install = crate::ui_components::primary(
                ui,
                install_label,
                enabled && self.tool.install_available() && !pinned_installed,
            );
            if install.gained_focus() || (tool_changed && install.has_focus()) {
                install.scroll_to_me(None);
            }
            if install.clicked() {
                action = Some(Action::Install);
            }
            if self.busy && crate::ui_components::ghost(ui, c::TOOL_CANCEL, self.busy).clicked() {
                self.signals.cancel.store(true, Ordering::Release);
            }
        });
        if pinned_installed {
            ui.weak(c::TOOL_PINNED_ALREADY_INSTALLED);
        }
        if self.tool == ToolId::YtDlp {
            if let Ok((manifest, bytes)) = candidate() {
                ui.small(format!(
                    "{} {:.1} MiB · {} {:.1} MiB",
                    c::TOOL_DOWNLOAD_SIZE_LABEL,
                    bytes as f64 / 1048576.0,
                    c::TOOL_INSTALL_FILES_SIZE_LABEL,
                    bytes as f64 / 1048576.0
                ));
                crate::accessible::collapsing(ui, c::TOOL_DETAILS, |ui| {
                    ui.label(format!("{}: {}", c::TOOL_VERSION_LABEL, manifest.version));
                    ui.label(format!("{}: {}", c::TOOL_PLATFORM_LABEL, manifest.target));
                    ui.label(format!(
                        "{}: {}",
                        c::TOOL_SOURCE_URL_LABEL,
                        manifest.source_url
                    ));
                    ui.label(format!("{}: {}", c::TOOL_SHA256_LABEL, manifest.sha256));
                    ui.label(format!("{}: {}", c::TOOL_LICENSE_LABEL, manifest.license));
                    ui.label(c::TOOL_VERIFICATION_DETAILS);
                    ui.label(c::TOOL_DISTRIBUTION_REVIEW_STATUS);
                });
            } else {
                ui.label(c::TOOL_UNSUPPORTED);
                crate::accessible::collapsing(ui, c::TOOL_DETAILS, |ui| {
                    ui.label(c::TOOL_LICENSE);
                    ui.label(c::TOOL_DISTRIBUTION_REVIEW_STATUS);
                });
            }
        } else if let Some(package) = crate::managed_candidates::archive_candidate(self.tool) {
            let installed_bytes: u64 = package
                .components
                .iter()
                .map(|component| component.bytes)
                .sum();
            ui.small(format!(
                "{} {:.1} MiB · {} {:.1} MiB",
                c::TOOL_DOWNLOAD_SIZE_LABEL,
                package.archive_bytes as f64 / 1048576.0,
                c::TOOL_INSTALL_FILES_SIZE_LABEL,
                installed_bytes as f64 / 1048576.0
            ));
            crate::accessible::collapsing(ui, c::TOOL_DETAILS, |ui| {
                ui.label(format!(
                    "{}: {}",
                    c::TOOL_VERSION_LABEL,
                    package.tool_version
                ));
                ui.label(format!("{}: {}", c::TOOL_PLATFORM_LABEL, package.target));
                ui.label(format!(
                    "{}: {}",
                    c::TOOL_SOURCE_URL_LABEL,
                    package.source_url
                ));
                ui.label(format!(
                    "{}: {}",
                    c::TOOL_SHA256_LABEL,
                    package.archive_sha256
                ));
                ui.label(format!("{}: {}", c::TOOL_LICENSE_LABEL, package.license));
                ui.label(c::TOOL_VERIFICATION_DETAILS);
                ui.label(c::TOOL_DISTRIBUTION_REVIEW_STATUS);
            });
        } else {
            ui.label(c::TOOL_UNSUPPORTED);
            crate::accessible::collapsing(ui, c::TOOL_DETAILS, |ui| {
                ui.label(if self.tool == ToolId::Deno {
                    c::TOOL_DENO_LICENSE_SUMMARY
                } else {
                    c::TOOL_FFMPEG_LICENSE_SUMMARY
                });
                ui.label(c::TOOL_DISTRIBUTION_REVIEW_STATUS);
            });
        }
        if let Some(status) = self.background_status() {
            let help = match self.background_work {
                Some(BackgroundWork::Lookup) if self.inventory.is_some() => {
                    c::TOOL_CACHE_REFRESHING
                }
                Some(BackgroundWork::Lookup) => c::TOOL_AUTO_CHECKING,
                Some(BackgroundWork::VersionCheck) => c::TOOL_UPDATE_CHECKING,
                _ if self.updates.pending.is_some() => c::TOOL_UPDATE_BUSY_BOUND,
                _ => c::TOOL_BUSY,
            };
            ui.label(status).on_hover_text(help);
        }
        if !enabled && !self.busy && !self.signals.stop.load(Ordering::Acquire) {
            ui.label(c::TOOL_RUNNING);
        }
        if let Some(error) = &self.error {
            ui.colored_label(ui.visuals().error_fg_color, error);
        }
        if self.updates.render_dirty || self.updates.render_snapshots.is_none() {
            self.updates.render_snapshots = None;
            self.updates.render_snapshots = Some(Box::new(self.update_snapshots()));
            self.updates.render_dirty = false;
        }
        let update_intent = self.updates.renderer.show_for_tool(
            ui,
            self.tool,
            self.updates
                .render_snapshots
                .as_ref()
                .expect("initialized renderer snapshot"),
            enabled,
            self.signals.stop.load(Ordering::Acquire),
            Some(c::TOOL_UPDATE_BUSY_BOUND),
        );
        if let Some(intent) = update_intent {
            self.submit_update(intent, enabled);
        }
        if let Some(evidence) = &self.updates.evidence[tool_slot(self.tool)] {
            ui.push_id(("managed-update-evidence", self.tool.id()), |ui| {
                let status = match evidence.state {
                    EvidenceState::Prepared => c::TOOL_UPDATE_EVIDENCE_PREPARED,
                    EvidenceState::InstalledInactive => c::TOOL_UPDATE_EVIDENCE_INSTALLED,
                    EvidenceState::InstallFailed => c::TOOL_UPDATE_EVIDENCE_FAILED,
                    EvidenceState::InstalledIncomplete => c::TOOL_UPDATE_EVIDENCE_INCOMPLETE,
                };
                ui.label(status);
                if let Some(installed) = &evidence.installed {
                    ui.label(format!(
                        "{} · {}",
                        installed.manifest.version,
                        c::TOOL_UPDATE_INSTALL_ONLY_HELP
                    ));
                }
                let label = ui.label(c::TOOL_UPDATE_EVIDENCE_PATH);
                let path = evidence.directory.display().to_string();
                let mut exact = path.as_str();
                let response = ui
                    .add(
                        egui::TextEdit::multiline(&mut exact)
                            .id(ui.make_persistent_id("path"))
                            .interactive(false)
                            .desired_width(ui.available_width())
                            .desired_rows(2),
                    )
                    .labelled_by(label.id);
                ui.ctx()
                    .accesskit_node_builder(response.id, |node| node.set_read_only());
            });
        }
        if let Some(inventory) = &self.inventory {
            if inventory.recovery.removed > 0 {
                ui.label(format!(
                    "{} {}",
                    c::TOOL_STAGING_RECOVERED,
                    inventory.recovery.removed
                ));
            }
            if inventory.recovery.preserved > 0 {
                ui.label(format!(
                    "{} {}",
                    c::TOOL_STAGING_PRESERVED,
                    inventory.recovery.preserved
                ));
            }
            if inventory.installed.is_empty() {
                ui.label(c::TOOL_EMPTY);
            }
            // Inventory shares the page viewport; no nested wheel boundary.
            ui.push_id("managed-tool-inventory", |ui| {
                for tool in &inventory.installed {
                    let active = inventory.active.as_ref().is_some_and(|current| {
                        current.manifest.version == tool.manifest.version
                            && current.manifest.target == tool.manifest.target
                    });
                    let state = if inventory.inspection.is_none() {
                        RowState::Verified
                    } else {
                        inventory
                            .rows
                            .get(&row_key(&tool.manifest))
                            .cloned()
                            .unwrap_or_default()
                    };
                    let row_enabled = enabled
                        && !active
                        && !matches!(state, RowState::Checking | RowState::Rejected(_));
                    let select_label = if matches!(state, RowState::Verified)
                        && !matches!(
                            self.display_state[tool_slot(self.tool)],
                            DisplayState::Stale | DisplayState::Failed | DisplayState::Refreshing
                        ) {
                        c::TOOL_SELECT
                    } else {
                        c::TOOL_VERIFY_SELECT
                    };
                    // Existing installation entity key; global row scope keeps native
                    // automatic action IDs stable when the inventory order changes.
                    let row_id = ui.make_persistent_id((
                        tool.manifest.tool.as_str(),
                        tool.manifest.version.as_str(),
                        tool.manifest.target.as_str(),
                    ));
                    ui.scope_builder(egui::UiBuilder::new().id(row_id), |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(format!(
                                "{} · {:.1} MiB · {}",
                                tool.manifest.version,
                                tool.bytes as f64 / 1048576.0,
                                if active {
                                    c::TOOL_ACTIVE
                                } else {
                                    c::TOOL_INACTIVE
                                }
                            ));
                            if crate::ui_components::secondary(ui, select_label, row_enabled)
                                .on_hover_text(c::TOOL_SELECT_HELP)
                                .on_disabled_hover_text(c::TOOL_SELECT_HELP)
                                .clicked()
                            {
                                action = Some(Action::Select(tool.manifest.clone()));
                            }
                            if crate::ui_components::danger(ui, c::TOOL_REMOVE, enabled && !active)
                                .clicked()
                            {
                                action = Some(Action::Remove(tool.manifest.clone()));
                            }
                        });
                        // Feedback follows actions so the original autoID seed stays intact.
                        match state {
                            RowState::Checking => {
                                ui.weak(c::TOOL_AUTO_CHECKING);
                            }
                            RowState::Rejected(reason) => {
                                ui.colored_label(ui.visuals().error_fg_color, reason);
                            }
                            _ => (),
                        }
                    });
                }
            });
            if let Some(active) = &inventory.active {
                ui.add_space(crate::ui_tokens::CONTROL_GAP);
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!(
                        "{} {} · {}",
                        self.tool.label(),
                        active.manifest.version,
                        c::TOOL_ACTIVE
                    ));
                    let apply = crate::ui_components::secondary(ui, c::TOOL_USE, enabled)
                        .on_hover_text(c::TOOL_USE_HELP)
                        .on_disabled_hover_text(c::TOOL_USE_HELP);
                    ui.ctx().accesskit_node_builder(apply.id, |node| {
                        node.set_description(c::TOOL_USE_HELP);
                    });
                    if apply.clicked() {
                        action = Some(Action::Use);
                    }
                });
                ui.weak(c::TOOL_USE_HELP);
                ui.push_id("managed-active-executable-details", |ui| {
                    crate::accessible::collapsing(ui, c::TOOL_EXECUTABLE_DETAILS, |ui| {
                        let label = ui.label(c::TOOL_EXECUTABLE_PATH);
                        let path = active.executable.display().to_string();
                        let mut exact = path.as_str();
                        let response = ui
                            .add(
                                egui::TextEdit::multiline(&mut exact)
                                    .id(ui.make_persistent_id("managed-active-executable-path"))
                                    .interactive(false)
                                    .desired_width(ui.available_width())
                                    .desired_rows(2),
                            )
                            .labelled_by(label.id);
                        ui.ctx().accesskit_node_builder(response.id, |node| {
                            node.set_read_only();
                        });
                        if crate::ui_components::ghost(ui, c::TOOL_EXECUTABLE_COPY, true).clicked()
                        {
                            ui.ctx().copy_text(path);
                        }
                    });
                });
            }
        }
        if let Some(action) = action {
            self.submit(action, enabled);
        }
        if self.busy || (self.signals.stop.load(Ordering::Acquire) && !self.stopped()) {
            ui.ctx().request_repaint_after(Duration::from_millis(100));
        }
    }
}
impl Drop for Panel {
    fn drop(&mut self) {
        self.request_stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        sync::atomic::AtomicU64,
        time::{Instant, SystemTime, UNIX_EPOCH},
    };
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            for _ in 0..256 {
                let path = std::env::temp_dir().join(format!(
                    "dlpad-cpu-tools-{}-{}-{}",
                    std::process::id(),
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_nanos(),
                    NEXT.fetch_add(1, Ordering::Relaxed)
                ));
                match fs::create_dir(&path) {
                    Ok(()) => return Self(path),
                    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                    Err(e) => panic!("{e}"),
                }
            }
            panic!("fixture limit")
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn wait(panel: &mut Panel) {
        let deadline = Instant::now() + Duration::from_secs(4);
        while panel.busy() {
            panel.poll();
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(5));
        }
    }
    fn stop(panel: &mut Panel) {
        panel.request_stop();
        let deadline = Instant::now() + Duration::from_secs(4);
        while !panel.stopped() {
            panel.poll();
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(5));
        }
    }
    fn frame(panel: &mut Panel, enabled: bool) {
        let context = egui::Context::default();
        let _ = context.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| panel.show(ui, enabled));
        });
    }
    fn cache_test_inventory(tool: ToolId, version: &str) -> Inventory {
        let (mut manifest, _) = candidate().unwrap();
        manifest.tool = tool.id().to_owned();
        manifest.version = version.to_owned();
        Inventory {
            installed: vec![InstalledTool {
                manifest,
                executable: PathBuf::from("synthetic-metadata-cache-not-runtime-authority"),
                bytes: 1,
            }],
            active: None,
            recovery: StagingRecovery::default(),
            inspection: None,
            auto_selected: None,
            explicit_selected: None,
            rows: Default::default(),
        }
    }
    fn cache_test_panel() -> (
        Panel,
        Receiver<Command>,
        SyncSender<Result<Outcome, String>>,
    ) {
        let (commands, requests) = mpsc::sync_channel(1);
        let (acks, results) = mpsc::sync_channel(1);
        let panel = Panel {
            updates: Default::default(),
            commands,
            results,
            signals: Arc::new(Signals::default()),
            inventory: Some(cache_test_inventory(ToolId::YtDlp, "cached-a")),
            cached: [
                None,
                Some(cache_test_inventory(ToolId::Ffmpeg, "cached-ff")),
                Some(cache_test_inventory(ToolId::Deno, "cached-b")),
            ],
            display_state: [DisplayState::Fresh; 3],
            errors: std::array::from_fn(|_| None),
            request_seq: 0,
            pending_dispatch: None,
            background_work: None,
            initial_refresh_pending: false,
            error: None,
            selected: None,
            auto_selected: None,
            inspection: None,
            automatic_tickets: [None; 3],
            pending_row: None,
            tool: ToolId::YtDlp,
            busy: false,
            context: egui::Context::default(),
        };
        (panel, requests, acks)
    }
    fn cache_draw(
        panel: &mut Panel,
        ctx: &egui::Context,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(960.0, 900.0),
                )),
                focused: true,
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| panel.show(ui, true));
            },
        )
    }
    fn cache_tab(out: &egui::FullOutput, label: &str) -> egui::accesskit::NodeId {
        out.platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .find(|(_, n)| n.role() == egui::accesskit::Role::Button && n.label() == Some(label))
            .unwrap()
            .0
    }
    fn cache_click(id: egui::accesskit::NodeId) -> egui::Event {
        egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Click,
            target: id,
            data: None,
        })
    }
    fn cache_painted(out: &egui::FullOutput, needle: &str) -> bool {
        out.shapes.iter().any(
            |s| matches!(&s.shape,egui::epaint::Shape::Text(t) if t.galley.text().contains(needle)),
        )
    }
    #[test]
    fn actual_compact_tools_status_and_install_fit_above_footer_at_800_200_percent() {
        for dark in [false, true] {
            for (width, scale) in [(800.0, 2.0), (960.0, 1.0), (960.0, 2.0)] {
                let ctx = egui::Context::default();
                crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
                ctx.set_visuals(if dark {
                    egui::Visuals::dark()
                } else {
                    egui::Visuals::light()
                });
                ctx.set_zoom_factor(scale);
                ctx.enable_accesskit();
                let (mut panel, requests, _acks) = cache_test_panel();
                panel.display_state[0] = DisplayState::Stale;
                let mut observed = None;
                for _ in 0..8 {
                    let mut clip = egui::Rect::NOTHING;
                    let mut footer = 0.0;
                    let out = ctx.run(
                        egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::vec2(width / scale, 600.0 / scale),
                            )),
                            focused: true,
                            time: Some(ctx.input(|input| input.time) + 0.05),
                            ..Default::default()
                        },
                        |ctx| {
                            egui::TopBottomPanel::top("compact-tools-fixture-navigation")
                                .exact_height(48.0)
                                .show(ctx, |ui| {
                                    ui.label(c::TOOLS);
                                });
                            footer = egui::TopBottomPanel::bottom("compact-tools-fixture-footer")
                                .exact_height(54.0)
                                .show(ctx, |ui| {
                                    ui.label("fixture footer");
                                })
                                .response
                                .rect
                                .top();
                            egui::CentralPanel::default().show(ctx, |ui| {
                                ui.label(
                                    egui::RichText::new(c::TOOLS)
                                        .text_style(egui::TextStyle::Name("screen-title".into())),
                                );
                                ui.add_space(12.0);
                                egui::ScrollArea::vertical()
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| {
                                        clip = ui.clip_rect();
                                        panel.show(ui, true);
                                    });
                            });
                        },
                    );
                    let Some(tree) = out.platform_output.accesskit_update.as_ref() else {
                        continue;
                    };
                    let install = tree.nodes.iter().find(|(_, node)| {
                        node.role() == egui::accesskit::Role::Button
                            && node.label() == Some(c::TOOL_INSTALL_YTDLP)
                    });
                    let status = tree.nodes.iter().find(|(_, node)| {
                        node.role() == egui::accesskit::Role::Label
                            && node.value() == Some(c::TOOL_STATUS_INSTALLED)
                    });
                    if let (Some((_, install)), Some((_, status))) = (install, status)
                        && cache_painted(&out, c::TOOL_STATUS_INSTALLED)
                    {
                        observed = Some((install.clone(), status.clone(), clip, footer, out));
                        break;
                    }
                }
                let (install, status, clip, footer, out) = observed
                    .expect("actual shaped status and accessible install action must be observed");
                for node in [install, status] {
                    let b = node.bounds().unwrap();
                    let rect = egui::Rect::from_min_max(
                        egui::pos2(b.x0 as f32, b.y0 as f32),
                        egui::pos2(b.x1 as f32, b.y1 as f32),
                    );
                    assert!(
                        clip.contains_rect(rect) && rect.bottom() <= footer,
                        "main tools state/action remain in initial viewport above footer: width={width} scale={scale} dark={dark} rect={rect:?} clip={clip:?} footer={footer}"
                    );
                }
                assert!(cache_painted(&out, c::TOOL_CACHE_STATUS_SHORT));
                assert!(
                    !cache_painted(&out, c::TOOL_YTDLP_ROLE)
                        && !cache_painted(&out, c::TOOL_STATUS_RECORD_HELP)
                        && !cache_painted(&out, c::TOOL_CACHE_STALE),
                    "long optional context is not painted inline before primary actions"
                );
                assert!(
                    out.platform_output
                        .accesskit_update
                        .as_ref()
                        .unwrap()
                        .nodes
                        .iter()
                        .any(|(_, node)| node.label() == Some(c::TOOL_DETAILS_HELP_TITLE)
                            && node.description().is_some_and(|description| description
                                .contains(c::TOOL_YTDLP_ROLE)
                                && description.contains(c::TOOL_YTDLP_ORIGIN_MANAGED)
                                && description.contains(c::TOOL_STATUS_RECORD_HELP)
                                && description.contains(c::TOOL_CACHE_STALE)))
                );
                assert!(
                    requests.try_recv().is_err(),
                    "render-only layout never inspects or mutates tools"
                );
            }
        }
    }
    #[test]
    fn actual_recorded_pin_disables_duplicate_install_but_keeps_explicit_update_check() {
        let ctx = egui::Context::default();
        crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
        ctx.enable_accesskit();
        let (mut panel, requests, _) = cache_test_panel();
        let (manifest, _) = candidate().unwrap();
        let mut inventory = cache_test_inventory(ToolId::YtDlp, &manifest.version);
        inventory.installed[0].manifest = manifest.clone();
        panel.inventory = Some(inventory);
        cache_draw(&mut panel, &ctx, vec![]);
        let out = cache_draw(&mut panel, &ctx, vec![]);
        assert!(cache_painted(&out, c::TOOL_STATUS_INSTALLED));
        assert!(cache_painted(&out, &manifest.version));
        assert!(cache_painted(&out, c::TOOL_UPDATE_CURRENT_UNKNOWN));
        let nodes = &out.platform_output.accesskit_update.as_ref().unwrap().nodes;
        let (install_id, install) = nodes
            .iter()
            .find(|(_, node)| {
                node.role() == egui::accesskit::Role::Button
                    && node.label() == Some(c::TOOL_INSTALL_YTDLP)
            })
            .unwrap();
        assert!(
            install.is_disabled(),
            "existing pinned record never invites duplicate installation"
        );
        assert!(
            nodes
                .iter()
                .any(|(_, node)| node.role() == egui::accesskit::Role::Button
                    && node.label() == Some(c::TOOL_UPDATE_CHECK)
                    && !node.is_disabled()),
            "latest discovery is an independent explicit action"
        );
        cache_draw(&mut panel, &ctx, vec![cache_click(*install_id)]);
        panel.submit(Action::Install, true);
        assert!(
            requests.try_recv().is_err(),
            "both actual UI and dispatch refuse duplicate install without granting runtime authority"
        );
        for change in 0..4 {
            let mut other = manifest.clone();
            match change {
                0 => other.version.push_str("-other"),
                1 => other.target.push_str("-other"),
                2 => other.sha256 = "0".repeat(64),
                _ => other.ownership = dlpad_engine::tools::Ownership::External,
            }
            panel.inventory.as_mut().unwrap().installed[0].manifest = other;
            let out = cache_draw(&mut panel, &ctx, vec![]);
            assert!(
                out.platform_output
                    .accesskit_update
                    .as_ref()
                    .unwrap()
                    .nodes
                    .iter()
                    .any(|(_, node)| node.role() == egui::accesskit::Role::Button
                        && node.label() == Some(c::TOOL_INSTALL_YTDLP)
                        && !node.is_disabled()),
                "unrelated metadata does not masquerade as current managed pin"
            );
        }
    }
    #[test]
    fn actual_unknown_and_failed_inventory_never_render_false_not_installed() {
        let ctx = egui::Context::default();
        crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
        ctx.enable_accesskit();
        let (mut panel, requests, _) = cache_test_panel();
        panel.inventory = None;
        panel.display_state[0] = DisplayState::Empty;
        let out = cache_draw(&mut panel, &ctx, vec![]);
        assert!(cache_painted(&out, c::TOOL_STATUS_CHECKING));
        assert!(!cache_painted(&out, c::TOOL_STATUS_NOT_INSTALLED));
        panel.failure_for(ToolId::YtDlp, c::TOOL_FAILED.into());
        let out = cache_draw(&mut panel, &ctx, vec![]);
        assert!(cache_painted(&out, c::TOOL_STATUS_FAILED));
        assert!(!cache_painted(&out, c::TOOL_STATUS_NOT_INSTALLED));
        assert!(requests.try_recv().is_err());
    }
    #[test]
    fn actual_cached_tool_tab_immediately_paints_last_rows_and_failed_refresh_retains_them() {
        let ctx = egui::Context::default();
        crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
        ctx.enable_accesskit();
        let (mut panel, requests, acks) = cache_test_panel();
        cache_draw(&mut panel, &ctx, vec![]);
        let out = cache_draw(&mut panel, &ctx, vec![]);
        let id = cache_tab(&out, c::TOOL_DENO);
        let out = cache_draw(&mut panel, &ctx, vec![cache_click(id)]);
        assert_eq!(panel.tool, ToolId::Deno);
        assert!(
            cache_painted(&out, "cached-b"),
            "same action frame shows cached metadata without waiting for IO"
        );
        assert!(cache_painted(&out, c::TOOL_CACHE_STATUS_SHORT));
        assert!(
            out.platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node.label() == Some(c::TOOL_DETAILS_HELP_TITLE)
                    && node
                        .description()
                        .is_some_and(|text| text.contains(c::TOOL_CACHE_STALE)))
        );
        assert!(
            requests.try_recv().is_err(),
            "cached navigation starts no tool inspection"
        );
        let refresh = cache_tab(&out, c::TOOL_REFRESH);
        cache_draw(&mut panel, &ctx, vec![cache_click(refresh)]);
        let out = cache_draw(&mut panel, &ctx, vec![]);
        assert!(cache_painted(&out, "cached-b"));
        assert!(cache_painted(&out, c::TOOL_CACHE_STATUS_SHORT));
        assert!(
            out.platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node.label() == Some(c::TOOL_DETAILS_HELP_TITLE)
                    && node
                        .description()
                        .is_some_and(|text| text.contains(c::TOOL_CACHE_REFRESHING)))
        );
        assert!(cache_painted(&out, c::TOOL_STATUS_INSTALLED));
        assert!(cache_painted(&out, c::TOOL_STATUS_CHECKING));
        let cmd = requests.try_recv().unwrap();
        assert_eq!(cmd.tool, ToolId::Deno);
        assert!(matches!(cmd.action, Action::Refresh));
        acks.send(Ok(Outcome::Tagged {
            tool: cmd.tool,
            request_seq: cmd.request_seq,
            result: Box::new(Err(c::TOOL_HASH_FAILED.into())),
        }))
        .unwrap();
        panel.poll();
        let out = cache_draw(&mut panel, &ctx, vec![]);
        assert!(cache_painted(&out, "cached-b"));
        assert!(cache_painted(&out, c::TOOL_CACHE_STATUS_SHORT));
        assert!(
            out.platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node.label() == Some(c::TOOL_DETAILS_HELP_TITLE)
                    && node
                        .description()
                        .is_some_and(|text| text.contains(c::TOOL_CACHE_FAILED)))
        );
        assert!(cache_painted(&out, c::TOOL_STATUS_FAILED));
        assert_eq!(panel.error.as_deref(), Some(c::TOOL_HASH_FAILED));
        assert!(panel.take_selected_path().is_none());
        assert!(
            requests.try_recv().is_err(),
            "failed refresh never silently retries every frame"
        );
        let a = cache_tab(&out, c::TOOL_YT_DLP);
        let out = cache_draw(&mut panel, &ctx, vec![cache_click(a)]);
        assert!(cache_painted(&out, "cached-a"));
        assert_eq!(
            panel.inventory.as_ref().unwrap().installed[0].manifest.tool,
            "yt-dlp"
        );
        assert!(
            requests.try_recv().is_err(),
            "cached return remains metadata-only"
        );
    }
    #[test]
    fn actual_readonly_navigation_keeps_origin_use_ticket_and_rejects_stale_cross_slot_ack() {
        let ctx = egui::Context::default();
        crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
        ctx.enable_accesskit();
        let (mut panel, requests, acks) = cache_test_panel();
        cache_draw(&mut panel, &ctx, vec![]);
        let out = cache_draw(&mut panel, &ctx, vec![]);
        let b = cache_tab(&out, c::TOOL_DENO);
        panel.submit(Action::Use, true);
        let cmd = requests.try_recv().unwrap();
        let out = cache_draw(&mut panel, &ctx, vec![cache_click(b)]);
        assert_eq!(panel.tool, ToolId::Deno);
        assert!(cache_painted(&out, "cached-b"));
        assert!(cache_painted(&out, c::TOOL_CACHE_STATUS_SHORT));
        assert!(
            out.platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node.label() == Some(c::TOOL_DETAILS_HELP_TITLE)
                    && node
                        .description()
                        .is_some_and(|text| text.contains(c::TOOL_CACHE_STALE)))
        );
        assert!(
            requests.try_recv().is_err(),
            "latest tab query waits for existing owned command"
        );
        let wrong = PathBuf::from("wrong-slot-never-executed");
        acks.send(Ok(Outcome::Tagged {
            tool: ToolId::Deno,
            request_seq: cmd.request_seq,
            result: Box::new(Ok(Outcome::Selected {
                tool: ToolId::Deno,
                path: wrong,
            })),
        }))
        .unwrap();
        panel.poll();
        assert!(panel.busy());
        assert!(panel.take_selected_path().is_none());
        let path = PathBuf::from("origin-a-verified-worker-receipt-not-native-proof");
        acks.send(Ok(Outcome::Tagged {
            tool: cmd.tool,
            request_seq: cmd.request_seq,
            result: Box::new(Ok(Outcome::Selected {
                tool: cmd.tool,
                path: path.clone(),
            })),
        }))
        .unwrap();
        panel.poll();
        assert_eq!(panel.take_selected_path(), Some((ToolId::YtDlp, path)));
        assert_eq!(panel.tool, ToolId::Deno);
        assert_eq!(
            panel.inventory.as_ref().unwrap().installed[0]
                .manifest
                .version,
            "cached-b"
        );
        let out = cache_draw(&mut panel, &ctx, vec![]);
        assert!(
            requests.try_recv().is_err(),
            "origin ACK never causes a cached tab to silently re-probe"
        );
        let refresh = cache_tab(&out, c::TOOL_REFRESH);
        cache_draw(&mut panel, &ctx, vec![cache_click(refresh)]);
        let latest = requests.try_recv().unwrap();
        assert_eq!(latest.tool, ToolId::Deno);
        assert!(latest.request_seq > cmd.request_seq);
        assert!(matches!(latest.action, Action::Refresh));
        acks.send(Ok(Outcome::Tagged {
            tool: cmd.tool,
            request_seq: cmd.request_seq,
            result: Box::new(Ok(Outcome::Inventory(Box::new(cache_test_inventory(
                ToolId::YtDlp,
                "stale-reorder",
            ))))),
        }))
        .unwrap();
        panel.poll();
        assert!(panel.busy());
        assert_eq!(
            panel.pending_dispatch,
            Some((latest.tool, latest.request_seq))
        );
        assert_eq!(
            panel.inventory.as_ref().unwrap().installed[0]
                .manifest
                .version,
            "cached-b"
        );
        assert!(panel.take_selected_path().is_none());
    }
    #[test]
    fn actual_cached_row_identity_survives_return_reordered_refresh_and_old_action_selects_exact_entity()
     {
        let ctx = egui::Context::default();
        crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
        ctx.enable_accesskit();
        let (mut panel, requests, acks) = cache_test_panel();
        let mut second = panel.inventory.as_ref().unwrap().installed[0].clone();
        second.manifest.version = "cached-a-second".into();
        panel
            .inventory
            .as_mut()
            .unwrap()
            .installed
            .push(second.clone());
        cache_draw(&mut panel, &ctx, vec![]);
        let out = cache_draw(&mut panel, &ctx, vec![]);
        let original = out
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .filter(|(_, n)| {
                n.role() == egui::accesskit::Role::Button && n.label() == Some(c::TOOL_SELECT)
            })
            .min_by(|a, b| {
                a.1.bounds()
                    .unwrap()
                    .y0
                    .total_cmp(&b.1.bounds().unwrap().y0)
            })
            .unwrap()
            .0;
        let b = cache_tab(&out, c::TOOL_DENO);
        let out = cache_draw(&mut panel, &ctx, vec![cache_click(b)]);
        assert!(requests.try_recv().is_err(), "cached B visit has no I/O");
        let refresh = cache_tab(&out, c::TOOL_REFRESH);
        cache_draw(&mut panel, &ctx, vec![cache_click(refresh)]);
        let cmd = requests.try_recv().unwrap();
        acks.send(Ok(Outcome::Tagged {
            tool: cmd.tool,
            request_seq: cmd.request_seq,
            result: Box::new(Ok(Outcome::Inventory(Box::new(cache_test_inventory(
                ToolId::Deno,
                "new-b",
            ))))),
        }))
        .unwrap();
        panel.poll();
        let out = cache_draw(&mut panel, &ctx, vec![]);
        let a = cache_tab(&out, c::TOOL_YT_DLP);
        let out = cache_draw(&mut panel, &ctx, vec![cache_click(a)]);
        assert!(requests.try_recv().is_err(), "cached A return has no I/O");
        let refresh = cache_tab(&out, c::TOOL_REFRESH);
        cache_draw(&mut panel, &ctx, vec![cache_click(refresh)]);
        let cmd = requests.try_recv().unwrap();
        let mut reordered = cache_test_inventory(ToolId::YtDlp, "cached-a");
        reordered.installed.insert(0, second);
        acks.send(Ok(Outcome::Tagged {
            tool: cmd.tool,
            request_seq: cmd.request_seq,
            result: Box::new(Ok(Outcome::Inventory(Box::new(reordered)))),
        }))
        .unwrap();
        panel.poll();
        let out = cache_draw(&mut panel, &ctx, vec![]);
        assert!(
            out.platform_output
                .accesskit_update
                .unwrap()
                .nodes
                .iter()
                .any(|(id, n)| *id == original
                    && n.role() == egui::accesskit::Role::Button
                    && n.label() == Some(c::TOOL_SELECT)),
            "original exact entity keeps action ID across cache/refresh reorder"
        );
        cache_draw(&mut panel, &ctx, vec![cache_click(original)]);
        let cmd = requests.try_recv().unwrap();
        let Action::Select(manifest) = cmd.action else {
            panic!("old row identity must route to Select");
        };
        assert_eq!(cmd.tool, ToolId::YtDlp);
        assert_eq!(manifest.version, "cached-a");
        assert_eq!(manifest.tool, "yt-dlp");
    }

    #[test]
    fn disconnected_pending_cache_refresh_marks_only_origin_failed_and_preserves_latest_slot() {
        let (mut panel, requests, acks) = cache_test_panel();
        panel.submit(Action::Refresh, true);
        let cmd = requests.try_recv().unwrap();
        panel.choose_tool(ToolId::Deno);
        drop(acks);
        panel.poll();
        assert!(!panel.busy());
        assert!(panel.pending_dispatch.is_none());
        assert!(matches!(
            panel.display_state[tool_slot(cmd.tool)],
            DisplayState::Failed
        ));
        assert_eq!(panel.tool, ToolId::Deno);
        assert_eq!(
            panel.inventory.as_ref().unwrap().installed[0]
                .manifest
                .version,
            "cached-b"
        );
        assert_eq!(
            panel.cached[tool_slot(ToolId::YtDlp)]
                .as_ref()
                .unwrap()
                .installed[0]
                .manifest
                .version,
            "cached-a"
        );
        assert!(panel.take_selected_path().is_none());
        assert!(panel.take_auto_selected_path().is_none());
    }

    #[test]
    fn first_enabled_page_loads_empty_inventory_and_does_not_refresh_each_frame() {
        let fixture = Fixture::new();
        let mut panel = Panel::new(fixture.0.clone());
        frame(&mut panel, false);
        assert!(!fixture.0.join("managed-tools").exists());
        frame(&mut panel, true);
        wait(&mut panel);
        assert!(panel.inventory.as_ref().unwrap().installed.is_empty());
        // Change only this owned namespace after completion: a repeated refresh
        // would report an owner error. Completed-frame rendering must not do so.
        fs::write(
            fixture.0.join("managed-tools/.dlpad-owner"),
            b"foreign marker",
        )
        .unwrap();
        for _ in 0..3 {
            frame(&mut panel, true);
        }
        assert!(!panel.busy());
        assert!(panel.error.is_none());
        panel.submit(Action::Refresh, true);
        wait(&mut panel);
        assert!(panel.error.is_some(), "explicit Refresh remains available");
        stop(&mut panel);
    }
    #[test]
    fn failed_first_page_does_not_retry_or_adopt_foreign_root() {
        let fixture = Fixture::new();
        let managed = fixture.0.join("managed-tools");
        fs::write(&managed, b"foreign bytes").unwrap();
        let mut panel = Panel::new(fixture.0.clone());
        frame(&mut panel, true);
        wait(&mut panel);
        assert!(panel.error.is_some());
        assert_eq!(fs::read(&managed).unwrap(), b"foreign bytes");
        // Removing our synthetic obstruction cannot silently trigger a retry.
        fs::remove_file(&managed).unwrap();
        for _ in 0..3 {
            frame(&mut panel, true);
        }
        assert!(!managed.exists());
        assert!(!panel.busy());
        panel.submit(Action::Refresh, true);
        wait(&mut panel);
        assert!(panel.inventory.is_some());
        assert!(panel.error.is_none());
        stop(&mut panel);
    }
    #[test]
    fn stopped_before_first_enabled_page_performs_no_manager_io() {
        let fixture = Fixture::new();
        let mut panel = Panel::new(fixture.0.clone());
        stop(&mut panel);
        frame(&mut panel, true);
        assert!(!fixture.0.join("managed-tools").exists());
        assert!(panel.inventory.is_none());
    }
    #[test]
    fn first_page_observes_standard_api_active_install_without_manual_refresh() {
        let fixture = Fixture::new();
        let root = fixture.0.join("managed-tools");
        let mut manager = ToolManager::new(&root).unwrap();
        let target = if cfg!(windows) {
            "x86_64-pc-windows-msvc"
        } else {
            "x86_64-unknown-linux-gnu"
        };
        let directory = root.join("yt-dlp/synthetic-first-open").join(target);
        fs::create_dir_all(&directory).unwrap();
        let executable = if cfg!(windows) {
            "yt-dlp.exe"
        } else {
            "yt-dlp"
        };
        fs::write(directory.join(executable), b"synthetic").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(
                directory.join(executable),
                fs::Permissions::from_mode(0o700),
            )
            .unwrap();
        }
        let manifest = InstallManifest {
            tool: "yt-dlp".into(),
            version: "synthetic-first-open".into(),
            target: target.into(),
            source_url: "https://example.invalid/synthetic-first-open".into(),
            sha256: "b3cc0475bb78a5026098858e9889acf666d31062d513d303314eca31d36e72f2".into(),
            license: "synthetic inventory only; no executable/version/distribution proof".into(),
            ownership: dlpad_engine::tools::Ownership::Managed,
            executable: executable.into(),
        };
        fs::write(
            directory.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        manager.select_existing(&manifest).unwrap();
        drop(manager);
        let mut panel = Panel::new(fixture.0.clone());
        frame(&mut panel, true);
        wait(&mut panel);
        let inventory = panel.inventory.as_ref().unwrap();
        assert_eq!(inventory.installed.len(), 1);
        assert_eq!(
            inventory.active.as_ref().unwrap().manifest.version,
            "synthetic-first-open"
        );
        assert_eq!(fs::read(directory.join(executable)).unwrap(), b"synthetic");
        assert!(
            panel.take_selected_path().is_none(),
            "inventory does not implicitly Use"
        );
        stop(&mut panel);
    }
    #[test]
    fn no_active_tool_does_not_publish_path_and_worker_survives() {
        let fixture = Fixture::new();
        let mut panel = Panel::new(fixture.0.clone());
        panel.submit(Action::Use, true);
        wait(&mut panel);
        assert_eq!(panel.error.as_deref(), Some(c::TOOL_EMPTY));
        assert!(panel.take_selected_path().is_none());
        panel.submit(Action::Refresh, true);
        wait(&mut panel);
        assert!(panel.inventory.as_ref().unwrap().installed.is_empty());
        stop(&mut panel);
    }
    #[test]
    fn refresh_failure_keeps_inventory_and_foreign_file() {
        let fixture = Fixture::new();
        let mut panel = Panel::new(fixture.0.clone());
        panel.submit(Action::Refresh, true);
        wait(&mut panel);
        assert!(panel.inventory.is_some());
        fs::remove_dir_all(fixture.0.join("managed-tools")).unwrap();
        fs::write(fixture.0.join("managed-tools"), b"foreign").unwrap();
        panel.submit(Action::Refresh, true);
        wait(&mut panel);
        assert!(panel.error.is_some());
        assert!(panel.inventory.is_some());
        assert_eq!(
            fs::read(fixture.0.join("managed-tools")).unwrap(),
            b"foreign"
        );
        stop(&mut panel);
    }
    #[test]
    fn disabled_and_stopping_panel_never_accept_commands() {
        let fixture = Fixture::new();
        let mut panel = Panel::new(fixture.0.clone());
        panel.submit(Action::Install, false);
        assert!(!panel.busy());
        assert!(!fixture.0.join("managed-tools").exists());
        panel.request_stop();
        panel.submit(Action::Refresh, true);
        assert!(!panel.busy());
        stop(&mut panel);
        assert!(!fixture.0.join("managed-tools").exists());
    }
    #[test]
    fn panic_always_acknowledges_stop() {
        let signals = Arc::new(Signals::default());
        supervise(signals.clone(), egui::Context::default(), || {
            panic!("synthetic worker panic")
        });
        assert!(signals.stopped.load(Ordering::Acquire));
        assert!(signals.failed.load(Ordering::Acquire));
    }
    #[test]
    fn removal_of_external_manifest_preserves_external_bytes() {
        let fixture = Fixture::new();
        let external = fixture.0.join("external-tool");
        fs::write(&external, b"external").unwrap();
        let (mut manifest, _) = candidate().unwrap();
        manifest.ownership = dlpad_engine::tools::Ownership::External;
        assert!(
            perform(
                &fixture.0,
                ToolId::YtDlp,
                Action::Remove(manifest),
                &Arc::new(AtomicBool::new(false))
            )
            .is_err()
        );
        assert_eq!(fs::read(external).unwrap(), b"external");
    }
    #[test]
    fn all_tools_empty_inventory_and_use_do_not_publish_selection() {
        let fixture = Fixture::new();
        for tool in [ToolId::YtDlp, ToolId::Deno, ToolId::Ffmpeg] {
            let mut panel = Panel::new(fixture.0.clone());
            panel.tool = tool;
            panel.submit(Action::Refresh, true);
            wait(&mut panel);
            assert!(panel.inventory.as_ref().unwrap().installed.is_empty());
            panel.submit(Action::Use, true);
            wait(&mut panel);
            assert_eq!(panel.error.as_deref(), Some(c::TOOL_EMPTY));
            assert!(panel.take_selected_path().is_none());
            stop(&mut panel);
        }
    }
    #[test]
    fn runtime_install_precancel_and_mismatched_manifest_are_rejected() {
        let fixture = Fixture::new();
        let (mut manifest, _) = candidate().unwrap();
        for tool in [ToolId::Deno, ToolId::Ffmpeg] {
            assert_eq!(
                tool.install_available(),
                cfg!(all(target_os = "windows", target_arch = "x86_64"))
            );
            let result = perform(
                &fixture.0,
                tool,
                Action::Install,
                &Arc::new(AtomicBool::new(true)),
            );
            let expected = if tool.install_available() {
                c::TOOL_CANCELLED
            } else {
                c::TOOL_UNSUPPORTED
            };
            assert!(matches!(result, Err(message) if message == expected));
            let manager = ToolManager::new(&fixture.0.join("managed-tools")).unwrap();
            assert!(manager.installed(tool.id()).unwrap().is_empty());
            assert!(manager.active(tool.id()).unwrap().is_none());
            assert!(
                perform(
                    &fixture.0,
                    tool,
                    Action::Select(manifest.clone()),
                    &Arc::new(AtomicBool::new(false))
                )
                .is_err()
            );
            assert!(
                perform(
                    &fixture.0,
                    tool,
                    Action::Remove(manifest.clone()),
                    &Arc::new(AtomicBool::new(false))
                )
                .is_err()
            );
            manifest.tool = tool.id().into();
            manifest.ownership = dlpad_engine::tools::Ownership::External;
            assert!(
                perform(
                    &fixture.0,
                    tool,
                    Action::Remove(manifest.clone()),
                    &Arc::new(AtomicBool::new(false))
                )
                .is_err()
            );
            manifest.tool = "yt-dlp".into();
        }
    }
    #[test]
    fn typed_selection_retains_tool_identity_and_disabled_commands_are_inert() {
        let fixture = Fixture::new();
        let mut panel = Panel::new(fixture.0.clone());
        for tool in [ToolId::Deno, ToolId::Ffmpeg] {
            panel.tool = tool;
            panel.submit(Action::Install, false);
            assert!(!panel.busy());
            let path = fixture.0.join(tool.id());
            panel.selected = Some((tool, path.clone()));
            assert_eq!(panel.take_selected_path(), Some((tool, path)));
        }
        assert!(!fixture.0.join("managed-tools").exists());
        stop(&mut panel);
    }
    fn inspection_fixture_record(
        root: &std::path::Path,
        version: &str,
        target: &str,
    ) -> InstallManifest {
        let directory = root.join("yt-dlp").join(version).join(target);
        fs::create_dir_all(&directory).unwrap();
        let executable = if cfg!(windows) {
            "yt-dlp.exe"
        } else {
            "yt-dlp"
        };
        fs::write(directory.join(executable), b"synthetic").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(
                directory.join(executable),
                fs::Permissions::from_mode(0o700),
            )
            .unwrap();
        }
        let manifest = InstallManifest {
            tool: "yt-dlp".into(),
            version: version.into(),
            target: target.into(),
            source_url: "https://example.invalid/inspection-fixture".into(),
            sha256: "b3cc0475bb78a5026098858e9889acf666d31062d513d303314eca31d36e72f2".into(),
            license: "policy-seam fixture, not a native/version verification receipt".into(),
            ownership: dlpad_engine::tools::Ownership::Managed,
            executable: executable.into(),
        };
        fs::write(
            directory.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        manifest
    }
    // Obtain an opaque ticket only through the real Controller API. No worker/test
    // construction, transmute, private revision access or forged equality.
    fn controller_ticket(index: usize) -> ManagedAutoTicket {
        let fixture = Fixture::new();
        let mut controller = crate::accessible_tool_preferences::Controller::new(
            fixture.0.clone(),
            [None, None, None],
        );
        let ctx = egui::Context::default();
        let mut fields = [String::new(), String::new(), String::new()];
        let until = Instant::now() + Duration::from_secs(3);
        let ticket = loop {
            let [first, second, third] = &mut fields;
            controller.poll(&ctx, [first, second, third]);
            if let Some(ticket) = controller.managed_auto_ticket(index) {
                break ticket;
            }
            assert!(
                Instant::now() < until,
                "real Controller default Auto load did not mint a ticket"
            );
            thread::sleep(Duration::from_millis(1));
        };
        controller.request_stop();
        let until = Instant::now() + Duration::from_secs(3);
        while !controller.stopped() {
            let [first, second, third] = &mut fields;
            controller.poll(&ctx, [first, second, third]);
            assert!(
                Instant::now() < until,
                "owned Controller worker stop ACK missing"
            );
            thread::sleep(Duration::from_millis(1));
        }
        ticket
    }
    #[test]
    fn managed_inspection_missing_precancel_and_foreign_roots_do_not_create_or_adopt() {
        let fixture = Fixture::new();
        let cancel = Arc::new(AtomicBool::new(false));
        let missing = inspect_managed(&fixture.0, &cancel).unwrap();
        assert!(
            missing
                .slots
                .iter()
                .all(|slot| matches!(slot, Availability::Unavailable(_)))
        );
        assert!(!fixture.0.join("managed-tools").exists());
        assert_eq!(
            inspect_managed(&fixture.0, &Arc::new(AtomicBool::new(true))).unwrap_err(),
            c::TOOL_CANCELLED
        );
        assert!(!fixture.0.join("managed-tools").exists());
        fs::create_dir(fixture.0.join("managed-tools")).unwrap();
        fs::write(fixture.0.join("managed-tools/foreign"), b"preserve").unwrap();
        assert!(inspect_managed(&fixture.0, &cancel).is_err());
        assert_eq!(
            fs::read(fixture.0.join("managed-tools/foreign")).unwrap(),
            b"preserve"
        );
        assert!(!fixture.0.join("managed-tools/.dlpad-owner").exists());
        assert!(!fixture.0.join("managed-tools/activation.sqlite").exists());
    }
    #[test]
    fn managed_inspection_partial_failures_keep_other_slots_and_activation_readonly() {
        let fixture = Fixture::new();
        let root = fixture.0.join("managed-tools");
        let mut manager = ToolManager::new(&root).unwrap();
        let manifest = inspection_fixture_record(&root, "policy-valid", "policy-target");
        let selected = manager.select_existing(&manifest).unwrap();
        let active = manager.active("yt-dlp").unwrap();
        let database = fs::read(root.join("activation.sqlite")).unwrap();
        let mut called = Vec::new();
        // Failure injection at the readonly backend seam: aggregation behavior,
        // not synthetic executable/version acceptance. Actual owned bytes are read.
        let inspection = inspect_managed_with(
            &fixture.0,
            &Arc::new(AtomicBool::new(false)),
            |manager, tool, _| {
                called.push(tool);
                match tool {
                    ToolId::YtDlp => manager.active_installed("yt-dlp"),
                    ToolId::Ffmpeg => Err(dlpad_engine::tools::ToolError::HashMismatch),
                    ToolId::Deno => Err(dlpad_engine::tools::ToolError::VersionMismatch),
                }
            },
        )
        .unwrap();
        assert_eq!(called, [ToolId::YtDlp, ToolId::Ffmpeg, ToolId::Deno]);
        assert!(
            matches!(&inspection.slots[0], Availability::Available(path) if *path == selected.executable)
        );
        assert!(
            matches!(inspection.slots[1], Availability::Unavailable(reason) if reason == c::TOOL_HASH_FAILED)
        );
        assert!(
            matches!(inspection.slots[2], Availability::Unavailable(reason) if reason == c::TOOL_VERSION_FAILED)
        );
        assert_eq!(manager.active("yt-dlp").unwrap(), active);
        assert_eq!(fs::read(root.join("activation.sqlite")).unwrap(), database);
        assert_eq!(fs::read(selected.executable).unwrap(), b"synthetic");
    }
    #[test]
    fn managed_inspection_pending_is_fatal_without_fallback_or_later_slots() {
        let fixture = Fixture::new();
        let root = fixture.0.join("managed-tools");
        let manager = ToolManager::new(&root).unwrap();
        let mut called = Vec::new();
        let result = inspect_managed_with(
            &fixture.0,
            &Arc::new(AtomicBool::new(false)),
            |_, tool, _| {
                called.push(tool);
                Err(dlpad_engine::tools::ToolError::ExecutionPending)
            },
        );
        assert_eq!(result.unwrap_err(), c::TOOL_EXECUTION_PENDING);
        assert_eq!(called, [ToolId::YtDlp]);
        assert!(manager.active("yt-dlp").unwrap().is_none());
        assert!(manager.active("ffmpeg").unwrap().is_none());
        assert!(manager.active("deno").unwrap().is_none());
    }
    #[test]
    fn explicit_failed_row_verification_preserves_active_and_never_publishes_path() {
        let fixture = Fixture::new();
        let root = fixture.0.join("managed-tools");
        let mut manager = ToolManager::new(&root).unwrap();
        // A foreign-target prior record cannot launch on this host; preserve it,
        // rather than using a real executable or changing an external install.
        let old = inspection_fixture_record(&root, "prior", "foreign-target");
        let prior = manager.select_existing(&old).unwrap();
        let before = manager.active("yt-dlp").unwrap();
        let (pinned, _) = candidate().unwrap();
        let directory = root
            .join(&pinned.tool)
            .join(&pinned.version)
            .join(&pinned.target);
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join(&pinned.executable), b"tampered").unwrap();
        fs::write(
            directory.join("manifest.json"),
            serde_json::to_vec(&pinned).unwrap(),
        )
        .unwrap();
        let mut rows = std::collections::BTreeMap::new();
        let outcome = perform_worker(
            &fixture.0,
            ToolId::YtDlp,
            Action::Select(pinned.clone()),
            &Arc::new(AtomicBool::new(false)),
            std::array::from_fn(|index| Some(controller_ticket(index))),
            &mut rows,
            &mut std::array::from_fn(|_| None),
        )
        .unwrap();
        let Outcome::Inventory(inventory) = outcome else {
            panic!("failed verification must not publish Selected");
        };
        assert!(inventory.explicit_selected.is_none());
        assert!(inventory.auto_selected.is_none());
        assert!(matches!(
            inventory.rows.get(&row_key(&pinned)),
            Some(RowState::Rejected(_))
        ));
        assert_eq!(manager.active("yt-dlp").unwrap(), before);
        assert_eq!(fs::read(prior.executable).unwrap(), b"synthetic");
        assert_eq!(
            fs::read(directory.join(&pinned.executable)).unwrap(),
            b"tampered"
        );
    }
    #[test]
    fn actual_unknown_row_command_captures_auto_ticket_then_rejection_preserves_identity_and_selection()
     {
        let ctx = egui::Context::default();
        crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
        ctx.enable_accesskit();
        let (commands, requests) = mpsc::sync_channel(1);
        let (acks, results) = mpsc::sync_channel(1);
        let signals = Arc::new(Signals::default());
        let (manifest, _) = candidate().unwrap();
        let item = InstalledTool {
            manifest: manifest.clone(),
            executable: PathBuf::from("synthetic-unknown-row-never-executed"),
            bytes: 1,
        };
        let prior = PathBuf::from("prior-explicit-publication");
        let inspection = ManagedInspection {
            slots: std::array::from_fn(|_| Availability::Unavailable(c::TOOL_AUTO_UNAVAILABLE)),
        };
        let mut panel = Panel {
            updates: Default::default(),
            commands,
            results,
            signals,
            inventory: Some(Inventory {
                installed: vec![item],
                active: None,
                recovery: StagingRecovery::default(),
                inspection: Some(inspection),
                auto_selected: None,
                explicit_selected: None,
                rows: std::collections::BTreeMap::new(),
            }),
            cached: std::array::from_fn(|_| None),
            display_state: [DisplayState::Empty; 3],
            errors: std::array::from_fn(|_| None),
            request_seq: 0,
            pending_dispatch: None,
            background_work: None,
            initial_refresh_pending: false,
            error: None,
            selected: Some((ToolId::YtDlp, prior.clone())),
            auto_selected: None,
            inspection: None,
            automatic_tickets: [None; 3],
            pending_row: None,
            tool: ToolId::YtDlp,
            busy: false,
            context: ctx.clone(),
        };
        let mut time = 0.0;
        let draw = |panel: &mut Panel, time: &mut f64, events| {
            *time += 0.05;
            ctx.run(
                egui::RawInput {
                    time: Some(*time),
                    focused: true,
                    events,
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(960.0, 640.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| panel.show(ui, true));
                },
            )
        };
        let output = draw(&mut panel, &mut time, vec![]);
        let unknown = output
            .platform_output
            .accesskit_update
            .unwrap()
            .nodes
            .into_iter()
            .find(|(_, node)| {
                node.role() == egui::accesskit::Role::Button
                    && node.label() == Some(c::TOOL_VERIFY_SELECT)
            })
            .expect("actual unknown row check/use action");
        assert!(!unknown.1.is_disabled());
        let click = || {
            egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
                action: egui::accesskit::Action::Click,
                target: unknown.0,
                data: None,
            })
        };
        let ticket = controller_ticket(0);
        let deno_ticket = controller_ticket(2);
        panel.set_automatic_tickets([Some(ticket), None, Some(deno_ticket)]);
        draw(&mut panel, &mut time, vec![click()]);
        panel.set_automatic_tickets([None; 3]);
        let command = requests.try_recv().expect("actual selected row command");
        assert_eq!(
            command.automatic_tickets,
            [Some(ticket), None, Some(deno_ticket)],
            "opaque dispatch tickets survive later UI mode change unchanged"
        );
        let Action::Select(sent) = command.action else {
            panic!("unknown row dispatches existing typed explicit Select");
        };
        assert_eq!(
            serde_json::to_vec(&sent).unwrap(),
            serde_json::to_vec(&manifest).unwrap()
        );
        assert!(panel.busy());
        acks.send(Err(c::TOOL_HASH_FAILED.to_owned())).unwrap();
        panel.poll();
        assert!(!panel.busy());
        assert!(
            matches!(panel.inventory.as_ref().unwrap().rows.get(&row_key(&manifest)), Some(RowState::Rejected(reason)) if *reason == c::TOOL_HASH_FAILED)
        );
        assert_eq!(panel.selected, Some((ToolId::YtDlp, prior)));
        assert!(panel.take_auto_selected_path().is_none());
        let rejected = draw(&mut panel, &mut time, vec![])
            .platform_output
            .accesskit_update
            .unwrap();
        let node = rejected
            .nodes
            .iter()
            .find(|(id, _)| *id == unknown.0)
            .expect("same actual row action ID after error");
        assert!(node.1.is_disabled());
        assert!(
            rejected
                .nodes
                .iter()
                .any(|(_, n)| n.role() == egui::accesskit::Role::Label
                    && n.value() == Some(c::TOOL_HASH_FAILED)),
            "actual inline error is visible node content, not tooltip-only"
        );
        draw(&mut panel, &mut time, vec![click()]);
        assert!(
            requests.try_recv().is_err(),
            "disabled exact entity cannot submit another action"
        );
    }

    #[test]
    fn automatic_ticket_receipt_echoes_exact_ticket_without_explicit_selection() {
        let (commands, _) = mpsc::sync_channel(1);
        let (acks, results) = mpsc::sync_channel(1);
        let ticket = controller_ticket(0);
        let mut panel = Panel {
            updates: Default::default(),
            commands,
            results,
            signals: Arc::new(Signals::default()),
            inventory: None,
            cached: std::array::from_fn(|_| None),
            display_state: [DisplayState::Empty; 3],
            errors: std::array::from_fn(|_| None),
            request_seq: 0,
            pending_dispatch: None,
            background_work: None,
            initial_refresh_pending: false,
            error: None,
            selected: None,
            auto_selected: None,
            inspection: None,
            automatic_tickets: [None; 3],
            pending_row: None,
            tool: ToolId::YtDlp,
            busy: true,
            context: egui::Context::default(),
        };
        let path = PathBuf::from("synthetic-worker-verified-path-receipt-not-runtime-proof");
        acks.send(Ok(Outcome::Inventory(Box::new(Inventory {
            installed: vec![],
            active: None,
            recovery: StagingRecovery::default(),
            inspection: None,
            auto_selected: Some((ToolId::YtDlp, path.clone(), ticket)),
            explicit_selected: None,
            rows: Default::default(),
        }))))
        .unwrap();
        panel.poll();
        assert_eq!(
            panel.take_auto_selected_path(),
            Some((ToolId::YtDlp, path, ticket))
        );
        assert!(
            panel.take_auto_selected_path().is_none(),
            "receipt is consumed once"
        );
        assert!(
            panel.take_selected_path().is_none(),
            "Auto receipt never becomes explicit intent"
        );
        acks.send(Err(c::TOOL_CANCELLED.to_owned())).unwrap();
        panel.poll();
        assert!(
            panel.take_auto_selected_path().is_none(),
            "failed completion cannot synthesize a ticket/path"
        );
        assert!(panel.take_selected_path().is_none());
    }

    #[test]
    fn restore_active_missing_root_and_unrequested_custom_paths_do_not_create() {
        let fixture = Fixture::new();
        let cancel = Arc::new(AtomicBool::new(false));
        let requested = [
            (ToolId::YtDlp, true),
            (ToolId::Deno, true),
            (ToolId::Ffmpeg, true),
        ];
        assert!(
            restore_active(&fixture.0, &requested, &cancel)
                .unwrap()
                .is_empty()
        );
        assert!(!fixture.0.join("managed-tools").exists());
        let no_restore = requested.map(|(tool, _)| (tool, false));
        fs::write(fixture.0.join("managed-tools"), b"foreign").unwrap();
        assert!(
            restore_active(&fixture.0, &no_restore, &cancel)
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            fs::read(fixture.0.join("managed-tools")).unwrap(),
            b"foreign"
        );
    }
    #[test]
    fn restore_active_foreign_directory_is_preserved_and_not_adopted() {
        let fixture = Fixture::new();
        let root = fixture.0.join("managed-tools");
        fs::create_dir(&root).unwrap();
        fs::write(root.join("sentinel"), b"foreign").unwrap();
        let requested = [
            (ToolId::YtDlp, true),
            (ToolId::Deno, true),
            (ToolId::Ffmpeg, true),
        ];
        assert!(restore_active(&fixture.0, &requested, &Arc::new(AtomicBool::new(false))).is_err());
        assert_eq!(fs::read(root.join("sentinel")).unwrap(), b"foreign");
        assert!(!root.join(".dlpad-owner").exists());
        assert!(!root.join("activation.sqlite").exists());
    }
    #[test]
    fn restore_active_precancel_precedes_empty_request_and_filesystem_access() {
        let fixture = Fixture::new();
        let requested = [
            (ToolId::YtDlp, false),
            (ToolId::Deno, false),
            (ToolId::Ffmpeg, false),
        ];
        assert_eq!(
            restore_active(&fixture.0, &requested, &Arc::new(AtomicBool::new(true))).unwrap_err(),
            c::TOOL_CANCELLED
        );
        assert!(!fixture.0.join("managed-tools").exists());
    }
    #[test]
    fn restore_active_existing_manager_without_active_does_not_select() {
        let fixture = Fixture::new();
        let manager = ToolManager::new(&fixture.0.join("managed-tools")).unwrap();
        let requested = [
            (ToolId::YtDlp, true),
            (ToolId::Deno, true),
            (ToolId::Ffmpeg, true),
        ];
        assert!(
            restore_active(&fixture.0, &requested, &Arc::new(AtomicBool::new(false)))
                .unwrap()
                .is_empty()
        );
        for (tool, _) in requested {
            assert!(manager.active(tool.id()).unwrap().is_none());
        }
    }
    #[test]
    fn actual_inventory_row_actions_keep_entity_identity_after_reorder_prepend_and_removal() {
        type Entity = (String, String, String);
        type Actions = [(egui::accesskit::NodeId, egui::Rect); 2];
        fn entity(m: &InstallManifest) -> Entity {
            (m.tool.clone(), m.version.clone(), m.target.clone())
        }
        fn draw(
            panel: &mut Panel,
            ctx: &egui::Context,
            events: Vec<egui::Event>,
            enabled: bool,
        ) -> egui::FullOutput {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(960.0, 900.0),
                    )),
                    focused: true,
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| panel.show(ui, enabled));
                },
            )
        }
        fn actions(
            out: &egui::FullOutput,
            installed: &[InstalledTool],
            disabled: bool,
        ) -> std::collections::BTreeMap<Entity, Actions> {
            let mut columns = [Vec::new(), Vec::new()];
            for (column, label) in [c::TOOL_SELECT, c::TOOL_REMOVE].iter().enumerate() {
                for (id, n) in &out.platform_output.accesskit_update.as_ref().unwrap().nodes {
                    if n.role() == egui::accesskit::Role::Button && n.label() == Some(*label) {
                        assert_eq!(n.is_disabled(), disabled, "actual row command gate");
                        let b = n.bounds().expect("actual row command bounds");
                        let rect = egui::Rect::from_min_max(
                            egui::pos2(b.x0 as f32, b.y0 as f32),
                            egui::pos2(b.x1 as f32, b.y1 as f32),
                        );
                        assert!(rect.width() > 0.0 && rect.height() > 0.0);
                        columns[column].push((*id, rect));
                    }
                }
                columns[column].sort_by(|a, b| a.1.top().total_cmp(&b.1.top()));
                assert_eq!(
                    columns[column].len(),
                    installed.len(),
                    "one actual action per rendered row"
                );
            }
            installed
                .iter()
                .enumerate()
                .map(|(i, tool)| {
                    assert!(
                        (columns[0][i].1.center().y - columns[1][i].1.center().y).abs() < 1.0,
                        "select/remove share actual row geometry"
                    );
                    (entity(&tool.manifest), [columns[0][i], columns[1][i]])
                })
                .collect()
        }
        fn click(id: egui::accesskit::NodeId) -> egui::Event {
            egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
                action: egui::accesskit::Action::Click,
                target: id,
                data: None,
            })
        }
        for dark in [false, true] {
            let ctx = egui::Context::default();
            ctx.set_visuals(if dark {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            });
            crate::ui_theme::configure_dlpad_theme(&ctx);
            ctx.enable_accesskit();
            let (commands, requests) = mpsc::sync_channel(4);
            let (acks, results) = mpsc::sync_channel(4);
            let mut panel = Panel {
                updates: Default::default(),
                commands,
                results,
                signals: Arc::new(Signals::default()),
                inventory: None,
                cached: std::array::from_fn(|_| None),
                display_state: [DisplayState::Empty; 3],
                errors: std::array::from_fn(|_| None),
                request_seq: 0,
                pending_dispatch: None,
                background_work: None,
                initial_refresh_pending: false,
                error: None,
                selected: None,
                auto_selected: None,
                inspection: None,
                automatic_tickets: [None; 3],
                pending_row: None,
                tool: ToolId::YtDlp,
                busy: false,
                context: ctx.clone(),
            };
            let (base, _) = candidate().unwrap();
            let make = |target: &str| {
                let mut manifest = base.clone();
                manifest.version = "2026.01.01".into();
                manifest.target = target.into();
                InstalledTool {
                    manifest,
                    executable: PathBuf::from("synthetic-never-executed"),
                    bytes: 3,
                }
            };
            // Same tool/version but distinct target exercises the existing full install key.
            let (a, b, c) = (make("fixture-a"), make("fixture-b"), make("fixture-c"));
            let states = [
                vec![a.clone(), b.clone()],
                vec![b.clone(), a.clone()],
                vec![c.clone(), b.clone(), a.clone()],
                vec![c.clone(), a.clone()],
            ];
            let mut observed = std::collections::BTreeMap::<Entity, Actions>::new();
            for installed in states {
                panel.inventory = Some(Inventory {
                    installed: installed.clone(),
                    active: None,
                    recovery: StagingRecovery::default(),
                    inspection: None,
                    auto_selected: None,
                    explicit_selected: None,
                    rows: Default::default(),
                });
                let output = draw(&mut panel, &ctx, vec![], true);
                let current = actions(&output, &installed, false);
                for tool in &installed {
                    let key = entity(&tool.manifest);
                    if let Some(prior) = observed.get(&key) {
                        for (index, previous) in prior.iter().enumerate() {
                            assert_eq!(
                                current[&key][index].0, previous.0,
                                "same installation action identity after list change: {key:?} action={index} dark={dark}"
                            );
                            assert_eq!(
                                current[&key][index].1.size(),
                                previous.1.size(),
                                "layout size preserved while row position changes"
                            );
                        }
                    } else {
                        observed.insert(key.clone(), current[&key]);
                    }
                }
                // Retained original provider IDs drive the actual Panel.show->submit channel,
                // not an expected ID hash or a direct call to submit/perform.
                for (index, (original_id, _)) in
                    observed[&entity(&a.manifest)].iter().copied().enumerate()
                {
                    draw(&mut panel, &ctx, vec![click(original_id)], true);
                    let request = requests
                        .try_recv()
                        .expect("original entity action reached actual command channel");
                    assert_eq!(request.tool, ToolId::YtDlp);
                    let sent = match (index, request.action) {
                        (0, Action::Select(m)) | (1, Action::Remove(m)) => m,
                        _ => panic!("wrong original row action variant"),
                    };
                    assert_eq!(
                        serde_json::to_value(sent).unwrap(),
                        serde_json::to_value(&a.manifest).unwrap(),
                        "action still targets entire original installation manifest"
                    );
                    assert!(
                        requests.try_recv().is_err(),
                        "bounded single action, no duplicate submit"
                    );
                    // Synthetic worker ACK uses existing poll to unlock; no worker/file/DB/tool runs.
                    acks.send(Ok(Outcome::Inventory(Box::new(Inventory {
                        installed: installed.clone(),
                        active: None,
                        recovery: StagingRecovery::default(),
                        inspection: None,
                        auto_selected: None,
                        explicit_selected: None,
                        rows: Default::default(),
                    }))))
                    .unwrap();
                    panel.poll();
                    assert!(!panel.busy());
                    draw(&mut panel, &ctx, vec![], true);
                }
                let disabled = draw(&mut panel, &ctx, vec![], false);
                let disabled_ids = actions(&disabled, &installed, true);
                assert_eq!(
                    disabled_ids[&entity(&a.manifest)][0].0,
                    observed[&entity(&a.manifest)][0].0
                );
                assert_eq!(
                    disabled_ids[&entity(&a.manifest)][1].0,
                    observed[&entity(&a.manifest)][1].0
                );
                for (id, _) in observed[&entity(&a.manifest)] {
                    draw(&mut panel, &ctx, vec![click(id)], false);
                }
                assert!(
                    requests.try_recv().is_err(),
                    "disabled row actions do not enqueue"
                );
                draw(&mut panel, &ctx, vec![], true);
            }
        }
    }
    #[test]
    fn actual_update_renderer_snapshot_reuses_large_manifest_storage_until_real_source_or_inventory_change()
     {
        let (mut panel, _requests, acks) = cache_test_panel();
        let inventory = panel.inventory.as_mut().unwrap();
        inventory.installed[0].manifest.license = "bounded large notice ".repeat(2048);
        inventory.active = Some(inventory.installed[0].clone());
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        cache_draw(&mut panel, &ctx, vec![]);
        let snapshot = panel.updates.render_snapshots.as_ref().unwrap();
        let records = snapshot[0].installed.as_ptr();
        let license = snapshot[0].installed[0].manifest.license.as_ptr();
        assert!(
            snapshot[1].installed.is_empty() && snapshot[2].installed.is_empty(),
            "inactive slot manifests are not duplicated into renderer"
        );
        for _ in 0..8 {
            panel.set_update_current_sources([CurrentSource::Unknown; 3]);
            cache_draw(&mut panel, &ctx, vec![]);
            let snapshot = panel.updates.render_snapshots.as_ref().unwrap();
            assert_eq!(snapshot[0].installed.as_ptr(), records);
            assert_eq!(snapshot[0].installed[0].manifest.license.as_ptr(), license);
            assert!(!panel.updates.render_dirty);
        }
        panel.set_update_current_sources([
            CurrentSource::Managed,
            CurrentSource::Unknown,
            CurrentSource::Unknown,
        ]);
        assert!(panel.updates.render_dirty);
        let out = cache_draw(&mut panel, &ctx, vec![]);
        let snapshot = panel.updates.render_snapshots.as_ref().unwrap();
        assert!(
            matches!(&snapshot[0].current,update_ui::CurrentVersion::Managed(v) if v.version=="cached-a")
        );
        // The original renderer's smaller string limit refuses rather than silently truncates.
        assert!(cache_painted(&out, c::TOOL_UPDATE_INVALID_SNAPSHOT));
        assert!(cache_painted(&out, "cached-a"));
        panel.submit(Action::Refresh, true);
        let seq = panel.pending_dispatch.unwrap().1;
        acks.send(Ok(Outcome::Tagged {
            tool: ToolId::YtDlp,
            request_seq: seq,
            result: Box::new(Ok(Outcome::Inventory(Box::new(cache_test_inventory(
                ToolId::YtDlp,
                "changed-record",
            ))))),
        }))
        .unwrap();
        panel.poll();
        assert!(panel.updates.render_dirty);
        cache_draw(&mut panel, &ctx, vec![]);
        assert_eq!(
            panel.updates.render_snapshots.as_ref().unwrap()[0].installed[0]
                .manifest
                .version,
            "changed-record"
        );
        assert!(!panel.updates.render_dirty);
    }
}

#[cfg(test)]
mod update_integration_regressions {
    use super::*;
    fn panel() -> (
        Panel,
        Receiver<Command>,
        SyncSender<Result<Outcome, String>>,
    ) {
        let (commands, requests) = mpsc::sync_channel(1);
        let (responses, results) = mpsc::sync_channel(1);
        (
            Panel {
                cached: std::array::from_fn(|_| None),
                display_state: [DisplayState::Empty; 3],
                errors: std::array::from_fn(|_| None),
                request_seq: 0,
                pending_dispatch: None,
                background_work: None,
                updates: UpdateState::default(),
                commands,
                results,
                signals: Arc::new(Signals::default()),
                inventory: None,
                initial_refresh_pending: false,
                error: None,
                selected: None,
                auto_selected: None,
                inspection: None,
                automatic_tickets: [None; 3],
                pending_row: None,
                tool: ToolId::YtDlp,
                busy: false,
                context: egui::Context::default(),
            },
            requests,
            responses,
        )
    }
    fn tagged_update(completion: UpdateCompletion, request_seq: u64) -> Outcome {
        Outcome::Tagged {
            tool: completion.tool,
            request_seq,
            result: Box::new(Ok(Outcome::Update(Box::new(completion)))),
        }
    }
    fn observation(tag: &str) -> Observation {
        Observation {
            tool: ReleaseTool::YtDlp,
            release_id: 1,
            tag: tag.into(),
            published_at: "synthetic".into(),
            checked_unix_seconds: 1,
            asset_id: 2,
            asset_name: "yt-dlp.exe".into(),
            asset_url: "https://supply.invalid/test".into(),
            asset_bytes: 1,
            metadata_sha256: "a".repeat(64),
            publisher_trust_verified: false,
        }
    }
    fn item(version: &str) -> InstalledTool {
        InstalledTool {
            manifest: InstallManifest {
                tool: "yt-dlp".into(),
                version: version.into(),
                target: "synthetic".into(),
                source_url: "https://supply.invalid/test".into(),
                sha256: "a".repeat(64),
                license: "synthetic".into(),
                ownership: dlpad_engine::tools::Ownership::Managed,
                executable: "yt-dlp.exe".into(),
            },
            executable: PathBuf::from(format!("synthetic/{version}/yt-dlp.exe")),
            bytes: 1,
        }
    }
    fn inventory() -> Inventory {
        let original = item("1");
        let next = item("2");
        Inventory {
            installed: vec![original.clone(), next],
            active: Some(original),
            recovery: StagingRecovery::default(),
            inspection: None,
            auto_selected: None,
            explicit_selected: None,
            rows: Default::default(),
        }
    }
    #[test]
    fn feature_tool_cancelled_setup_navigation_keeps_origin_receipts_and_cached_inventory() {
        let (mut panel, requests, responses) = panel();
        panel.inventory = Some(inventory());
        panel.submit(Action::Refresh, true);
        let command = requests.try_recv().unwrap();
        panel.signals.cancel.store(true, Ordering::Release); // same request as existing cancel control
        responses
            .send(Ok(Outcome::Tagged {
                tool: command.tool,
                request_seq: command.request_seq,
                result: Box::new(Err(c::TOOL_CANCELLED.into())),
            }))
            .unwrap();
        panel.poll();
        assert!(!panel.busy());
        assert_eq!(panel.error.as_deref(), Some(c::TOOL_CANCELLED));
        panel.select_tool(ToolId::Ffmpeg);
        assert_eq!(panel.selected_tool(), ToolId::Ffmpeg);
        assert!(
            requests.try_recv().is_err(),
            "targeted navigation alone cannot install, select, or execute"
        );
        panel.select_tool(ToolId::YtDlp);
        assert_eq!(panel.inventory.as_ref().unwrap().installed.len(), 2);
        assert_eq!(panel.error.as_deref(), Some(c::TOOL_CANCELLED));
    }
    #[test]
    fn actual_background_labels_follow_dispatched_actions_and_success_without_hiding_cache() {
        let manifest = item("2").manifest;
        for (action, expected, selected) in [
            (Action::Refresh, c::TOOL_BACKGROUND_LOOKUP, false),
            (Action::Install, c::TOOL_BACKGROUND_INSTALL, false),
            (
                Action::Select(manifest.clone()),
                c::TOOL_BACKGROUND_VERIFY,
                false,
            ),
            (Action::Remove(manifest), c::TOOL_BACKGROUND_REMOVE, false),
            (Action::Use, c::TOOL_BACKGROUND_SELECT, true),
        ] {
            let (mut p, requests, responses) = panel();
            p.inventory = Some(inventory());
            assert_eq!(p.background_status(), None);
            p.submit(action, true);
            let command = requests.try_recv().unwrap();
            assert!(p.busy());
            assert_eq!(p.background_status(), Some(expected));
            assert_eq!(p.inventory.as_ref().unwrap().installed.len(), 2);
            assert_eq!(
                p.inventory.as_ref().unwrap().installed[0].manifest.version,
                "1"
            );
            p.submit(Action::Refresh, true);
            assert!(requests.try_recv().is_err(), "busy safety gate unchanged");
            assert_eq!(p.background_status(), Some(expected));
            let outcome = if selected {
                Outcome::Selected {
                    tool: ToolId::YtDlp,
                    path: PathBuf::from("synthetic/1/yt-dlp.exe"),
                }
            } else {
                Outcome::Inventory(Box::new(inventory()))
            };
            responses
                .send(Ok(Outcome::Tagged {
                    tool: command.tool,
                    request_seq: command.request_seq,
                    result: Box::new(Ok(outcome)),
                }))
                .unwrap();
            p.poll();
            assert!(!p.busy());
            assert_eq!(
                p.background_status(),
                None,
                "completion clears running label"
            );
            assert_eq!(p.inventory.as_ref().unwrap().installed.len(), 2);
        }
    }
    #[test]
    fn actual_background_lookup_survives_stale_receipt_navigation_and_failed_refresh() {
        let (mut p, requests, responses) = panel();
        p.inventory = Some(inventory());
        p.submit(Action::Refresh, true);
        let command = requests.try_recv().unwrap();
        responses
            .send(Ok(Outcome::Tagged {
                tool: command.tool,
                request_seq: command.request_seq + 1,
                result: Box::new(Err(c::TOOL_FAILED.into())),
            }))
            .unwrap();
        p.poll();
        assert!(p.busy());
        assert_eq!(p.background_status(), Some(c::TOOL_BACKGROUND_LOOKUP));
        p.choose_tool(ToolId::Ffmpeg);
        assert_eq!(
            p.background_status(),
            Some(c::TOOL_BACKGROUND_LOOKUP),
            "label follows origin job across navigation"
        );
        assert_eq!(
            p.cached[tool_slot(ToolId::YtDlp)]
                .as_ref()
                .unwrap()
                .installed
                .len(),
            2
        );
        responses
            .send(Ok(Outcome::Tagged {
                tool: command.tool,
                request_seq: command.request_seq,
                result: Box::new(Err(c::TOOL_FAILED.into())),
            }))
            .unwrap();
        p.poll();
        assert!(!p.busy());
        assert_eq!(p.background_status(), None);
        p.choose_tool(ToolId::YtDlp);
        assert_eq!(p.inventory.as_ref().unwrap().installed.len(), 2);
        assert_eq!(
            p.inventory.as_ref().unwrap().installed[0].manifest.version,
            "1"
        );
        assert!(matches!(p.display_state[0], DisplayState::Failed));
        assert!(p.error.is_some());
        p.submit(Action::Refresh, true);
        let retry = requests.try_recv().unwrap();
        assert_eq!(p.background_status(), Some(c::TOOL_BACKGROUND_LOOKUP));
        responses
            .send(Ok(Outcome::Tagged {
                tool: retry.tool,
                request_seq: retry.request_seq,
                result: Box::new(Ok(Outcome::Inventory(Box::new(inventory())))),
            }))
            .unwrap();
        p.poll();
        assert_eq!(p.background_status(), None);
        assert!(matches!(p.display_state[0], DisplayState::Fresh));
        assert!(p.error.is_none());
    }
    #[test]
    fn actual_background_version_check_and_verified_use_keep_pending_receipt_policy() {
        let (mut p, requests, responses) = panel();
        p.submit_update(
            update_ui::Intent::Check {
                tool: ToolId::YtDlp,
                generation: 0,
            },
            true,
        );
        let command = requests.try_recv().unwrap();
        assert_eq!(
            p.background_status(),
            Some(c::TOOL_BACKGROUND_VERSION_CHECK)
        );
        responses
            .send(Ok(tagged_update(
                UpdateCompletion {
                    tool: ToolId::YtDlp,
                    generation: 0,
                    kind: UpdateKind::Check,
                    evidence: None,
                    result: Err(c::TOOL_FAILED),
                },
                command.request_seq,
            )))
            .unwrap();
        p.poll();
        assert!(p.busy());
        assert_eq!(
            p.background_status(),
            Some(c::TOOL_BACKGROUND_VERSION_CHECK)
        );
        responses
            .send(Ok(tagged_update(
                UpdateCompletion {
                    tool: ToolId::YtDlp,
                    generation: 1,
                    kind: UpdateKind::Check,
                    evidence: None,
                    result: Ok(UpdateValue::Observed(observation("2"), false)),
                },
                command.request_seq,
            )))
            .unwrap();
        p.poll();
        assert_eq!(p.background_status(), None);
        assert!(!p.busy());
        p.inventory = Some(inventory());
        p.submit_update(
            update_ui::Intent::UseInstalled {
                tool: ToolId::YtDlp,
                generation: 1,
                manifest: item("2").manifest,
            },
            true,
        );
        let command = requests.try_recv().unwrap();
        assert_eq!(p.background_status(), Some(c::TOOL_BACKGROUND_VERIFY));
        responses
            .send(Ok(tagged_update(
                UpdateCompletion {
                    tool: ToolId::YtDlp,
                    generation: 2,
                    kind: UpdateKind::Use,
                    evidence: None,
                    result: Err(c::TOOL_FAILED),
                },
                command.request_seq,
            )))
            .unwrap();
        p.poll();
        assert_eq!(p.background_status(), None);
        assert!(!p.busy());
        assert!(
            p.selected.is_none(),
            "failed verification cannot publish selection"
        );
    }
    #[test]
    fn actual_background_rejected_dispatch_disconnect_and_stopped_worker_never_leave_running_label()
    {
        let (mut p, requests, responses) = panel();
        p.submit(Action::Refresh, false);
        assert_eq!(p.background_status(), None);
        assert!(requests.try_recv().is_err());
        p.submit(Action::Refresh, true);
        requests.try_recv().unwrap();
        assert_eq!(p.background_status(), Some(c::TOOL_BACKGROUND_LOOKUP));
        drop(responses);
        p.poll();
        assert!(!p.busy());
        assert_eq!(p.background_status(), None);
        let (mut p, requests, _responses) = panel();
        p.submit(Action::Refresh, true);
        requests.try_recv().unwrap();
        p.signals.stopped.store(true, Ordering::Release);
        p.poll();
        assert!(!p.busy());
        assert_eq!(p.background_status(), None);
        let (mut p, requests, _responses) = panel();
        drop(requests);
        p.submit(Action::Refresh, true);
        assert!(!p.busy());
        assert_eq!(p.background_status(), None);
        assert!(p.error.is_some());
    }
    #[test]
    fn bounded_update_command_and_actual_poll_ignore_stale_observation_without_old_pin_fallback() {
        let (mut p, requests, responses) = panel();
        p.submit_update(
            update_ui::Intent::Check {
                tool: ToolId::YtDlp,
                generation: 0,
            },
            true,
        );
        let command = requests.try_recv().unwrap();
        assert!(command.automatic_tickets.iter().all(Option::is_none));
        assert!(matches!(
            command.action,
            Action::Update(UpdateRequest::Check { generation: 1 })
        ));
        assert!(p.busy());
        p.submit_update(
            update_ui::Intent::Check {
                tool: ToolId::YtDlp,
                generation: 0,
            },
            true,
        );
        assert!(requests.try_recv().is_err());
        responses
            .send(Ok(tagged_update(
                UpdateCompletion {
                    tool: ToolId::YtDlp,
                    generation: 0,
                    kind: UpdateKind::Check,
                    evidence: None,
                    result: Ok(UpdateValue::Observed(observation("999.0.0"), false)),
                },
                1,
            )))
            .unwrap();
        p.poll();
        assert!(p.busy());
        assert!(p.updates.observed[0].is_none());
        assert_eq!(p.updates.pending, Some((ToolId::YtDlp, 1)));
        responses
            .send(Ok(tagged_update(
                UpdateCompletion {
                    tool: ToolId::YtDlp,
                    generation: 1,
                    kind: UpdateKind::Check,
                    evidence: None,
                    result: Ok(UpdateValue::Observed(observation("999.0.0"), false)),
                },
                1,
            )))
            .unwrap();
        p.poll();
        assert!(!p.busy());
        assert_eq!(p.updates.observed[0].as_ref().unwrap().tag, "999.0.0");
        p.submit_update(
            update_ui::Intent::InstallApprovedObserved {
                tool: ToolId::YtDlp,
                generation: 1,
                version: update_ui::VersionKey {
                    version: "999.0.0".into(),
                    target: "synthetic".into(),
                },
            },
            true,
        );
        assert!(
            requests.try_recv().is_err(),
            "unknown observed release cannot enqueue some old compiled install"
        );
    }
    #[test]
    fn exact_installed_manifest_dispatch_and_close_wait_receipt_preserve_prior_explicit_path() {
        let (mut p, requests, responses) = panel();
        p.inventory = Some(inventory());
        let manifest = p.inventory.as_ref().unwrap().installed[1].manifest.clone();
        let mut altered = manifest.clone();
        altered.sha256 = "b".repeat(64);
        p.submit_update(
            update_ui::Intent::UseInstalled {
                tool: ToolId::YtDlp,
                generation: 0,
                manifest: altered,
            },
            true,
        );
        assert!(
            requests.try_recv().is_err(),
            "same key with different complete manifest must refuse"
        );
        p.selected = Some((ToolId::YtDlp, PathBuf::from("original-explicit-path")));
        p.submit_update(
            update_ui::Intent::UseInstalled {
                tool: ToolId::YtDlp,
                generation: 0,
                manifest: manifest.clone(),
            },
            true,
        );
        let command = requests.try_recv().unwrap();
        assert!(command.automatic_tickets.iter().all(Option::is_none));
        match command.action {
            Action::Update(UpdateRequest::Use {
                generation: 1,
                manifest: actual,
            }) => assert_eq!(
                serde_json::to_vec(&actual).unwrap(),
                serde_json::to_vec(&manifest).unwrap()
            ),
            _ => panic!("exact use command"),
        }
        p.request_stop();
        responses
            .send(Ok(tagged_update(
                UpdateCompletion {
                    tool: ToolId::YtDlp,
                    generation: 1,
                    kind: UpdateKind::Use,
                    evidence: None,
                    result: Ok(UpdateValue::Selected {
                        inventory: Box::new(inventory()),
                        path: PathBuf::from("too-late-path"),
                    }),
                },
                1,
            )))
            .unwrap();
        p.poll();
        assert_eq!(
            p.take_selected_path(),
            Some((ToolId::YtDlp, PathBuf::from("original-explicit-path")))
        );
        assert!(p.take_auto_selected_path().is_none());
    }
    #[test]
    fn inactive_update_inventory_receipt_never_publishes_manual_environment_or_automatic_field_choice()
     {
        let (mut p, _requests, responses) = panel();
        p.set_update_current_sources([
            CurrentSource::External,
            CurrentSource::Unknown,
            CurrentSource::Unknown,
        ]);
        p.selected = Some((
            ToolId::YtDlp,
            PathBuf::from("preserved-manual-env-session-path"),
        ));
        p.busy = true;
        p.updates.pending = Some((ToolId::YtDlp, 7));
        p.pending_dispatch = Some((ToolId::YtDlp, 7));
        p.request_seq = 7;
        responses
            .send(Ok(tagged_update(
                UpdateCompletion {
                    tool: ToolId::YtDlp,
                    generation: 7,
                    kind: UpdateKind::Install,
                    evidence: None,
                    result: Ok(UpdateValue::Inventory(Box::new(inventory()))),
                },
                7,
            )))
            .unwrap();
        p.poll();
        assert_eq!(
            p.take_selected_path(),
            Some((
                ToolId::YtDlp,
                PathBuf::from("preserved-manual-env-session-path")
            ))
        );
        assert!(p.take_auto_selected_path().is_none());
        assert_eq!(
            p.inventory
                .as_ref()
                .unwrap()
                .active
                .as_ref()
                .unwrap()
                .manifest
                .version,
            "1"
        );
        assert_eq!(p.inventory.as_ref().unwrap().installed.len(), 2);
        assert!(matches!(
            p.update_snapshots()[0].current,
            update_ui::CurrentVersion::External
        ));
    }
    #[test]
    fn accepted_dynamic_capture_dispatches_exact_version_once_without_recapture_or_auto_ticket() {
        let (mut p, requests, responses) = panel();
        // Enter through the real dispatcher so its command generation advances.
        // Manually setting pending(1) left serial at0, an unreachable fixture state.
        p.submit_update(
            update_ui::Intent::Check {
                tool: ToolId::YtDlp,
                generation: 0,
            },
            true,
        );
        let check = requests.try_recv().expect("actual Check dispatch");
        assert!(check.automatic_tickets.iter().all(Option::is_none));
        assert_eq!(check.tool, ToolId::YtDlp);
        assert_eq!(check.request_seq, 1);
        assert!(matches!(
            check.action,
            Action::Update(UpdateRequest::Check { generation: 1 })
        ));
        assert_eq!(p.updates.serial, 1);
        assert_eq!(p.updates.pending, Some((ToolId::YtDlp, 1)));
        assert_eq!(p.pending_dispatch, Some((ToolId::YtDlp, 1)));
        assert!(p.busy);
        responses
            .send(Ok(tagged_update(
                UpdateCompletion {
                    tool: ToolId::YtDlp,
                    generation: 1,
                    kind: UpdateKind::Check,
                    evidence: None,
                    result: Ok(UpdateValue::Observed(observation("999.0.0"), true)),
                },
                1,
            )))
            .unwrap();
        p.poll();
        let snapshots = p.update_snapshots();
        let eligible = matches!(
            snapshots[0].observed.as_ref().unwrap().install,
            update_ui::Eligibility::Ready
        );
        assert_eq!(
            eligible,
            cfg!(all(windows, target_arch = "x86_64", feature = "cpu"))
        );
        if !eligible {
            return;
        }
        p.submit_update(
            update_ui::Intent::InstallApprovedObserved {
                tool: ToolId::YtDlp,
                generation: 1,
                version: update_ui::VersionKey {
                    version: "different".into(),
                    target: "x86_64-pc-windows-msvc".into(),
                },
            },
            true,
        );
        assert!(requests.try_recv().is_err());
        assert_eq!(
            p.updates.serial, 1,
            "rejected version cannot advance command generation"
        );
        assert!(
            p.updates.captured[0],
            "rejected version cannot consume captured observation"
        );
        p.submit_update(
            update_ui::Intent::InstallApprovedObserved {
                tool: ToolId::YtDlp,
                generation: 1,
                version: update_ui::VersionKey {
                    version: "999.0.0".into(),
                    target: "x86_64-pc-windows-msvc".into(),
                },
            },
            true,
        );
        let request = requests.try_recv().unwrap();
        assert!(request.automatic_tickets.iter().all(Option::is_none));
        let install_sequence = request.request_seq;
        assert_eq!(install_sequence, 2);
        assert_eq!(request.tool, ToolId::YtDlp);
        let Action::Update(UpdateRequest::Install {
            generation,
            observation_generation,
            version,
        }) = request.action
        else {
            panic!("exact install command")
        };
        assert_eq!((generation, observation_generation), (2, 1));
        assert_eq!(version.version, "999.0.0");
        assert!(!p.updates.captured[0]);
        assert!(p.take_selected_path().is_none());
        assert!(p.take_auto_selected_path().is_none());
        assert!(
            requests.try_recv().is_err(),
            "only one accepted Install queued"
        );
        // An actual matching failed receipt ends busy but cannot restore a consumed pin.
        responses
            .send(Ok(tagged_update(
                UpdateCompletion {
                    tool: ToolId::YtDlp,
                    generation: 2,
                    kind: UpdateKind::Install,
                    evidence: None,
                    result: Err(c::TOOL_UPDATE_UNVERIFIED),
                },
                install_sequence,
            )))
            .unwrap();
        p.poll();
        assert!(!p.busy);
        assert_eq!(p.updates.serial, 2);
        assert!(!p.updates.captured[0]);
        p.submit_update(
            update_ui::Intent::InstallApprovedObserved {
                tool: ToolId::YtDlp,
                generation: 2,
                version: update_ui::VersionKey {
                    version: "999.0.0".into(),
                    target: "x86_64-pc-windows-msvc".into(),
                },
            },
            true,
        );
        assert!(
            requests.try_recv().is_err(),
            "failed accepted install needs a new Check, not recapture/replay"
        );
        assert_eq!(p.updates.serial, 2);
        assert!(p.take_selected_path().is_none());
        assert!(p.take_auto_selected_path().is_none());
    }
    #[test]
    fn failed_completion_keeps_partial_inactive_publication_and_exact_evidence_path_without_selection()
     {
        let (mut p, _requests, responses) = panel();
        let original = PathBuf::from("manual/original.exe");
        p.selected = Some((ToolId::YtDlp, original.clone()));
        p.updates.pending = Some((ToolId::YtDlp, 4));
        p.pending_dispatch = Some((ToolId::YtDlp, 4));
        p.request_seq = 4;
        p.busy = true;
        let exact = PathBuf::from("owned/attempt-0000000000000004");
        responses
            .send(Ok(tagged_update(
                UpdateCompletion {
                    tool: ToolId::YtDlp,
                    generation: 4,
                    kind: UpdateKind::Install,
                    evidence: Some(RetainedEvidence {
                        directory: exact.clone(),
                        state: EvidenceState::InstalledIncomplete,
                        installed: Some(item("2")),
                    }),
                    result: Err(c::TOOL_UPDATE_RECEIPT_UNAVAILABLE),
                },
                4,
            )))
            .unwrap();
        p.poll();
        assert!(!p.busy());
        assert!(matches!(p.updates.status[0], update_ui::Status::Failed(_)));
        let evidence = p.updates.evidence[0].as_ref().unwrap();
        assert_eq!(evidence.directory, exact);
        assert!(matches!(evidence.state, EvidenceState::InstalledIncomplete));
        assert_eq!(evidence.installed.as_ref().unwrap().manifest.version, "2");
        assert_eq!(p.take_selected_path(), Some((ToolId::YtDlp, original)));
        assert!(p.take_auto_selected_path().is_none());
    }
    #[test]
    fn unknown_deno_ffmpeg_observations_never_gain_dynamic_install_eligibility() {
        let (mut p, _requests, _responses) = panel();
        for tool in [ToolId::Deno, ToolId::Ffmpeg] {
            p.tool = tool;
            let slot = tool_slot(tool);
            let mut observed = observation("unknown-new-version");
            observed.tool = release_tool(tool);
            p.updates.observed[slot] = Some(observed);
            p.updates.captured[slot] = true;
            let snapshots = p.update_snapshots();
            let row = snapshots.iter().find(|s| s.tool == tool).unwrap();
            assert!(matches!(
                row.observed.as_ref().unwrap().install,
                update_ui::Eligibility::Unavailable(_)
            ));
        }
    }
    #[test]
    fn serialized_receipt_cannot_authorize_install_without_worker_captured_pin() {
        let root = std::env::temp_dir().join(format!(
            "dlpad-ui-receipt-no-authority-{}",
            std::process::id()
        ));
        std::fs::create_dir(&root).unwrap();
        std::fs::write(
            root.join("supply-receipt.json"),
            b"{\"tool\":\"yt-dlp\",\"stage\":\"prepared-before-install\"}",
        )
        .unwrap();
        let mut cache: UpdateCache = std::array::from_fn(|_| None);
        let mut rows = Default::default();
        let done = perform_update(
            &root,
            ToolId::YtDlp,
            UpdateRequest::Install {
                generation: 1,
                observation_generation: 1,
                version: update_ui::VersionKey {
                    version: "fake".into(),
                    target: "x86_64-pc-windows-msvc".into(),
                },
            },
            &Arc::new(AtomicBool::new(false)),
            &mut rows,
            &mut cache,
        );
        assert!(matches!(done.result, Err(c::TOOL_UPDATE_UNVERIFIED)));
        assert!(done.evidence.is_none());
        assert!(!root.join("managed-tools").exists());
        assert!(root.join("supply-receipt.json").is_file());
        std::fs::remove_file(root.join("supply-receipt.json")).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
    fn seeded_cross_tab_panel() -> (
        Panel,
        Receiver<Command>,
        SyncSender<Result<Outcome, String>>,
    ) {
        let (mut panel, requests, responses) = panel();
        panel.inventory = Some(inventory());
        let mut deno = inventory();
        for item in &mut deno.installed {
            item.manifest.tool = "deno".into();
        }
        deno.active = None;
        panel.cached[tool_slot(ToolId::Deno)] = Some(deno);
        panel.display_state = [DisplayState::Fresh; 3];
        (panel, requests, responses)
    }
    #[test]
    fn merged_update_outer_sequence_and_inner_release_origin_reject_stale_without_clearing_pending()
    {
        let (mut p, requests, responses) = seeded_cross_tab_panel();
        p.submit_update(
            update_ui::Intent::Check {
                tool: ToolId::YtDlp,
                generation: 0,
            },
            true,
        );
        let cmd = requests.try_recv().unwrap();
        assert_eq!(cmd.request_seq, 1);
        p.choose_tool(ToolId::Deno);
        let complete = || UpdateCompletion {
            tool: ToolId::YtDlp,
            generation: 1,
            kind: UpdateKind::Check,
            evidence: None,
            result: Ok(UpdateValue::Observed(observation("999.0.0"), true)),
        };
        responses
            .send(Ok(tagged_update(complete(), cmd.request_seq + 1)))
            .unwrap();
        p.poll();
        assert!(p.busy());
        assert_eq!(p.pending_dispatch, Some((ToolId::YtDlp, cmd.request_seq)));
        assert!(p.updates.observed[0].is_none());
        let mut wrong_release = complete();
        let Ok(UpdateValue::Observed(o, _)) = &mut wrong_release.result else {
            panic!("observation fixture")
        };
        o.tool = ReleaseTool::Deno;
        responses
            .send(Ok(tagged_update(wrong_release, cmd.request_seq)))
            .unwrap();
        p.poll();
        assert!(p.busy());
        assert_eq!(p.updates.pending, Some((ToolId::YtDlp, 1)));
        assert!(p.updates.observed[0].is_none());
        responses
            .send(Ok(tagged_update(complete(), cmd.request_seq)))
            .unwrap();
        p.poll();
        assert!(!p.busy());
        assert!(p.pending_dispatch.is_none());
        assert!(p.updates.pending.is_none());
        assert_eq!(p.tool, ToolId::Deno);
        assert_eq!(
            p.inventory.as_ref().unwrap().installed[0].manifest.tool,
            "deno"
        );
        assert_eq!(p.updates.observed[0].as_ref().unwrap().tag, "999.0.0");
        assert!(p.updates.observed[tool_slot(ToolId::Deno)].is_none());
        assert_eq!(
            p.cached[0]
                .as_ref()
                .unwrap()
                .active
                .as_ref()
                .unwrap()
                .manifest
                .version,
            "1"
        );
        assert!(p.take_selected_path().is_none());
        assert!(p.take_auto_selected_path().is_none());
    }
    #[test]
    fn merged_verified_update_use_ack_publishes_origin_a_after_readonly_b_navigation_only_exact_sequence()
     {
        let (mut p, requests, responses) = seeded_cross_tab_panel();
        let target = p.inventory.as_ref().unwrap().installed[1].manifest.clone();
        p.submit_update(
            update_ui::Intent::UseInstalled {
                tool: ToolId::YtDlp,
                generation: 0,
                manifest: target,
            },
            true,
        );
        let command = requests.try_recv().unwrap();
        assert!(command.automatic_tickets.iter().all(Option::is_none));
        p.choose_tool(ToolId::Deno);
        let source = PathBuf::from("origin-a-exact-worker-use-receipt");
        let complete = || {
            let mut inventory = inventory();
            inventory.active = Some(item("2"));
            UpdateCompletion {
                tool: ToolId::YtDlp,
                generation: 1,
                kind: UpdateKind::Use,
                evidence: None,
                result: Ok(UpdateValue::Selected {
                    inventory: Box::new(inventory),
                    path: source.clone(),
                }),
            }
        };
        responses
            .send(Ok(tagged_update(complete(), command.request_seq + 1)))
            .unwrap();
        p.poll();
        assert!(p.busy());
        assert!(p.take_selected_path().is_none());
        responses
            .send(Ok(tagged_update(complete(), command.request_seq)))
            .unwrap();
        p.poll();
        assert_eq!(p.take_selected_path(), Some((ToolId::YtDlp, source)));
        assert!(p.take_auto_selected_path().is_none());
        assert_eq!(p.tool, ToolId::Deno);
        assert_eq!(
            p.inventory.as_ref().unwrap().installed[0].manifest.tool,
            "deno"
        );
        assert_eq!(
            p.cached[0]
                .as_ref()
                .unwrap()
                .active
                .as_ref()
                .unwrap()
                .manifest
                .version,
            "2"
        );
        assert!(!p.busy());
        assert!(p.pending_dispatch.is_none());
    }
    #[test]
    fn merged_worker_update_error_uses_outer_origin_and_sequence_without_overwriting_visible_b() {
        let (mut p, requests, responses) = seeded_cross_tab_panel();
        p.submit_update(
            update_ui::Intent::Check {
                tool: ToolId::YtDlp,
                generation: 0,
            },
            true,
        );
        let command = requests.try_recv().unwrap();
        p.choose_tool(ToolId::Deno);
        let error = |request_seq| Outcome::Tagged {
            tool: ToolId::YtDlp,
            request_seq,
            result: Box::new(Err(c::TOOL_HASH_FAILED.into())),
        };
        responses.send(Ok(error(command.request_seq + 1))).unwrap();
        p.poll();
        assert!(p.busy());
        assert!(p.errors[0].is_none());
        responses.send(Ok(error(command.request_seq))).unwrap();
        p.poll();
        assert!(!p.busy());
        assert!(p.pending_dispatch.is_none());
        assert!(p.updates.pending.is_none());
        assert_eq!(p.errors[0].as_deref(), Some(c::TOOL_HASH_FAILED));
        assert!(p.error.is_none());
        assert!(p.errors[tool_slot(ToolId::Deno)].is_none());
        assert!(matches!(p.display_state[0], DisplayState::Failed));
        assert!(matches!(p.updates.status[0], update_ui::Status::Failed(_)));
        assert_eq!(p.tool, ToolId::Deno);
        assert_eq!(
            p.inventory.as_ref().unwrap().installed[0].manifest.tool,
            "deno"
        );
        assert!(p.cached[0].is_some());
        assert!(p.take_selected_path().is_none());
        assert!(p.take_auto_selected_path().is_none());
    }
}

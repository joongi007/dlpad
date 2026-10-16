# S00 Linux isolated Hangul IME experiment

## Scope and result

Linux/WSL2, Iced 0.14 / iced_winit 0.14.1 / winit 0.30.13, tiny-skia, owned Xvfb and private D-Bus. This does not verify Windows/macOS, Wayland, accessibility, or a user desktop. S00 IME acceptance remains **incomplete**: real key composition and commit work; readable inline preedit has not been demonstrated.

`python3 scripts/ime_probe.py target/release/dlpad /tmp/dlpad-ime-probe-20261001-fonts` exited 1, intentionally preserving the preedit failure gate. Physical ASCII keys `gksrmf` followed by Space committed exactly `한글 `. Active Hangul `gk` followed by two BackSpace keys cancelled the composition without changing committed text. Switching to inactive IME and typing `abc` produced exactly `한글 하abc`. Escape committed the pending `하`: an independent GTK3 XIM entry produced the same `한글 하`, so Escape is not reported as composition cancellation. No Unicode clipboard paste generated these results; clipboard reads only inspected committed text.

## Preedit diagnosis and corrected hypothesis

Initial legacy-XIM screenshots showed a dark 31×31 block at app-local (46,315). Owned X window inventory identified it as the separate **Fcitx5 Input Window**, root (206,445), while the app origin was (160,130). Temporary bounded instrumentation (32 input-method requests, 32 preedit updates, 16 draws maximum) observed empty Iced preedit content and no Preedit update/draw calls. Moving Iced selection background before glyphs had no effect. Therefore this experiment does not establish an Iced glyph draw-order defect.

Setting only the owned Fcitx configuration `UseOnTheSpot=True` was confirmed through its private D-Bus GetConfig. Xlib queryInputStyle reported [1028,1026,1032,516,514,520], including callback style 1026. The external Fcitx Input Window moved to root (160,780), outside the app (160,130,960,640); inline preedit remained absent. Adding a temporary fontconfig directory with a symlink to the existing Malgun font did not resolve it. Callback-style advertisement alone does not prove that the application selected it or received callbacks. The remaining boundary is XIM context/style/event delivery, not yet isolated to a specific component.

The pixel check deliberately fails on the empty inline crop. Contrast counts are supporting evidence and require screenshot review; white background is not proof of readable composition. Evidence folders under `/tmp/dlpad-ime-probe-20261001-{inventory,onspot,fonts,baseline}` contain PNGs, owned window inventory, configuration observation and exact committed text. They are temporary local artifacts, not repository fixtures or proof of all OS compatibility.

[Exact fcitx5-hangul 5.0.7 source](https://raw.githubusercontent.com/fcitx/fcitx5-hangul/5.0.7/src/engine.cpp) handles BackSpace through hangul_ic_backspace and flushes unhandled keys; Escape behavior was independently observed in GTK. [Fcitx5 5.0.14 XIM configuration](https://raw.githubusercontent.com/fcitx/fcitx5/5.0.14/src/frontend/xim/xim.h) defaults UseOnTheSpot to false; [its frontend implementation](https://raw.githubusercontent.com/fcitx/fcitx5/5.0.14/src/frontend/xim/xim.cpp) provides the advertised styles. Local winit 0.30.13 source in src/platform_impl/linux/x11/ime/input_method.rs prefers callbacks when available, but actual application context selection is not proven by this experiment.

## Temporary vendoring and cleanup

The experimental vendor copy came from iced_winit 0.14.1, crate SHA256 `7589888c8e951899cc688247a69933bb7a0511f0b4b2e122ac3fcd5dacb37f72`, upstream VCS `38237dd294da3256c2ac4d3f8288fc73956c91e0`, path `winit`. Selective source + normalized Cargo + README + MIT license cost approximately 164KiB (compressed original crate 52,352 bytes); no registry-cache source was modified. [Original license](https://raw.githubusercontent.com/iced-rs/iced/38237dd294da3256c2ac4d3f8288fc73956c91e0/LICENSE) was preserved.

The ineffective draw-order change and all temporary Rust trace instrumentation were removed. vendor/src/window.rs is byte-identical to the original registry source. The probe no longer enables DLPAD_IME_TRACE. There is no retained product-source fix from this hypothesis; the integration owner removes the Cargo path patch and restores the registry lock entry. Vendoring exposed two pre-existing deprecated try_next warnings and must not be kept merely to suppress them.

## Isolation and costs

Project-only Ubuntu Jammy Fcitx5 5.0.14-1 / Hangul 5.0.7-1 dependencies use 5,496,506 archive bytes and 20,453,529 extracted bytes. These test tools are not bundled with the product. PRoot maps the extracted libhangul dictionary directory; no system installation or user IME configuration changes were made. Every run owns a virtual display, empty Xauthority file, private D-Bus, temporary XDG directories and process groups. HOME is unchanged. No visible-desktop fallback occurs. Existing Windows font files are read only, not redistributed.

Baseline build after restoring the exact upstream source: `cargo build -p dlpad --bin dlpad --release --locked`, exit 0, 45.59s; two upstream vendoring deprecation warnings. `PYTHONPYCACHEPREFIX=/tmp/dlpad-ime-pycache python3 -m py_compile scripts/ime_probe.py` passed. Final baseline `python3 scripts/ime_probe.py target/release/dlpad /tmp/dlpad-ime-probe-20261001-baseline` exited 1 with commit=true, cancel=true, Latin transition=true, GTK same Escape=true, preedit=false. `python3 scripts/check_docs.py` passed (93 documents). Exact upstream window.rs comparison passed and no DLPAD_IME_TRACE remains in vendor source or probe.

# CPU 접근성 host 구현 검증 범위

2026-10-03, S00/R08. `software_host.rs`는 기존 Candidate UI를 그대로 실행하고 egui-winit의 AccessKit 초기화·native event·action request·platform output 경로를 직접 연결한다. 외부 CPU backend의 접근성 없는 helper app은 사용하지 않는다. IME와 clipboard는 같은 egui-winit adapter가 처리하며 CloseRequested/CancelClose는 Candidate의 작업 종료 정책에 전달한다.

repaint 요청은 shared earliest-deadline와 queued bit로 pending wake를 한 개로 합친다. event loop는 기본 Wait이며 deadline에만 WaitUntil로 깨어난다. 화면은 최대 16M pixels, texture는 총 4M pixels/64개/side4096, geometry는 vertices1M/indices3M로 제한한다. backend가 GPU callback을 조용히 무시하지 않도록 host가 거절한다. 최소화/zero-size에서는 owned 1x1 dummy buffer와 empty primitives로 검증된 texture 변경만 적용하고 실제 surface resize/present를 하지 않는다. 다음 유효 크기에서 전체 UI를 tessellate한다. softbuffer XRGB의 상위 byte는 0으로 지운다.

View drop 순서는 input(clipboard/AccessKit), surface, window, context이며 Wayland clipboard lifetime을 보존한다. 새 renderer archive3개 총505827bytes의 pin/source 근거는 software-renderer-pin.json에 있다. font/renderer primitive cache/native buffers의 실제 RSS는 제한값만으로 보장되지 않으므로 별도 실제 실측을 인수 조건으로 유지한다.

직접 실행: `rustfmt --edition 2024 crates/app/src/software_host.rs`, exit0. Cargo/lock은 통합 담당 소유이고 host freeze 이후 check/test/build를 수행한다. 작성한 5개 회귀시험은 zero-size/frame cap, repaint coalescing, malformed partial texture rollback, hidden font texture와 visible partial update, invalid mesh index를 다룬다. 이 문서 시점에는 아직 실행하지 않았다.

미검증: CPU binary의 실제80MiB/idleCPU, private virtual GUI의 AccessKit reader·IME·clipboard·Pause/StopExit, DPI/zero-size 실제window, Windows/macOS/Wayland actual 동작, 10000항목 데이터와 runtime cache 비용. 이전 Glow SHA에 대한 측정을 새 CPU 성공으로 확대하지 않는다. screenshot viewport command는 현재 fail-closed이며 제품 Candidate는 해당 command를 요청하지 않는다.

후속 실제 실행: docs/validation/windows-cpu-feature.md에 Windows GNU CPU feature Clippy/test/release exit0 및 host5회귀 통과를 기록했다. 실제 GUI/RSS/3OS 게이트는 유지한다.

## 실제 Windows 첫 GUI 실패와 초기화 순서 수정

통합 담당의 실제 private desktop GUI 첫 실행은 실패했다. `docs/validation/evidence/windows-cpu-cli-startup-2.log`에 accesskit_winit0.29.2 lib198의 "adapter must be created before window is shown" panic과 input desktop unchanged가 기록돼 있다. 앞선 Windows CLI의 compile/test 성공은 이 native GUI 초기화 순서를 검증하지 못했다.

전용 host에서 모든 OS의 창을 `with_visible(false)`로 생성하고, surface와 egui-winit State 및 init_accesskit 준비를 마친 View를 보관한 뒤 `set_visible(true)`를 호출하도록 수정했다. 담당 파일 rustfmt exit0. Cargo/GUI/build는 실행하지 않았으며 새 source/binary의 Linux 및 Windows 실제 GUI 재검증은 통합 담당이 직렬 수행한다. 최초 CLI binary SHA에 대한 성공 범위는 그대로 보존한다.

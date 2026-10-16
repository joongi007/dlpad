# S00: 같은 접근성 UI를 사용하는 CPU renderer host

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R08, S00 (R11의 3OS 동작은 별도 미검증 게이트)
- 담당/수정 범위: library_identity — 신규 crates/app/src/software_host.rs 및 전용 validation/workitem
- 선행 작업: root Cargo exactpins/Candidate 공유API/CPU entry 연결

## 사용자 결과

Rust native egui의 같은 다운로드/Pause/StopExit UI와 AccessKit/IME를 유지하며 GL/LLVM 없이 CPU로 그리는 수직 후보를 구현한다. 메모리80MiB 인수는 실제 실측으로 판단한다.

## 계약과 범위

pub run(data)->Result<(),Box<dynError>>, Candidate::new/show_ui, configure_context/font_bytes를 root가 제공한다. rendereronly egui_software_backend0.0.2/std, egui_winit0.33.3/accesskit, softbuffer0.4.8,winit0.30.13을 root가 Cargo에 연결한다. 신규host만 수정하며 기존제품/Cargo/lock/harness수정 또는 공유합의 전 빌드하지 않는다. 보조library의 run_app는 AccessKit 경로가 없어 사용하지 않는다.

## 인수 조건

- init_accesskit/플랫폼event/ActionRequest/tree 업데이트와 실제 IME·clipboard 경로를 유지한다.
- CloseRequested는Candidate로전달하고CancelClose/StopExit완료를 존중한다.
- repaint wake 한개+합친 deadline, 기본 Wait로 idle연속60FPS를 피한다.
- zero size skip, frame16Mpixel/texture·primitive예산 및 softbufferXRGB상위byte를 확인한다.
- sourcefreeze후 root가컴파일/통합검증, private Xvfb실제CPU/RSS/reader/IME를 확인한다.

## 검증과 handoff

신규 host 구현 및 담당 파일 rustfmt exit0 완료. 5개 회귀시험 작성, 컴파일/test는 root 통합 대기. docs/validation/cpu-accessible-host.md에 구현 계약과 미검증 범위를 기록했다. 이전Glow숫자를CPU성공으로사용하지않는다. 예산완화/접근성제거/egui0.34wholeupgrade 없음.

실제 Windows 첫 GUI에서 AccessKit-before-show panic 재현됨. 숨긴 native window → surface/input/accesskit 준비 → host View 보관 → 최초 show 순서로 전용 host만 수정했다. own rustfmt exit0, 실제 새 binary GUI 확인 전 완료로 전환하지 않는다. 이전 CLI unit 성공과 native GUI 인수를 분리한다.

# winit vendor 패치 후 Windows GNU 재검증

2026-10-02. 주 담당이 고정한 winit0.30.13 vendor path patch/Cargo.lock를 실제 Windows GNU native 환경에서 다시 검증했다. 이전 Windows53source 증거를 새 입력의 증거로 확대하지 않았다. [작업 계약](../work-items/S00-windows-winit-revalidation.md), [최종 입력과 결과](windows-winit-final.json)를 보존한다.

## 실제 실행과 결과

`python3 scripts/windows_winit_revalidate.py`가 기존 native runner를 순차 실행했다. 사용한 argv는 operation별 inputs JSON과 최종 manifest에 있다.

| Native 명령 | 결과 | 실제 경과 |
|---|---|---|
| cargo fmt --all -- --check | exit0 | 5.42초 |
| cargo clippy --workspace --all-targets --locked -- -D warnings | exit0, 경고0 | 11.53초 |
| cargo test --workspace --locked | exit0 | 20.72초 |
| cargo build --workspace --release --locked | exit0 | 28.31초 |

시험은 app14/core21/engine71/diagnostic3/Windows process6 pass, engine1 ignored다. Windows activation12 native시험이 engine에 포함된다. Unix 전용 guardian/local_media/local_media_failures/download_soak의 Windows0tests를 해당 기능 검증으로 계산하지 않는다. 시험에서 별도 same-user client IPC, 소유 프로세스 자손/취소/timeout/부모crash 회귀가 통과했다.

최종 `dlpad.exe`는 6,807,040 bytes, SHA-256 `85cdad770ad3c1db25c25f3c50d6193636cc8b15b7d705f253b314ef48ffc59b`다. 실제 build log가 `winit v0.30.13 (D:\project\dlpad\vendor\winit)` 컴파일을 확인한다.

## 동일 입력과 격리

기존 windows_native_probe.py의 snapshot은 workspace Rust/Cargo.toml, root Cargo.lock/toolchain과 vendor Rust/Cargo.toml을 포함한다. 신규 orchestrator는 vendor의 모든 파일(매니페스트·build.rs·플랫폼 소스·기타 assets), CLI runner/native host 소스·exe를 더한 총278개의 입력을 해시한다. 네 명령 각각의 baseline/before/after가 모두 동일하며 native runner의 before/after도 동일하다. 신규 orchestrator만 추가했고 기존 runner/product/Cargo/schema를 변경하지 않았다.

네 실행 모두 native host가 `private-desktop=true`, `dlpad-input-desktop-unchanged=True`를 보고했다. 사용자 desktop/SwitchDesktop/visible fallback을 사용하지 않았다. 기존 정상 Git 저장소가 없어 .git을 보존했다. 네 원본 로그 및 runner input JSON을 `windows-winit-*` 전용 자료로 복사했다.

## 인수의 한계

이 증거는 새 dependency snapshot의 실제 Windows GNU CLI 컴파일/시험/릴리스 빌드다. Windows MSVC·macOS·실제 GUI 한글 preedit 시각과 접근성·IME 후보창·백그라운드 창 복귀를 검증하지 않는다. Linux vendor 패치 결과와도 OS 시각 인수를 합치지 않는다. 이후 새 source가 추가되면 여기 보존한 baseline에 포함된 입력까지만 성공을 주장한다. 원격 CI나 공개 release는 수행하지 않았다.

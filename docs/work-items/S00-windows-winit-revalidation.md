# S00: winit vendor 패치 후 Windows native 재검증

- 상태: done (Windows GNU native CLI; GUI/IME 게이트는 별도)
- 분류: test
- 우선순위: P0
- 요구사항: R11, R10
- 담당/수정 범위: library_identity — 전용 workitem/validation 증거, 필요시 신규 재검증 script
- 선행 작업: 주 담당 Cargo.lock/winit0.30.13 vendor patch 고정, 기존 native CLI private desktop 환경

## 사용자 결과

Linux에서 검증한 새 IME vendor 패치가 Windows GNU 전체 빌드/회귀 시험도 통과하는지 실제 Windows에서 확인한다. 이 결과로 Windows GUI IME 완료를 주장하지 않는다.

## 계약과 범위

기존 windows_native_probe.py는 workspace Rust/Cargo/toolchain과 vendor의 모든 Rust/Cargo.toml을 before/after hash하며 변경 시 exit3한다. build.rs도 Rust glob에 포함한다. 별도 신규 orchestrator는 vendor 전체 파일 및 runner/native host까지 추가 hash해 동일입력을 확인하고4실행로그를 전용증거로 보존한다. 기존 script/product/Cargo/schema는 변경하지 않는다. 사용자 desktop으로 전환하지 않고 기존 소유 private desktop native host만 사용한다. 정상 Git 저장소가 없어 .git을 보존한다.

## 인수 조건

- 실제 native fmt/clippy/workspace tests/release를 순차 실행하고 정확한 argv·exit·source manifest를 보존한다.
- vendor 패치 입력이 포함되고 네 명령의 before/after가 모두 동일하다.
- 실패는 성공으로 기록하지 않고 담당자에게 전달한다.
- Windows GNU와 MSVC/macOS/GUI·IME 인수를 구분한다.

## 검증과 handoff

새 vendor 전체 파일 포함278input의 네 명령 baseline/before/after가 모두 동일하며 fmt/clippy/test/release exit0다. [실제 결과](../validation/windows-winit-validation.md), [최종 manifest](../validation/windows-winit-final.json)에 로그·argv·hash를 보존했다. Windows app14/core21/engine71/diagnostic3/process6 pass, engine1 ignored다. 이전 Windows53source 증거는 새 snapshot으로 확대하지 않는다. MSVC/macOS/Windows GUI IME는 미검증이다.

# S00: Windows CLI 환경 조사와 격리 GUI 게이트

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R11, R08, R02, R06
- 담당/수정 범위: library_identity 에이전트; 본 문서, docs/validation/windows-cli.md, 신규 scripts/windows_environment.ps1/windows_desktop_capability.ps1/windows_tools_setup.py/windows_native_probe.py/windows_cli_host.cs/windows_cli_fixture.cs
- 선행 작업: Linux 검증, 주 담당의 도구 설치 선택 조정

## 사용자 결과

실제 Windows native 빌드/시험 가능 여부를 로컬 증거로 확인한다. WSL Linux 통과와 Windows 통과를 구분하며 사용자 데스크톱에 시험 창을 띄우지 않는다.

## 계약과 범위

먼저 WSL interop와 기존 native 도구를 읽기 전용으로 조사한다. Windows CLI는 hidden/no-console 방식으로 실행하고 시스템/기존 사용자 데이터/설정을 변경하지 않는다. 추가 설치가 필요하면 시간·용량·툴 선택을 주 담당에게 먼저 보고한다. Rust/Cargo/lock/UI/기존 시험 script는 수정하지 않는다. Git 저장소가 없고 .git을 보존하며 원격 CI 실행을 주장하지 않는다.

## 인수 조건

- 실제 로컬 Windows 접근 성공/실패와 권한 모드를 기록한다.
- 기존 Rust/MSVC/native build 도구 존재를 확인한다.
- CLI와 private desktop GUI/IME/a11y 검증의 한계를 구분한다.
- 실제 Windows 시험 없는 CI YAML/크로스 컴파일만으로 OS 완료를 보고하지 않는다.

## 검증과 handoff

실제 Windows 접근과 프로젝트 전용 Rust 1.94.0/LLVM-MinGW 설치를 완료했다. 다운로드 322926828바이트, 도구/archives 약1.89GB, 추가 cargo/target 별도. 최초20~40분 추정은 예산이며 실제 전체 소요시간은 별도 측정하지 않았다. private CLI timeout/자손 정리/무관한 fixture 생존/input desktop 불변을 실증했다. native fmt/check 성공, test에서 Windows canonical prefix 오류11개를 발견하고 주 담당/해당 파일 담당에게 fix 요청했다. [검증 기록](../validation/windows-cli.md)에 실패 로그와 미완료 Windows MSVC/GUI/IME/macOS 게이트를 기록했다. 최종 동일 native Windows GNU source snapshot에서 fmt/clippy/test82개/release 모두 성공했다. release앱6723584바이트, hash/원시로그를 보존했다. R11 전체 3 OS/원격 CI/MSVC와 S00 GUI/IME는 완료라고 주장하지 않는다.

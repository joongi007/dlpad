# Windows CPU 공통 스타일 높이 수정 CLI 검증

2026-10-03. root가 accessible.rs configure_context의 현재 스타일 설정을 all_styles_mut spacing32로 수정한 동결 소스를 검증했다. 기존 프로젝트 Windows GNU runner의 CPU clippy/test/release가 각각 exit0, 6.29/20.84/18.35초이며 실제 188 passed/1 ignored다. 세 private CLI 로그에서 input-desktop-unchanged=True를 확인했다.

238개 compiled input은 각 작업 전후와 작업간·현재소스 대조가 모두 동일하고 runner/lock SHA는 불변이다. 정확한 해시·argv·시간과 원시 로그/manifest는 windows-cpu-appearance-sizing/result.json 및 같은 폴더에 보존했다. 새 .local-tools/windows/target/release/dlpad-cpu.exe는 5,553,152바이트, SHA256 d210a7828f384d4422d456c103c6acdaaf5002ae1b0b156d6b0ad91b291bbebf다.

이전 appearance와 popupfix 증거는 해당 이전 소스의 결과로 보존한다. CLI 검증은 실제 32px 높이·비포인터 메뉴 닫힘·Tab focus·테마·IME의 GUI 인수를 대체하지 않는다. 사용자 desktop/clipboard는 접근하지 않았으며 MSVC/macOS/공개 릴리스는 실행하지 않았다. 제품/Cargo/runner 수정 없이 Windows 전용 target을 사용했다.

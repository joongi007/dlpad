# Windows CPU 비포인터 메뉴 닫힘 수정 CLI 증거

2026-10-03. accessible_settings.rs의 테마·배율 선택 응답 clicked 시 ui.close()를 호출하는 수정 소스로 기존 project-only Windows GNU Rust runner의 CPU clippy/test/release를 수행했다. Private CLI desktop과 소유 Job으로 실행했고 세 로그의 input-desktop-unchanged=True를 확인했다. 사용자 GUI와 clipboard는 사용하지 않았다.

세 검사 exit0이며 elapsed 6.20/20.79/18.81초, 실제 188 passed/1 ignored다. appearance Windows 9회귀는 두 CPU 후보 엔트리에서 통과했다. 238개 compiled input은 각 before/after·검사간·현재소스 대조가 같고 runner/lock SHA도 불변이다. 정확한 값과 로그·manifest는 windows-cpu-appearance-popupfix/result.json 및 같은 폴더에 보존했다.

새 .local-tools/windows/target/release/dlpad-cpu.exe는 5,555,712바이트, SHA256 2f53a16348535a349ea04a46c645bf46f2aa8fddf3ea81a6477c41f15a135ae6이다. 이전 dd0241... appearance 바이너리 증거는 이전 소스에 대한 범위로 보존한다.

CLI 시험은 실제 AccessKit 메뉴 닫힘과 Tab focus의 GUI 인수가 아니다. 추가 비포인터 egui 단위 회귀는 미작성이며, 실제 Linux/Windows GUI 재검증은 통합 담당자의 후속 범위다. Windows final symlink readonly 실패 경로는 cfgUnix 시험으로 대체하지 않으며 기존 미검증 게이트를 유지한다. MSVC/macOS/공개 릴리스는 수행하지 않았다.

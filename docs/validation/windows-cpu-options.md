# Windows CPU typed 옵션 통합 CLI 검증

2026-10-03. typed DownloadForm, Quality validation, 입력 최소32px 및 docked actions/scroll 통합 동결 소스로 실제 Windows GNU CPU clippy/test/release를 수행했다. 기존 프로젝트 runner와 Windows 전용 target/private CLI Job을 사용했으며 제품/Cargo/runner 변경과 GUI/clipboard 접근은 없다.

세 검사 exit0이며 elapsed 8.57/26.42/30.70초, 원시 결과는 200 passed/1 ignored다. 239개 compiled input은 각 before/after·작업간·현재소스 대조가 모두 동일하며 runner/lock SHA도 불변이다. 세 로그에서 input-desktop-unchanged=True를 확인했다. 정확한 argv/manifest/SHA/time과 원시 시험 출력은 windows-cpu-options/result.json 및 같은 폴더에 보존한다.

새 .local-tools/windows/target/release/dlpad-cpu.exe는 5,571,584바이트, SHA256 345b0fa9689e62924168ef1457d50c9698fa68f8e04b73c1fe4b8c497d20739a다. 이전 appearance-sizing 등 증거는 각각 이전 source 입력에 대한 결과로 유지한다.

CLI 성공은 실제 옵션 UI·32px 높이·IME·Tab focus·테마의 GUI 인수와 별개다. MSVC/macOS 및 공개 릴리스는 실행하지 않았다. 미실행 GUI와 OS를 이 증거로 완료라고 주장하지 않는다.

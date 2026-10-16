# S02: Windows 인증 private GUI 실제 harness

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R08, R10, R11
- 담당/수정 범위: soak_recovery_audit — scripts/windows_cookie_gui_probe.ps1, 이 문서
- 선행 작업: S02-windows-cookie-private-gui-review 및 root 최신 immutable CPU app/example READY

## 사용자 결과

사용자 화면·클립보드와 분리된 owned desktop에서 실제 쿠키 입력·작업 완료/실패/취소를 검증한다. Capability가 없으면 실패를 기록하고 사용자 화면으로 전환하지 않는다.

## 계약과 범위

읽기 검토 문서의 HWND/MTA/UIA/owned Job/argv/literal synthetic source 계약을 따른다. 새 script만 작성하고 제품 Rust/Cargo/기존 scripts를 바꾸지 않는다. 앱과 fixture tool은 명시 절대경로+SHA이며 root가 실행한다. OS temp 아래 owned prefix/cap/source를 만들고 앱 data=root/private-cache, 실제 cookie cache=data/session-cookies로 고정해 기존 example 허용 prefix를 만족한다. TEMP/TMP는 변경하지 않는다.

## 인수 조건

- capability: private desktop에서 owned CPU HWND/UIA Edit label/실제 text/Start Invoke 지원을 읽고 닫기·회수·input desktop 불변을 확인한다.
- success/failure/cancel: 실제 source path 소비 후 empty, job/Attempt/artifact/정제 preview/privacy/DB/rawcalls/정상 copy 정리/source sentinel 보존.
- MTA thread desktop 연결·복귀, suspended app를 owned kill-on-close Job에 먼저 묶기, exact HWND PID/start 검증. 전역 input/clipboard/visible desktop 사용 없음.
- UIA setter 미반영은 구분된 owned WM_CHAR fallback+실제 값 읽기로 검증하며 반환 성공만으로 PASS를 쓰지 않는다. HWND subtree256nodes/text64KiB/file2MiB/전체8MiB/파일128개/입력1024자를 제한한다. UIA worker는 90초 join deadline이며 개별 COM 호출을 중단한다고 주장하지 않는다. Hidden host 외부 controller deadline도 필요하다.

## 검증과 handoff

새 harness source를 구현했다. Cargo/GUI 실행은 하지 않았다. Root가 먼저 capability를 실제 실행하고 성공할 때 다음 흐름을 선택한다. Pending restart actual seed는 별도 dev fixture 계약이 필요하므로 이 첫 harness에서 완료로 주장하지 않는다.

Capability는 실제 빈 Edit text와 Start Invoke 지원을 읽고 private PrintWindow, WM_CLOSE 종료, MTA 복귀, owned Job active=0, input desktop 이름 불변을 요구한다. UIA subtree는 owned HWND에서만 순회하며 duplicate label 또는 unsupported pattern은 실패한다. 입력 mode는 ValuePattern 반환값에 의존하지 않고 실제 readback을 확인하며, 미반영 시 owned HWND client 안의 실제 UIA bounds로만 WM_CHAR를 보낸다.

인증 mode는 `REVIEW_REQUIRED`를 반환한다. Cookie field 소비/empty와 실제 start/cancel 조작 후 앱이 종료된 경우에만 DB·WAL·toolcalls·artifact 등 bounded 원시 파일을 `owned-files/`로 복사하고 size/SHA manifest를 기록한다. Root가 SQLite job/Attempt/state, artifact literal/hash, version no-cookie·metadata/download 동일 private path, privacy, 정상 copy 정리, source/sentinel 보존을 독립 검토해야 인증 PASS가 된다. Controller stdout/stderr 로그는 root hidden host가 보존한다. 합성 temp root는 독립 검토를 위해 그대로 보존하며 앱/자손은 owned Job으로 회수한다.

실행 계약: Windows hidden CLI host에서 `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/windows_cookie_gui_probe.ps1 -Binary <immutableCPU> -ExpectedBinarySha <sha> -OutputDirectory <freshEvidence> -Mode capability`. 이후 root가 제공한 immutable example을 `-FixtureTool <exe> -ExpectedToolSha <sha>`로 명시하고 fresh evidence에 success/metadata-failure/download-failure/cancel을 각각 실행한다. TEMP/TMP는 기존 OS temp를 유지하며 기존 인증 example direct-parent/prefix/cap 계약을 만족한다. 사용자 데스크톱 fallback은 없다.

현재 실제 Windows Add-Type/UIA/GUI runtime은 미검증이며 root 실행 대기다. 새로운 상태 전이·제품 source/Cargo·기존 scripts는 수정하지 않았다.

첫 root capability 실행은 `windows-cookie-gui-capability-first`에 보존됐다. UIA tree는 Pane 한 개만 반환했고 PS5의 BOM 없는 UTF-8 해석으로 field 오류 label도 깨졌다. 실제 field 접근은 FAIL이며 input desktop 불변·owned Job empty 정리만 통과했다. 후속 script는 C# label을 ASCII Unicode escape로 바꾸고 exact owned HWND에 `SendMessageTimeout(WM_GETOBJECT, UiaRootObjectId=-25, SMTO_ABORTIFHUNG, 2000ms)`를 보낸 뒤 최대 5초 동안 root subtree를 재조회한다. AccessKit Windows 0.29.2 `adapter.rs::handle_wm_getobject`와 accesskit_winit 0.29.2 `request_initial_tree`를 읽었다. 초기 tree 요청이 비동기 event를 보내고 placeholder를 먼저 반환하는 계약에 근거하며, 재시도에서도 실제 fields가 없으면 실패를 유지한다. 제품 Cargo에는 accesskit feature가 이미 활성화돼 있다. 새 실제 retry는 root가 별도 evidence에 실행한다.

Root 첫 retry도 Pane 한 개로 field FAIL이었다. Source 추가 검토에서 `software_host.rs`는 실제 `dlpad` 제목 창에 `init_accesskit`을 연결하고 InitialTreeRequested에서 enable/redraw를 처리한다. 현재 후속 harness는 같은 PID의 임의 visible HWND 대신 제목이 정확히 `dlpad`인 유일한 owned HWND를 요구한다. 소유 HWND의 class/client dimensions·접근성 object -25(UIA root)/-4(OBJID_CLIENT)의 delivery/response·bounded subtree count를 보존하고 field 실패 전 private screenshot을 남긴다. UIA tree에도 native HWND/process/framework/automation ID를 기록한다. 다른 desktop의 ancestor나 global UIA root는 조회하지 않는다. Root 새 runtime 결과 전 원인은 미확정이며 제품 구현을 바꾸지 않았다.

Root diagnostic capability는 실제 field labels/readability 지원을 확인했다. 이후 `windows-cookie-gui-auth-success-first`의 실제 인증 입력은 readback mismatch로 FAIL이며 cleanup/input desktop 불변만 통과했다. 후속 harness는 field별 setter/readback 단계의 label·expected/actual length·SHA256·고정 합성 값 UIA readback·bounds와 private screenshot을 보존한다. 소유 HWND에 mouse move/down/up과 WM_PAINT를 분리해 대기하고 문자마다 local keydown/WM_CHAR/keyup/paint를 보낸다. Setter가 일부 text를 넣었다면 local posted Ctrl+A/clear 후 실제 empty를 확인할 때만 새 text를 입력한다. Posted modifier가 적용되지 않으면 실패하며 전역 key 상태 변경이나 input으로 우회하지 않는다. 새 actual retry 전 입력 성공을 주장하지 않는다.

후속 download-failure 원시 URL은 expected29/actual30이며 `hattps://fixture.invalid/watch`였다. Root가 printable keydown A의 Windows 문자 번역과 직접 WM_CHAR가 중복될 수 있음을 확인했다. 최신 source는 printable keydown/up과 가짜 Ctrl modifier를 제거했다. 최초·setter 이후 actual empty면 clear 없이 WM_CHAR만 보낸다. 부분 text가 있으면 owned End key 후 actual 길이(최대1024)만큼 WM_CHAR backspace를 보내고 actual empty를 확인할 때만 입력한다. 이 경로도 empty/readback 실패 시 그대로 FAIL이며 root retry 전 성공으로 표현하지 않는다. 이전 실패 원시는 보존한다.

`windows-cookie-gui-auth-download-failure-input-retry/input-url-readback.json`은 expected29/actual0이었다. Winit 0.30.13 Windows `keyboard.rs::process_message`를 읽어 WM_CHAR 전에 pending `event_info`가 없으면 문자 이벤트를 버리는 실제 조건을 확인했다. 후속 수집기 시험은 printable A 대신 문자 번역 없는 F24(0x87) keydown → 고정 합성 WM_CHAR → F24 keyup을 owned HWND에 연속 enqueue한다. F24가 자체 문자를 생성한다고 가정하지 않으며 Windows 실제 retry 전 동작은 미검증이다. Partial clear는 control text를 직접 보내는 대신 owned Backspace keydown/up으로 바꾸고 actual empty를 요구한다. Queue scheduling으로 pending envelope가 잡히지 않거나 외부 modifier가 관찰되어 text가 달라지면 정확한 readback FAIL을 유지하며 사용자 desktop/global key 상태를 조작하지 않는다. 제품 source/Cargo 변경과 agent Windows 실행은 없다.

## Root 실제 GUI 인증 흐름 독립 검증

고정 CPU SHA2049eca19ed20ee2fcf4aec361f303147d2f9ddcfc0e342be23c8faface54bfd의 success·metadata-failure·download-failure·cancel을 실제 private desktop에서 실행했다. root는 각 SQLite job/Attempt/artifact 및 version→metadata→download 호출, 쿠키 private copy 제거·원본/sentinel 보존·DB 비밀 누락을 독립 확인했다. 취소는 Cancelled/attempt1/artifact0이며 .part 보존도 확인했다. download-failure-key-envelope와 cancel-key-envelope/root-review.json에 후속 증거를 기록했다. 입력 수집기 실패 원시 증거는 모두 보존했다. report preview privacy UI, pending restart, 실제 IME와 최신 source 전체 GUI 검증은 남아 있으므로 이 작업 전체를 완료로 바꾸지 않는다.

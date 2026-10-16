# S10: Windows private 보고서 저장·편집·생략 인수 준비

- 상태: in-progress (runner staging READY; native 인수 미실행)
- 분류: validation
- 우선순위: P0
- 요구사항: R13, R08
- 담당/수정 범위: diagnostic_cache; `/tmp/dlpad-windows-s10-report-save-probe.ps1` 및 본 문서만
- 선행 작업: S10-windows-private-clipboard-next-audit; 최신 정상960x640 report-focus-scroll 및 root frozen Windows release/합성 session cookie fixture

## 사용자 결과

최신 Windows 제품에서 clipboard를 사용하지 않고 Markdown/JSON 새 파일 저장, 로그 수정 재정제·생략, 로컬 MPD 구조 첨부·생략과 원본 보존을 실제 private GUI로 인수할 수집기를 준비한다. 전체 S10/Windows clipboard/P0 완료가 아니다.

## 계약과 범위

AGENTS 읽기 순서와 template, 실제 `windows_cookie_gui_probe.ps1` 및 제품 DiagnosticPanel/Report worker/catalog를 조사했다. 기존 observer의 private desktop·원 process HANDLE/PID/start FILETIME·HWND·Job kill-on-close·MTA UIA·bounded local WM input·provider activation·cleanup 코드를 `/tmp` 독립 복제했다. 원 scripts/Rust/Cargo는 수정하지 않는다. 복사/Paste/clipboard API·Ctrl+V·사용자 입력·SwitchDesktop·SetForegroundWindow·station 설정 변경·visible fallback 없음. 기존 사용자 clipboard가 같은 station에 있음을 인지하며 접근하지 않는다.

원 observer SHA256 `0d9a9f953ab8281b07efa9689e5b50b022c8f1b94ffad6f6d95c9b17451bf30c`. staging runner SHA256 `a5b7a5fb6373519c6aedc9b94465533f806a373ea498f23e18853cdb3a8836fb`. default mode metadata-failure, report verification 필수, 원 이미 구현된 synthetic tool/cookie fixture 프로토콜만 사용한다. 실행 예: root 승인된 private CLI host 안에서 `powershell.exe -NoProfile -File <staging> -Binary <immutable CPU exe> -ExpectedBinarySha <sha> -FixtureTool <synthetic fixture exe> -ExpectedToolSha <sha> -OutputDirectory <fresh evidence> -Mode metadata-failure`. 정책 변경/ExecutionPolicy bypass는 추가하지 않는다. 실제 실행은 root 담당이다.

## 구현된 수집 흐름

1. 합성 인증 실패 후 실제 report header를 열고 JSON/MD 각각 새 파일 저장. JSON logs≥1 fixture 전제를 명시적으로 확인한다.
2. log0 수정: 공개 관측 문장+합성 signature URL을 입력하고 반영→새 JSON에서 관측 문장 유지/domain/query값 제거·로그 개수 유지 확인. 개별 제외 한 개 감소→전체 제외0 확인.
3. create_new MPD fixture의 URL/range 구조 attach→JSON attachment1/URL값 누락→개별 제외0→재첨부1. 새 DTD/entity와 1MiB+1 입력의 실패 뒤 attachment1·omission 존재·원 bytes 불변 확인. 전체 MPD 제외0.
4. 원 sentinel 경로 저장은 create-only 실패 label과 sentinel SHA 불변. 원 MPD/source/cookies/제품 binary/fixture SHA와 Failed DB 상태는 기존 runner 결과 및 root 독립 리뷰로 대조한다.
5. 정상 WM_CLOSE·원 process 종료·Job0·desktop 복귀/input desktop 불변. `closewait_inflight_verified=false`를 명시해 정상 종료를 진행 중 worker 종료 대기의 증거로 확대하지 않는다.

UIA subtree/queue≤256, UI text≤65536chars, input≤1024chars, exports≤2MiB/파일, MPD fixture≤1MiB+1, 선택별 polling≤80×100ms 및 원 UIA worker90초 deadline을 유지한다. report save 성공은 새 파일 존재와 저장 label뿐 아니라 실제 Save enabled(보고서 worker 반환 뒤)에 의해 확인한다. named controls를 새로 찾고 정확 field readback을 검사한다. 모든 GUI mutation 전 원 HWND/PID/start 확인, 실제 visible content 확인, owned-window WM wheel을 방향별 최대32회 사용한다. 주어진 budget 내 UIA provider 호출이 못 끝나면 원 native host Job deadline이 failclosed 종료하며 retry/budget 확대 없이 원시 실패를 보존한다.

## 인수 조건과 남은 정확한 gate

준비된 runner는 edit/omit/attach/count/privacy/export/source/sentinel/정상 종료를 독립 assert한다. root는 실제 최신 binary/source SHA와 SQLite Failed/Attempt/artifact/원 cookie 회수, 새 export와 PNG를 직접 검토한다. UIA 문구/ControlType/스크롤·MTA 호출이 실제 Windows에서 지원되는지는 아직 미검증이다. 자동 observer status는 REVIEW_REQUIRED를 유지하며 이미지/DB 인수 없이 PASS로 승격하지 않는다.

**진행 중 report-worker CloseWait는 아직 수집하지 못한다.** 로컬 작은 파일 save/parse는 빠르고 임의 sleep으로 busy를 가정할 수 없다. 기존 제품에는 worker가 파일 capability를 유지한 상태에서 의도적으로 대기하는 관측 장벽이 없다. deterministic 후속 시험이 필요하면 root가 별도 fixture-only 계약을 채택해야 한다: report worker가 실제 요청을 인수한 뒤 작업 전 `started` native event를 signal하고, owned observer가 `release` event를 signal하기 전까지 취소 가능 유한 budget으로 대기; main CloseWait가 요청 취소를 전달해도 release/worker outcome ack 전 실제 process exit 불가를 확인한다. gate는 `--fixture`와 테스트 빌드 feature 양쪽 조건, 전달된 원 event HANDLE 타입/권한 검증, source fixture 소유권·bounded timeout을 필요로 하며 일반 제품 환경변수나 사용자 파일로 켜지지 않아야 한다. 이 hook은 구현하지 않았다. 새 hook을 채택하지 않으면 실제 빠른 save-in-flight 관측 성공 시에만 좁게 기록하고 실패/미관측을 완료로 쓰지 않는다.

## 검증과 handoff

/tmp 실제 runner 코드 작성 및 원 helper/source와 문자열·금지 API 읽기 점검을 수행했다. PowerShell/C# native compile·GUI·Cargo는 source freeze에 따라 실행하지 않았다. root native collector로 첫 실제 결과를 보존한 뒤 수집기 오류와 제품 오류를 구분해야 한다. original named clipboard ERROR5/NULL183 실패는 그대로 남는다.

## 인증 로그 전제 독립 확인 및 omit case 분리

root 요청에 따라 실제 `windows-cookie-gui-auth-report-ancestor-first/owned-files/report-export.md`와 readonly-scope-retry/report-preview.txt를 읽었다. 내부 JSON logs는 정확 `["[authenticated tool output omitted]"]` 한 개, omitted_lines=0이다. 현재 download.rs1108/1114가 인증 raw-log를 이 고정 문장으로 치환하고 accessible.rs1220이 report.add_log로 넣는다. core redactor는 이 문장을 생략하지 않는다. 따라서 logs≥1 전제는 기존 실제 Windows fixture에서 확인되며 조건 완화·가짜로그·raw 인증 로그 노출은 필요 없다. 최신 binary에서 달라지면 초기 assert가 실제 실패한다.

logs1에서 single omit 뒤 all omit은 빈 배열의 idempotent action만 증명한다. 이를 숨기지 않도록 staging runner에 `-LogOmission single|all` 두 fresh-run case를 분리했다. single은 실제 한 개 감소를 확인하고 all은 초기 nonempty logs에 직접 전체 제외를 실행해0을 확인한다. `all_logs_nonempty_before_action_verified`는 단일 case의 실제 count가1이면 false이며 all case에서는 true다. root가 서로 다른 fresh evidence/root에서 두 case를 실행해야 두 종류의 의미 있는 제거 인수가 모두 확보된다. 하나의 성공으로 두 case를 완료하지 않는다.

최신 staging SHA256 `0122334d72f6e21e187b2dfb510347bdddc3e29960a3f26cbd2155699f26b60e`. 이전 staging SHA는 위 준비 이력이며 프로젝트 runner/Rust/Cargo 변경과 native 실행은 계속 없다.

## root 통합 및 첫 실행 계약

첫 staging SHA 확인은 agent가 독립 로그전체제거 case를 추가한 뒤 원SHA 불일치를 발견해 게시를 중단했다. 최종 원 SHA0122334d72f6e21e187b2dfb510347bdddc3e29960a3f26cbd2155699f26b60e를 확인하여 scripts/windows_s10_report_save_probe.ps1에 게시했다. root가 실제 격리 GUI 실행과 source/immutable binary/tool/SHA·DB/PNG/export independent review를 담당한다. 인증 initial logs1개는 현재 실제 raw-output-omitted 고정 문장이고 가짜로그가 아니다. single/all은 fresh 실행으로 나눠 비어있는 배열 idempotent 결과와 nonempty 전체제거를 구분한다. 작업중 CloseWait 및 actualclipboard 인수는 별도다.

## 첫 실제 실행 실패와 안전한 대안

[첫 collector](../validation/evidence/windows-s10-report-save-first-20261004T124219Z/collector-result.json)는 fixture build를 통과했지만 powershell.exe -NoProfile -File이 시스템의 스크립트 실행 정책 때문에 UnauthorizedAccess/exit1로 앱 실행 전에 실패했다. GUI 인수 성공이 아니다. source/binary/tool SHA와 host private/inputdesktop 불변을 보존했다. ExecutionPolicy 변경·Bypass·파일 내용을 -Command로 재해석하는 우회는 수행하지 않는다.

기존 private Desktop/원 HWND·Job·UIA 코드를 standalone C# executable collector로 옮기는 독립 대안을 준비한다. 정책/계정/보안설정은 바꾸지 않고 개발 범위의 소유 C# native compile/argv 실행으로 같은 GUI 인수를 수행한다. root 실행·원 bytes·cleanup 보장과 clipboard0 조건은 유지한다. 단계가 바뀌더라도 완료 기준을 줄이지 않는다.

## Restricted 정책 실패 뒤 standalone native collector

첫 실제 PowerShell `-File`은 Restricted 정책으로 앱 실행 전에 거절됐다. 이 원시 실패를 보존하며 ExecutionPolicy 변경·Bypass·파일 내용을 `-Command`로 재해석하는 우회는 하지 않는다. root 요청에 따라 독립 `/tmp/windows_s10_report_save_probe.cs`에 기존 embedded CookiePrivateGui와 standalone Main을 작성했다. `.ps1` 실행 없이 기존 native csc로 C# 소스를 직접 컴파일하고 private CLI host로 collector exe를 실행하는 경로이며 정책 변경은 없다.

소스 SHA256 `f302a4a7cbd74fd5207bd521840bae46d996ec4063c76a2b1c1b2d7c6fcfa4ff`, 49,662 bytes. argv 정확7개: `binary binarySHA fixtureTool fixtureSHA freshEvidenceDir metadata-failure|download-failure single|all`. SHA는64 hex, source 경로는 absolute normalized/no-reparse이며 binary/fixture 원 read HANDLE을 FILE_SHARE_READ만으로 보유하고 최대256MiB streaming SHA를 실행 전후 확인한다. copy source도 retained HANDLE에서64KiB씩 읽고 create_new·sync하며 copied exe SHA를 확인한다.

fresh evidence와 GUID owned Windows TEMP directchild는 CreateDirectoryW+currentSID protected 단일 FullControl DACL로 create-only 생성하고 observed ACL/owner를 검증한다. 이미 있는 output을 채택하지 않으며 신규 output 소유권이 성공한 뒤에만 실패 JSON을 쓴다. 같은 원 fixture.cap/mode/source.cookies/sentinel, private-cache/media, test child process env와 toolcopy를 준비한다. 이는 이번 standalone 프로세스의 시험 환경에 한정하며 사용자 설정·windowstation·clipboard를 바꾸지 않는다.

Run의 원 privateDesktop/HWND/startidentity/Job/cleanup은 유지한다. 실제 원 process exit와 Job0을 모두 확인한 뒤에만 owned-root 파일을 fresh evidence/owned-files로 create-only 보관한다. traversal entry≤256/queue≤128, 파일≤128/각≤2MiB/합계≤8MiB, exe는 보관에서 제외, 모든 경로 no-reparse 재검증, JSON≤64KiB. 원 임시 root는 자동 삭제하지 않고 보존한다. REVIEW_REQUIRED는 그대로이며 Failed SQLite/Attempt/privacy/PNG·원 cookie 회수는 root 리뷰가 필요하다.

C# native compile·collector 실행은 아직 하지 않았다. 기존 clipboard capability failure와 save-inflight CloseWait 미검증은 유지한다. Rust/Cargo/프로젝트 scripts와 정책 설정은 수정하지 않았다.

## standalone collector 통합

root가 최종 standalone staging SHA f302a4a7cbd74fd5207bd521840bae46d996ec4063c76a2b1c1b2d7c6fcfa4ff를 대조해 scripts/windows_s10_report_save_probe.cs에 게시했다. 기존 private Run과 권한/Job/원 HWND 확인 및 새 private fixture/원 source stream pin/종료 후 bounded archive를 유지한다. C# compile 및 직접 argv native 실행을 인수하며 PowerShell 정책·설정은 변경하지 않는다. rootreview 전 REVIEW_REQUIRED이며 실행중 CloseWait 성공으로 주장하지 않는다.

## standalone 첫 실제 GUI 결과

windows-s10-report-save-native-20261004T124853Z의 native csc compile은 통과했지만 report header Invoke pattern unavailable로 첫 action에서 exit2다. 실제 private960x640 HWND·합성 URL/cookie 입력·Failed SQLite 상태를 확인했고 original Job 종료/desktop closed/input Default 불변/clipboard0를 보존했다. report export는 미수행이며 S10 PASS가 아니다. 헤더의 UIA Invoke 비지원은 기존 bound-click 관측 경로와 일치하므로 root가 action pattern이 없는 enabled control에 한해 원 HWND/PID/FILETIME와 실제 보이는 content bounds를 검증한 로컬 WM click을 사용하도록 수집기를 수정했다. disabled control은 거절하고 전역 입력/visible fallback은 도입하지 않는다.

## action 역할 보강

원 bounds-click 재시험 windows-s10-report-save-bound-click-20261004T125351Z는 JSON을 Button으로 찾는 수집기 전제가 실패했다. 실제 selectable format/header의 action role을 Button/RadioButton/CheckBox/TabItem 범위에서 exact label+unique로 찾도록 수정하고 각 action 전 bounded UIA role/enabled tree와 실제PNG를 추가했다. Text/Edit role은 클릭 대상에 포함하지 않는다. 단순 action반환을제품저장PASS로간주하지않으며 원 실패/root/Job0/input불변근거를보존했다. 재현 가능한 native 실행 runner scripts/windows_s10_report_save_native_probe.py도 root담당파일로 추가했다.

## header 실제 활성화 경로

windows-s10-report-save-action-roles-20261004T125627Z의 action tree/PNG를 root가 직접 확인했다. format이 다른role이라누락된것이아니라 header가 실제닫힌채유지돼JSON자체가없었다. 기존observer는같은mouse-open실패뒤원UIAfocus+localSpace경로를이미사용했다. 새collector도nonInvoke/focusable대상에원focus반영과실제content범위확인후원HWND에만WM_SPACE를보내도록맞췄다. 전역keyboard/clipboard는없으며마우스/IME실제OS입력인수로확대하지않는다. 앞선action역할확대는정확지원actionrole탐색방어로유지한다.

## 실제 scroll anchor clip 오류

windows-s10-report-save-owned-space-20261004T130013Z는 header keyboardopen을확인했지만JSONscroll32회후불변으로실패했다. root가PNG와actualscrollrows를직접대조해anchor_y44가고정탭바위에있는offscreenEdit임을확인했다. 기존InContent는footer하한만확인하고topnavbarclip을누락했다. 새collector는실제다섯navButton의bottom좌표에서원contenttop을측정해그위anchor/target를제외한다. 좌표상수나budget확대로실패를덮지않으며root에서표준960x640로재실행한다.

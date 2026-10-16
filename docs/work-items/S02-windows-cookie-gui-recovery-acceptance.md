# S02: Windows 실제 GUI 시작 시 쿠키 receipt 회수 인수

- 상태: proposed
- 분류: test
- 우선순위: P0
- 요구사항: R03, R04, R08, R11
- 담당/수정 범위: metrics_recovery — 이 문서만; 후속 collector 구현·실행은 root 조정
- 선행 작업: 최신 Windows gated DownloadRepository adapter, public receipt recovery, 실제 독립 helper 회귀와 원래 native anchor fixture

## 사용자 결과

새 Windows GUI 프로세스가 시작 시 public recover_sessions를 실행하여 이전의 원래 receipt로 증명된 복사본만 회수하고, 미확인 복사본은 보존하면서 고정 안내를 표시하는지 실제 사설 desktop에서 확인한다. 이 문서는 인수 계획이며 실행·채택 완료가 아니다.

## 현재 코드와 기존 수집기

accessible.rs의 begin_recovery worker는 data/session-cookies를 recover_sessions에 전달한다. deferred/rejected/truncated이면 cookie_recovery_pending을 설정하고 catalog COOKIE_RECOVERY_PENDING을 표시한다. 회수 오류는 COOKIE_RECOVERY_FAILED로 전달하며 준비를 완료하지 않는다. 기존 scripts/windows_cookie_gui_probe.ps1는 사설 CreateDesktop, 원래 process handle/creation, UIA root, PrintWindow, input desktop 전후 비교, 소유 Job 정리를 제공한다. 실제 success/metadata-failure/download-failure/cancel 4모드와 동일 앱 바이너리의 보통 guardian 등록을 사용한다. 정상 시작 fixture는 매번 새 루트이므로 현재 collector 그대로는 이전 receipt 시작 회수를 입증하지 못한다.

## 빌드와 원래 fixture 준비

root는 소스 동결 manifest를 확보한 뒤 최신 Windows 앱 CPU 바이너리와 session_cookie_fixture_tool.exe, 독립 helper를 빌드한다. 크기·SHA·toolchain·source input hash를 기록한다. 기존 구 GUI 결과나 구 helper 결과를 새 제품 검증으로 재사용하지 않는다. 새 앱 바이너리는 일반 등록 경로로 자기 자신을 helper로 사용하며 helper override 환경을 제거해야 한다.

먼저 기존 collector의 4모드를 새 출력 폴더에서 실행하고 실제 DB/Attempt/artifact/calls/receipt와 비밀정보 생략을 root가 검토한다. REVIEW_REQUIRED는 PASS가 아니다. 취소는 worker-ready 후 UIA 취소로 유발한다. metadata 실패7/download 실패9 및 성공 artifact를 검사한다. 원본 source와 외부 sentinel의 바이트·SHA를 유지한다.

회수 전용 fixture는 같은 합성 데이터 루트 아래 private-cache/session-cookies에 제품 importer로 만들며 원 recovery marker와 모든 원 FileId/ACL을 보존한다. root 소유 ProductFixture/실제 helper 경로를 이용해 Intent→Prepared 원생성 시각→Bound→잠금 인계→Begin→실제 terminal receipt를 생성하고 lease를 defer한다. 이미 public recover를 호출하는 defer_and_recover()는 GUI 선행 fixture에 그대로 사용할 수 없다. 해당 helper는 회수 전에 종료/reap하고 원 journal lock을 놓아야 한다. 새 FileId를 관측해 원 anchor를 대체하거나 JSON Resolved만 써서 receipt를 모사하지 않는다.

## 실제 인수 행렬

1. 완료 receipt: 원 pinned receipt256 및 Bound/Resolved fixture를 준비한 뒤 같은 data 루트의 새 앱을 사설 desktop에서 시작한다. 초기 recovering→ready 실제 UIA를 유한 대기로 확인하고 원 cookie/marker/journal/receipt만 제거됐음을 검증한다. source와 외부 sentinel은 그대로여야 한다. pending 안내가 없어야 하며 새 쿠키 선택으로 추가 합성 작업이 가능해야 한다.
2. Intent 또는 Bound/no receipt: 실제 원 fixture를 보존하고 시작한다. 앱은 ready와 COOKIE_RECOVERY_PENDING을 표시한다. 모든 원 바이트를 보존하고 새 쿠키 입력은 비어 있으며 새 선택을 허용한다. PID 숫자의 종료 여부를 회수 근거로 사용하지 않는다.
3. lock Busy: 원래 helper 또는 명시 소유 fixture의 잠금을 유지한 상태에서 시작하여 바이트 보존·pending 안내를 확인한다. 소유 fixture 종료 후 다음 새 앱 시작에서 실제 receipt 조건을 다시 판단한다.
4. receipt 변조/부분 및 foreign 추가: 원 ID를 유지한 합성 변조와 모르는 항목을 각각 준비하고 회수를 거절·보존하는 실제 안내를 확인한다. recovery API가 rejected로 반환하면 pending, API Err면 COOKIE_RECOVERY_FAILED와 시작 버튼 비활성/ready 미완료가 기대 동작이다. 권한 또는 non-directory cache 오류는 고정 실패 안내와 외부 바이트 보존을 별도 확인한다.

## collector 최소 확장 계약

기존 private desktop/원 process identity/캡처/cleanup을 재사용하고 새 수집기는 기존 루트를 명시 입력받아 재시작만 수행한다. 새 루트 자동생성이나 같은 이름으로 anchor 재생성을 하지 않는다. 시작 전후 원래 fixture manifest와 UIA/초기·ready·pending/error PNG, native process creation, InputDesktop 비교를 기록한다. 사용자 clipboard, SendInput, desktop switching, visible fallback은 금지한다. 현재 collector가 종료 시 자기 outer Job 전체를 정리하므로 fixture helper는 GUI 실행 전에 종료/reap하거나 별도 소유 observer host에 있어야 한다. 실행 중 helper를 같은 Job에 두고 collector 종료를 독립 app kill 증거로 확대하지 않는다.

## 검증과 handoff

읽기 전용으로 코드·collector를 확인했다. Rust/Cargo/프로세스/GUI는 실행하지 않았고 기존 수집기를 수정하지 않았다. sourcefreeze 종료 및 root GO 뒤 fixture builder와 루트 재사용 startup collector 구현이 필요하다. 실제 GUI 4모드와 시작 회수 행렬 원시 결과가 나오기 전 이 작업은 proposed다. Windows 이전 boot ID 삭제 권한은 도입하지 않으며 receipt 없는 이전 boot 회수는 이 인수 범위로 성공 처리하지 않는다.

## collector 구현 인계

신규 scripts/windows_cookie_startup_probe.ps1을 구현했다. 기존 수집기의 private desktop/Job/UIA/원 process identity/PrintWindow/input desktop 보존 구현을 재사용한다. 입력은 Binary/ExpectedBinarySha/SeedManifest/OutputDirectory이다. manifest version1에는 owned_root, appdata, library, source, sentinel, cookie_owner 절대경로와 expected(recovered/pending/error), source_sha256, sentinel_sha256이 필요하다. 전부 원래 명시 소유 temp root 아래이고 cookie_owner는 appdata/session-cookies 아래여야 한다. producer/helper 종료·reap와 원 journal lock 해제 뒤 manifest를 발행해야 한다.

수집기는 원 owner 파일의 최대32개/총2MiB bounded SHA·size를 자체 채집하며 recovered는 원 owner 부재, pending/error는 원 파일 그대로와 count를 요구한다. UIA 필드 비움/시작 버튼 준비 또는 오류 비활성/고정 pending·failed 안내, startup/recovery PNG와 UIA를 기록한다. 시작 동작은 호출하지 않으며 기존 calls.jsonl 변화도 거절한다. source/sentinel/binary hash를 전후 확인한다. 새 seed 생성·publicrecover 선행 호출·visible fallback·clipboard 입력은 없다.

구현 파일을 게시했으나 PowerShell parser/Add-Type/native GUI는 아직 실행하지 않았다. root GO 후 실제 seed builder와 최신 바이너리로 컴파일·실행해야 한다. 기존 수집기에서 재사용한 미사용 report/input helper는 실행 경로에 호출되지 않는다.

## Root collector 사전 검증

windows-cookie-startup-collector-compile-first는 Powershell script execution policy UnauthorizedAccess로실패했다. 구문/C#compile 및GUI는실행되지않았으며실패자료보존. 시스템전원/보안정책변경과ExecutionPolicy bypass는사용하지않았다. 프로젝트전용C#parser/compile검증도구와새합성seed를별도준비한다. 이실패는자동승인거절이아닌Windows런타임정책오류다.

## 보안 정책 유지와 독립 native runner

root의 PowerShell 실행은 ExecutionPolicy UnauthorizedAccess로 거절됐다. 해당 실패 로그를 보존하고 정책 변경·-ExecutionPolicy 우회를 하지 않는다. 신규 scripts/windows_cookie_startup_runner.cs는 기존 CookieStartupGui C# 본문과 manifest/hash/원 owner 검증을 native Main으로 연결한다. argv는 binary expectedSHA seedmanifest outputdirectory 4개이며 PowerShell runtime이나 .ps1 실행을 사용하지 않는다. 기존 host의 csc/UIAutomationClient/UIAutomationTypes/WindowsBase/System.Drawing/System.Web.Extensions 참조로 root가 컴파일한다. GUI 실행은 root GO와 실제 seed 준비 뒤 수행한다.

error seed는 원 cache를 preserved-session-cookies로 이동하고 session-cookies를 원 합성 plain file로 만드는 계약이므로 error에서만 cookie_owner의 session-cookies 하위 제한을 제외한다. 모든 경로는 여전히 동일 소유 temp root 아래이고 original owner의 실제 바이트·파일 count를 검사한다. 이는 error seed 원본 보존 위치를 지원하며 recovered/pending owner 경로 제한은 유지한다. native runner는 아직 컴파일·GUI 실행되지 않았다.

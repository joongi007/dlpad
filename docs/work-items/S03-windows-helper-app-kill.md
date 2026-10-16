# S03: Windows helper 실제 app-like process 종료

- 상태: in-progress
- 분류: test
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: metrics_recovery — process/windows.rs cfg(test), process/windows_helper_launch.rs tests, cookies/windows/test_fixture.rs 및 이 문서
- 선행 작업: 독립 helper 실패3nativePASS; root test-only original-cap duplicate/producer launch/observer bridge 통합

## 사용자 결과

부모 observer가 실제 producer app-like process의 원래 process handle만 종료한 뒤 독립 helper가 actual tool Job를 정리하고 원래 receipt를 쓰는지 확인한다. controlEOF만 직접 발생시킨 이전시험과 구분한다. 아직 public app startup recovery와 실제 GUI app kill 인수는 아니다.

## 계약과 범위

observer는 private parent TempDir 및 자신의 PROCESS_DUP_HANDLE|QUERY capability를 준비한다. cfgtest 전용 native producer launcher가 이 cap 및 NUL stdio만 정확 HANDLE_LIST로 상속한다. producer는 자신의 TestFixture를 parent-owned base 안에 생성하고 실제 helper launch/Prepared nativecreation/app durableBound/release/Ack/Begin을 수행한다. 합성 tool은 cookiecopy를 읽은 뒤 started marker를 기록하고 유한10초 동안 실행한다.

producer는 자신이 보유한 원helperprocess/events/stdout/stderr 핸들을 observer 원processcap에 DuplicateHandle한다. private bounded create-new ready manifest는 원Bound/anchors 및 원creation/대상handle만 담는다. observer는 원producercreation과 원helpercreation을 실제 retainedhandle에서 대조하고 original native file/ACL pins를 kill 전에 확보한다. helperlive/receipt0/events0/actualtoolstarted를 확인한 뒤 producer 원handle에만 TerminateProcess를 수행한다. producer Job 전체를 종료하거나 PID로 process를 다시 찾지 않는다.

producer는 Tool Job 밖에 있고 별도 kill-on-close producer Job를 만들지 않는다. hidden host의 외부 Job는 observer까지 포함하여 계속 살아있다. 따라서 app-only kill과 host Job 종료가 다른시험이다. breakaway fallback은 없다. helper는 자신의 Tool Job를 소유한다.

## observer proof와 실패 보존

producer가 죽으며 controlwriter OSclose→helper originalJob empty→receipt flush→Terminal→helper originalprocess exit 순서를 확인한다. observer가 보유한 originalpipe/processhandle로 유한5초 frame/exit/EOF를 관측한다. 원receiptFileId/ACL/256B fullmatch와 originalBound journal re-lock/resolve를 검사하고 source/copy/외부sentinel을 보존한다. actualfileID를 새로 관측하여 원anchor를 대체하지 않는다. manifest partial/교체/identity불일치/이미helperterminal/미시작tool은 실패다.

## 인수 조건

별도 producer 프로세스가 실제 종료됐고 helper는 그전 live여야 한다. actualstarted 없는 empty fixture_tool로 확대하지 않는다. Terminal/status/actualreceipt를 함께 확인하고 tool stdout의 started marker 및 pipeEOF를 기록한다. 모든wait/read/childcleanup은 originalhandle+절대기한으로 유한 처리한다. 실패시 소유helper만 originalhandle로 정리하고 proof없는record는 보존한다.

## 검증과 handoff

현재 /tmp 구현 초안이며 shared product/Cargo/schema/GUI를 수정하거나 native를 실행하지 않는다. root가 test-only accessors/bridges를 검토·통합한 뒤 helperbinary를 먼저 빌드하여 실제 Windows 검증한다. 초안compile/runtimePASS를 주장하지 않는다. 실제계정/사용자Documents/전원/설정 변경 없음.

## concrete 초안 handoff

root 승인에 따라 /tmp/dlpad-helper-app-kill-draft.rs, /tmp/dlpad-helper-app-kill-fixture-bridge.rs, /tmp/dlpad-helper-app-kill-native-accessors.rs를 준비했다. 첫 파일은 launcher tests include fragment, 둘째는 현재 create_seeded/base_digest/reopen_seed를 보존하는 create_in 및 original ObserverPin bridge, 셋째는 cfgtest native Child duplicate/terminate 및 exact3cap producer launcher다. producer environment는 fixture base와 SystemRoot만 명시하며 global set_var·부모 secret 환경 복사를 하지 않는다.

구체 case는 Begin 뒤 실제10초 합성 reader가 private cookiecopy를 읽고 stdout을 flush한 다음 started marker를 게시한다. producer가 원helperprocess와3readpipe를 observer 원processcap으로 복제한다. READY 이전 오류는 해당 target duplicate만 역DuplicateHandle/CLOSE_SOURCE로 정리하는 RAII 초안을 포함한다. parent는 nativecreation/원Bound/원파일pins/helperlive/receipt0/events0 검증 후 실제producer 원handle만 종료한다. 따라서 controlEOF직접drop fixture 또는 emptytool 자연종료를 actual appkill로 확대하지 않는다.

개별 rustfmt 최종 exit0. 초안 SHA256: app-kill 69f7e77572f752703ef73ce60473d56ffe38e61f67416707e30e4b3b7ecc944c; fixture bridge a48a57fb3c207a8cc1a88ea1333826e9124ba8916f9c42a8f55678c011d8d27d; nativeaccessors f48b03f215a194c461e30fb3a4a2dadc011ef82e6ff3c41bd2558fcd376936c3. root가 소유 module에 연결해야 하므로 현재 compile 가능성과 runtime 결과는 미검증이다. project Rust/Cargo/공유schema 변경 없음. observer 회수는 원journal receipt/re-lock/resolve 기반으로 public recover_sessions 인수를 대신하지 않는다.

## 프로젝트 통합

root GO로 지정된 세 프로젝트 파일에 test-only 구현을 통합했다. 기존 ProductFixture와 marker 회귀를 보존했다. 포맷·Cargo·native 실행은 root 동결 검증을 기다리며 수행하지 않았다. 실제 앱 유사 프로세스 종료 인수는 아직 미검증이다.

## Root 실제 native 통합 검증

windows-actual-app-kill-native-first: current debugguard build exit0, exact actual_app_like_process_kill_leaves_helper_owned_cleanup_receipt 1 PASS/156 filtered/exit0. root raw log/실제 code/operation JSON 확인; source_unchanged 및 input_desktop_unchanged true. 실제toolstarted→helperlive/receipt0 확인→originalproducer processhandle TerminateProcess→nativehelper Terminalnonzero/nativeexit동일→eventsEOF/actualreceipt256→originalnativepins journal reopen/resolve와 source·foreign보존을 검증했다. 실제 제품 GUI crash/publicrecover 통합전체를 이 app-likefixture로 대체하지 않는다. 현재 미연결 capture facade로 deadcode/unused 경고5개가 있어 이번run은 Clippy PASS를 주장하지 않는다. 전체검증과 제품capture연결후경고제거가 남는다.

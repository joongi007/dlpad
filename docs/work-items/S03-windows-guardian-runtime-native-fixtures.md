# S03: Windows guardian runtime native fixtures

- 상태: done (Request native5case; 독립 helper/appcrash 인수 미완료)
- 분류: test
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: metrics_recovery — /tmp 초안2개, GO 후 신규 cookies/windows/test_fixture.rs 및 process/windows_guardian/tests.rs, 이 문서만
- 선행 작업: root PreparedChild/GuardianJournal/Request runtime; source freeze99334 종료 및 GO

## 사용자 결과

원래 Windows Job/process/native FileId와 durable Bound/lock handoff를 실제 runtime Request로 검증한다. 제품 guardian 독립 process entry/app crash/public recovery 완료와는 구분한다. 공유 runtime/schema/Cargo를 수정하지 않는다.

## 계약과 범위

TestFixture는 원 private cache/owner/copy/journal/receipt와 actual SID/ACL/native IDs를 준비하고 create/owner_path/begin/record/bind/release/reacquire/resolve/all_receipts_match/receipt_bytes/copybytes보존 API를 제공한다. runtime tests는 actual Prepared creation→app durablebind/release→appAck→helper ownlockAck→Begin→Terminal 순서를 인수한다. anonymouspipe4096은 비상속이며 parent event/stdout 읽기는 PeekNamedPipe+절대기한/byte상한을 사용한다.

정상0, 실제 toolnonzero(고정 unknown Rusttestflag), helperAck 후EOF, Begin+Cancel 한256Bwrite, wrongBeginDigest 다섯case를 추가한다. 취소 전 gate cases는 toolstdout0을 확인한다. 실제 Terminal 및 receipt256 이후 app reacquire/resolve/fullproof로 원Jobempty runtime 계약을 검증한다. Request는 scopedthread 안에서 실행하며 spec3초+cleanup2초 범위와 EOF실패정리를 사용한다. parent read 무한대기와 guessedPID/foreignkill은 금지한다.

## 인수 조건

tool fixture는 현재 Rust test executable의 정확 fixture_tool 필터이며 로그4KiB이하를 확인한다. frame actual slot/nonce/creation/digest 및 원cookie/sourcebytes가 보존되어야 한다. 미시작gatecase는 도구 test output이 없어야 한다. parser에 값만넣은 성공을 actual runtime 결과로 확대하지 않는다. malformed/EOF/queuedCancel결과와 genuine tool exit를 구분한다.

## 검증과 handoff

현재 /tmp 초안과 문서만 준비한다. rootGO 전 제품 Rust는 게시하지 않는다. 자체 Cargo/native실행은 없고 초안compile가능/시험PASS를 주장하지 않는다. GO 후 개별rustfmt/sourceSHA동결을 인계하고 root 실제 Windows native 검증 결과를 후속 기록한다. 사용자desktop/실제cookie/전원/설정 변경 없음.

## 게시와 실제 검증 경계

root GO 후 신규 test_fixture.rs와 windows_guardian/tests.rs만 게시했다. root는 부모 module/reexport 및 기존 modtests 안 include! 연결을 담당한다. TestFixture API와 helperthread runtime fixture는 위 계약대로 작성했으며 parent 이벤트와 tool output은 native PeekNamedPipe를 사용하여 유한읽기한다. Terminal후 원 events EOF를 확인하고 실제 receipt256/reacquire/resolve/fullproof 및 원source/copybytes를 검사한다.

개별 rustfmt exit0. test_fixture SHA256 3796e27c3e84c5deaa5d33249bc2cfe8c12ef5a608ab6c8e9d38d7b8a05d0e7d; runtime tests SHA256 8a23f4193a636af17ae115ee432c1be76fcf7dda958b66732b938c64bcc26e07. 자체 Cargo/native는 미실행이다. root config/entry 통합 후 컴파일·실제 Windows 결과가 필요하며 현재 코드를 compile PASS 또는 runtime PASS로 보고하지 않는다. 독립 guardian process entry/app crash는 후속 범위다.

## Root 실제 Windows 검증

`cargo clippy -p dlpad-engine --all-targets --locked -- -D warnings`와 `cargo test -p dlpad-engine --lib --locked native_runtime_ -- --nocapture`는 windows-guardian-runtime-fixtures-native-first에서 exit0, sourceunchanged/inputdesktopunchanged이다. root가 raw로그의 정확5case PASS/130filtered를 대조했다. 정상/native비영 exit, helperAck뒤EOF, 단일256write queuedCancel, wrongBeginDigest를 실제Job과 원파일 receipt/resolve로검증했다. elapsed0.10초의짧은합성범위다. 최종 tests SHA는54003486fd12a3027ab569f8ddce07481c21b766080164d625a70e5a03f7b8ba이며singlewriteassert를반영한다.

이는scopedthread helper Request의 actualnativeJob 인수이고 독립helperprocess/appcrash/제품startup scanner 인수가아니다. 전체Windows/Linux workspace는새전용snapshot에서진행중이다.

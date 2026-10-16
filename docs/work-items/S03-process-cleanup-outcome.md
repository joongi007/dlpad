# S03: 미확인 프로세스 정리의 타입 전달

- 상태: in-progress
- 분류: fix
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: root — process.rs; metrics_recovery — download.rs; stress_recovery — cookies.rs 및 전용 시험; UI 통합 root
- 선행 작업: 세션 쿠키 실행 연결 및 startup reaper

## 사용자 결과

소유 프로세스 정리가 미확인인 경우 인증 파일을 즉시 삭제하지 않고 명시적으로 정리를 보류한다.

## 계약과 범위

문자열 매칭 없이 io::Error 내부의 typed OwnershipCleanupError를 전달한다. startup deferred/unconfirmed와 output drain interrupted를 구별한다. 임의 도구 exit126이나 stderr는 정리 실패 증거로 추론하지 않는다. guardian/service terminal ownership acknowledgment는 별도 남은 계약이며 이 타입만으로 전체 프로세스 종료를 보장하지 않는다.

## 인수 조건

정확한 typed error 분류, 일반 같은문구 오류를 미분류, 기존 error kind 유지, authenticated omission 뒤 타입 유지 및 복사본 보존/원본보존/FD회수를 시험한다.

## 검증과 handoff

구현 중. guardian terminal acknowledgment와 startup stale-copy 회수는 추가 구현 필요하다.

## root 최초 실행

process.rs의 세 오류 경로와 cleanup_is_unconfirmed를 구현했다.`cargo test -p dlpad-engine --lib --locked cleanup_marker_tests` actual1PASS/0FAIL/exit0。[結果](../validation/evidence/session-cookie-cleanup-marker/process-unit.json)。같은 문자열의 일반 Io 오류는 분류하지 않는다. download 분류와 cookie FD 회수·보류 통합은 병렬 구현 중이다.

## 보류 연결 구현

download adapter는 typed 오류를 SessionCleanupDeferred로 전달하고 인증 출력 생략 뒤에도 유지한다. root native worker는 해당 오류에만 lease.defer_cleanup()을 호출한다. importer는 파일을 지우지 않고 cache/owner/file FD를 닫으며 보류된 lease를 재사용할 수 없게 한다. 일반 성공/실패/확인된 취소는 명시 cleanup을 유지한다. 신규 전체 engine lib 및 cookie 통합시험은 실행 중이며 아직 통과로 기록하지 않는다. 부팅 후 보류 복사본 회수와 guardian/service 최종 정리 acknowledgment는 별도 남은 게이트다.

## terminal확인 공유 구현 시작

root는process.rs의Ownership에실제command가선택한service여부와terminal확인호출을연결하고TerminalUnconfirmed타입을추가한다. metrics는guardian.rs/user_service.rs에서GC/GU·LC/LU를발신하고stress는새terminal.rs의유한parser만담당한다. 명시protocol은direct GC, service GC+LC순서이며불명/누락/중복/비정상에는copy를보존한다. supervisor오류는보수적으로U이고genuine tool126은정상GC와원래status를유지한다. service의startup launcher가reap되어도서비스정리는확인되지않으므로startup실패는typedUnconfirmed다. 소스동결/컴파일/실제회귀이전이며프로토콜채택완료가아니다. GUI시험은기존immutable5cb52a14를사용해변경중인소스와구분한다.

## terminal 최초 실제 parser 검증

새인터페이스통합컴파일은성공했다. `cargo test -p dlpad-engine --lib --locked process::terminal` 실제5PASS/1FAIL/exit101. 사전timeout요청73ms와kernel관측76ms를직접비교한시험이실패하여실제사전read_timeout관측을기준으로복원검증하도록담당자에게수정인계했다. [실패 결과](../validation/evidence/terminal-cleanup-first/parser-first.json). 제품의timeout복원/프로토콜성공으로아직표시하지않는다.

## parser 재실행

같은명령재실행은actual6PASS/0FAIL/exit0(1.01초). [재실행 결과](../validation/evidence/terminal-cleanup-first/parser-retry.json). 이전실패근거를보존하며실제guardian/service회귀는아직별도다. 쿠키GUI3flow보고는5cb52a14 immutable snapshot이고이새terminal변경을포함하지않는다. root는원시DB·artifact근거보존을보강하도록추가수집을요청했다.

## root 엔진·기존 GUI 독립 대조

새 terminal source의engine clippy와전체engine test는actualexit0,218PASS/0FAIL/7ignored. [명령 결과](../validation/evidence/terminal-engine-regression/result.json), [집계](../validation/evidence/terminal-engine-regression/counts.json). 이시험은기본directfixture이며사용자managerstrict회귀를대신하지않는다. 실제native용driver/helper를immutable로보존하고typeddriver를추가컴파일중이다.

이전5cb52쿠키GUI의success/cancel rawSQLite정합성·정확jobJSON·revision4/1attempt·cookiepath DB배제·literalhash·artifactbytes/hash·source/sentinelhash·PID/cgroup부재를root가독립대조했다. [독립 검토](../validation/evidence/session-cookie-native-gui-20261004-root-review/root-review.json). malformed의화면인수는agent실행근거이며root가직접렌더링재검증했다고표현하지않는다. 새 terminal 변경은해당GUI에포함되지않는다.

## 실제 사용자 서비스와 raw protocol 인수

새typeddriver/helper에서actual22caseexit0/PASS: API17·rawfast3·missingack2. root가원시결과의genuineexit126보존·guardianSIGKILL/missingacktyped미확인·rawfast0/42/126의RGCLC/원exit/socket+pipeEOF를대조하고정확78PID/14cgroup경로부재를확인했다. [독립 검토](../validation/evidence/terminal-native-first-20261004/root-review.json). fakehelpermissingack는manager독립fixture이며실제서비스고장인수로확대하지않는다.

root가download의typed오류생략회귀에TerminalUnconfirmed를추가했다. startuprecovery파일을병렬구현중이므로소스동결후통합재검증한다. 등록앱releaseGUI·추가비용·Windows/다른OS·실제쿠키fault수명·startup회수는아직별도게이트다.

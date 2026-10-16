# S03: keeper receipt 이후 debug 관찰 barrier

- 상태: ready (source freeze, 실제 검증 대기)
- 분류: native-test-support
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: stress_recovery; process/user_service.rs 및 이 문서
- 선행 작업: root full default/CPU7check ALL0, socketpair 상세 계약 승인; 신규 native sibling fixture는 metrics 담당

## 사용자 결과와 계약

실제 receipt sync와 identity/keeperalive 확인 뒤 EOF직전 debug-only K/B socketpair로 sibling 관찰을 결정적으로 배치한다. 숫자FD>=3, currentUID Unix stream/nonlistener/connected/peercred/원controlalias거절 뒤 ownedCLOEXECFD로 capture한다. channelargv/manager env전달 없음. release 환경조회 및 I/O없음. total2초 absolute poll/EINTR bounded, malformed/EOF/timeout failclosed. 제품timeout/strictreceipt완화0.

## 인수 조건

순수 parser 경계, 실제 UnixStream positive/nativealias, pipe/regular/listener 등 invalid검증. owned FD cleanup, readyK/ackB/bytewrong/extra/EOF2초 상한. 실제제품live/pendingsibling 인수와helperbuild/Cargo는root 담당이며작성PASS와구분한다.

## 검증과 handoff

작성 뒤 소유 rustfmt/sourcefreeze. 자체 Cargo/native 실행 없음.

## 소스 인계

run_gated에서 debug-only env를 시작 시 capture한다. parser는 ASCII십진수 길이≤10/i32범위/FD≥3이며 비UTF8·부호·공백을 거절한다. current control rawFD 및 fstat dev/inode alias, currentUID socket, AF_UNIX/SOCK_STREAM/nonlistener/connected와 peercred UID를 확인하고 delivery originalFD를 owned CLOEXEC로 보유한다. validate 실패는 foreignFD ownership을 획득하지 않는다. readonly regular receipt/lock는 socket검증에서 거절된다. sockaddr_storage는 kernel short-return에도 초기화되지 않은 tail을 읽지 않도록 zeroed로 구성했다.

debug gated Environment에서 exact channel env키를 제외하며 channel FD/값을 manager/keeper argv로 전달하지 않는다. release는 capture와exchange 호출/환경 개별조회가 cfg로 제외된다. receipt sync→원 identity/keeperalive 뒤 EOF직전에 readyK와 ackB를 합친2초 absolute pollremaining budget으로 교환한다. send는 MSG_NOSIGNAL/DONTWAIT, recv DONTWAIT 및 queuedbyte beforeHUP 정책이다. wrong byte/extraqueuedbyte/preackEOF/timeout은 failclosed이며 ack뒤peerEOF는 queuedextra가 없으면 허용한다. delayed futurebytes까지 기다려 검증했다는 주장은 하지 않는다.

순수 parser와 실제 UnixStream/nativealias/regular/pipe/closed 입력, K/B·wrong·extra·EOF, missingack 실제2초 timeout unit3개를 추가했다. globalenv mutation이나 arbitrary fixture sleep 없이 peer 수명으로 timeout을 유발한다. 소유 rustfmt PASS. frozen source SHA256 `45b757a8f3944d7cd6f6630058dfc9a3a5a365caa6ad44517d3f43e407d6a364`. Cargo/native 자체 실행 없음. metrics 신규 native sibling fixture와 root helper compile/실제 단위·release 검증 대기이며 현 작성 상태를 PASS라고 표현하지 않는다.

Root 실제 첫 unit3PASS/helperbuild0/engine source 일치를 확인했다. release는 env 조회와 channel I/O를 수행하지 않는다. 기존 environment collection에서 reserved barrier key를 제거하는 filter는 root가 모든 build에 적용했다. 이후 수정 source는 새 증거로 다시 검증한다.

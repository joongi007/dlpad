# S03: 제품 EOF keeper 종료의 sibling 보존 native 인수 설계

- 상태: proposed (읽기 감사·fixture 설계, 구현/실행 없음)
- 분류: native-regression
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: stress_recovery; 이 문서만
- 제안 후속 파일 담당: 신규 tests/cookie_keeper_sibling.rs는 fixture 담당, user_service.rs debug 관찰 지점은 root 합의 후 해당 모듈 담당; 기존 cookie_service_gate.rs 및 prototype scripts는 변경하지 않음
- 의존성: EOF keeper 제품 actual startup2/service3/download4/recovery2 scoped PASS, 최신 immutable helper+driver, actual user manager와 root shared debug 계약 승인

## 사용자 결과와 현재 제품 경계

keeper 종료 뒤 같은 parent에 살아 있는 service 또는 queued start가 있으면 이를 중단하지 않고 parent를 manager dependency가 유지하는지 확인한다. 합성 sibling은 fixture가 생성하지만 launcher의 소유 reader/keeper와 다른 Invocation을 가지므로 제품 cleanup에서 foreign sibling에 해당한다. 기존 사용자 unit은 전혀 사용하지 않는다.

현재 user_service.rs는 원 reader terminal/pidfd 확인→parent pinned.confirm_empty→receipt write/sync→parent/keeper identity 확인→keeper writerEOF→keeper terminal/Unref→reader exact release→parent Unref다. 정상 parent 직접Stop은 없다. populated1과 빈 directprocs를 strict empty로 바꾸거나 receipt 원칙을 완화해서는 안 된다.

## prototype와 제품 proof 차이

prototype holder6case는 keeper 유지 중 strict empty를 관측한 뒤 sibling을 넣고 writerEOF로 sibling active/pending 보존과 자동 GC를 확인했다. 제품 fixture는 실제 helper가 guardian I/B와256byte receipt를 생성하고 cleanup terminal GC/LC/EOF를 제공하는 경로를 시험해야 한다. prototype에는 journal bind/receipt/durable flush가 없어 이를 실제 제품 PASS로 대체할 수 없다.

기존 cookie_service_gate NativeProof::terminal은 parent GetUnitENOENT까지 요구한다. 살아 있는 sibling을 넣은 case에서 이 인수를 그대로 사용하면 정당한 parent 보존을 실패 처리하므로 새로운 fixture에서 reader/keeper terminal과 parent preserved를 분리해야 한다. 기존 성공 시험을 완화하지 않는다.

## deterministic 관찰 barrier 제안

필요한 race window는 strict empty와 receipt sync를 완료한 후 keeper writer를 닫기 전이다. I 직후 sibling을 넣으면 parent populated1로 proof가 정당하게 거절되는 다른 시험이고, 고정 sleep이나 receipt 파일 polling만으로 이 window에 들어왔다고 보장할 수 없다.

따라서 debug-only 정확 opt-in 관찰 barrier를 receipt sync 및 기존 post-receipt identity/keeperalive 검증 뒤, writerEOF 바로 앞에 두는 계약을 root에 제안한다. fixture에만 ready checkpoint를 알리고 fixture가 sibling actual state/native membership을 확인한 뒤 bounded release acknowledgement를 준다. timeout/EOF/malformed acknowledgement는 실패로 반환하여 source proof를 생략하지 않는다. release 빌드에서는 환경 조회·관찰 I/O·대기가 없어야 한다. 현재 공유CLI/FD 계약에 이 channel은 없으므로 새FD 또는 owned-temp channel의 보안·소유권 binding은 root가 승인하기 전 구현하지 않는다.

선호 channel은 별도 fixture 소유 UnixStream의 bounded1byte ready/ack이며 launcher만 참가하고 manager/guardian/tool/keeper에 상속하지 않는다. FD 전달 계약과 CLOEXEC 해제/복구를 별도 합의해야 한다. 파일 channel을 택하면 임시0700 fixture directory와 nofollow/create-new0600 고정 bounded marker/ack를 native identity로 binding하고 최대2초 absolute 관찰예산으로 제한해야 한다. arbitrary 사용자 path를 debug env로 읽게 해서는 안 된다. 기존 startup fault의1초 sleep 또는 stdout pipe를 막는 우회는 deterministic barrier가 아니다. 새로운 debug checkpoint는 credential내용·실제 cookiepath·URL을 로그에 쓰지 않는다.

## live 및 pending 두 인수

부모 fixture가 독립0700 temporary namespace/source/sentinel/journal과 nonce를 만든다. 실제 gated launcher로 C→I 검증→journal.bind→B/R→receipt checkpoint까지 진행한다. keeper는 originalInvocation/PID/start/pidfd 및 parent밖 native identity로 실제 살아 있어야 한다. fixture는 해당 parent에 별도 random sibling을 생성하며 exact serviceInvocation, Slice 관계, nonDelegate, own PID/start/pidfd/native membership을 capture한다.

live case는 실제 ActiveState=active와 살아 있는 sibling PID를 요구한다. pending case는 Typeoneshot의 control pipe EOF를 기다리는 합성 worker로 activating/nonzero Job을 유지한다. 단순2초 sleep 대신 부모 fixture의 private EOF writer로 sibling 수명을 제어해 keeper release 이후에도 같은 pending Job을 deterministically 확인한다. sibling writer는 fixture만 소유하고 launcher/manager에는 readFD만 준다. 생산 timeout 정책에 추가 상한을 넣지 않고 시험 자체의 bounded deadline만 적용한다.

sibling이 실제 필요한 dependency라는 점을 확인한 뒤 barrier ack한다. 원 guardian terminal/pidfd 소멸, keeper EOF 정상exit/pidfd 소멸 및 GetUnitENOENT, 실제 terminal frame LC/GC+EOF와 원256byte receipt identity를 검증한다. parent는 동일 Invocation/native mount/dev/inode 및 active 상태로 남아야 한다. sibling originalPID/start alive/pidfd notready, 동일 Invocation/Job이 보존돼야 한다. parent cgroup.events populated1이어도 이미 이전 시점의 실제 strict empty receipt와 그 뒤 들어온 sibling을 구분하며 새로운 empty proof라고 승인하지 않는다.

그 후 fixture의 sibling writerEOF로 합성 worker를 정상 종료시킨다. 원 exit0/PID/pidfd terminal과 exact service Unref 및 parent dependency auto-GC/GetUnitENOENT/cgroup 부재, FD baseline을 확인한다. 제품 launcher가 sibling에 Stop 요청을 보내서는 안 된다. 같은 native fixture에서 sibling-before-proof 별도 negative case를 추가한다면 receipt 미발급/cleanup unconfirmed와 foreign 보존을 요구하며 strict proof를 완화하지 않는다.

## parent Stop 금지의 검증 방식

현재 source의 ReaderContainer.release/Drop은 Invocation확인 후 Unref뿐이며 stop_owned 호출 권한은 service guard에 한정된다. native sibling/PID/pendingJob 지속은 parent Stop의 피해가 없었다는 실제 근거다. 하지만 Stop 호출0을 직접 계측한 것은 아니므로 이를 구분한다.

더 강한 회귀 후보는 stop_owned가 service 타입만 허용하는 고정 정책과 순수 시험을 root 승인 후 추가하여 slice Stop을 모든 제품 cleanup 경로에서 거절하는 것이다. 동적 D-Bus 감사를 쓰면 이번 launcher의 정확 unique sender와 own parent arg0만 match해 bounded record를 남기고 다른 unit/사용자 traffic은 수집하지 않는다. broad dbus-monitor 또는 단순 전체 log에서 Stop 문자열 부재를0호출 증명으로 쓰지 않는다. 어떤 계측을 선택하든 source 근거와 actual sibling 보존을 함께 기록해야 한다.

## 실패 정리·미검증 범위

자기 fixture writer를 닫고 owned child reap/원 pidfd terminal 및 exact own Invocation service만 정리한다. identity를 못 얻은 unit은 이름 추정Stop하지 않는다. parent 직접Stop을 실패 fallback으로 넣지 않으며 unknown 잔여물은 원시와 정확 known 상태를 root에 전달한다. fixture 자체 최대30초와 각 child/manager query/ack 상한을 기록하고 기존8h/foreign PID/사용자 desktop에 접근하지 않는다.

현재 작업은 읽기 설계만 수행했다. Rust/제품/Cargo/native 실행 없음. 새 debug channel·service-only Stop guard는 제안 상태이며 현재 구현 완료로 쓰지 않는다. 제품 timeout/receipt/cgroup strict proof 정책은 변경하지 않았다.

## 전용 socketpair 상세 계약 (root 구현 전 합의용)

파일 path channel은 사용하지 않는다. fixture가 UnixStream::pair()를 만들고 controller endpoint를 부모에 보유, peer endpoint 하나만 launcher에 explicit inherit한다. 제안 debug env 이름은 `DLPAD_TEST_COOKIE_RECEIPT_BARRIER_FD`이며 값은 공백/부호/전후문자 없는 ASCII십진수, 범위3..=i32::MAX다. 이름 최종 채택은 root 계약이다. channel은 기존 control/receipt writer/record lock 및 stdin/out/err와 다른 FD여야 한다. 설정값 오류·닫힌FD·alias·미검증 socket은 고정 invalid 오류로 실패하며 무관 FD를 임의로 닫지 않는다. release 빌드는 env 조회·FD 검증·channel I/O가 전혀 없어야 한다.

기존 tests/cookie_service_gate.rs spawn은 peer control, launch.writer, launch.lock 세 FD를 Command::pre_exec에서 F_GETFD 후 FD_CLOEXEC만 해제한다. 신규 sibling fixture의 별도 spawn helper에서 네 번째 channel peer FD를 descriptors에 포함하고 모든 FD>=3/상호상이와 실제 native identity를 부모에서 확인한다. 같은 child pre_exec 안에서 이 네 FD만 상속 허용하고 할당·filesystem I/O는 하지 않는다. 부모는 spawn 뒤 delivery peer를 drop하고 controller endpoint만 보유한다. 기존 fixture의 세 FD 계약이나 일반 공개 DownloadSession 실행 경로를 테스트 때문에 전역적으로 확장하지 않는다. Public API까지 관찰 channel이 필요해지는 경우 process spawn 소유자와 별도 합의가 필요하다.

launcher 초기 marker entry 또는 gated run 시작에서 env FD를 검증한다. fstat의 S_IFSOCK/currentUID, getsockopt SO_DOMAIN=AF_UNIX/SO_TYPE=SOCK_STREAM/정확optlen, SO_PEERCRED의 current effectiveUID 및 유효peerPID를 확인한다. fstat 전후 dev/inode를 저장해 clone에도 같은 socket identity인지 확인한다. 정상 연결 socketpair endpoint인지 SO_ACCEPTCONN=false 및 getpeername AF_UNIX connected 상태로 확인한다. listening socket이나 regular/pipe/TCP/다른UID/unconnected stream을 수락하지 않는다. root entry는 기존 세 전달FD와 alias 거절에 필요한 raw descriptors를 알고 있으므로 user_service에 넘기기 전에 이 검증 또는 명확한 distinctness 계약을 수행한다.

FD 상속 후 즉시 원 delivery FD의 CLOEXEC를 복구하고 try_clone/try_clone_to_owned가 만드는 duplicate에도 CLOEXEC를 확인한다. clone은 launcher가 소유한 UnixStream/OwnedFd로 관리하고 검증 실패 시 foreign FD ownership을 획득한 것으로 간주하지 않는다. valid delivery 원본은 명확하게 ownership을 이전해 clone 후 닫거나 자체를 보유하는 한 가지 구현을 택하고 중복 열린 descriptor를 남기지 않는다. clone 전후 fstat만으로 sameUID의 적대 프로세스를 인증했다고 주장하지 않는다. SO_PEERCRED/currentUID와 부모 fixture가 소유한 socketpair/explicit inheritance는 합성 테스트 channel의 경계이며 실제 사용자 인증 경계가 아니다. controller가 살아 있는지 poll/read로 확인하되 peerPID 숫자만으로 signal 권한을 만들지 않는다.

channel env 키는 gated 서비스 Environment property에서 제외하고 keeper에도 전달하지 않는다. numericFD가 닫혀 있어도 manager/tool env로 유출해 관찰 fixture를 제품 흐름에 노출해서는 안 된다. channel은 helper CLI argv, receipt shape 및 control protocol에 추가하지 않는다. FD_CLOEXEC와 explicit manager stdFD 목록으로 control/keeper/툴에 channel FD가 상속되지 않게 하고 실제 synthetic tool/keeper FD inventory에서 channel native socket dev/inode 부재를 확인한다. parent controller endpoint는 처음부터 CLOEXEC이며 child에 상속하지 않는다.

barrier는 receipt sync 및 post-receipt 원 reader/parent/keeper identity 검증 뒤 writerEOF 직전에 한 번만 동작한다. 고정 ready byte `K`를 controller에 보내고 ack byte `B` 하나만 받는다. 두 작업을 합친 absolute2초 deadline을 사용해 poll의 remaining budget을 재계산하며 EINTR/WouldBlock만 budget 안에서 재시도한다. read/write는1byte만, partialEOF/timeout/unknownbyte/queued extra bytes는 고정 failure다. socket에는 SIGPIPE 없는 send 또는 Rust UnixStream write의 해당 동작을 확인하고 오류를 Result로 반환한다. sleep으로 조건을 맞추거나 log 출력이 blocking barrier가 되게 하지 않는다. marker ready는 native receipt 자체가 아니므로 fixture는 실제256byte receipt의 digest/identity를 별도로 읽어 검증해야 한다.

fixture는 ready 수신 뒤 owned sibling을 실제 active 또는 activating+nonzeroJob까지 관찰한 다음 ack한다. 준비가2초 안에 완료되지 않으면 시험 실패이며 timeout을 임의로 늘리거나 ack 전에 false 조건을 통과시키지 않는다. cold manager start 비용 때문에 fixture가 안정적으로 못 들어오면 sibling 전용 job을 미리 구성해 start transaction을 빠르게 하거나 별도의 agreed test budget을 근거로 검토한다. 제품 전체 작업 timeout이나 receipt strict proof에는 이 시험예산을 적용하지 않는다. 실제 writerEOF 전에 새 sibling이 등장하므로 이때 재차 strict empty를 요구해 race를 숨기거나 새 populated 상태를 empty로 인정하지 않는다. receipt는 직전 실제 strict proof의 기록이며 keeper release 정책은 그 이후 sibling 보존을 검증하는 별도 인수다.

negative 순수/OS fixture는 env없음,0/1/2/음수/overflow/nonUTF8/nondigit, closedFD, pipe/regular/TCP/listener/unconnected, 세 전달FD alias, missing/bytewrong/extra ack/EOF/2초deadline, release compiled env 무시를 구분한다. currentUID만 허용하므로 다른UID 시험을 위해 관리자/계정 전환을 도입하지 않는다. 해당 negative는 환경이 제공하는 범위만 실제 검증으로 보고한다. 모든 소유 endpoint 종료 후 FD baseline과 owned child reap를 확인한다.

본 상세 계약은 문서 제안만이며 실제 root hook/신규Rustfixture 구현은 현재 wholechecks freeze 해제와 root 합의 후에 수행한다. 기존 source/Cargo/native 실행을 변경하지 않았다.

root가 제안한 ready K/ack B를 상세 channel 계약에 반영했다. marker 문자 자체의 고유성은 소유권 증명이 아니며 타입/UID/native socket identity/explicit launcher-only FD 수명을 우선 검증한다. 현재 Rust freeze 유지다.

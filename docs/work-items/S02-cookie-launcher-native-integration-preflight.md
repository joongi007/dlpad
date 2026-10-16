# S02: cookie launcher native 연결 사전 검토

- 상태: done (읽기 검토, 구현 아님)
- 분류: interface-review
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: stress_recovery; 이 신규 문서만
- 선행 작업: S02-cookie-launcher-fd-gate-audit, guardian wait_begin_only 실제 native 분기 시험

## 사용자 결과

같은 boot 쿠키 복구를 위한 launcher의 I 및 cleanup receipt 연결을 검증 가능한 작은 단계로 분리한다. 이 문서는 제품 연결이나 P0 완료를 주장하지 않는다.

## 현재 코드와 빈 연결

`process/guardian.rs`의 GATED_MARKER/GATED_USER_MARKER와 supervise(user_service,gated)는 index/nonce를 argv에서 검증하고, 유일 reader인 wait_begin_only가 성공한 뒤 ControlReader와 tool spawn을 시작한다. 현재 native 검증은 이 guardian 분기만의 결과다.

반면 entry의 launcher 판정은 LAUNCHER_MARKER 하나이고 launch_service는 control FD→--→tool argv만 받는다. guardian::command pre_exec도 control FD 하나만 CLOEXEC를 해제한다. `user_service.rs::run`은 항상 USER_MARKER를 prefix로 만들며 StartTransientUnit 이후 control을 drop하고 GetUnit/Invocation을 얻어 legacy wait_unit을 호출한다. launcher I 송신, pinned cgroup identity, capability bootstrap, receipt/record-lock 상속, receipt sync는 아직 연결되지 않았다.

`start`가 sd-bus로 넘기는 FD는 control/stdout/stderr뿐이다. 이 경계는 유지한다. wait_unit의 PathBuf와 empty_group/wait_cleanup의 NotFound 또는 빈 ControlGroup 성공을 durable receipt 증거로 사용하지 않는다.

## 최소 1단계: launcher 전용 capability 수신과 FD 소유권

Root가 공유 인터페이스로 확정할 최소안은 별도 gated launcher marker를 추가하고, 내부 argv를 `MARKER control_fd receipt_writer_fd record_lock_fd expected_record_dev expected_record_ino -- tool argv...`로 구분하는 것이다. index/nonce/boot/digest/receipt nativeID는 argv에 반복하지 않고 exact C packet에서 읽는다. record ID는 현재 Capability에 없으므로 위 credential-free 숫자 인수를 검증하거나 C schema에 추가하는 선택이 필요하다. 두 방식을 섞어 native identity 검증을 생략하지 않는다. 실제 source/쿠키/record path는 internal argv나 환경에 넣지 않는다.

변경 지점은 guardian::command의 optional owned cookie launch capability, entry launcher marker 판정, launch_service의 gated parsing, user_service::run의 optional gated context 또는 분리 run_gated다. 기존 ProcessSpec 필수 field와 nongated tool/version/metadata 경로는 유지한다.

ReaderLaunch.writer와 lock은 한 launcher exec의 명시 FD allowlist에만 추가한다. parent는 spawn 성공 후 전달용 writer/lock 사본을 즉시 drop하고 journal의 본 lock 소유권은 유지한다. 같은 OFD lock을 try_clone한 이유는 app crash 뒤 launcher가 종료할 때까지 잠금 수명을 유지하기 위해서다. launcher startup은 세 FD를 서로 다른 descriptor로 검증하고 즉시 CLOEXEC를 복구한다. writer는 O_WRONLY/regular0600/current UID/nlink1 및 C의 receipt dev/ino와 일치해야 한다. lock은 expected nativeID/regular0600/current UID/nlink1을 확인하며 receipt와 동일 inode이면 거절한다. lock에 임의 쓰기를 제공하지 않는다.

launcher는 StartTransientUnit 전에 control에서 정확한 C frame 하나만 bounded 읽고 decode_capability로 확인한다. 현재 reader_gate의 packet read는 내부 helper이므로 root가 read_capability_only 또는 동등한 단일 수신 API를 조정해야 한다. 최대1024bytes/20ms polling/절대 deadline/취소 정책을 재사용하고 overread로 B/T/K를 소비하지 않는다. 이 수신을 완료하면 launcher는 control의 reader 역할을 영구 종료한다. B를 launcher가 기다리는 combined send_identity_then_wait_begin은 사용하지 않는다.

## 최소 2단계: gated guardian 시작과 native I 송신

user_service::run의 command prefix만 gated context에서 `guardian, GATED_USER_MARKER, index, nonce_hex, --, tool argv...` 계약으로 바꾼다. 실제 user guardian은 control socket이 fd0에 전달되고 별도 FD argv를 소비하지 않는 기존 경로를 유지한다. direct GATED_MARKER는 descriptor argv가 필요하지만 service GATED_USER_MARKER에는 넣지 않는다. 이 구조는 현재 supervise(user_service=true)의 parsing과 일치한다. nonce는 합성 권한 token이며 credential은 아니지만 diagnostic에 노출하지 않는다. receipt/record FD는 ExecStart argv·Environment·D-Bus property에 절대 추가하지 않는다.

기존 start 호출에 control만 전달하되, I를 쓰기 위한 launcher의 UnixStream writer alias를 보유한다. sd-bus message FD 사본과 원 전달 control은 start 이후 기존대로 unref/drop한다. gated writer alias는 I 완료 후 drop한다. launch_service의 startup_control alias는 성공/실패 종료 처리까지 유지하되 read하지 않는다. 서비스 guardian만 B/T/K를 읽는다.

GetUnit/Invocation 직후 capture_gate_identity를 추가한다. 동일 unit/object/Invocation을 전후 확인하고 live MainIdentity(startticks+pidfd), /proc/PID/cgroup의 unified membership과 manager ControlGroup 일치, PinnedCgroup mount/dev/inode/leading-slash path를 확보한다. cgroup path root/512bytes 초과/다른 namespace 또는 path binding 변경은 거절한다. ReaderIdentity의 capability는 C에서 받은 값이며 observed receipt ID로 교체하지 않는다. unit의 random token16과 Invocation16, canonical identity digest를 codec 계약으로 계산한다.

launcher는 send_identity_only로 I만 쓴 뒤 기존 manager lifecycle 관찰로 이동한다. app은 C를 보낸 뒤 receive_identity_then_begin의 bind callback에서 journal.bind와 file/directory sync가 실제 성공한 다음 B를 쓴다. 따라서 app read_startup_controlled가 R부터 읽는 기존 초기화 앞에 이 선택적 교환이 필요하다. guardian은 B 전 tool/R이0이며 B 뒤 R과 기존 terminal만 쓴다. I 실패/partial/nonce 불일치/만료/취소에서는 B0과 copy 보존을 인수한다.

## 최소 3단계: exact empty와 durable receipt 후 release

wait_unit의 초기 known_pid/group를 위 native pin에서 시작하는 gated 전용 관찰을 둔다. legacy nongated 동작은 보존한다. terminal inactive/failed, 같은 Invocation, active job 부재와 같은 pinned cgroup의 populated0+정확히 빈 procs를 전후 확인한다. guardian pidfd와 membership/namespace/path binding의 수명 및 native proof는 root caller 계약이다. unit GC, path NotFound, ControlGroup 빈 문자열은 receipt 승인 근거가 아니다.

성공 proof에서 exact CleanupReceipt256bytes를 receipt writer에 write_all→sync_all하고 동일 Invocation을 다시 확인한 뒤 unref한다. launch_service는 이후 LC를 보낸다. failure 경로 stop_owned/wait_cleanup가 legacy 성공이어도 별도 pinned proof가 없으면 receipt 쓰기를 허용하지 않는다. guardian GU라도 독립 native empty 증명과 receipt가 가능할 수 있지만 현재 실행은 GC+LC+real EOF/child reap/drain 확인이 없으면 성공으로 바꾸지 않는다.

현재 systemd가 terminal 직후 cgroup을 제거하면 pinned events/procs open/read가 불가능할 수 있다. 이 경우 receipt 부재와 보수 보존이 정답이다. 실제 native 실험에서 read 가능한 시점과 proof 유지가 확인되기 전에는 pin이 있다는 이유만으로 empty를 추정하지 않는다. receipt 실패나 sync 실패에서 Resolved/cleanup을 앞당기지 않으며, Unref보다 receipt가 뒤로 가는 순서를 허용하지 않는다.

## 단계별 실제 인수 시험

| 단계 | 실제 확인 | 실패 인수 |
|---|---|---|
| FD/bootstrap | launcher의 writer/lock exact inode와 mode/access, guardian/tool의 두 FD 상속0, stdout/control 정상 | 중복 FD/동일 inode/잘못된 access/C partial·oversize·domain/expired는 tool0·foreign 보존 |
| I/B | 실제 wire C→I→durable bind→B→R, metadata/download 두 슬롯 각각 최대1회 | I 전/후 bind 실패와 before_B app crash는 tool-start0, copy/record 보존 |
| manager identity | same Invocation·MainPID/start/pidfd·membership·mount/dev/inode/path | unit 재사용/namespace/path 교체는 foreign Stop/PID signal0·receipt0 |
| receipt | actual populated0+procs empty→256bytes sync→Unref→LC+EOF | GU/escaped child/manager 누락/cgroup 조기삭제/receipt partial·sync 오류는 proof 없으면 copy 보존 |
| crash/recovery | receipt sync 후 app crash와 다음 startup anchored recovery | sync 전·launcher crash·reader live·boot 같고 receipt 없음은 자동삭제0 |

root가 새 debug driver/helper/source SHA를 고정하고 실제 사용자 manager에서 실행한다. 기존 guardian9병렬/default전체/22terminal/native cookie fixture와 FD baseline/sentinel/owned temp cleanup을 통합 확인한다. fixture checkpoint 파일은 보안 경계가 아니며 actual B wire와 제품 durable receipt를 함께 대조한다. 기존 8h unit/사용자 desktop/실계정/실영상은 대상이 아니다.

## 실행과 남은 결정

실제 수행은 기존 감사 문서와 guardian/user_service/reader_gate/reader_protocol/reader_journal/cgroup 소스 rg/sed 읽기 및 이 문서 작성뿐이다. Rust/Cargo/manager/FD 전달/native 시험은 실행하거나 수정하지 않았다. root가 record-lock expected ID bootstrap 위치, optional process adapter, bound deadline 전달 및 gated cgroup 삭제 timing 정책을 공유 인터페이스로 결정해야 한다. 이는 이전 hardlink 오류 원인 검토와 별도 scope이다.

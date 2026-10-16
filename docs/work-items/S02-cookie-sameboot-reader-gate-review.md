# S02: Linux same-boot reader gate 구현 분할 계약

- 상태: done
- 분류: review
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: metrics_recovery — 이 문서만; Rust/Cargo/기존 증거 읽기 전용
- 선행 작업: [기존 same-boot 계약](S02-same-boot-cookie-recovery-contract.md), terminal GC/GU·LC/LU, durable previous-boot record, reader unwind guard

## 현재 코드에서 해결해야 하는 공백

root는 옵션A인 I→durable bind→B→R를 선택했다. 현재 unix/recovery는 별도 파일이 아니라 cookies.rs 내부 모듈이다. begin_reader는 AtomicBool만 CAS하고 confirm은 메모리 flag만 지운다. recovery Record v1은 boot/nonce/cache/owner/cookie/record/marker identity뿐이며 same-boot는 검증 후 항상 deferred다. record4096B/marker512B, scan128+1/import128를 유지한다. Windows recovery Unavailable와 일반 DB/log의 credential 비저장은 그대로다.

현재 guardian은 tool spawn 뒤 R을 보낸다. launcher user_service::run은 Start→GetUnit→InvocationID→wait_unit→Unref 순서다. launcher의 control alias는 guardian과 **같은** socket endpoint다. launcher는 GC를 직접 관측하지 못하고 app만 GC를 읽는다. 따라서 app만 durable terminal receipt를 쓰면 app crash 후 unitGC에서 proof가 사라지는 공백을 해결하지 못한다. 같은 endpoint를 launcher도 읽는 구현은 app과 competing reader이므로 금지한다.

## root 채택: pre-B gate와 launcher cleanup-only receipt

root는 launcher가 exact Invocation/cgroup pinned identity와 empty를 확인한 후 Unref 전에 durable **cleanup-only** receipt를 쓰는 정책을 선택했다. guardianGC는 앱 execution 성공의 별도 증거로 유지한다. 같은boot copy 삭제용 receipt는 현재 native owned cleanup 검증과 동등한 wholecgroup empty+sameownedunit+same InvocationID+terminal/noactivejobs 증거다. GU/missingGC 자체는 삭제 허가가 아니지만 이후 launcher의 exact native empty 증명은 receipt를 만들 수 있다. guardian exit126 단독은 증명이 아니다.

따라서 relay나 추가 guardian ackFD는 필요 없다. launcher는 현재 socket endpoint의 write-only 역할을 유지하고 I packet만 먼저 쓴다. guardian은 gated marker에서 B를 받기 전에는 tool/R/GC/GU를 쓰지 않으므로 frame interleave가 없다. B의 유일 reader는 guardian이다. launcher는 실제 identity/cgroup pin 확인→I write를 완료한 뒤 wait_unit으로 진행한다. app은 I durable bind 후 B를 쓰고 guardian은 B를검증 후tool을spawn/R한다. EOF/T/K는 preB에서tool미시작/cleanup으로 처리한다. 기존control-reader thread를B이전에는시작하지않거나같은한reader가gate와postB를담당한다.

launcher receipt write는 app socket 생존에 의존하지 않는다. wait_unit의 actual sameidentity/nativeempty terminal 증명 성공 또는 stop_owned+wait_cleanup의 동등 증명 성공 뒤에만 receipt sync하고 그 다음 Unref/LC 또는LU를 보낸다. execution이 GU/LU여도 independent cleanup-only receipt가 valid하면 nextstartup exact copy 정리가 가능하다. receipt 실패는 execution cleanup 성공을 durable recovery 성공으로 확대하지 않으며 copy보존이다. root의 bounded Child kill이 launcher를 receipt 전에 죽이거나 manager unavailable이면deferred를유지한다. 모든crash를sameboot에반드시회복한다는보장은없다.

## root 구현 선택: 미리 생성한 write-only receipt FD

root는 directory FD writer 대신 더 좁은 write-only receipt FD와 record flock FD를 선택했다. app은 anchored owner 안의 fixed reader-0.receipt/reader-1.receipt를 O_CREAT|O_EXCL|O_NOFOLLOW,0600으로 먼저 생성하고 native fileID를 검증한다. 그 identity/nonce를 Intent에 기록하고 file/record/marker/directory sync를 마친 뒤 launcher를 실행한다. launcher는 receipt write-only FD와 record lock FD만 한 exec에 명시 상속하고 즉시 CLOEXEC로 복구한다. service/tool에는 두 FD를 전달하지 않는다. argv에 기록 path를 넣지 않는다.

launcher는 전달된 receipt FD의 regular/uid/mode/nlink/native fileID를 capability와 비교하고 고정256B cleanup receipt만 한 번 쓴다. generic basename/path/offset/data writer는 제공하지 않는다. I packet은 미리 생성된 receipt identity를 포함하고 app의 Bound durable commit이 성공해야 B를 보낸다. capability hash는 identity binding이며 같은 UID 악성코드에 대한 MAC/security boundary는 아니다. 이 선택은 구현·실험 예정이며 릴리스 아키텍처의 검증 완료로 표현하지 않는다.

## optional 실행 API와 정확한 메서드

공개 ProcessSpec 필드는 추가하지 않는다. 기존 run/run_metadata는 observer=None으로 내부 runner에 위임한다. cookie adapter만 run_with_reader(spec,cancel,progress,observer) / run_metadata_with_reader(spec,cancel,observer)를 사용한다. 아래는 구현 예정 계약이며 현재 API가 아니다.

- process::ReaderObserver trait: prepare(&mut self)->io::Result<ReaderLaunch>, bind(&mut self,&ReaderIdentity)->io::Result<()>, finish(&mut self,&TerminalReceipt)->io::Result<()>. Debug는 고정 redacted, 오류는 fixed typed. callback은 worker에서 실행되며 source/cookie path/argv를 받지 않는다.
- ReaderLaunch는 reader index0/1, random16B intent nonce, write-only receipt OwnedFd와 record flock OwnedFd이다. prepare는 미리 생성한 receipt native identity와 pending intent를 record에 sync한 뒤 반환한다. launcher exec까지 전달하는 private receipt/record-lock FD는 CLOEXEC를 원칙으로 하고 그 한 exec만 명시 상속 후 즉시 CLOEXEC 복구한다. manager/service/tool로 receipt FD를 전달하지 않는다.
- ReaderIdentity는 unit16Bnonce, InvocationID16B, boot UUID, cgroup dev/inode/mount identity 및 validated canonical path≤512B. I packet은 version1/fixed magic/intent nonce/index/identity/길이 field이며 전체≤1024B다. user_service는 actual Start/GetUnit/Invocation/cgroup 검증값만 생성한다. 불완전/oversize/잘못된nonce면 B를 보내지 않는다.
- bind는 기존 anchored cache/owner/cookie/record/marker를 재검증하고 해당 slot을 Bound로 durable commit한다. 성공 후에만 runner가 B(index+intentnonce)를 보낸다. read_startup_controlled의 전체 deadline/cancel polling을 packet까지 확장한다. 기존 GU/LU/EP typed mapping과 unknown preservation을 유지한다.
- TerminalReceipt는 version/index/intentnonce/별도 full-native identity_digest/cleanup-only proof kind/owned-native-terminal-empty=true를 고정 binary로 기록한다. launcher write_cleanup_receipt(&ReceiptCapability,&ReaderIdentity,...)가 sync 성공한 뒤 Unref/LC를 수행한다. 파일 길이≤256B, 크기 변경/partialwrite는 invalid이다. launcher는 정확한 precreated write-only receipt FD만 사용한다. arbitrary path는 받지 않는다.
- finish는 app가 GC+LC/pipeEOF/ownedChild reap까지 확인했을 때 durable receipt와recordbinding을 재검증한 뒤 guard confirm을 허용한다. 기존 confirm()->()는 durable failure를 표현하지 못하므로 cookie-only confirm_durable(self)->Result<(),CookieImportError> 또는 observer finish가 먼저 성공한 후 기존 confirm을 호출한다. sync 오류/panic/typed unknown에서 메모리 flag를 먼저 해제하지 않는다.

## record v2 최소 schema와 crash commit

Record v1을 기존 exact identity와 boot 정리용으로 계속 읽고 같은boot는 deferred로 둔다. v2는 기존필드 + 고정 slots[2]만 추가한다. slot은 Empty 또는 Intent { nonce16B, receiptFileIdentity, generation } 또는 Bound { 앞필드, receiptFileIdentity, unit16Bnonce, invocation16B, cgroup { dev,ino,mountId,path≤512B } }다. 문자열 nonce/IDs는 canonical lowercase hex만 사용한다. metadata index0/download index1이며 미확인 slot을 덮어쓰지 않는다. metadata 완료 후 download Intent durable commit은 metadata receipt를 보존한다. 세 번째 reader는 새 lease를 요구하고 fail-closed한다. tool version에는 slot/observer/cookie를 넣지 않는다.

두 bound slot의 최악 JSON 크기를 실제 serialize 시험으로4096B 이하로 제한한다. 초과하면 B0/고정오류다. owner의 추가 파일은 고정 reader-0.receipt/reader-1.receipt 두 개만 허용하며 receipt native identity를 slot에 바인딩한다. nonce file명을 무제한 생성하지 않는다. partial file/extra/foreign 항목은 삭제하지 않고128capacity를 차지한다.

기존 record+marker digest를 in-place 두번 쓰는 crash는 mismatch를 남기므로 보존한다. 정상 단계 전환은 record sync→marker digest sync→owner directory sync로 commit하고 B는 마지막 성공 뒤다. receipt 작성은 record/marker를 수정하지 않는 독립 append-once proof라 launcher와 app writer가 충돌하지 않는다. valid receipt는 slotnonce/identitydigest와 일치해야 하며 cache/owner/cookie/record/marker/receipt 각각 exact native identity 검증 후에만 deletion 근거다. marker mismatch가 crash로 생긴 경우라도 receipt만 보고 삭제하지 않는다. atomic record revision을 별도 구현하지 않는 이 보수 정책은 일부 orphan을 남기지만 외부 파일 삭제를 확대하지 않는다.

## recovery 판단과 live owner

same-boot Bound는 valid cleanup-only durable receipt 또는동등한exactnative cleanup 조회증명으로만resolved다. launcherreceipt에는sameunit/InvocationID/nativepinnedcgroup identity/terminalempty를바인딩하고guardianGC의수신여부를삭제proof로사용하지않는다. 모든nonempty slot이resolved여야cookie삭제한다. unitGC 이후에도valid durable receipt가독립증거를남긴다. unitGC/managerunavailable/Invocationreuse/cgrouprename/PID없음은receipt를대신하지않는다. syscall/manager timeout은finite이며foreignunit Stop/PIDkill을하지않는다.

restart recovery는 살아 있는 app 또는 launcher의 record lock을 nonblocking flock으로 확인한다. app과 launcher는 같은 record open description의 lock을 유지하고 recovery는 별도 open description으로 lock을 얻지 못하면 deferred다. service/tool에는 lock FD를 상속하지 않는다. 기존 same-boot 보존 정책을 lock 없이 없애면 live lease 삭제 경쟁이 생긴다. 외부 동일 UID 코드가 advisory lock을 무시하는 경우를 보안 경계로 주장하지 않는다.

## 독립 파일 담당 분리

| 담당 | 수정/신규 파일 | 공유 의존성 |
|---|---|---|
| root 통합 | process.rs, guardian.rs, user_service.rs, download.rs, cookies.rs의nestedmodule hook, 필요 ADR/catalog | observer API/gate/LC순서와legacy경로, 기존 record구현 단독 조정 |
| record 담당 | 신규 cookies/reader_records.rs | root가 기존 unix::OwnedCopy/Identity/Records 최소pub(super) bridge를 제공; slot/receipt/lock/schema와 순수 검증 unit 독점 |
| packet 담당 | 신규 process/reader_protocol.rs | purebounded I/B/receipt codec/state machine; socket I/O/manager/spawn 없음, root가module연결 |
| native fixture 담당 | 신규 tests/cookie_reader_gate.rs 및 scripts/cookie_reader_gate_probe.py | immutable rootbuilt helper/driver; OS실행은root조정 |

현재 nestedunix/recovery를 통째 새파일로 옮기는 리팩터링은 필요조건이 아니다. root만 기존 record writer/owner allowlist를 만지고 신규 pure codec/record policy는 독립파일로 병렬 작성한다. Cargo/schema를 agent가 중복 변경하지 않는다.

## 실제 crash 인수 조건

| crash/실패 지점 | 필수 관측/판정 |
|---|---|
| prepare 전/beforeI | toolstart0;intent 없음 lease는기존 소유cleanup정책, pending/partialrecord는deferred |
| Intent durable 후 I 이전 | toolstart0/B0; samebootdeferred, managerlookup/PIDabsence로삭제0 |
| I 수신 후 bind sync 이전 또는 record/marker mismatch | B0/toolstart0/삭제0 |
| bound sync 후 preB | toolstart0; receipt없는Bound는보존(안전한Begin 미전송 증거없음) |
| B 전송 도중/postB/preR | 실제B수신+toolstart evidence; appSIGKILL→guardianEOF→exactownedtreegone; receipt없으면deferred |
| R 이후 appSIGKILL | 실제Invocation/cgroup와receiptwriterlauncher생존확인; nativeempty cleanupreceipt sync→Unref→unitGC→restart exactcopycleanup 가능 |
| GC 직후 receipt sync 이전 crash | receiptpartial/없음이면삭제0, UnitGC라도보존 |
| receipt sync 이후/LC이전 appcrash | app수신LC가없어도validnativeempty cleanup-only durable receipt로nextstartupcleanup;원본 source/sentinel보존 |
| metadata receipt 후 download Intent/Bound crash | download unresolved가metadata receipt로덮이지않음 |
| guardianSIGKILL/GU/launcherSIGKILL/managerqueryfail | nativeempty proof/validreceipt없으면deferred; unrelatedunit/PID시그널0 |
| replacedreceipt/marker/cache/cookie/fileID/nonce mismatch/oversize packet | B0 또는삭제0, fixederror/secretlog0/FDbounded |
|128copy full/scan129 | unknown보존/CacheFull, scan/thread/reader count상한유지 |

모든 crash는 이번fixture owned pidfd/startticks 대상만 수행한다. 실제 계정/쿠키/사용자desktop/기존8h manager unit은 건드리지 않는다. root빌드 immutable 입력SHA/driverSHA/raw packet(credential없는fixedidentity)/DBsecretabsence/receipt exactfileID/cleanup을 함께 기록한다. sameboot durable gate 채택은 위 gate+receipt native실증 후이며 현재 Unavailable/deferred 제한을 먼저 제거하지 않는다.

## 새 pure codec 배정용 함수 목록

신규 process/reader_protocol.rs 독점은 root가 다음과 같이 배정할 수 있다: const MAX_IDENTITY_PACKET=1024/MAX_CAPABILITY_PACKET=512/MAX_RECEIPT=256; encode_identity(&ReaderIdentity)->Result<Vec<u8>,ProtocolError>; Decoder::push(&[u8])->Result<DecodeProgress,ProtocolError>(상한초과즉시거절/partialprefix누적); encode_begin(index,nonce)->fixedframe; decode_begin_exact(frame)->Result<Begin>; encode_capability/decode_capability; encode_cleanup_receipt/decode_cleanup_receipt. 타입은fixedarrays와boundedpath만가지고Debug는redacted. noI/O/process/env/Cargo. unit은overflow/truncation/nonce/indexmismatch/unknownversion/identityzero/pathnormalization/roundtrip/deterministicdigest. root가IPC와fixederrorcatalog연결을담당한다. 신규cookies/reader_records.rs는이typedcodec을참조하되root가sharedtypes visibility/module조정후배정한다.

## 실행과 미검증

실행은 rg/sed/cat 읽기와 이 문서 작성뿐이다. cookies/unix.rs와cookies/recovery.rs는 현재 없는파일임을 확인하고nestedcookies.rs를읽었다. 코드/Cargo/manager 수정 및 실제 crash시험은 하지 않았다. 새gate/receipt/lock/schema는 구현제안이며 LinuxP0완료가 아니다. OSAPI정체와fsync의hardrealtime보장도주장하지않는다.

record v2 구현 시험은 최대 두 Bound 슬롯을 실제 JSON serialize하여 encoded bytes≤4096을 assert하고 초과 시 B 전 고정 오류를 반환한다. 모델 field 수 추정만으로 상한을 검증했다고 보고하지 않는다.

root 후속 공유 schema는 Capability.digest=intent token과 별도 ReaderIdentity/CleanupReceipt.identity_digest32를 구분한다. root native I producer와 record binder는 full identity/path canonical hash를 직접 검증해야 한다. pure codec의 provided digest equality만으로 path 증명을 완료하지 않는다. [순수 record 구현](S02-cookie-reader-record-state.md)이 실제 hash 검증과 아직 미구현 anchored durability 경계를 기록한다.

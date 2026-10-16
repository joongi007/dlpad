# S02: anchored reader journal 기반

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: metrics_recovery — 신규 cookies/reader_journal.rs 및 이 문서
- 선행 작업: reader_records/reader_protocol 순수 모델

## 범위 및 API 계약

Linux 전용 ReaderJournal::create(directory:File)는 caller가 이미 검증한 owned directory FD를 받는다. directory0700/currentUID/native devino와 고정 recordstate.json0600 create-new/no-follow/CLOEXEC/link1을 확인하고 record FD에 nonblocking exclusive flock을 유지한다. root OwnedCopy bridge는 기존 cache/owner/cookie identity를 별도로 검증한다.

begin(index,nonce16,intentdigest32,boot16)->ReaderLaunch는 fixed reader0.receipt/reader1.receipt0600 create-new와 정확 native fileID를 만들고 Intent를 record4096B 내에 write/sync한 뒤 parentdirectory sync한다. 그 뒤에만 narrowed write-only receipt FD와 같은 open-description record lock clone/capability를 반환한다. bind(identity)는 canonical hash/full capability를 검증하고 Bound durable commit 이후 반환하여 caller가 B를 보낼 수 있다. resolve(index)는 exact256B receipt decode/fullmatch와 Resolved write/sync를 완료한 뒤 memory state를 바꾼다. reader pending flag 해제는 caller가 성공 반환 뒤 수행한다.

모든 I/O 전에 anchored directory/file native identity/type/UID/mode/link를 재검증한다. in-place commit 실패/partialwrite/receipt missing 또는 tamper는 poison/failclosed이며 기존 파일을 삭제하거나 incomplete proof를 정상 처리하지 않는다. Drop은 FD만 닫는다. unknown entry는 삭제하지 않는다. 이 모듈은 nextstartup recovery/실제 guardian writer 연결 완료가 아니다.

## 인수 조건

actualTempDir에서 begin/bind/receipt/resolve durable 순서, same lock busy, replaced file identity, zero/oversize/partial receipt, injected write failure/partial record, user/source/unknown 파일 보존을 확인한다. fault injection은 private generic write adapter unit만 허용하고 env hook은 추가하지 않는다. 새 dependency/thread/Cargo 수정 없음. rustfmt/sourcefreeze 뒤 root가 Cargo를 실행한다. syscall/fsync kernel 정체까지 hard realtime 보장하지 않는다.

## 구현 source freeze

reader_journal.rs는 fixed recordstate.json 및 reader0.receipt/reader1.receipt만 create-new로 생성한다. 모든 handle의 CLOEXEC, currentUID/0700 directory/0600 regular file/link1/devino를 검증하고 fstatat NOFOLLOW로 basename와 원 handle identity를 대조한다. record는4096B 이하 read_at으로 현재 memory encode와 동일한지 재검증하며 receipt는 정확256B만 읽는다. absolute filesystem path/로그/env는 사용하지 않는다. directoryFD의바깥basename/cache/owner/cookie바인딩은이모듈이확인할수없으므로rootOwnedCopybridge가별도로검증한다.

ReaderJournal::create(File), begin(index,nonce,intentdigest,boot), bind(identity), resolve(index), cleanup_eligible()를제공한다. ReaderLaunch는capability/write-onlywriter/same-open-description flockrecordlockclone이다. Intent와Bound/Resolved commit은filewrite+sync_all→parentdirectorysync→candidatecontent/nativeidentity재검증을성공한뒤memory반영한다. bind는puremodel의canonicalfullidentitySHA256을검증한다. resolve는actualreceiptdecode/fullmatch성공후durablecommit하며caller의pendingflag는성공반환전지우지않는다.

private persist_with writer adapter로Intent상태의partialJSONwrite후syntheticIoerror를주입한다. commit 실패는poisoned가되어추가조회/전이가실패하고memory는이전Intent그대로다. receipt생성직후의I/O오류도기존파일을삭제하지않고FD반환에실패하므로tool gate를열수없다. Drop은ownedFD만닫고cookie/record/receipt/unknownentry를unlink하지않는다. 실제recovery는이파일을기존v1cleanupallowlist에무조건추가하지말고별도검증해야한다.

5개시험을작성했다: 실제Intent/Bound/receiptwrite-sync/Resolved와launcherlockclone수명,Intentpartialcommit/memorypreserve,receiptinode교체+unknownsentinel보존,partial/zero/oversize/wrongreceipt unresolved,record4097Btamper보존. actualTempDir만사용한다. 아직시험을실행하지않았으며source SHA256 `605773402c0ab52584a433acd276f2dd5a04d49f85786c9e37c97a1840d0cfb4`,rustfmt exit0/Cargo미실행이다. rootmodule선언/OwnedCopybridge/launcherFDallowlist/actualnative시험대기다.

이모듈은독립journal4096B를검증하지만기존recovery.json/owner.marker의완전한v2schema합산과atomiccrossfilecommit은아직없다. journal은확인된memory를기록하는기반이며새앱시작에서record+receipt를안전하게reload/cleanup하는기능이아니다. confirmedreceipt가있어도liveownerlock/원cache-owner-cookie-fileID와전체recordmarker검증없이삭제하면안된다. fsync/metadata/lock의kernel정체까지hardrealtime보장을주장하지않는다.

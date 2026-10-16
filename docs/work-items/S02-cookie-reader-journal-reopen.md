# S02: reader journal의 증거 기반 reopen

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: metrics_recovery — cookies/reader_journal.rs 및 이 문서
- 선행 작업: durable journal13 unit 실제 rootPASS

## 계약 및 인수

root는 JournalAnchor 필수 계약을 채택했다. create의 anchor() 값은 root가 기존 exact owned recovery marker에 durable bind해야 한다. reopen(directory:File,expected:JournalAnchor)은 현재 inode 관측으로 원본 anchor를 새로 만들지 않는다. directory/record exact native identity, mode/uid/link/CLOEXEC, nonblocking flock, bounded4096 record 및 fixed receipt256을 재검증한다. record/receipt open에CREATE/TRUNCATE/DELETE를 사용하지 않는다.

Resolved reload는 full receipt match를 필수로 요구한다. Intent/Bound 미증명 상태는 cleanup 불가이며 reopen이자동resolved로전환하지않는다. boundproof는명시resolve로별도durablecommit한다. unknownentry를읽거나삭제하지않는다. wrongrecord/receipt/tamper/lockbusy/partial/oversize의actualTempDir인수와원본bytes보존을시험한다. rootOwnedCopy/launcher/nextstartup전체복구는여전히미연결이다. Cargo는root전담이며자체실행하지않는다.

## 구현 동결 handoff

JournalAnchor는directory_dev/ino와record_dev/ino를가진redactedtype다. anchor()는현재ownedjournal의원값을반환하지만그값을외부durablemarker에바인딩하는것은root의별도담당이다. ReaderJournal::reopen(File,expected)는expected필수이며현재observe한inode로anchor를추정하지않는다. O_RDWR는유한flock과후속명시resolve를위한것이고reopen자체는read_only행동이다. CREATE/TRUNCATE/unlink/자동repair를하지않는다.

reopen은정확directory와recordidentity를확인하고nonblockinglock을획득한뒤4096B이하read_at으로읽는다. fixedslotreceipt는recordcapability의expectedfileID와맞아야하며최대256B를읽는다. ReaderRecords::decode는Resolved상태마다actualreceiptfullmatch를요구한다. Intent/Bound는유효receipt가있어도자동Resolved로바꾸지않는다. Bound에서나중에명시resolve(index)를호출해야별도durablewrite/sync가수행된다. partial/zero/invalidproof가있다면Resolvedreopen은실패하며기존files는그대로다.

Resolved proof는reopen에서한번만검증하지않고모든validate/cleanup_eligible및commitcandidate검증에서실제receiptfile내용과length를재확인한다. 따라서같은inode파일이0byte로바뀌거나잘못된256Bproof로교체되어도기존memoryResolved를삭제proof로계속사용하지않는다. root의anchoredcookie/marker/liveownerlock검증은여전히필수다.

신규4개시험은Boundpending→receipt resolve→Resolvedreopen/receiptzero변조후조회및reload실패,Intentreopen과app/launcherclone의busylock/no byte mutation,wronganchor/replacedreceipt/replacedrecord보존,partialJSON/4097Brecord보존을확인한다. actualTempDir와합성receipt만사용한다. 아직실제Cargo시험은하지않았고root가수행한다. source SHA256 `63bfa4137922cc53ce29313a737685af54e7ea22a3d0ec51afdb3f7d13e94542`,rustfmt exit0. reader_records/reader_protocol/Cargo/기존cookiebridge수정없음.

일반앱시작이이API를호출하거나완전한recoverymarker에journalanchor를기록하는연결은아직없다. unknownentry/unusedfixedreceipt는읽거나삭제하지않고caller가전체ownerallowlist와capacity128을별도로판단해야한다. 이foundation을P0sameboot제품복구완료로보고하지않는다. syscall/fsync/advisorylock이kernel정체까지hardrealtime이라고주장하지않는다.

## Root 통합 검증

실제 `cargo test -p dlpad-engine --test cookie_reader_models --locked`는 수정 전후17 passed/0 failed다. `cargo clippy -p dlpad-engine --test cookie_reader_models --locked -- -D warnings`의 최초 needless_range_loop 실패를 보존하고 root가 iterator/enumerate로 수정한 뒤 exit0를 확인했다. `cargo fmt --all -- --check`도 수정 전 exit0였다. [최종 검토](../validation/evidence/cookie-reader-journal-reopen/root-review.json)는 최종 소스SHA와 원시 로그를 기록한다. 제품 복구 연결은 미완료다.

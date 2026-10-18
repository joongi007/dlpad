# S11: 버전 선택·실패 복구 기록 계약

- 상태: in-progress
- 분류: feature
- 우선순위: P1
- 요구사항: R08, R10, R11
- 담당/수정 범위: update_transport; 별도 update-journal worktree, 이 문서의 감사 결과만 우선 수정
- 선행 작업: 공개 v0.1.0 및 GitHub Pages, signed metadata/transport/staging/새 bundle 준비 인수

## 사용자 결과

Windows native identity를 보존하는 기존 A/B 새 bundle 준비와 앱 데이터 schema 호환성·선택 기록의 읽기/쓰기 경계를 감사한다.

## 계약과 범위

사용자가 2026-10-18 업데이트 구현을 명시 요청했다. 현재 v0.1.0은 첫 updater 지원 버전의 1회 수동 설치가 필요하다. 첫 지원 릴리스는 업데이트 및 관련 UX에 집중하고 별도 대형 기능은 섞지 않는다. 주 담당이 공유 schema·Cargo/lock·인터페이스를 조정한다. 기존 .git/index는 보존하고 정상 worktree를 사용한다. 공개 태그/Release·운영키 외부 공급은 구현·합성 검증과 구분하며 기존 비밀 전송 승인 범위를 유지한다. 검증은 임시 폴더/합성 fixture/private desktop만 사용한다.

## 인수 조건

앱의 idle/작업 중 gate, 서명·크기·해시 오류, 중단·stale 상태, 준비/실행 실패, 기존 A와 사용자 미디어·도구·프로필 보존을 검증한다. 원 owner 종료·새 버전 readiness·복구 결과를 구분한다. UI/OS 동작은 단위 검사로 대체하지 않는다. 운영키 없음을 임의 test key 신뢰로 대체하지 않는다.

## 검증과 handoff

진행 중. D: 여유 약1.6GiB, /tmp 여유610GiB를 확인했다. 기본 PATH의 rustc는 없으며 기존 pinned Rust/Windows 도구 경로를 확인 후 사용한다. Git 정상 worktree/main은 fc6f559이며 root의 과거 index·불명 lock을 삭제하지 않는다. 낮은 D: 공간에 새 Cargo cache를 만들지 않는다.

## 2026-10-18 감사 결과: 권고 계약 (제품 미구현)

현재 `app_install/update_prepare.rs`는 새 extraction/bundle/journal 세 경로를 create-only로 만들고 원 source·bundle capability를 보존한다. 기존 A나 global 선택은 변경하지 않는다. `active_schema.rs`의 immutable schema1은 한 registered/receipt/executable의 native identity와 digest를 묶는 개발 관측이며 predecessor·업데이트 sequence·프로필 호환·전역 선택 head가 없다. 이를 덮어쓰거나 그 generation suffix를 업데이트 revision으로 쓰지 않는다. `active_selection.rs` 재검증은 namespace/원HANDLE/bytes를 반복 검사한다. 새 coordinator는 A와 B의 정확한 active/registered/receipt binding을 모두 별도로 유지해야 한다.

`active_launch.rs`의 Started는 original suspended process의 resume 시도/ACK다. 실제 GUI·DB 준비 성공이 아니다. 현재 실행 API는 argv0 하나와 ambient environment를 사용하고 별도 readiness IPC를 전달하지 않는다. updater용 child handshake는 새 typed interface로 설계해야 하며 PID·HWND만으로 readiness 또는 교체 완료를 기록하면 안 된다. Accepted ADR0009의 원 owner/Job 자연종료 규칙은 유지한다. supervisor/child 구조 자체는 lifecycle 담당과 Root가 조정한다.

### 단일 writer와 별도 durable namespace

최소 인터페이스 권고는 `UpdateJournalLease::open_existing_or_create` → `read_head` → `begin(expected_head, request_binding)` → `append(expected_head, exact_transition)` → `inspect_cold`다. 이는 새 제안이며 현재 제품 API가 아니다. lease는 별도 private per-install update namespace의 original directory/lock HANDLE을 보유한다. persistent lock 파일을 unlink하거나 PID로 소유자를 채택하지 않는다. cold writer는 원 parent에 상대적인 정확한 leaf open, private ACL·native identity·정확 namespace·원 record 전후 검증을 통과해야 하며 `ReadRoot`나 JSON만으로 mutation capability를 복원하지 않는다. 기존 scratch revision/cold writer 시험은 해당 경계의 실험 근거이지 제품 구현이 아니다.

기록은 64KiB 이하 canonical JSON의 immutable generation/event 파일과 정확한 predecessor digest를 사용하고, 완성된 원 생성 HANDLE에서 flush/sync/readback 뒤 non-replacing publication한다. 파일명만으로 최신 generation을 선택하지 않는다. 시작 anchor에서 유일한 연쇄만 인정하고 gap/fork/중복 generation/unknown file/잘린 기록/future schema/sequence overflow는 `RecoveryRequired`로 멈춘다. 첫 구현은 journal event 상한(예: 1,024개)에 도달하면 원 기록을 보존하고 새 요청을 거절하며 자동 pruning/GC를 하지 않는다. 이 수치와 원 Windows flush/publication의 durability 수준은 Root가 실제 crash 시험 뒤 확정한다. SQLite profile DB와 file journal을 하나의 원자 transaction이라고 표현하지 않는다.

각 이벤트 최소 binding: schema/domain, journal root identity, generation/predecessor SHA, transaction nonce, A/B의 exact installation/registration/receipt/active/native exe identity 및 SHA/version, signed metadata 원 SHA·key_id·trust-policy generation·sequence/channel/target/issued/expires·package SHA/bytes, profile canonical root/native identity·compatibility contract ID, state·original process creation identity/readiness nonce/관측 결과. 긴 경로나 개인정보는 공개 로그에 복사하지 않는다. 기록은 관측/전이 사실이며 서명 검증·실행·삭제 권한을 대체하지 않는다.

### 상태 전이와 전원 소실

| 단계 | durable 사실 / 실패·cold 동작 |
|---|---|
| Prepared | B 원 bundle과 signed package를 재검증했으나 선택은 A. 취소·만료면 A 유지, B와 failure capability 보존. |
| HandoffRequested | exact A/B·expected head·profile contract를 고정. A의 모든 worker 종료 ACK 및 original owner terminal 확인 전 B의 profile 접근 금지. |
| StartingB | bounded original process/readiness nonce를 기록. Started만으로 성공 처리하지 않는다. crash/recovery에서 기존 B 프로세스 존재를 PID로 추측하지 않는다. |
| BReadOnlyReady | B가 exact build/profile 계약·필수 DB read-only preflight·필수 worker 초기화를 확인한 관측. profile 쓰기는 아직 금지. |
| ActivationCommitted | 원 head CAS와 durable event 성공 확인 후에만 B에 profile write/run 권한 ACK. crash가 ACK보다 먼저면 cold launcher는 B를 다시 검증해 재시작한다. |
| ReturnedToA | B 원 process/Job terminal 및 profile 호환이 확인된 실패만 exact A로 명시 복구. 실패 원자료·highest accepted sequence는 보존. |
| RecoveryRequired | append/publication 불명, B terminal 불명, profile 변경 불명, head/identity 충돌이면 자동 A 실행도 보류. UI가 원자료를 보존하며 재검증·명시 복구를 안내. |

commit 전에 B가 원 profile에 한 번이라도 migration/recovery/settings 쓰기를 할 수 있으면 위 원자성 주장은 성립하지 않는다. child는 commit ACK를 기다리는 별도 gated startup이어야 한다. `ActivationCommitted` 이후 정상 B 세션이 쓴 사용자 데이터를 과거 백업으로 되돌리는 자동 rollback은 금지한다. A 재실행이 안전한 것은 같은 profile compatibility ID를 엄격히 지킨 한정된 첫 지원군뿐이며, 실행 파일 복귀와 사용자 데이터 복구는 별도다. 원 A·B·패키지·journal은 보존하고 updater 성공을 근거로 제거하지 않는다.

### 프로필 보호: 첫 지원군은 schema뿐 아니라 직렬화 의미까지 동결

현재 CPU main(`accessible_cpu_main.rs`)은 `InstanceGuard::acquire`를 GUI 실행 전에 얻고 `software_host::run` 종료 뒤 해제한다. 두 번째 실행은 활성화 요청을 보내고 종료한다. 원 profile을 공유할 때 lock을 지우거나 A/B가 동시에 writer가 되는 우회는 금지한다. supervisor가 lock을 보유할지 child에게 명시 이전할지는 새 lifecycle 계약에서 하나만 선택해야 한다. 현재 API에 그런 이전은 없다.

`Store::from_connection`은 open 시 schema_migrations1/2와 settings/library 테이블 및 column/index를 생성·수정한다. `DownloadRepository::open`도 즉시 테이블/column과 adapter-local download_schema_migrations2..5를 쓴다. `PRAGMA user_version` 하나로 전체 profile을 판정할 수 없다. theme.sqlite, library/jobs DB의 실제 경로·schema_migrations와 download_schema_migrations, column/index, StoredOptions/job/attempt/publication JSON의 version·enum 의미를 함께 확인해야 한다. 현재 경로의 open/migration은 forward schema를 무조건 거절하는 사전 gate가 없으므로 기존 앱을 그대로 B readiness probe로 사용하면 안전한 downgrade를 보장하지 못한다. `Store::read_setting_readonly`/`DownloadRepository::open_readonly`는 제한된 읽기 재료이며 전체 profile compatibility 증명은 아니다.

권고안 A(최초 업데이트 지원): 첫 지원 수동 설치 A와 후속 B를 같은 명시적 compatibility ID로 고정하고 DB migrations·직렬화 필드/enum 의미·managed tool registry·소유 marker 계약을 변경하지 않는다. 업데이트 journal은 profile DB 밖에 둔다. B는 original profile instance ownership과 read-only preflight를 완료하고 commit ACK를 받은 후 기존 writable startup을 진행한다. 실제 synthetic A→B→A 시험에 queued/completed/failed/interrupted 및 settings/library/tool selection과 복구가 필요한 publication 상태를 포함한다. 과거 개발 A31/B32 Queued3·library2·theme 시험만으로 이 범위를 완료했다고 쓰지 않는다.

권고안 B(향후 incompatible release): 별도 private profile sandbox에 SQLite backup API/닫힌 원 DB의 검증된 snapshot을 만들고, 원 profile은 writer lock 아래 보존한다. live SQLite 단일 파일 복사로 WAL/SHM을 누락하거나 user media/tools/credential 전체를 복사하지 않는다. sandbox에는 download/scan/tool install 등 외부 side effect가 금지된 별도 mode가 필요하다. 성공 후 사용자 writer가 원 profile을 수정하면 과거 snapshot 복원으로 덮어쓰지 않는다. 이 isolation/merge/ownership 설계가 없으면 incompatible 자동 전환을 거절하고 수동 migration으로 분리한다. 단순 backup 존재를 rollback 보장으로 표현하지 않는다.

### trust/재시도/롤백

고정 trust key는 Root가 별도 관리하며 test key를 제품 기본 신뢰로 사용하지 않는다. key 변경은 signed metadata key_id와 무관하게 compiled/승인된 trust-policy generation으로 구분한다. 서명은 요청 시와 활성화 직전에 exact 원 bytes와 현재 keyring/time/channel/version policy로 다시 검증한다. `highest_accepted_sequence`를 journal에 단조 보존하고 rollback으로 낮추지 않는다. 새 요청은 그 high-water보다 커야 한다. 같은 transaction nonce+metadata digest+exact A/B의 재요청만 동일 진행 조회로 취급하고 중복 프로세스/새 B 생성을 금지한다. 다른 digest를 같은 sequence로 주거나 동일 nonce의 다른 binding은 충돌이다. cold restart의 이전 요청 재검증은 이미 받아들인 동일 candidate라는 별도 경로여야 하며 일반 `sequence > installed_sequence` 검증을 느슨하게 바꾸지 않는다.

이미 검증·준비한 exact A로 돌아가는 local failure recovery는 새 네트워크 downgrade 후보 수락과 다르다. manifest 만료를 무시해 새 B를 활성화하지 않으며, B가 활성화된 뒤 메타데이터의 유효기간이 지났다는 이유만으로 정상 실행을 종료하거나 자동 rollback하지 않는다. signed metadata만으로 기존 A의 복귀 권한/프로필 호환을 만들지 않는다.

## 감사 검증과 인계

실행: 위 지정 worktree에서 AGENTS/README/PRODUCT/CONTEXT, 요구사항 R08/R10/R11, ARCHITECTURE, ADR0012/0005/0009 및 `update_prepare.rs`, `active_schema.rs`, `active_selection.rs`, `active_launch.rs`, `journal_bundle.rs`, `store.rs`, `download.rs`, `instance.rs`, CPU main을 읽고 대조했다. 이 작업은 문서 감사이며 제품코드·schema·Cargo·key·기존 사용자 데이터 변경과 실제 교체·crash/전원소실 시험은 수행하지 않았다. 설계는 Root 합의와 새 인수 전까지 Proposed다. Root가 공유 계약·상태/용어·프로필 정책 및 lifecycle 담당과의 interface를 확정해야 한다.

## 후속 독립 구현 범위 합의

Root 승인으로 신규 `crates/engine/src/app_update/download.rs`와 `download/tests.rs`만 추가한다. worker 전용 signed metadata → package stream → original staged package → 별도 새 bundle 준비를 묶고 original candidate/package/bundle 및 실패 capability를 보존한다. activation/프로필/journal schema는 변경하지 않는다. 진행률은64KiB 이하 read와 동기 latest callback이며 queue는0개다. UI의 단일worker/cap1 mailbox 정책은Root가 통합한다. `PreparedDownload::verify_current`는 원 ZIP whole hash 검사에 원 reader가 필요하여 `&mut self`를 받는다. 운영키·시간·sequence의 활성화 직전 재검증은Root controller 책임이다. 루트가 공유 app_update 모듈 등록을 담당한다.

### 독립 download unit 실제 검증

`python3 scripts/check_docs.py --source-only`: PASS 1,131문서(로컬 증거 생략). 신규 reader의 단독 `/tmp/dlpad-download-reader-20261018` harness에서 `cargo test --offline --locked`는 최종6 PASS/0 FAIL, `cargo clippy --all-targets --offline --locked -- -D warnings`는 exit0이다. Rust1.94, 기존 CARGO_HOME/RUSTUP_HOME만 읽고 target은`/tmp/dlpad-download-reader-target`에 생성했다. 신규두파일에`rustfmt --edition 2024`를 적용했다. 실제 Windows 전체 chain6회귀는 소스에 추가했으나 이 agent가 실행하지 않았으며 Root native 통합 검증이 남는다. 새 dependency/version/lock/상시worker는 없다. CountingReader는64KiB이하 caller buffer를 이용하고 sample저장0/queue0이며 기존64KiB signedmetadata·stagebuffer 비용과 별도 wholepackage복사를 추가하지 않는다. native PE 증가·runtime 비용은 Root hosted/native 결과로 후속 측정한다.

## Root 합의 후 DPAPI 선택 journal 독립 구현

추가 소유 파일은 신규 `app_update/activation_journal.rs`와 `activation_journal/tests.rs`뿐이다. Windows 전용 Journal은 caller의 원 private root/고정 bootstrap File native identity·bootstrap version·profile_epoch를 DPAPI entropy 및 record header에 바인딩한다. persistent writer lock 하나, create-only pending→slot0001..0512 non-replacing publication, 원 file flush/readback/DPAPI/canonical record 재검증, predecessor cipher SHA/generation CAS를 사용한다. plaintext192KiB·ciphertext256KiB·512slot 상한이며 최대 암호문 총128MiB다. unknown/partial/fork/gap/tamper/미래schema는 보존하는 failclosed다. 원 pending 및 이미 공개한 capability는 실패/poisoned Journal에 유지하고 자동 삭제·repair하지 않는다. flush/readback이 실제 전원소실 전체 원자성 보장을 뜻하지 않으며 해당 OS 인수는 별도다.

`Journal::capture(SignedImageInput,now,cancel,deadline)`는 원 signed candidate와 원 PreparedUpdateBundle만 받아 readonly 원 bytes/identity/canonical active record SHA를 관측한다. 반환 CapturedImage는 private record만 보유하고 File/DELETE capability를 보유하지 않는다. Root lifecycle 실험에서 fresh bundle writer/delete cap이 strong ImagePin과 충돌해, Root가 capture+weak pin→fresh caps drop(삭제0)→원HANDLE strong downgrade→B RuntimeReady 순서를 수행하도록 합의했다. 이후 `commit(expected,&CapturedImage,now,..)`는 새 metadata 만료/현재 pinned keys와 exact active-selection canonical SHA·원 실행파일 nativeID/size/SHA를 재검증한 후에만 active 관측을 게시한다. UI Ready·프로필 write gate·원 A 종료 확인·owned Job은 이 module 밖 Root/lifecycle 책임이다.

활성 코드 재시작은 모든 load에서 현재 pinned keyring으로 metadata 원 서명을 다시 검증하되, 저장한 실제 validated_at으로 당시 유효기간을 확인한다. metadata가14/31일 뒤 만료되었다는 이유만으로 이미 설치된 코드를 실행 불능으로 만들지 않는다. 새로운 commit은 현재 now에서 만료/정책을 다시 확인한다. Evidence의 모든 필드는 private이며 getters는 관측을 전달할 뿐이다. Root bootstrap bridge는 실행 직전 original image cap을 새로 pin/hash/nativeID 검증해야 한다. baseline은 root/bootstrap nativeID/version/epoch에 고정된 원 host이며 임의 path가 아니다.

commit은 version·sequence 증가, exact 이전 active 보존을 요구한다. rollback은 previous signed evidence만 선택하고 첫 B의 previous=None이면 고정 BootstrapBaseline(active=None)로만 복귀한다. highest_sequence는 감소하지 않고 failed_sequence를 보존한다. 최초 B가 commit 전에 실제 실행 시도 후 실패한 경우 active=None 상태의 record_failed가 가능하며 같은 expected head/failed sequence의 재요청은 파일 추가 없는 동일 결과다. stale head CAS와 같은/낮은 sequence의 새 commit은 거절한다. Root는 단순 네트워크/hash/준비 취소에서 high-water를 올리지 않고 실제 B 실행 실패를 기록하기로 합의했다.

DPAPI는 현재 Windows 사용자 범위의 로컬 보호이며 publisher 서명을 대체하지 않는다. 같은 SID의 악성 코드, journal 전체 삭제 또는 과거의 완전한 known-prefix 복원까지 항상 탐지하는 hardware antirollback은 이번 범위 밖이라고 Root가 명시 확정했다. 정상 실패 복귀 뒤 high-water 보존과 네트워크 replay 거절은 구현한다. 운영 개인키·실제 계정·비밀 외부 전송은0이며 고정 운영키 공급·교체는Root의 별도 작업이다. 과거 journal 항목에 사용한 키를 current keyring에서 제거하면 해당 연쇄를 failclosed하므로 keyrotation 보존 정책이 필요하다.

### 실제 검증과 남은 인수

독립 `/tmp/dlpad-activation-journal-20261018` harness에서 Root 최신 metadata 단일 모듈(테스트 등록만 제거), identity enum과 pure journal 정책을 Rust1.94로 빌드했다. 첫 `cargo test --offline --locked` 6 PASS/0FAIL, `cargo clippy --all-targets --offline --locked -- -D warnings` exit0이다. getter/관측 정밀화 뒤 같은 검증을 다시 실행했다. 기존 CARGO_HOME/RUSTUP_HOME을 사용하고 target은`/tmp/dlpad-activation-journal-target`이다. 실제 engine workspace 잠금 버전/Windows API/DPAPI/native publication 검증은 Root가 수행해야 한다.

Windows 테스트 소스3개는 DPAPI wrong entropy/tamper, record_failed durable→cold open/idempotency/stale CAS, unknown 및 partial namespace 보존을 추가했다. 이 agent가 실제 Windows에서 실행했다고 보고하지 않는다. full capture→strong B Ready→commit→cold selection→rollback과 profile epoch/DB 의미 호환, 실제 crash 경계는 Root 통합 인수로 남는다. 새 crate/Cargo feature/lock 변경은 없고 Win32_Security_Cryptography 등 기존 feature를 재사용한다. record 및 pins 비용은 상한 안에서 worker가 지불하며 UI thread로 넘기지 않는다.

## 후속 stable-parent host 합의와 구현 범위

Root 요청으로 신규 `app_update/host.rs`와 `host/tests.rs`를 추가했다. 공유 main/UI/trust/catalog/control/Cargo는Root만 수정한다. `run(root,keys,key_id,bootstrap_version)`는고정OfficialHttps만사용하며, 별도 library/test `run_with_transport<T,F>`는외부Runtime변수/CLI/URL/key override없이 worker factory seam만제공한다. 실제 signed B ZIP과owned UI subprocess를검증할Rootfixture용이다. 제품idle에서network요청0이다.

부모는persistent updater-host.lock을unlink하지않고소유하며, 강한SelfImage와profile 원nativeID, private DPAPI journal을유지한다. 최초selected signed image의pin/preflight/RuntimeReady 실패는original B Job0을확인한경우에만실패sequence를기록하고exactprevious 또는boundbaseline으로한번복귀한다. journal의unknown/tamper/partial/decrypt/key오류는baseline으로우회하지않는다. active sequence보다높은과거실패high-water가있어도실제낮은active실패를관측할수있으며high-water는감소하지않는다. 이pure정책7번째회귀를추가했다.

다운로드는worker1개/latest Progress1개/join result1개이며worker는cancel+원join ACK없이detach하지않는다. UI종료·IPC EOF때는worker를취소하고원UI자연drain을유지한다. private session 최대32개·nonce별fresh생성,기존session/자료삭제0이다. 준비중실패의capability는다음명시요청까지보유하고Drop은원자료를보존한다. 10분전체전송deadline/120초startup을사용하지만협조적취소는transport의현재blocking read가끝나야관측되므로항상즉시종료된다고주장하지않는다.

Restart는current prepared generation만허용하며원package/bundle 재검증→capture+weak PreparedImage→freshcap해제(삭제0)→원UI RestartAccepted→원A leader/Job0→B strong pin/spawn/preflight/commit/RuntimeReady→journal 선택commit 순서다. ACK/IPC쓰기실패는B실행권한이아니며원UI살아있으면그수명을유지하고새update요청을막는drain모드로간다. Root검토에따라A CloseWait에는120초runtime deadline을두지않았다. 원leader소실후남은자손정리의bounded5초와살아있는UI의임의강제종료를구별한다. 준비capture실패는code4로clientcache를무효화한다. cold복귀notice(code9/gen0)의실제client표시는Root가통합검증한다.

Windows owned child/실제 A→B·실패rollback·coldreopen은Rootnativefixture에서검증해야한다. Linux단독harness의journal7+host generation/미지원4는11PASS/0FAIL,strictClippy0이며Windows지원증거로확대하지않는다. nativev2 첫회귀의journal publication sharing실패는원로그를보존했다. PublishedRelativeFile의reducedDuplicate가원FILE_OBJECT의WRITE/DELETE reservation을유지하므로,그원cap이살아있는exactrecord에만추가reader SHARE_ALL을허용하고coldreader는SHARE_READ를유지하도록수정했다. 원published share_READonly가외부write/delete를계속막는다. fixture도host와같이readonly원root를pin/verify한뒤freshcreation DELETEcap을해제한다. JournalFailure에raw_os_error관측을추가했고Rootnative재검증결과를기다린다. 공유created_relative_file기능/권한/파일은이agent가수정하지않았다.

### Root 후속 리뷰 반영

수동 재설치·보안 패치·고정키 교체 시 새 bootstrap nativeID를 과거journal의entropy로채택하지않는다. `app-update-selection-{SHA256(original bootstrap fullNativeID + bootstrap_version)}`별 독립namespace를선택하고과거기록은adopt/mutate/delete하지않는다. 같은bootstrapchain은같은namespace를coldload한다. profile최상위열거4096개/설치namespace32개상한이고기존32개상태에서33번째namespace생성은거절한다. 이것은사용자가명시수동설치한새baseline정책이며네트워크candidate의sequence회피로노출하지않는다.

Root nativev3는host Windows타입오류0이나collapsible_if4건으로strictClippy를실패했다. 원실패를보존하고let-chain으로수정했다. 성공복귀는공유 `FailureCode::Recovered=13`/generation0notice로명시하며,code9Prepared보존과구별한다. Empty-key Prepare도새generation을예약·실패완료한뒤TrustUnavailable을회신한다. Busy중복Prepare는현재worker를취소하고join ACK를받은뒤새generation Busy를한번회신하며대기중낡은progress를보내지않는다.추가intent는bool1개로병합하고Cancel은이를취소한다. 같은generation으로Busy를보내계속살아있는worker와UI를엇갈리게하지않는다.이전worker종료전새worker를시작하지않는state회귀를추가했다.

wire의ParentEventProgress는현재동기boundedpipepush이며client숨김/모달로읽기가늦으면pipebackpressure가생길수있다. worker측latest1개와wire상latest1개는같은보장이아니다. clientread ACK/명시poll에따른최신sample회신또는native nonblocking progress adapter를Root에요청했다. 숨김장기case의실제인수전에는wire전체cap1이나항상즉시취소를주장하지않는다.


### 실제 전체 전환 fixture 범위와 첫 결함 발견

Root 승인으로 제품 외 ignored `.local-tools/update-host-fixture-20261018/`에 독립 Rust fixture와 Windows direct-rustc driver를 추가했다. 동일 원 엔진 rlib를 read-only pin하고 테스트용 Ed25519 키로 합성 ZIP을 서명한다. 실제 네트워크 요청, 운영 서명키 사용, 실제 사용자 프로필·영상 사용은 없다. 각 시험은 고유 합성 프로필에 정상 SQLite DB와 sentinel을 만들고 모든 소유 Job 종료 뒤에만 자기 임시 root를 정리한다. 정상 A→B/같은 journal cold restart, B preflight 실패와 runtime crash 복귀·재생 거절, hash 실패, 취소, 중복 Busy, stale generation을 시험 대상으로 한다. 처음 성공 경로부터 실행하며 성공하지 않은 나머지를 통과로 보고하지 않는다.

최초 native-v6 실행은 A RestartAccepted와 B RuntimeReady까지 도달했으나 최종 선택이 active=None/failed sequence2가 되어 실패했다. native-v7의 고정 stage 진단은 `verify-captured / PermissionDenied / OS5`, mutation=false로 분리했다. 실제 `file_identity::open_regular_no_reparse`는 identity 조회용 FILE_READ_ATTRIBUTES만 요청하므로 그 HANDLE의 ReadFile이 거절된 결함이다. 원 identity HANDLE을 유지한 채 ReOpenFile(GENERIC_READ, SHARE_READ, OPEN_REPARSE_POINT)하고 원/새 full nativeID, 기록 ID·길이·최종 경로를 대조한 후에만 제한된 hash를 읽도록 수정했다. 원 metadata HANDLE을 읽기 권한으로 오인하지 않으며 write/delete 공유를 넓히지 않는다. native-v7 실패의 소스 관측은 unchanged=true, 원 Job natural zero, 사용자 입력 데스크톱 unchanged=true다. 수정 후 재검증 전이며 전체 전환 통과를 주장하지 않는다. 별도 recovered13/gen0 codec 거절은 Root/bootstrap 담당자가 원인을 확인하고 수정했다.


### 소스 checkout/CI 전환 회귀 범위 추가

Root 승인으로 신규 `crates/engine/src/bin/app_update_fixture.rs`를 소유한다. Cargo의 기본 OFF `app-update-fixture` feature와 required-feature bin 등록은 Root가 담당한다. 테스트 서명키와 합성 패키지는 이 bin 안에만 존재하며 제품 신뢰키를 수정하지 않는다. Windows만 실행하고 다른 OS는 Unsupported를 반환한다. `--case CASE --owned-parent ABS_EXISTING_DIR` 인수를 필수로 받아 알려진 임시 부모 아래 고유 fresh 자식 root만 생성한다. 해당 root만 성공 시 정리하고 실패 자료는 보존한다. 사용자 Documents/실제 영상/운영 계정·키에 접근하지 않는다. 작업 결과의 CASE_PASS JSON과 외부 original Job0 receipt는 서로 다른 증거로 유지한다.

native-v8의 전체 전환 정상/cold, B preflight 실패 복귀와 replay 거절, B runtime crash 복귀와 replay 거절, hash 실패가 실제 통과했다. 뒤의 취소 case는 실패 이벤트가 Cancelled1이 아닌 Disk5라 실패했다. source unchanged 및 사용자 desktop unchanged, 원 Job natural-zero는 확인했다. 원인은 stream 취소가 StageFailureKind::Io(Interrupted)로 전달되는 경로를 host가 일반 disk로 분류한 것이며, 해당 경로를 Cancelled1로, Stage IO TimedOut은 Network3으로 보존하도록 수정하고 native 실패 코드 회귀를 추가했다. busy/stale 및 전체 일괄 성공은 다음 엔진 재빌드 후 확인한다. journal의 실제 attributes-only HANDLE 읽기 거절/원 HANDLE ReOpen 읽기 성공/foreign write·rename 차단 회귀도 추가했다.


### 전체 전환 최종 실제 검증과 source freeze

추적되는 `crates/engine/src/bin/app_update_fixture.rs`를 최신 C: Windows MSVC 엔진 rlib에 직접 연결한 전체 7개 시험은 모두 통과했다. 명령은 `python3 .local-tools/update-host-fixture-20261018/run.py`이며 원 로그는 `.local-tools/update-host-fixture-20261018/run-1792313770810094027/host-fixture.log`다. 실제 bin SHA256은 `688ff2014c07356226b013de3ee78e977fd7435f92b8773e3662c49ceebbc07f`, 크기는5,494,272bytes다. success와 stale는 generation1/highest2/active0.1.2, start-fail은 generation1/highest2/failed2/activeNone, runtime-crash는 generation3/highest2/failed2/activeNone, hash·cancel·busy는 generation0/highest0/activeNone였다. 정상/cold는 worker1/request3, 실패복귀 후 replay는 worker2/request5이며 나머지는 worker1/request3을 정확히 확인했다. 모든 경우 SQLite 원 bytes와 synthetic sentinel을 보존했다. 원 engine/source/bin SHA 전후 동일, wrapper source unchanged=true, 사용자 input desktop unchanged=true, 원 Job natural-zero/강제정리0, 전체87.7초를 확인했다. 이를 실제 운영 release 다운로드/실제CPU GUI B 업데이트 시험이나 전원손실 복구 인수로 확대하지 않는다.

Root native-v11에서 새 fixture 전용 strictClippy와 engine library 빌드를 통과했다. 전체 7개 증거 후 host/journal의 임시 stage 출력만 제거하고 원 읽기권한·취소분류 수정과 신규 native 회귀를 유지했다. Root가 최종 두 파일을 통합하고 전체 workspace/release 및 CI wrapper sourcefixture 실행을 담당한다. 이 agent의 제품 소스는 freeze하며 공유 Cargo/schema/module/main/UI/trust 파일은 수정하지 않았다. first-support epoch1은 library schema2/jobs schema5를 변경하지 않는 호환 계약이다. readonly preflight는 backup·sandbox·DB rollback이 아니며 이후 schema 변경에는 별도 호환/격리 정책이 필요하다. Linux host/fixture는 명시 Unsupported이며 profile helper의 cross-platform 읽기 코드만으로 Linux updater 지원을 주장하지 않는다.

최종 전용 worktree에서 `python3 scripts/check_docs.py --source-only`는1,131문서 PASS다. 런타임 증거1,505링크는 source-only 범위에서 제외했으므로 그 명령을 원본 실행 증거 검증으로 해석하지 않는다. WSL의 해당 Windows 생성 worktree `.git`에는 `D:/Project/...` gitdir가 기록되어 `git status`가128로 실패했다. 기존 `.git`은 삭제·재구성하지 않았고 파일별 SHA로 Root 통합을 대조했다.

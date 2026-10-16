# S04: Managed pending automatic recovery authority

- 상태: proposed
- 분류: safety-contract
- 우선순위: P0
- 요구사항: R03, R05, R08, R13
- 담당/수정 범위: diagnostic_cache; 이 새 문서만
- 선행 작업: 원 root execution lock/marker, typed guardian capability handoff, platform terminal receipts

## 사용자 결과

앱 crash 후 끝난 managed execution의 pending gate만 실제 종료 증거로 회수한다. live/unconfirmed/unknown이면 관리 설치·선택·제거를 계속 보류한다. 자동 회수는 현재 미구현이며 제안이다.

## 현재 원 authority 비교

execution_marker.rs의 현 record는 고정버전 문자열과nonce뿐이다. 원root/marker File pins는live 객체에만 존재하며restart시재구성할originalIDs/receipt가없다. pending(root)는불명type/content도true이고Drop은clear하지않는다. 따라서현record만읽고새로연File의ID를original이라고채택하여clear하면안된다. execution_lease.rs의ManagedExecution.finish는정상소유scope의예약완료경계이며persistedrecord만으로복원할공개삭제API가아니다.

Windows cookie recovery(`cookies/windows/recovery.rs:49–106`)는SID보호ACL/원namespace chain과marker의원cache/owner/copy/marker nativeIDs를대조한뒤Journal::reopen에서원journal/receiptIDs·nonce·digest를확인한다. 실제all_receipts_match가없으면pristine/Resolved/boot정보로삭제하지않는다. Linux cookie reader_records/reader_journal의원record/lock/receipt capability도immutablebinding과actualnativeanchor를요구한다. Job journal의Interrupted 상태 또는guardian PID만으로실행파일삭제authority를만들지않는다.

## crash boundary별 권한

| 시점 | startup의 허용 행동 | 부족할 때 |
|---|---|---|
| marker 생성 전 | gate없음을관찰;새exclusive실행등록가능 | root검증실패면관리거절 |
| marker create 후/fullimmutablebinding 전 | 존재gate보존 | partial/empty도삭제금지 |
| binding 완성, launch Intent 전 | no-reader를restart에서추측하지않음 | 실제원receipt없으면보존 |
| Intent/Bound 후 tool live | 원lockbusy/reader미종료확인시보존 | PID미발견/mtime오래됨도보존 |
| guardian terminal receipt 작성 뒤 app finish 전 | 원ID/ACL/nonce/digest/모든stage terminalreceipt 일치+exclusivelock 후회수후보 | guardian process/receipt 하나라도불명 보존 |
| clear 도중 namespace교체 또는실패 | 원marker와foreign보존;다음start는gate유지 | 원파일누락만으로잔여receipt삭제권한없음 |

## 제안 immutable binding과 최소 interface

root가schema조정한다. `ManagedExecutionBinding`은formatversion, randomsessionnonce, original root/lock/marker native IDs, bounded original journal/receipt IDs, immutable digest, bounded selected package/component identities를포함한다. 파일명/path는고정allowlist, 사용자URL/비밀/원cookie를기록하지않는다. NativeIDs만은권한이아니며원ACL/linkcount/namespace/liveexclusivecap+원receiptbinding이필수다. self-referentialmarkerID는CREATE_NEW 원markerhandle에서얻어immutablebody완성후sync하며partial쓰기보존이다. 기존cookiebaseDigest→journal원IDs→marker원IDdigest순서를재사용가능하나별도domainnonce로cookie권한과혼용하지않는다.

최소 shared API 제안:

- `begin_managed_execution()`이원marker/binding/journalcap을생성하고typed`ManagedExecutionContext`반환. arbitrarypathclear는노출하지않음.
- `prepare_reader(stage, tool_identity)`는bounded Intent와원receiptcap을childlaunch이전에예약한다. same-stage counter재사용금지,상한초과failclosed. metadata/version/download 여러process를한session에서전부account해야한다.
- guardian의`bind_reader(native_process_creation)`/`confirm_owned_terminal(receiptcap)`가원identity와정확nativecleanupack에receipt를쓴다. helperexit/Jobkill요청만으로terminalreceipt를쓰지않는다.
- `recover_managed_pending(root)->RecoverySummary{cleared,pending,rejected}`는기존record읽기만으로authority를만들지않고strictnativechain+immutablebinding+actualallreceipt+exclusive원lock을확인한다. 모호함은pending/rejected로보존.

## 첫 구현 수직 slice

**제안: Linux의실제현재serviceguardian경로에서1개의syntheticmanagedreader crash후회수.** registeredapp을directfallback하지않는다. 원Unixcontrol SCM_RIGHTS context/receiptcap handoff→서비스guardian원ready→Intent/Bound실행→원cgroup전체cleanup→원terminalreceipt→앱SIGKILL→freshstartup검증→정확pending기록만clear의end-to-end를먼저완성한다. 기존root1pending으로동시session등록을거절하고첫slice는reader1개만명시지원한다. 나머지metadata/download 다중stage는전부account 구현전에는자동회수대상으로두지않고gate보존한다. Windows별도원SID ACL/Job/creationtime/receipt proof 구현후같은acceptance를native실행한다.

담당 조정 필요 파일: tools/execution_marker·execution_lease·새execution_journal/recovery, process typed context/reader protocol/Linux guardian+user_service keeper, Windows guardian config/capture/entry+native binding, download managed session stage registration 및 SessionCleanupDeferred, startup/appcatalog summary. 공유schema/Cargo/root 담당,동시파일수정금지. 외부툴은관리원삭제authority대상에서제외하고savedtypedresume는원선택snapshot을검증한다.

## 인수 조건

- 앱 crash 전/reader live/terminalreceipt뒤/clear중의서로다른kill checkpoints를소유임시fixture로실행.
- 원receipt없는Empty/Intent/Bound/unresolved/Resolved자기주장,wrongnonce/nativeID/ACLwiden/link/symlink/foreign/truncated/oversize를보존.
- priorboot/PIDreuse/mtime 및프로세스없음만으로clear하지않음. actualmatchingreceipt없으면이전boot도보존.
- 모든원reader receipts verified/exclusive원lock이있을때만원marker삭제; source/package/외부sentinel변경0.
- metadata/version/download multi-stage중하나라도미확정이면회수거절.
- OS별nativeproof와원sourcebinaryhash고정,원시실패보존. mock/syntheticprocess가공급trust나전체P0완료를증명하지않음.

## 검증과 handoff

읽기 감사와구체interface/첫slice제안완료. Rust/Cargo/기존문서수정없음. 실제pending자동회수실행·구현은없으며root의native실험후채택/수정한다.

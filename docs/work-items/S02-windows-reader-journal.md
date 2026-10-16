# S02: Windows reader native journal

- 상태: implemented (native I/O 기반 인수; 제품 연결 pending)
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: metrics_recovery — 신규 cookies/windows/journal.rs, root 승인에 따른 windows_records.rs receipt anchors 보강 및 이 문서
- 선행 작업: 순수 Windows records; root의 receipt original anchors 공유 계약; 실제 Windows compile/native 검증

## 사용자 결과

Windows 보류 쿠키 회수의 anchored native journal 기반을 준비한다. 순수 model을 실제 private Windows 파일/잠금/flush에 연결하지만 제품 importer/독립 guardian/public recovery는 root 후속 작업이다. 기반만으로 crash 회수 완료를 주장하지 않는다.

## 계약과 범위

부모 windows.rs의 private open/kind/acl/Descriptor/nativeidentity를 사용한다. Journal::create는 원래 owner 핸들/SID/path 및 cache/copy original anchors와 sessionnonce/originalrecorddigest를 받는다. 보호 DACL createNew journal과 receipt0/1 모두를 reader 시작 전에 만든다. receipt들의 실제 original FileId가 SessionAnchors.receipts에 포함되고 canonical bound hash에도 포함된다. 원 record와 root recovery marker에 전체 anchor를 durable로 고정해야 한다.

File::try_lock 비차단 journal lock을 생존 기간 보유한다. original owner/journal/receipt retainedhandle 및 현재 path의 ID/ACL/no-reparse/link1를 I/O 전 재검증한다. 기존 cache/copy/상위 pins와 전체 owner entry allowlist는 root caller가 검증한다. 현재 관측값으로 원본 anchor를 대체하지 않는다.

begin은 precreated empty receipt에 exact write-only 핸들을 열고 durable Intent를 commit한 뒤 ReaderWriter를 반환한다. bind는 creation identity를 포함한 Bound를 file seek/write/set_len/sync_all 후 메모리에 반영한다. resolve는 original receipt pin과 exact256 parser/fullmatch를 확인한 뒤 Resolved를 같은 방식으로 commit한다. partial state write/sync 실패는 poisoned 보존이며 rename/unlink fallback은 없다. drop은 핸들 close만 수행한다.

reopen은 원래 SessionAnchors/sessionnonce/originaldigest를 요구하고 original native identity 및 bounded4096 state/receipt native anchors를 검증하며 record lock을 다시 얻는다. all_receipts_match는 actual pinned receipts를 읽어 검증한다. persisted Resolved 자체가 삭제 권한은 아니다. 새 receipt256B와 state4096B 상한을 유지한다. record query/read 오류를0로 취급하지 않는다.

## 공유 API

Journal::create/reopen/anchors/record/receipt_anchors/begin/bind/resolve/all_receipts_match. ReaderWriter는 write-only File/original anchor/index를 소유한다. root가 helper 전달 전에 명시 handle allowlist/type/access 검증을 연결한다. 기존 windows.rs module 포함 및 public wrapper는 root 담당이다. guardian과의 lock handle 복제 수명은 실제 native fixture 전 미검증이다.

## 인수 조건

native tests는 원래 두 receipt의 정확 전이/flush/reopen, nonblocking Busy, partial record, mismatch receipt의 원 bytes 보존, 원 journal 교체 및 synthetic ACL 보호 해제를 검증한다. 실제 sourcecookie/사용자 Documents/GUI는 사용하지 않는다. root의 source freeze 동안 /tmp에서 준비하며 해제 전 제품 파일을 게시하지 않는다.

## 검증과 handoff

개별 rustfmt는 실제 exit0. 자체 Cargo/native 실행 없음. root가 Linux focused schema 및 WindowsGNU journal compile/test/clippy를 수행하고 actual 원시 evidence를 기록한다. 디렉터리 엔트리 crash 내구성, parent pinning, handle allowlist/lock duplication 및 app crash receipt는 후속 통합 인수다. 파일 sync_all만으로 directory namespace의 전원 손실 내구성을 확정하지 않는다. 새 dependency/Cargo/Unix schema 변경은 없다.

## 게시 상태

root의 전체workspace 두 번째 검증 terminal PASS 및 source freeze 해제 후 게시했다. createNew 전에 owner_path를 original handle identity/ACL과 대조하고 생성 동안 share-delete 없는 directory pin을 유지한다. write-only receipt handle은 META_ACCESS도 포함하여 data read 없이 FileId/ACL을 확인한다. 개별 rustfmt exit0, 문서479개 PASS. journal SHA256: 0379fc9247a0086e366ca2a9545edc5e2491ed0f9656b39ac7d4408d441de5a8; records SHA256: 1bb0549b41a9bda7351de73f605266eaefde2e4d6f9b33454d9cb2cb367968f9. 실제 native creationFileId/ACL/lock 검증은 root 다음 실행 전으로 미검증이다. 변경 전 records6/fullworkspace7의 증거를 이번 journal 성공 증거로 사용하지 않는다.

## root 실제 첫 native 검증 및 후속 경계

root가 첫 게시 source의 Windows native journal3시험 PASS를 확인했다(원시 Windows 실행19700). 이는 normal transitions/reopen/Busy, partial/mismatch 보존, identity swap/ACL tamper 범위이며 제품 startup 연결 증거가 아니다. 이후 root review에서 slot0Resolved+slot1Empty 상태의 unusedreceipt1 data가 기존 all_receipts_match에 포함되지 않는 결함을 발견했다. Empty/Intent original receipt도 실제 bounded read하여 비어있지 않으면 거절하도록 /tmp에 보강하고 실제 sentinel/record/payload 보존 회귀를 준비했다. default release source freeze 해제 전 제품 파일에는 반영하지 않는다. 기존3PASS가 이 새 failure path 통과를 뜻하지 않는다.

## Root actual native 검증

first 독립WindowsGNU journal3tests PASS/108filtered/exit0(private desktop/inputdesktopunchanged). 이후unusedreceipt보강을 포함한 Windows 전체4명령은 windows-journal-default-tab-workspace-native-first/result.json에서fmt/Clippy/test/release 모두exit0. Root가 원native journal4+records6 test이름 PASS 및 beforeafter/현재전체SHA 일치를 대조했다. unused receipt payload·record·외부sentinel 보존시험도 실제PASS. 최종journal sourceSHA dd231d3dae0e0ba2fd1d5eaec3d28f27a79122177418008d1c8a08a8dd9fb4db.

이것은 original FileId/ACL/lock/receipt I/O 모듈 인수다. 제품SessionCookieLease/import ownerallowlist/immutable marker/guardian 및 public recover 연결은 여전히 미구현이며 Windows startup회수는 활성화하지 않았다. 실제 appcrash/handle duplication/nativeguardian 비용은 후속 gate다.

Root 최신Linux전체7검증+2toolversion terminalcompleted/모두PASS 및현재sourceSHA일치를확인했다. Windows전체4도PASS. 제품import/recoverymarker/independentguardian/publicrecover가 연결되기전 첫릴리스 인증회수 완료로 보고하지 않는다.

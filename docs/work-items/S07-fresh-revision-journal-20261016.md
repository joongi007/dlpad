# S07: fresh revision journal 실제 파일 수직 실험

- 상태: proposed / 실제 Windows 검증 대기
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R10, R11 / ROADMAP S07
- 담당/수정 범위: field_reveal. 이 scratch의 revision_journal.rs, revision_journal_tests.rs와 검토 자료만
- 선행 작업: 원 FreshRoot/CreatedFile 비대체 게시, readonly UserBundleInspection, immutable active v1. Root 공유 schema 최종 검토

## 사용자 결과

향후 로컬 upgrade/rollback을 위한 최소 기반이다. 검증한 A/B 설치의 선택 사실을 새 sibling journal에 기록하고 cold readonly로 대조한다. 제품 UI·실제 선택 전환·실행은 없다. user upgrade/rollback 완료를 주장하지 않는다.

## 계약과 범위

ADR0005 Proposed 유지. 기존 active v1/journal/schema/Live/S/Cargo/lock 변경0, 새 deps0. 새 domain은 Proposed이며 Root만 채택한다. write API는 원 UserBundleInspection의 canonical active bytes를 직접 SHA256한다. caller JSON/hash/path만으로 권한 생성0.

한 FreshRoot를 세션 전체에 보유한다. CREATE_NEW pending→sync→원 HANDLE의 non-replacing fresh-name 게시→sync/readback 후 head ACK. 32세대·파일당64KiB·정확 namespace·NativeID·generation/predecessor/from/to·직전 선택 rollback fact를 검사한다. 취소/partial/unknown/stale/fork/상한/게시 불명확은 A/B 삭제·실행0, 자료/cap 보존과 재시도 차단. cold는 readonly pins만 보유.

제안은 새 Rust2파일과 test-only module patch뿐이다. 제품 파일 직접 수정0. Root가 채택하면 docs/work-items/에 초안을 옮기고 실제 결과/CONTEXT/ADR Proposed를 갱신한다.

## 인수 조건

- 실제 Windows TempDir의 합성 PE256B A/B로 baseline→upgrade fact→rollback fact sync/publish, writer drop 뒤 cold chain/live binding 대조. 합성 PE 실행0.
- 모든 경로에서 A/B bytes/native IDs·원 caps·외부 sentinel 보존. cold pins 생존 중 write-open 거절. 같은 root 두 번째 생성 비대체 거절.
- 취소/만료/Created·Written·Synced 취소/실제 NT rename 충돌/게시 후 취소의 phase·artifact·old head 구별. 불명확 삭제/재시도0.
- stale head/wrong from/arbitrary rollback/unknown member/partial/duplicate/unknown fields/fork/gap/replaced NativeID/future schema/overflow/33세대 거절 및 손상 원자료 보존.
- Root가 Windows 컴파일·9회귀·기존 app_install 회귀·strict clippy 실행. Linux 검사로 대체0.

## 검증과 handoff

rustfmt1.94 두 파일 exit0만 확인. Cargo/build/native/실제FS 시험 실행0. Git exit128, 기존 .git 보존. 인수 결과는 아직 없다. 상세 한계/API/비용/pins는 HANDOFF.md 및 SCHEMA_PROPOSAL.md 참조.

Root가새Proposed fact-schema와원caps·sync/nonreplace/불명결과경계를검토하고S에test-only2파일+module을stage했다.제품Live/source32/schema/lock/기본3PE변경0. 실제WindowsFS9회귀/engine strictclippy를실행하며UIprivate·package는별도동결Live/release만읽는다. 이실험은사용자upgrade나실제process/powerloss복구의완료가아니다.

Root 실제WindowsFS9회귀9PASS/0FAIL/0ignored/75.19초·strictenginealltargetsclippy0을원로그/guard/후보sha로독립검토했다. [Root검토](../validation/evidence/managed-tool-isolated-s07-fresh-journal-v1-20261016/root-review.json).256B합성PE두bundle의원bytes/nativeIDs/sentinel·32경계/취소/partial/비교체충돌/불명commit을검사했다. same-processReadonly새handle검증을별도processcrash/powerloss로과장하지않는다. 제안schema/코드는S/test-only에만있고제품Live/기존activev1/3PE/lock미변경이다.

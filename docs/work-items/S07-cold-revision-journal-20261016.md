# S07: cold journal-only append 및 lease 후보

- 상태: done-scoped experiment / 제품 채택 없음
- 분류: architecture-experiment
- 우선순위: 후속 updater 기반 / P0 제품 의무로 확정하지 않음
- 요구사항: R08, R10, R11 / S07
- 담당/수정 범위: field_reveal / s07-fresh-revision-journal-v2 새 scratch만. windows_caps는 복사본 patch 제안, 공유 Live파일은 Root 담당
- 선행 작업: v1 실제9회귀/strict clippy Root 확인. 신규 lease domain/명시 appendcap Root 검토

## 사용자 결과

후속 로컬 upgrade 기반에서 새 세션이 기존 fact journal에 새 generation만 추가하고 동시 cold writer를 명시 lease로 거절한다. 아직 제품 전환·실행 기능이 아니다.

## 계약과 범위

AUTHORITY_PROPOSAL.md 우선 검토. v1/Live/S/Cargo/lock/원 active·record schema/기존 기록 변경0. cold reader→FreshRoot 승격0. 기존 lease/records overwrite/delete0. 새 deps0. 실제 임시FS fixture와 constructor/append patch는 Root 승인 뒤 최소 범위로 준비한다.

## 인수 조건

같은 원 lease의 동시 writer 둘 중 하나만 진입. 첫 writer 정상 drop 뒤 두 번째 cold append 가능. A/B/sentinel/기존 generations bytes·NativeID 보존. readonly inspector는 쓰기권한 없음. stale head/partial/unknown/fork/replaced root·lease ID/ACL·reparse/overflow/rename불명확 시 진행 차단과 자료 보존. v1 readonly chain 호환. fresh v1과 lease cold 동시성 한계는 숨기지 않는다.

## 검증과 handoff

읽기와 제안 문서만. Cargo/native0/원파일수정0. 실제 process crash/powerloss/launcher/DB호환 인수0. Root 권한 검토 후 코드/Windows fixture 준비, Root만 실행한다.

Root 실행 전 상태: 후보/원 경계 검토 중. 실제 성공은 별도 원 결과와 독립 검토로만 기록한다. Live32/공유schema/lock 변경0.

## 실제 격리 실험 결과와 범위 정정

위 미실행 문단은 준비 당시 기록이다. v2는 실제 Windows17 중2통과/15실패였고 v3 계측에서 append용 ReOpenFile의 PermissionDenied/raw5를 관측했다. 원 실패는 보존한다. Root 검토 후 v4는 동일0x120083/share7 권한의 원 canonical parent HANDLE+정확한 leaf에 NtCreateFile FILE_OPEN을 사용했다. 권한 상향이나 CREATE/path fallback 없이17통과/0실패/0ignored와 strict engine/all-targets clippy0를 실제 확인했다. [Root 검토](../validation/evidence/managed-tool-isolated-s07-cold-journal-v4-full-20261016/root-review.json).

이 코드는 별도 scratch/S 복사본의 test-only 실험이며 Live32/설치 ZIP에는 채택하지 않았다. 정식 로컬 revision 전환·launcher·일반 DB 호환/rollback 제품 기능이 아니다. ADR0005 Proposed 전체를 사용자 확정 P0 의무로 승격하지 않으며, P0의 실제 지원 정책 시험은 [수동 A/B/A 작업](S07-manual-side-by-side-profile-20261016.md)으로 구분한다. 기존 app active/journal schema·Cargo.lock·의존성 변경 없음.

# S01: 정확히 동일한 목록의 불필요 DB 갱신 생략

- 상태: in-progress
- 분류: performance
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: root crates/engine/src/library.rs·공유통합·native 검증·문서, metrics_recovery /tmp 안전성검토/회귀만
- 선행 작업: native scanner 및 coarse 단계별 비용 실측

## 사용자 결과

파일 목록이 바뀌지 않은 주기 재확인에서 불필요한 DB 갱신 비용을 줄인다. 실제 추가·삭제·교체·이동은 기존 게시 정책으로 반영한다.

## 계약과 범위

Windows native 관측에서만 원 root 전체행 수와 관측 수, root/path 키 및 size/mtime/identity/present/SQLite storage class의 정확한 일치를 검사한다. 일치 시 candidate 검증과 기존 UPDATE/UPSERT/DELETE만 생략한다. observations clear, cancel checkpoint, 원 root 검증과 commit은 유지한다. 관측 시점 snapshot을 원 파일의 현재 HANDLE 권한으로 확대하지 않는다. schema/deps/watch/30초 주기는 변경하지 않는다.

## 인수 조건

정확 동일 scan에서 UPDATE/DELETE를 거절하는 자체 DB trigger를 두어 실제 기존행 mutation이 없음을 확인한다. 같은 수의 파일 교체·NULL/값 변경·present0·hardlinks·rename 실패복구를 검증한다. feature 회귀·전체 fmt/clippy/test/release 및 같은60초×3회 GUI 기준을 통과한 뒤 완료한다.

## 검증과 handoff

계측으로 unchanged SQL 비용을 관측했고 동등집합 증명을 독립 검토했다. 구현·실제 시험은 후속이며 성능 성공은 미확정이다.

실제 Windows feature27회귀 PASS: INSERT/UPDATE/DELETE 거절 trigger를 둔 동일 snapshot 두 번의 갱신 생략, 같은 수의 path/size/identity NULL/mtime NULL/present0 변경의 publication, hardlink alias 삭제의 publication을 검증했다. feature Clippy/release/profile 실행 PASS. 첫 CPU0.078125초·unchanged 두 회차0.09375/0.09375초이며 정확한 집합 검사 wall약35~50ms였다. GUI 예산 통과는 미확정이다. 최신 전체 workspace 검증 중이며 root가 source를 동결한다.

최초 정확동일 구현의 최신 Windows 전체 fmt/clippy/test/release도 모두PASS(소스/원CPU SHA17b57883003c… root확인)했다. 동일집합 query를 previous→observed 방향으로 뒤집는 작은 후속 후보는 같은 전체cardinality/유일path 증명을 유지하며 root조건 괄호로 모든mismatch를 제한한다. 원파일 identity 문자열도16개 임시String 대신64B 고정capacity의 직접16진수표기로 바꿔 byte동일성을 유지한다. 이후source의 native회귀/전체검증/GUI는 다시 인수해야 하므로 앞선PASS를 소급하지 않는다. 새deps/schema/clock변경0.

후속 String 직접표기·역방향 query의 실제 Windows27회귀/featureClippy/profile 실행도PASS다. 독립검토는 opaque128bit byte순서/leading0/소문자/u64최대volume에서64B 상한 및 root AND 괄호·NULL/타입동등집합 증명을 확인했다. 최신 원1만파일전체identity와 defaultCPU 전체workspace는 후속 실제 실행 중이다.

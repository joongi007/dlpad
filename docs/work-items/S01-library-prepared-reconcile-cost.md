# S01: 목록 재확인 INSERT 준비 비용 개선

- 상태: in-progress
- 분류: performance
- 우선순위: P0
- 요구사항: R01, R03, R08
- 담당/수정 범위: root crates/engine/src/library.rs, crates/engine/examples/windows_library_reconcile_cost.rs, 본문·관련성능증거; metrics_recovery /tmp preparedpatch만
- 선행 작업: 최신CPUdfae의60초×3회 loaded창 CPU 예산실패와thread계측

## 사용자 결과

파일목록의주기적재확인에서동일INSERT를행마다재준비하는비용을줄인다. 실제비용이줄었는지측정하며예산을완화하지않는다.

## 계약과 범위

한transaction에서statement를한번준비하고stream관측마다execute한뒤publication/commit전에drop한다. 취소/afterfirst/rollback/원WindowsfileID/경로·rename하드링크정책과30초fallback은그대로유지한다. schema/deps/Cargo변경0. 작업용고정native benchmark는원publicStore API를소유임시10k×3B fixture에서비교하며GUI전체예산을대신하지않는다. privateGUI/기존전체검증sourcefreeze 종료후에만root가Rust를게시한다.

## 인수 조건

원본과교정후native 동일개수·3회reconcile CPU/wall비교, 원파일/DB재열기/idempotent상태보존. 기존취소/실패rollback/identity회귀·workspace fmt/clippy/test/release를실행한다. 동일loaded60초6window/3회메모리80MiB·CPU1% 조건을모두검증한다. 개선이불충분하면후속원인을계측하며완료로승격하지않는다.

## 검증과 handoff

원60초각회평균4.60/4.89/5.26%,각3/6window13~15%CPU 초과를보존한다. 실제scan호출 stack 직접인과는아직미확정이며perfileSQL재준비는코드로확인한독립비용이다. 주담당native 실행·공유검증을통과하기전초안완료를제품완료로쓰지않는다.

2026-10-05 root가 원 library SHA와 초안 diff를 확인하고 prepared INSERT를 게시했다. 원 public API의10k×3회 CPU는1.15625/1.203125/1.171875초였으며 원파일 identity와 DB 재열기10000행을 확인했다. 수정 후 동일 native 비교 진행 중이다. 이전535소스 workspace 전체검증은 역사적 근거이고 새 example 포함536소스의 전체검증은 아직 수행하지 않았다.

同 조건 교정 후 CPU1.125/1.203125/1.140625초, 평균1.177083→1.15625초(약1.77% 감소)였다. 표본 오차와 quiet host 미확인을 고려하면 예산 해결 근거가 아니므로 in-progress를 유지한다. root가 ticks 공식·10000행·identity보존·재열기·원source와 natural Job0/private/input을 독립 대조했다. Windows feature 포함 library 회귀 실행 중이며 후속 identity/SQL 단계별 비용 계측이 필요하다.

최신536소스 Windows CPU workspace fmt/clippy/test/release 네명령을 실제 통과했고 root가 모든 source SHA·terminal 자연Job0·현재CPU SHA를 검토했다(windows-library-prepared-workspace-20261005-resume). 실제목록회귀12개 PASS. GUI CPU 예산 해결은 여전히 미검증이며 비용 분리 후속을 진행한다.

최신CPU0ed425의60초×3회 strict GUI 재시험은 모두 BUDGET_FAIL(평균5.04/4.74/4.82%, 3·6번째10초창13~14%). 메모리는 모두예산내, Job자연0/private종료/input불변이다. Thread churn으로원thread attribution은 미확정이지만 Job원CPU예산실패는유효하다. 원실패는 windows-prepared-loaded-thread-60sec-20261005-resume에보존했다. 새identity microprobe 게시로 보호소스537개가 됐으므로536개전체회귀는 이력이며 새 example all-target 검증은 후속이다.

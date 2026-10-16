# S01: 동일 CPU 예산에서 작업 구간과 대기 비용 비교

- 상태: in-progress
- 분류: experiment
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: root — 전용 validation/evidence의 독립 Cargo workspace·lock·native 실행·통합 검토. soak_recovery_audit — /tmp main.rs 초안만.
- 선행 작업: 실제 앱 원 thread 계측 성공 및 기능 180초 실패 기록.

## 사용자 결과

현재 주기 스캔이 기한을 넘는 원인을 처리 비용과 CPU/wall 대기로 분리해 비교한다. 결과가 없는 후보를 제품에 채택하지 않는다.

## 계약과 범위

A는 현재 공유 core의 CPU-first 2ms/500ms 정책, B는 복사한 4ms/1000ms wall 정책이다. 둘 모두 CPU 4/1000·전체 180초·100ms 취소 확인 대기 상한과 실제 engine scan/watch APIs를 유지한다. engine의 기존 기본 OFF scan-profile fixture로 9개 단계의 CPU/wall을 관측하고 실제 recv 대기 시간·횟수를 CPU/wall 결정별로 구분한다. 구간 활성화 횟수의 근사값을 정확한 window 수로 표현하지 않는다. 전용 Cargo와 lock만 복사하고 제품 workspace/lock/dependencies는 변경하지 않는다. 임시 합성 10,000개와 sentinel만 사용하고 native 실행은 직렬이다.

## 인수 조건

복사한 B의 구간 경계·CPU 대기 중 작업 구간 보존·지연 호출 시 보상 금지·180초 기한 우선 회귀를 확인한다. 입력 source/Cargo/lock/EXE SHA, 소유 Job/private desktop/input 보존을 대조한다. 직접 요청한 engine 스캔은 실제 GUI의 자연 periodic 완료·Next/Stop·CPU 기준을 대체하지 않는다. 후보가 기한·보존·CPU를 함께 만족하지 못하면 채택하지 않는다. 통과 후보도 별도 GUI·기능 인수가 필요하다.

## 검증과 handoff

현재 /tmp 초안은 미컴파일·미실행이다. root가 복사·회귀·컴파일을 확인한 뒤 AB/BA 비교를 직렬 실행하며 원 실패를 보존한다. 같은 CPU 예산으로 실제 처리량 개선이 없는 후보의 반복 실행은 확대하지 않는다.

독립 Windows workspace의 fmt/Clippy(all-targets -D warnings)/test/build(release --offline --locked)는 모두 exit 0이다. 복사한 B 정책의 경계·대기·기한 회귀 4개를 실제 실행했다. root가 제품 source543 불변과 독립 Cargo.toml/lock SHA 보존을 확인하고 별도 experiment.exe의 SHA를 고정했다. 초기 AB 비교를 직렬 실행하며 A의 실패가 B 비교 실행을 막지 않도록 원 case별 결과를 보존한다. 이는 제품 채택이나 GUI 성능 인수가 아니다.

후속 C 후보의 /tmp 초안도 준비했다. C는 wall 2ms/500ms·CPU 4/1000·CPU/Stop 대기 100ms·전체 180초를 유지하면서 스캔 중 watch.poll만 500ms로 바꾸는 실험이다. 매 poll의 root metadata 조회 비용 감소가 목표이며 root 복구·이벤트 인식은 최대 500ms+checkpoint/OS로 느려질 수 있다. 관련 요구사항은 목록 동기화 R03과 작업 반응 R08이며 R01 지표 cadence는 변경하지 않는다. 아직 실행·제품 채택하지 않았다.

A1 현재 2ms/500ms CPU-first 단독 scan은 180.001초·원 thread CPU 0.703125초에서 budget-deadline Cancelled로 실패했다. 전체 actual recv 대기는 CPU 결정 107.285초(997회)·wall 결정 72.073초(708회)이며 watch poll 1,551회·callback 68,932회였다. profile의 preflight 9.359초/CPU 0.03125초, traversal+INSERT 170.641초/CPU 0.671875초에서 중단되었다. baseline warmup은 별도 0.336초/CPU 0.328125초다. 원 native는 전체 10,000개 payload 및 sample ID 2개/128행을 보존하고 자연 Job0·private/input 정리를 확인했다. root는 B 비교가 끝난 뒤 전체 DB를 독립 대조한다. B1은 실행 중이며 A 실패를 덮지 않는다.

B1 4ms/1000ms도 180.014초·CPU 0.671875초에서 deadline Cancelled로 실패했다. watch poll 1,632회, CPU 결정 대기 127.169초, wall 결정 대기 52.228초였다. 따라서 B를 제품에 채택하지 않으며 실패 후보의 반복 AB/BA 확대는 하지 않는다. A1/B1 raw 결과와 별도 warmup/profile을 보존한다. 현재 C watch500 후보를 새 독립 workspace에 복사해 컴파일·회귀 후 동일 A/C 비교를 준비한다. 원 A/B evidence와 제품 Cargo/lock은 변경하지 않는다.

C 전용 workspace도 Windows fmt/Clippy/test/release 네 단계와 복사한 B의 회귀 4개를 exit 0으로 확인했다. C1→A2 순서로 동일 컴파일 입력의 두 후보를 직렬 실행해 순서·source 차이를 구분한다. A1/B1의 최종 readonly DB 10,000행 전체 root/경로/size/present/identity 형태·integrity ok와 sentinel 및 native 원 payload 보존 receipt도 root가 대조했다. baseline 전체 DB 사본은 없으므로 byte 단위 롤백 동일성은 주장하지 않는다. [한정된 DB 검토](../validation/evidence/windows-composed-budget-phase-20261006-continue/root-db-review.json).

# S01: 주기 스캔의 CPU 예산과 우선 명령

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: root crates/core/src/library_budget.rs·lib.rs, engine watch.rs·library.rs, app accessible_library.rs, CONTEXT·ADR0002·검증 문서; metrics_recovery /tmp 설계 독립 검토
- 선행 작업: 마지막 게시 목록 readonly Next, 최신 전체 Windows4검증, TEMP 동시성 실측

## 사용자 결과

조용한 폴더의 주기 확인 부하를 분산한다. 수동 새로고침·다음 페이지·실제 변경·복구·종료 요청을 대기 중 우선 처리한다.

## 계약과 범위

30초 확인 요청 정책은 유지하지만 주기 scan 완료는 최대180초의 협조적 기한을 갖는 제안이다. 개별 OS/SQLite 호출은 강제100ms응답을 보장하지 않는다. full fresh128bit identity와 게시 rollback을 유지한다. core 순수 시간 정책, engine checkpoint·typed watch cause, worker cap1 command와 고정 크기 pending만 도입한다. 추가 thread/mutex/dependency/Cargo/schema 변경 없이 root가 공유 계약을 조정한다. 실제 변경·복구·명시 명령은 빠른 scan, Next는 readonly 페이지를 유지한다.

## 인수 조건

가상 시간 예산 경계·기한·catchup burst 방지. debounce pending 변경의 cause 보존. 명령 serial ACK·root 변경 경계·Stop·채널 종료·예상 preemption·진짜 실패 구분. 마지막 게시 목록 보존. Windows TEMP/read/write 실측, 실패 rollback 회귀, 격리 CloseWait/Next 및60초GUI3회 원 CPU 기준 검증. 단위검증만으로 GUI 성능 완료를 주장하지 않는다.

## 검증과 handoff

현재 제품 후보는1500µs/500ms wall pacing에 원Windows worker CPU counter/0.4% 부채 대기를 추가한 source543이다. 전체Windows4명령PASS지만 GUI3회중첫10초1회FAIL이어서 인수는미완료다. [CPU 후보와현재증거](S01-periodic-thread-cpu-budget.md), [구간별후속계측](S00-gui-idle-window-attribution.md)을따른다. 실제최신완료/Next/Stop은진행중이다.

### 이전 후보의 구현과 측정 이력

아직 제품 pacing 미통합이다. 설계는 /tmp 독립 검토했고 native fixture 선행 인수를 실행 중이다. 기존 CPU 기준을 완화하지 않는다. snapshot 지연과 총 scan 비용을 실측하여 채택 여부를 결정한다.

구현 후보는 active2ms/slot500ms로 이상적 처리시간 약82초의 기한 여유를 확보한다. hard CPU quota는 아니다. core 가상 시간3회귀와 worker Next serial/rollback·cause 우선 회귀를 추가했다. 실측 전 최종 채택은 아니다.

Windows CPU 전체 fmt/clippy/test/release4명령 PASS, source542/root source equality·현재CPU c744b913f06b3dd77911dd5aa6d4482edceabb046059b5a443e072fcf5ebced6·원 natural Job0/private/input guard PASS. 새 core3/worker2각후보/watch cause 회귀 PASS. [전체 증거](../validation/evidence/windows-periodic-budget-workspace-20261006-continue/result.json). 격리GUI60초3회 실행 중이며 성능 완료는 아니다.

3회 GUI 평균CPU0.57275/0.49434/0.67650%, 최대10초0.92789/1.56127/1.09273%로 두 회차 strictCPU FAIL, 메모리/원 자연Job/private/input guard PASS. [원증거](../validation/evidence/windows-periodic-budget-gui-60sec-20261006-continue/gui/result.json). 전체 source/CPU 당시일치 root검토. DB/payload 전수재검토는 아직 미완료다. 마지막 구간 부하의 원인을 단정하지 않고 단일 주기 scan 단계계측 후속을 준비한다.

2ms/500ms 후보는실제GUI완료118.8초와Next/Stop rollback/ACK/자연종료를통과했다. 엄격10초CPU2회실패 때문에active1500µs/slot500ms를다음후보로비교한다. wake 주기/30초요청/180초기한·빠른변경/명시명령은유지한다. 예상전체완료약160초는추정이며실제완료인수를다시실행한다. 원CPU기준이나기한을완화하지않는다.

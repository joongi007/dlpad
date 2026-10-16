# S00: 현재 MSVC 앱의 격리 GUI 비용

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R03, R11
- 담당/수정 범위: root — scripts/windows_msvc_current_install_gui_cost_probe.py 통합·실행·증거/문서; soak_recovery_audit — /tmp collector 초안; 독립 검토는 측정 후 별도 배정
- 선행 작업: 현재 source576 MSVC 전체4검사와 설치 선택본 격리 GUI scoped PASS.

## 사용자 결과

MSVC 앱의 목록 표시 후 대기 상태 CPU·메모리 비용을 현재 바이너리에서 확인한다.

## 계약과 범위

기존 GNU 비용 collector/증거를 유지하고 새 collector에서 MSVC CPU SHA121f02cd…/8,709,632B를 복사해 기존 private observer로 측정한다. 세 합성 root 각1만파일, 세60초·18개 고정10초 window, 원 Job 전체와 thread 비용, 80MiB·단일 core CPU1% 예산을 사용한다. concurrent dlpad probe는 실행하지 않는다. 합성 파일과 DB·원GNU/lock/source 보존을 확인한다. 새 의존성이나 앱runtime 변경은 없다. physical IME/reader·cleanOS/최소OS/8시간 GUI 시험으로 확대하지 않는다.

## 인수 조건

원 native compile/measurement exit0·자연 Job0·input desktop 불변, 현재source/CPU SHA와 GUI metrics 원 Job/thread 샘플, 18개 window CPU<1%/WS·private<=80MiB, 원 합성파일3만·DB3개 보존. root가 원 counter에서 재계산하고 readonly DB·화면을 확인하며 독립 검토를 저장한다.

## 검증과 handoff

private measurement를 시작했다. 기존 GNU의 통과 결과를 MSVC에 소급하지 않으며 실제 terminal 뒤 인수한다.

첫 [실제 MSVC 측정](../validation/evidence/windows-msvc-current-gui-cost-20261006-continue/collector-result.json)은 FAIL로 보존했다. 실제3round·18window 중 round2/window6의 CPU1.0902568715%가1%미만 기준을 초과했고 나머지17window와 모든 memory 예산은 통과했다. Job/GUI는 자연 종료·private/input 복원됐으며 FAIL은 취소/권한 거절이 아니다. 원 thread counter에서 library62.5ms, appearance/metrics/미분류thread 각15.625ms가 관측됐으나 thread churn 때문에 이 window 전체 attribution은 미완료다. 원 counter를 임의로 빼거나 기준을 낮추지 않는다.

diagnose 절차로 ①목록 주기 점검과 worker 동시 실행 ②Windows CPU counter 분해능 ③MSVC 조회 비용 차이의 가설을 기록했다. 먼저 동일 예산의1round60초 최소 재현으로 신호를 확인하고 필요하면 defaultOFF phase 경계 계측을 추가한다. 원3round 인수와1round 진단을 구분하며 원인은 아직 미확정이다. root/독립 counter·readonly DB 검토는 실패 자체를 성공으로 바꾸지 않는다.

최소1round60초은 같은 CPU1%/80MiB 기준의6window를 통과했고 최대CPU0.4722557385%다. 원18window FAIL이 해결됐다고 판단하지 않는다. 기존 defaultOFF engine library-scan-profile-fixture의9phase 계측을 사용해 scripts/windows_msvc_reconcile_profile_collect.py에서 합성 public reconcile API의 MSVC 실제 단계 비용을 수집한다. 새feature/dependency/Rustsource는 추가하지 않고 원 GUI default 바이너리를 보존한다. console형 합성 example의 stderr를 원host에서 제한해 읽으며, private GUI의 stderr를 수집했다고 표현하지 않는다. 이 분리 측정은 GUI 원 실패의 자동 재현이나 P0 비용 인수를 대신하지 않는다.

동일 바이너리의 새3round/18window 반복은 actual PASS, round별최대CPU0.629951/0.936544/0.782472%다. 원1.09% FAIL을 지우지 않으며 원인이나 수정 완료를 확정하지 않는다. 이후 root가 원18counter를 재계산하고 원3만 합성 payload01,02,03 및 readonlyDB3 integrity/count10000·SHA불변을 확인했다. [실제 profile](../validation/evidence/windows-msvc-reconcile-profile-20261006-continue/result.json)의 unpaced public API3회 CPU343.75/343.75/328.125ms에서 traversal+INSERT가 주요 비용으로 확인됐지만 GUI pacing/원window의 직접 attribution은 별도다. 새 감독 수명 구현 뒤 source map은 변하므로 이전 고정 바이너리 비용과 새제품 source 인수를 구분한다.

후속 source 변경을 위한 새 담당 파일 scripts/windows_msvc_whole_gui_cost_probe.py를 root가 통합했다. 원121f 고정 collector는 보존하고 새 collector는 strict current whole proof의 정확 CPU SHA/bytes·128MiB 상한과 원파일 identity-aware SHA를 사용한다. 기존 private3×60초/18창·CPU<1%/80MiB·동시dlpadprobe금지·원GNU/lock/source 보존 기준은 그대로다. 실제 측정은 descendant native 인수 종료 후 직렬 실행할 예정이며 아직 새 source 성능 완료를 주장하지 않는다.

현재 source577 CPU594676ba…의 [새 실제18구간 측정](../validation/evidence/windows-msvc-source577-current-gui-cost-20261006-continue/collector-result.json)은 FAIL이다. round2/window1은 duration10.0202568초·Job CPU1,250,000ticks(100ns)로 CPU1.2474730188551655%다. thread churn 없이 main781,250ticks/library468,750ticks가 관측됐으나 단계별 원인 귀속은 미확정이다. 다른17구간 CPU와 전체18구간 메모리는 통과했다(최대WS72,036,352B). 실제 GUI3회 exit0·원Job0·private/input 복원은 확인됐고, wrapper는 budget FAIL 직후 최종 source/pin 검사 전에 종료했으므로 그 검사를 통과했다고 보고하지 않는다. 후속 독립 source/payload/readonly DB 검토와 defaultOFF 프레임 계측을 분리 기록한다. 이전 바이너리의 반복 PASS를 이 실패 해결로 소급하지 않는다.

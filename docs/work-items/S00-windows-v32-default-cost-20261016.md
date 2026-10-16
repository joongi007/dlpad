# S00: 기본 v32의 실측 비용과 누락 라운드 복구

- 상태: done-scoped
- 분류: validation
- 우선순위: P0
- 요구사항: R06, R11 / S00·S06 / PERFORMANCE
- 담당/수정 범위: Root 실행·통합, field_reveal은 새 third-round probe 후보만, p0_audit는 원 결과 읽기 감사만. 제품/공유 Cargo/lock 변경 없음.
- 선행 작업: whole32/기본CPU·3PE·source675 일치, 모든 다른 dlpad 컴파일/GUI/대용량 hash 종료 확인

## 사용자 결과

실제 기본 설치 후보를 합성10,000개 파일의 로드된 목록에서 관측한다. 기능 feature 진단이나 과거 버전 수치를 현재 shipping 인수로 대신하지 않는다.

## 계약과 범위

원 v24 observer/collector를 바이트 동일하게 사용하는 current32-default18 준비의 실제 실행이 선행했다. 첫 두60초 라운드는 완료했고, 세 번째는 측정 전 UIA ElementNotAvailable로 중단됐다. 원 결과 전체는 FAIL로 보존한다. 새 후보는 누락된 세 번째만 측정하며 원12구간을 다시 실행하거나 버리지 않는다.

Windows의 원 HWND/PID/creation·private desktop을 유지해 UIA 읽기 setup에서만 bounded 재조회한다. 기존 Ready/LoadedReady·10안정표본·2초 안정화·30초/150초 상한과 WholeJobCost·ThreadCostPins 산식/원 counter/고정10초·CPU<1%·WS/private80MiB를 바꾸지 않는다. 다른 observer 준비 버전의 결과를 결합한다는 사실과 동일 측정 코드 증거를 명시한다.

## 인수 조건

각 실제60초의 mean 원 counter/elapsed와6구간을 독립 재계산한다. strict 실패는 보존하고 평균으로 숨기지 않는다. 원 measured 세 process의 합성30k 파일/readonly3DB·6PNG·소유 종료·제품 source/3PE를 Root가 확인한다. setup 실패 root의 추가10k는 측정 데이터로 세지 않는다. quiet_host_verified=false이며 물리 host의 무부하를 주장하지 않는다.

## 실제 실행과 handoff

`cost-collector.py ...whole32/result.json ...current32-default18-budget-20261016 --no-concurrent-dlpad-probes`는 원 FAIL이다. 두평균은0.5202217476412626%/0.5982905234453654%, 메모리12PASS·strictCPU11PASS/1FAIL(두 번째3구간1.2475643800608072%). 세 번째는 cost없음/원Job active1→owned termination→empty·desktop 복구다. 앱 crash 또는 자연종료3회 성공으로 해석하지 않는다. [원 결과](../validation/evidence/current32-default18-budget-20261016/collector-result.json). 세 번째만 복구 실행해 native0/Job0·60.0577074초·평균0.41626630589631863%·6CPU/6메모리 PASS를 확인했다. 측정 전 setup 조회만 바뀐 혼합 observer 준비 버전이며 측정 prefix는 바이트 동일하다.


## 최종 실제 검증

Root의 `python3 -B /tmp/dlpad-root-performance-final.py`가 원 counter로 평균과18구간을 재계산하고 source675 hash·동일 측정 prefix·원 중단 시도 보존·자연종료3회를 확인했다. 별도 Windows readonly `verify-30000`는 파일30,000개의 실제 payload/native128bit ID/mtime과 DB3개 integrity·전후SHA를 확인해 native0/Job0이다. 원2회12구간은 재측정하지 않았다. Root가 측정6PNG를 모두 직접 열었다.

최종 strict CPU17 PASS/1 FAIL, 메모리18 PASS. 합산 관측 시간180.1952167초, CPU9,218,750 ticks(100ns), 시간 가중 평균0.511597930779036%. 평균으로1.2475643800608072% 구간 실패를 숨기지 않는다. 무부하 host·물리 IME·전체 성능 인수는 미검증이며 추가 기능 개발로 확대하지 않는다. [원 검토](../validation/evidence/current32-default18-composite-root-20261016/root-review.json).

# S01: 유휴 worker의 주기적 스캔 구간 비용

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: root 새 standalone evidence/원실행/통합; metrics_recovery /tmp 초안
- 선행 작업: 실제 GUI CPU FAIL, scan없는 감시 및 channel 대조

## 사용자 결과

유휴 CPU 초과를 스캔 횟수와 저장소 열기·목록 조사·페이지 읽기 비용으로 분리한다.

## 계약과 범위

제품 소스/Cargo.lock/schema 수정 없음. 기존 default-off engine profile fixture만 isolated manifest에서 사용한다. actual worker의 명령 대기/WatchSession/Store open+reconcile+page 경로를 합성 media/data 분리 root에서 재현한다. 원 GUI의 스레드 비용과 standalone을 동일한 측정으로 주장하지 않는다. metadata freshness/실패경로/정책 유지.

## 인수 조건

warmup 제외60초 bounded loop와90초 deadline, scan최대16건·원 elapsed/threadCPU/phase, 원10k파일/sentinel과 DB 정확행 보존, native Job 자연종료/가상desktop/input불변을 확인한다. poll dirty를 raw event원인 확정으로 해석하지 않는다.

## 검증과 handoff

실제 baseline 실험은 60.0575초/CPU평균1.32685%, 두 scan 각CPU0.34375초였다. 원 phase에서 traversal/INSERT가 지배했고 Store open/page는CPU계수 해상도이하였다. native 본체가 원1만 파일 payload/sentinel·2개 identity와 before/after를 확인하고 root가 source·fixture 두 핀·DB1만행/integrity를 검토했다. root의 추가 모든nativeID 재열기는 하지 않았다. self-hint 증폭은 관측하지 않았으나 aggregate dirty만으로 event 원인을 확정하지 않는다. [원증거](../validation/evidence/windows-quiet-worker-cost-20261006-continue/result.json).

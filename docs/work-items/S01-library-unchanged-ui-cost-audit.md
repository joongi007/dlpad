# S01: 동일 목록의 UI 반영 비용 검토

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: soak_recovery_audit accessible_library.rs/accessible_main.rs 읽기 전용 /tmp 검토; root 원계약·후속통합판단
- 선행 작업: engine native 비용 최적화와 최신 GUI60초 측정

## 사용자 결과

목록이 그대로인 재확인에서 화면 반영의 불필요 비용이 남아 있는지 확인하고, 의미 있는 경우 기존 오류·새 파일 관측·취소 상태를 유지하는 후속을 선정한다.

## 계약과 범위

DESIGN/ARCHITECTURE와 기존 snapshot 흐름을 기준으로 동일128행의 재구성·메시지·렌더링 여부를 원code에서 확인한다. 실제CPU 원인을 코드만으로 확정하지 않는다. 현재 GUI시험 중 보호소스는 변경하지 않으며 원시험을 방해하는 별도native/Cargo 실행도 하지 않는다. 실험·구현은 root와 별도후속계약으로 조정한다.

## 인수 조건

실제code 위치·samepoint snapshot/error/cancel 정책·원본과 변경후 동작 차이를 기록한다. 시험하지 않은 UI 비용 감소를 성공으로 집계하지 않는다.

## 검증과 handoff

읽기 검토를 병렬로 배정했다. actual GUI 비용 측정은 root만 실행한다.

# S06: CPU 지표·진단 fixture 공유 통합

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R01, R06, R08, R13
- 담당/수정 범위: root; accessible.rs 및 두 entry, 본 문서와 통합 검증
- 선행 작업: S06-cpu-metrics, S10-cpu-report-stress, callback 숨김 GUI

## 사용자 결과

대상 볼륨 여유·선택 인터페이스·실제 처리 배속을 CPU 후보에 표시하며 종료 시 지표 worker ack를 기다린다.

## 계약과 범위

worker 단일/초당1회/latest revision/3초 stale unknown. UI filesystem I/O 없음. report stress는 --fixture와 DLPAD_TEST_REPORT_STRESS=1 모두 있을 때 실제 실패 보고서 표시만 교체한다. 원래 persisted job/attempt·프로세스 실패는 변경하지 않는다. 고정 유효 builder 외부 입력 없음. Cargo/schema/new dependency 없음. 지표 관측에 따른 초당 repaint·단일 worker 비용을 실제 측정한다.

## 인수 조건

CPU clippy/실패경로 unit/release, 격리 지표 GUI와 최대 유효 진단 copy/save/collision/원job 보존. 기본 workspace와 native Windows 입력 대조, 최신 snapshot 비용을 별도 측정한다.

## 검증과 handoff

callback b8bb 두 숨김복귀 GUI terminal PASS 뒤 source freeze 해제하고 공유 연결한다. 새 source 검증 진행 중이며 이전 snapshot 결과를 재사용해 성공 처리하지 않는다.

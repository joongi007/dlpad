# S00: 현재 입력 인수표 정합성

- 상태: completed (문서 인수 범위)
- 분류: documentation
- 우선순위: P0
- 요구사항: R04, R07, R08, R13
- 담당/수정 범위: root docs/development/input-coverage-audit.md·이 작업 기록; diagnostic_inputs는 /tmp의 표 교정 후보만 준비한다.
- 선행 작업: 현재 진단 세 입력 및 고급 숫자/실행 계획의 실제 Windows 인수

## 사용자 결과

입력별로 실제 눌러보고 읽은 증거와 아직 빠진 시험을 구분한다. 이미 검증된 입력을 미완료로 두거나 전수 입력을 검증했다고 오해하게 하지 않는다.

## 계약과 범위

현재 source와 raw 결과·root-review를 기준으로 문서 표를 정정한다. source/GUI 증거는 각각 snapshot 범위를 유지한다. 실제 포인터·IME와 private provider 입력을 구분하고, 새 검증이 필요하면 좁은 후속을 남긴다. 제품 코드·native 실행·Cargo·공유 schema 변경 없음.

## 인수 조건

S10 세 singleline, advanced 숫자3곳, 도움말/readonly plan의 현재 actual 증거가 표에서 정확히 연결되고 custom tool path·multiline caret 등 실제 미검증 행이 유지된다. root가 원 결과와 후보를 대조하고 check_docs를 통과한다.

## 검증과 handoff

초기 범위 기록. 과거 공개 URL walkthrough를 현재 binary의 전수 시험으로 소급하지 않는다.

최신 S10 및 option-plan root-review의 PASS_SCOPED와 원 결과 범위를 대조해 표를 통합했다. 현재 CPU에서 S10 전수 입력·물리 포인터·IME 검증을 완료했다고 주장하지 않는다. 문서 검사 결과는 통합 검사로 기록한다.

실제 통합 검증: `python3 scripts/check_docs.py` exit0, 884 documents 및 114 hash-matched notices PASS. 입력 구현을 복제하는 추가 테스트는 만들지 않았다.

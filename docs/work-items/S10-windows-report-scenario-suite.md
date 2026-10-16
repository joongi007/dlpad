# S10: Windows 독립 진단 시나리오 전체 실행

- 상태: in-progress
- 분류: test
- 우선순위: P0
- 요구사항: R13, R08, R11
- 담당/수정 범위: root; scripts/windows_s10_report_suite_native_probe.py 및 본 문서
- 선행 작업: 일반 Windows release 확정, 단계별 저장 수집기

## 사용자 결과

보고서의 저장·기존 파일 보존·편집·로그 생략·MPD 첨부/제외/재첨부/안전하지 않은 입력 거절을 독립 fixture로 실제 시험한다.

## 계약과 범위

11개 시나리오마다 fresh private GUI와 임시 합성 파일을 사용한다. 기존 worker90초/outer150초 상한을 유지한다. 프로젝트 source·수집기·바이너리는 전체 실행 동안 SHA로 고정한다. 실패 시 원 결과를 보존하고 종료 정리가 확인된 독립 시나리오를 이어서 실행한다. Clipboard·CloseWait·IME 인수는 별도 근거와 구분한다.

## 인수 조건

각 native assertion 및 root DB/export/PNG 독립 검토가 통과해야 기능 인수를 완료한다. suite 종료 코드만으로 전체 P0를 완료 처리하지 않는다.

## 검증과 handoff

root가 실제 저장 시나리오의 누적90초 실패를 보존하고 formats/collision을 분리한 수집기를 통합했다. 새로운 suite는 반복 실행을 재현 가능하게 만들며 실제 실행은 최신 일반 제품 전체 검증 이후 진행한다.

`windows-s10-report-suite-20261004T135454Z`를 실행했다. 첫 save scenario native assertion과 정상 종료·private desktop 닫힘은 통과했다. root는 JSON/Markdown의 동일 report ID, Metadata exit7, export SHA/정제, DB Failed·artifact0·snapshot 쿠키 미저장, 원 synthetic source/sentinel 보존 및 final PNG를 직접 대조하고 해당 case의 `root-review.json`에 기록했다. 나머지 case는 실행 중이며 전체 완료로 표시하지 않는다. suite는 GUI가 실행된 경우 원 Job 비움·원 프로세스 종료·사용자 input desktop 불변을 확인하지 못하면 다음 case를 실행하지 않는다.

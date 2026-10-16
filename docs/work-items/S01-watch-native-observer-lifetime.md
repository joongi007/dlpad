# S01: Windows 감시 GUI 관측 도구의 종료 수명

- 상태: in-progress
- 분류: testing
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: root — scripts/windows_library_watch_probe.cs, scripts/windows_library_watch_native_runner.py 및 본 문서; metrics_recovery — /tmp 초안만
- 선행 작업: 현재 source 전체 검증과 실제 periodic 시험 종료 후 native 실행

## 사용자 결과

사용자 화면을 방해하지 않는 Windows 파일 변경 시험을 안전하게 종료하고 실제 add/delete/rename 결과를 검증한다.

## 계약과 범위

기존 private desktop 관측 도구의 90초 worker timeout 뒤 HANDLE close 경계를 보강한다. cooperative stop 설정, 원 Job 종료, 5초 join 뒤 worker가 종료된 경우만 HANDLE을 닫고 종료되지 않으면 FAIL과 retain을 기록한다. 앱 기능·Cargo·schema는 변경하지 않는다. wrapper는 현재 default 전체 검증 proof와 복사된 CPU/C#·컴파일 핀, Job0/private/input을 대조하며 실제 수집 성공도 root pixel/DB 검토 전 REVIEW_REQUIRED로 남긴다.

## 인수 조건

Windows 컴파일과 실제 private GUI 파일 추가/삭제/이름 변경 관측, 자연 app exit0·Job0, 사용자 input desktop 불변, 원 1만 fixture와 rename identity 결과를 확인한다. 강제 종료·살아 있는 worker의 HANDLE close·root recovery 미실행을 성공으로 집계하지 않는다.

## 검증과 handoff

하위 읽기 검토가 원 timeout HANDLE close 결함을 확인하고 /tmp 초안을 작성했다. root 검토와 실제 실행은 진행 중이며 기존 도구는 보강 전 새 실행하지 않는다.

## 통합 검토

root가 stop→Job 종료→join5초→safeClose 조건을 검토하고 통합했다. wrapper는 강제 종료·retained·unsafe close를 거절한다. createNew 외부 파일 보존·symlink/traversal·copy/JSON 상한·잘못된 JSON/UTF8 순수 실패 회귀 5개를 실행해 통과했으며 로컬 CI 설정에 추가했다. 실제 Windows 컴파일/GUI는 제품 Stop 시험 후 실행한다. UIA 호출 자체의 취소와 root recovery 인수는 별개로 남는다.

실제 Windows 컴파일·watch-only GUI 수집과 root 픽셀/DB/full payload 검토가 통과했다. [root 결과](../validation/evidence/windows-scan-watch500-watch-gui-20261006-continue/root-review.json). 원 앱 exit0/Job0·private/input·safe HANDLE close 확인, 추가/삭제/이름 변경과 rename ID→DB 일치 확인을 포함한다. timeout 실패 경로는 코드 순서와 순수 wrapper 실패 테스트로 검토했으며 실제 UIA hang 강제 재현은 하지 않았다. root recovery 및 paced scan preemption 인수는 별도로 남긴다.

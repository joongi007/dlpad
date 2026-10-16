# S01: Windows 시작 시 없는 목록 root의 자동 회복

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: root — scripts/windows_library_missing_root_probe.cs, scripts/windows_library_missing_root_native_runner.py 및 본 문서; metrics_recovery — /tmp 초안만
- 선행 작업: 기본 제품 전체 검증·watch-only 완료·GUI CPU 종료

## 사용자 결과

초기 목록 폴더를 찾지 못한 상태에서 해당 폴더가 생기면 수동 새로고침 없이 오류가 해소되고 목록에 파일이 표시되는지 확인한다.

## 계약과 범위

원 기존 DLPAD_LIBRARY_ROOT 시작 경로와 fresh private data를 사용한다. owned TEMP private root 아래 media만 초기 부재로 두고 warning/실제 부재 확인 후 새 폴더와 합성 3B recovered.mp4를 createNew한다. 기존 폴더 삭제·rename·ACL 변경·새 product debug hook·Cargo/schema 변경은 없다. 원 private HWND/PID/creation/Job·current CPU/source·compiled pins 및 cooperative timeout 수명을 검증한다.

## 인수 조건

초기 warning/실제 부재→createNew→10초 이내 실제 row/감시 warning 해소, manual Refresh/reentry 없음, 자연 app exit0/Job0/private/input/safe HANDLE close, exact final DB 1row·payload·sentinel·PNG를 root가 대조한다. 이는 실제 startup missing→create recovery이며 paced scan 중 Recovery preemption이나 hard 10초 제품 보장으로 확대하지 않는다.

## 검증과 handoff

하위가 CPU startup 경로를 추적해 worker WatchSession이 초기 실패 후 살아 있음을 확인했다. 일반 Apply는 먼저 폴더를 생성하므로 이 사례로 쓰지 않는다. 초안을 root가 통합하며 CPU 측정 종료 후 실제 Windows 실행한다.

## 실제 결과

첫 native 컴파일은 클래스 간 PrivateDirectory 참조 CS0103으로 실패했다. root가 Program.PrivateDirectory 명시 호출·public helper로 수정하고 새 evidence에서 재실행했다. 실제 missing warning과 부재를 확인한 뒤 create-only 폴더/파일이 2.010초에 자동 표시되고 경고가 해소됐다. 원 앱 자연 exit0/Job0·private/input·safe close, 최종 DB 1row/integrity와 payload/sentinel·두 PNG를 root가 대조했다. [root 검토](../validation/evidence/windows-scan-watch500-missing-root-fixed-gui-20261006-continue/root-review.json). 실제 native FileID 전체 재열기나 paced-scan Recovery 중단은 미검증이며 이를 완료로 확대하지 않는다.

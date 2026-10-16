# Windows UI 검수 후보 — 2026-10-17

현재 후보는 **v37**입니다. 설치기 초기 검사 중단 뒤 재검사 복구와 작업별 중단 문구를 수정했습니다. 앱 실행 파일과 UI는 v36과 같습니다.

## 설치기 수정 확인

1. [설치 ZIP](../validation/evidence/dlpad-install-ui-feedback-v37-20261017.zip)을 새 폴더에 모두 풀고 `payload`와 함께 `dlpad-install.exe`를 실행합니다.
2. 기존 앱은 삭제할 필요 없습니다. 설치기에 전에 지정한 설치 폴더(`registered` 폴더를 포함하는 위치)를 선택합니다.
3. 초기 검사 중 **검사 중단**을 누르고 중단 완료 안내가 나올 때까지 기다립니다. **다시 검증**을 누르면 현재 설치와 기록을 다시 찾아 앱 실행이 활성화돼야 합니다. 아주 빨리 검사가 끝나면 중단 버튼이 비활성화될 수 있습니다.
4. 새 설치 위치에서 같은 순서로 재검사하면 **설치**가 활성화돼야 합니다. 설치 도중 중단은 이미 만들어진 파일·기록을 보존합니다.

기존 설치의 앱 실행은 그 폴더의 앱을 여는 동작입니다. 새 앱 설치에는 새 위치를 선택하며 기존 앱을 자동 교체하지 않습니다. 설치 후 실행 파일은 선택한 폴더의 `registered/dlpad.exe`입니다. SemVer0.1.0과 검수 번호v37은 별개이며 서명되지 않은 개발 검수본입니다.

## 앱 UI 확인

1. 기존 앱을 트레이에서도 완전히 종료합니다.
2. [portable ZIP](../validation/evidence/dlpad-user-preview-ui-feedback-v37-20261017.zip)을 새 폴더에 전부 풀고 `dlpad-cpu.exe`를 실행합니다.
3. 파일 목록에서 영상을 더블클릭/Enter 또는 **선택한 영상 확인**으로 로컬 상세를 엽니다. 로컬 준비에는 FFprobe가 필요합니다.
4. **탐색기에서 파일 선택**으로 저장 폴더와 해당 파일 선택을 확인합니다. 이 동작은 준비된 로컬 파일을 대상으로 합니다.

## 확인 범위

Windows whole37 fmt·clippy·test·release와1685통과/0실패/43ignored·43harness, source675·3PE·lock을 Root가 대조했습니다. 새 ignored nativeState 회귀는 별도로 명시 실행해 통과했습니다. 실제 private GUI의 원36 실패→새37 재검사 복귀·현재 기록 suffix-1·앱 실행 활성과 원자료 보존을 확인했습니다. 이 GUI는 원35 payload와 원36/새37 설치기를 고정한 혼합 합성 fixture이며 새37 bundle 전체 runtime 시험은 아닙니다. [수정 검토](../validation/ui-review-v37-20261017.md).

v35 UI·설치 앱 실행, v36 native 경로 파싱과 이전 성능·긴 경로·로컬CI·8h는 각 snapshot 범위로 유지합니다. 실제 사용자 Explorer 선택과 Windows·외부 환경 인수는 [현재 P0](../validation/P0-current-20261016.md)에 남습니다.

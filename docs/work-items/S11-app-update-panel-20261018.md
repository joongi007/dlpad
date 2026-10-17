# S11: 앱 업데이트 조회와 native 설정 진입점

- 상태: in-progress
- 분류: feature
- 우선순위: P1
- 요구사항: R08, R11
- 담당/수정 범위: updater_audit isolated worktree의 accessible_app_update.rs·catalog.rs·engine/app_update/discovery.rs, Root의 accessible.rs·CPU/대체 entrypoint·engine module export·실제 Windows 검증
- 선행 작업: 소개 페이지와 자동 preview 릴리스 준비, 서명 후보/owned 저장 계약

## 사용자 결과

설정 → 앱 업데이트에서 현재 Cargo 버전, 안정/미리보기 채널과 조회 진행·중단·실패·이전 결과를 확인한다. 조회는 다운로드 작업으로 표시하거나 시작을 차단하지 않는다. 운영키·자동 교체가 아직 없음을 명확히 안내하고 canonical 공식 Release를 사용자가 열 수 있다.

## 계약과 범위

새 panel 생성/시작 시 외부 조회·idle thread0. 요청당 worker1/result channel1, 최대20초·256KiB·20release, 공식 GitHub API redirect0·자격증명0. native UI는 기존 GTMUX theme/components/중앙 catalog를 유지한다. Root가 모든 탭/CloseWait global poll과 원thread join/ACK에 연결하고 일반 조회를 mutation busy에서 제외한다. 자동 설치·임의URL·운영키 등록·공개 배포는 수행하지 않는다.

## 인수 조건

실제 navigation으로 진입해 draft/도구 owner 보존, 중복 조회 차단·오류·취소·정상 stop ACK, 조회중 다른 작업 조작, Windows 화면/한글/hover/배율/종료를 확인한다. 유효 signature가 있다고 현재 설치 fresh transaction을 자동 upgrade 권한으로 해석하지 않는다.

## 검증과 handoff

Root가 소스 diff와 worker/close 연결을 검토하고 통합했다. 하위 독립 harness22PASS 중 신규는discovery4+UI6이며 기존12를 신규 회귀로 중복 집계하지 않는다. Root Windows workspace strict lint·전체 test·release build와 private GUI 시험 진행 중. 실제 네트워크/새 화면/전체 workspace 결과는 아직 완료 표시하지 않는다.


Root Windows workspace strict lint/전체test/CPUrelease build는 실제 통과했다(주1728+filteredchild1PASS, 0FAIL, 43ignored). 원로그는 `.local-tools/update-integration-20261018/native-full-v1/native-tests.log`다. 새 화면 actual private GUI는 별도 진행하며 이 통합 테스트로 시각/키보드/OS 배율 인수를 대체하지 않는다.


## Root 실제 private Windows 화면 — 2026-10-18

실제 CPU SHA73c661d45be9b3b373ccbec461c8a74e261e51e818bf7c5b2ae4aca448012e41를 실행했다. 기존 windows_ui_rework_private_probe.cs의 HWND/원process시각/Job/PrivateDesktop 소유 절차를 유지한 로컬 probe로 설정→앱업데이트, 960×640/800×600, Light→Dark/restart, 100%/200%, 실제 공개GitHub 조회를 실행했다. 두run 모두 '아직 공개된 버전이 없습니다.'를 실제 관측했고 WM_CLOSE exit0/Job자연empty/강제정리0/inputdesktop불변이다. `.local-tools/update-integration-20261018/gui-v6/`의 원 JSON/PNG를 Root가 직접 읽고 Light/Dark와200% screenshot을 열어 검수했다. observer의 REVIEW_REQUIRED는 사람 이미지 검수를 요청하는 원상태로 보존했고 원파일을 PASS로 바꾸지 않았다.

초기 probe 컴파일과 접근성 Invoke 미지원, content 전용 클릭을 nav에 사용한 시험도구 오류는 v1~v4 실패로 보존했다. 제품코드를 그 실패 때문에 바꾸지 않았다. 기존 진단 시험처럼 private HWND 전용 Space 활성화·UIA focus/readback으로 v5/v6를 실행했다. 사용자 화면 전환/전역SendInput/클립보드0이다. 200% 작은 창에서는 버튼이 초기viewport 아래에 있었고 focus 후 실제 ScrollArea가 움직여 버튼이 footer 위에 표시됨을 v6 PNG로 확인했다. 단순UIA client bounds만으로 시각 통과를 선언하지 않았다. 이는 합성private키 이벤트 검증이며 실제물리키보드/OSDPI변경/실제앱자동교체 성공이 아니다.

원명령 `python3 .local-tools/update-integration-20261018/run-native-gui.py`는 기존 pinned Windows Python→windows-cli-host→C# private observer를 호출한다. 사용자 Documents/실영상/계정데이터는 사용하지 않았다. 개발CPU/미디어 작업산출물은 그대로 두고 현재 작업에 사용한 재빌드 가능한 Cargo debug cache만 정리했다(6286files/2,769,614,500B). 삭제목록/이유는 /tmp/dlpad-debug-cache-cleanup-20261018.json에 남겼고 release 실행파일·검수 PNG는 보존했다.


Root는 실제49c CI ZIP에서 임시설치한 원 dlpad.exe(SHA4a60a714...)도 별도로실행했다. gui-hosted49-v1의Light/Dark·960/800·200%focus실제스크롤·공개NoRelease조회·restart·두번자연exit0/Job0/강제0/inputdesktop불변을 확인하고 원PNG를직접열었다. raw REVIEW_REQUIRED를변조하지않고 root-review.json으로검수를추가했다. 현재Root의bundle후속소스빌드나물리IME/OSDPI/실제자동교체를검증한것은아니다.

# S00: 시스템 플레이어 열기와 앱 감독 Job 경계

- 상태: in-progress
- 분류: investigation
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: advanced_units — 새 독립 조사/후보; root — library_open.rs, app_install/active_launch.rs 공유 감독 계약/통합
- 선행 작업: S00-library-file-actions-20261013

## 사용자 결과

파일을 기본 앱으로 열어도 dlpad 종료가 사용자의 플레이어를 종료시키거나 무한히 기다리는 결과를 만들지 않아야 한다.

## 계약과 범위

현재 library_open은 ShellExecuteEx 전달이며 private 자동화에서 실제 Shell 제공자는 거절한다. 설치기 감독 Job은 KILL_ON_JOB_CLOSE다. Shell이 별도 broker로 실행하는지 직접 자식으로 실행하는지는 아직 실제 인수하지 않았고, Job 상속 가능성을 소스/소유 fixture로 조사한다. 기존 감독·다운로드 소유 프로세스 보호를 약화시키지 않는다. Default desktop/실제 사용자 연결 프로그램/외부 메시지/기존 registry 설정은 조작하지 않는다. source 검토만으로 실제 association 동작을 확정하지 않는다.

## 인수 조건

외부 사용자 앱과 앱 소유 작업의 수명 분리. 불확실한 Job 경계에서는 실제 외부 앱 요청을 안전하게 거절하거나 근거 있는 broker를 쓴다. owned synthetic subprocess 시험, 취소/timeout/CloseWait 및 기존 설치 launch/도구 프로세스 회귀를 Root 통합 검증한다.

## 검증과 handoff

초기 source 확인: app_install/active_launch.rs job은 KILL_ON_JOB_CLOSE, register_self_guardian은 executable path 등록만 한다. Shell의 실제 Job 상속 여부는 미검증이다. 소스/후보 및 원자료를 독립 namespace에 보존하고 실제 PASS와 구분한다.

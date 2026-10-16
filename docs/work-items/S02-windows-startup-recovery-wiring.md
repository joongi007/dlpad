# S02: Windows 제품 startup receipt 회수 연결

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — cookies.rs, cookies/windows/recovery.rs, 이 문서; diagnostic_cache — windows.rs module 선언
- 선행 작업: 원 import marker/journal/cleanup, actual guardian receipt

## 사용자 결과

Windows startup는 최대128개 항목을 조사하고 원 marker/native IDs/ACL/journal 잠금과 실제 receipt를 확인한 사본만 회수한다. Empty/Intent/누락 receipt는 보존하며 이전 boot 추측 권한은 도입하지 않는다.

## 계약과 범위

원 namespace pins와 원 marker IDs를 검증한 후 Journal::reopen한다. selfclaimed marker만으로 삭제하지 않는다. 삭제 직전 OwnedCopy cleanup으로 원 모든 DELETE handle·allowlist·receipt를 다시 검증한다. foreign sibling/큰디렉터리/partial은 보존한다.

## 인수 조건

실제 public import/defer/recover에서 receipt없는 session deferred, foreign 보존, malformed rejected,128상한을 native 시험한다. 실제 gated guardian 완료 회수와 appkill은 별도 통합 인수로 미완료를 유지한다.

## 검증과 handoff

구현 전 상태. OS별검증은 native 실행 결과가 있는 범위만 보고한다.

## Root native 통합 결과

first native Clippy exit101: NativeFileIdentity 비Copy 값의 이동 3곳; 실패 자료 보존. root가 원ID clone으로 수정 후 second engine all-target Clippy 및 cookies::windows:: native module tests exit0. 모든 operation source_unchanged/input_desktop_unchanged true. actual public recovery 미완료 receipt 보존·malformedmarker 거절·foreign130scan 상한 회귀를 실행했다. 실제 guardian completed receipt를 생성해 public recovery 성공하는 통합 인수와 제품 gated reader wiring/appkill/전체workspace 검증은 아직 남는다.

# S00: 설치 체크박스 실제 색상과 native paint 경로 진단

- 상태: in-progress
- 분류: UX
- 우선순위: P0
- 요구사항: R01, R07, R13
- 담당/수정 범위: installer_batch — /tmp 독립 Win32 실험 및 docs/validation/evidence/installer-checkbox-standalone-native-20261013만; root — 제품 채택·통합·최종 검토
- 선행 작업: 최신 v7 설치·8popup·54scene actual, whole-v3 1272/0/30

## 사용자 결과

설치의 선택된 체크박스도 앱과 같은 상태 표현을 제공한다. 코드에 색상이 있다는 이유만으로 실제 화면이 맞다고 주장하지 않는다.

## 계약과 범위

현재 checked PNG에서 PRIMARY 계열이0pixel이고 RGB178/194/206 marker158pixel을 관측했으며 같은 화면 Install 버튼은PRIMARY이다. enabledtrue/pressedtrue JSON은 체크 캡처 이전 값이므로 paint 당시 state를 증명하지 않는다. 이 불일치를 Source/GDI clip/notification/후속 defaultpaint/contrast/borrow실패로 구별한다. 처음에는 source를 수정하지 않고 같은 Win32 styles·fonts·v6manifest·painter의 최소 독립 실험으로 이벤트와 clipping을 계측한다. fixedtrace256/JSON64KiB/ownedprivateJob/원binary·source pins를 보존한다. standalone 결과는 제품 원인 확정이 아니다.

## 인수 조건

actual NM stage/flags/native check/state/enabled/contrast/clipbox/DC origin/return과 checked PNG를 연결한다. 필요 시 bounded client clip 또는 post-native indicator overlay를 독립 케이스로 비교한다. 정확한 private desktop·프로세스 소유권·원 log·Source/binary 보존을 root가 검토한다. physical input과 global UI를 사용하지 않는다. 제품 수정 채택 후에는 새 whole와 실제 설치 화면으로 재검증한다.

## 검증과 handoff

source b0ccd4c0…/installer b6c78297…에서 readonly ROI mismatch를 확인했다. standalone compile/native는 아직 시작 전이며 원인은 미확정이다. GDI stock/resource 소유권과 native BS_AUTOCHECKBOX/provider/keyboard semantics를 유지한다.

## 독립 Win32 실험 결과

소유 private desktop/Job의 C# minimal repro에서 v3/v4/v5 actual compile/native 자연exit0·Job0, source/binary/helper pins 보존을 확인했다. root는 최종64trace와 PNG를 직접 대조했다. 실제 PREPAINT·id206·정확parent·checked1·enabledtrue·HCfalse·fullclip·return4와 파란 표시를 관찰했다. 제품의 회색 표시를 재현하지 못했으므로 제품 원인이나 수정 성공을 주장하지 않는다. 원 default sandbox interop socket 실패와 C# 초기화 컴파일 실패도 보존했다. clip reset/무근거 overlay는 채택하지 않았다. 후속은 실제 제품의 동기재진입/presentation gate를 조사한다. [root 독립 검토](../validation/evidence/installer-checkbox-standalone-native-20261013/attempt-v5/root-review.json).

## 2026-10-13 실제 v7 안정 화면 검증

root가 같은 제품 v7의 checkbox206/207/208을 각각 BM_SETCHECK=1로 바꾼 뒤 immediate와 200ms settled 화면을 비교했다. 동일 HWND/check1/enabled 상태에서 immediate gray와 settled blue를 실제 PNG로 확인하고 원 check0을 복구한 뒤 5파일 설치·readonly 재검증·원 source/5 payload 보존·자연종료/Job0를 검토했다. 세 settled 상태의 파란 체크 표시는 PASS다. native animation/정확한 draw-time 원인은 미확정이며 물리 입력 인수는 아니다. 제품 source에 추측성 paint 변경을 추가하지 않는다. [root 검토](../validation/evidence/windows-msvc-installer-checkbox-settle-v7-20261013/root-review.json).

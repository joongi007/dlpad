# S00: 현재 앱의 실제 렌더 상태와 Windows pointer 입력 분리 검증

- 상태: in-progress
- 분류: UX
- 우선순위: P0
- 요구사항: R01, R07, R13
- 담당/수정 범위: root — feature 빌드·격리 GUI·원 로그/PNG/ACK 검토; advanced_units — /tmp 전용 collector/observer/receipt 후보
- 선행 작업: S00-ui-cross-surface-audit-20261013, whole-v8 통합 PASS

## 사용자 결과

공통 버튼과 입력의 hover/pressed/focus/disabled 표현을 실제 Windows 앱 렌더링에서 확인한다. 자동 메시지 전달 실패를 렌더링 성공이나 물리 입력 검증으로 바꾸지 않는다.

## 계약과 범위

기존 first-paint-fixture 및 owned synthetic egui input만 사용한다. 프로젝트 UI/worker/schema/Cargo.lock 수정 없음. private desktop/Job/원 PID 및 소스·기본 binary pin을 유지한다. advanced_units는 /tmp/dlpad-synthetic-pointer-current-20261013만 수정하며 다른 담당 파일을 수정하지 않는다. 원 default WM pointer 시험 FAIL은 보존한다.

## 인수 조건

현재 whole/feature 4검사의 source·artifact·log 일치, bounded sequence/presented ACK, 실제 UIA 역할·값·ID 유지, 9개 state PNG 및 기존 픽셀 threshold, disabled Start의 비활성·비반응, enabled Windows network input의 실제 focus와 값 보존을 확인한다. root가 캡처를 직접 확인하고 자연 close/Job0/Default desktop 보존을 검토한다. synthetic-egui-fixture 성공은 OS pointer/physical IME/reader 인수와 구별한다.

## 검증과 handoff

root 실행: /tmp/dlpad-synthetic-pointer-native/build_feature.py --default-proof ...whole-v8.../result.json --configuration ...whole-v8.../effective-configuration.json --output ...windows-msvc-ui-cross-surface-pointer-feature-v8-20261013 --reuse-owned-target --target-tag 20261012-corrected. 결과는 실행 중이며 완료 주장하지 않는다. 과거 observer는 Windows network input을 disabled로 가정했으나 현재 기능은 enabled이므로 현재 provider/focus 동작에 맞춰 /tmp 검증 후보를 고친다. 실제 baseline UI 규칙을 삭제하거나 threshold를 낮추지 않는다.

실제 최신진단feature4검사:1273/0/30 및root로그/source/originalpin대조PASS. 9actualPNG·4presentedACK·disabledStart·URL/network실제providerfocus/ID/value/naturalcloseJob0·Default보존을root재검토했다. hover1356pixel/pressed1370pixel(기준>20)·실제settings선택ACK PASS. root가idle/hover/pressed/selected/networkafterPNG를직접보았다. root-review.json PASS_SCOPED_SYNTHETIC_RENDER_AND_PROVIDER_ACK. OSpointer전달FAIL은그대로이며inactive창focus의시각표현·물리caret/IME/reader·전체버튼state는미인수다. fixture가DB를생성하지않아SQLiteintegrity시험은실행하지않았다.

## 2026-10-13 새 source655 실제 합성 상태

새 whole1278/feature1312 proof와 exact immutable featureCPU로9state를 실제 캡처했다. root는 원로그/source/pins,4presented ACK/호버·누름 pixel delta>20, 네트워크 input 원RuntimeID/value/focus 및 자연종료0/Job0/private/MTA/Default input불변을 독립 검토했다. idle/hover/pressed/selected 대표4PNG를 직접 확인했다. 이 성공은 합성 egui 렌더링/공급자 ACK이며 OS마우스 bridge·inactive focus paint·물리IME/reader/전체UI인수는 미검증이다. 원OSpressedFAIL을유지한다. [root검토](../validation/evidence/pointer-state-synthetic-egui-feedback-library-20261013/root-review.json).

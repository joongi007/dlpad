# S00: 높이 입력 오류 문맥 공개

- 상태: completed
- 분류: UX
- 우선순위: P0
- 요구사항: R07, R08
- 담당/수정 범위: diagnostic_cache — /tmp 후보; root — crates/app/src/accessible_options.rs, crates/app/src/accessible.rs 통합, 이 문서
- 선행 작업: Windows provider current native 검사 종료 및 source 고정 해제

## 사용자 결과

높이 입력이 잘못되면 입력과 고치는 방법을 함께 보고 수정한다. 오류 문구가 footer 아래에 가려진 상태를 방치하지 않는다.

## 계약과 범위

DESIGN.md의 label·오류·포커스 문맥을 따른다. 실제 label/input/error response 기하를 사용하고 focus 또는 focused edit 변화에서 필요한 만큼 한 번 공개한다. 사용자 후속 휠을 매 frame 되돌리지 않는다. 원 ID/caret/Invalid/description/DANGER marker/core parser·기존 field 호출 계약을 유지한다. 프로젝트 파일은 root만 수정한다.

## 인수 조건

실제 Form.show와 좁은 ScrollArea에서 오류 전체 glyph와 입력이 클립 안에 들어온다. 정상 값 복구와 disabled·원 focus/caret 회귀를 유지하고 후속 휠이 재공개되지 않는다. Light/Dark native 확대 캡처에서 실제 설명을 확인한다.

## 검증과 handoff

root가 provider-fixed Light/Dark800×640/앱200% first height-invalid PNG를 직접 봤다. 입력과 label·빨간 outline은 보이지만 실제 오류 설명은 viewport 아래였다. provider full-control-visible와 fullDescription 통과는 오류 문구의 실제 glyph 가시성 증거가 아니다. 후보 준비 중이며 구현 완료를 주장하지 않는다.

- root는 실제 Form/ScrollArea 회귀만 먼저 추가했다. runtime 공개는 바꾸지 않는 test-only None getter로 후보의 parent hook 계약을 선언했다. native 탐색48/footer100 logical을 반영한400×300/native-ppp2 fixture에서 원 error glyph clip을 검사한다. 이것은 physical OS DPI 인수가 아니다.
- 첫98286은 후보가 PanelState::load에 Memory를 넘겨 Context 타입과 달라 compile 실패했다. 제품 clip 실패 재현으로 계산하지 않는다. 현재 egui API에 맞게 고치고 RED-v2를 실행했다. 원 실패를 보존하며 core/glyph assertion은 그대로다.
- 실제 RED24431 exit101: error glyph y198..211, actual clip bottom195/footer top200으로 설명이 가려졌다. root native 캡처와 같은 문제를 actual Form에서 재현했다. source 불변·Job 자연 종료 확인.
- 실제 label/input/error rect를 묶고 focus 또는 focused edit 변화에서만 current-frame 공개를 요청하도록 통합했다. 모든 nested scroll 이후 outer download-content에서 비애니메이션·최소 이동으로 적용한다. 다음 idle/user wheel frame에는 재공개하지 않으며 body보다 큰 union은 control+error를 우선한다. core/parser/metadata/ID·기존 field wrapper 계약은 유지한다. GREEN 및 최신 전체/native 검증은 아직 남아 있다.
- GREEN39210은 실제 오류 glyph fullclip까지 통과했으나 뒤의 휠 이동 assertion에서 실패했다. 옵션10회귀는 통과했고 전체성공으로 쓰지 않는다. max-scroll 또는 요청 재발행을 구별하기 위해 before/after 실제offset·현재pending을 계측한 새 실행을 시작했다. 인수 기준을 낮추지 않는다.
- trace37135은before=217.5/after=217.5/pending=None을 관측했다. 실제 content_size와 inner_rect로 max offset을 측정해 현재 위치가 하단 경계임을 assertion하고, 경계에서 위로 이동하는 실제 휠을 시험한다. glyph fullclip과20logical 이상 이동·후속idle 미복귀·focus/값 보존 조건은 유지하며 새 GREEN-v2를 실행했다. 아직 전체성공으로 기록하지 않는다.
- GREEN98351 exit0: 두 UI harness 각각 옵션11회귀 통과. actual max offset과 before가 같은 하단 경계를 확인했고 실제 위쪽 휠20logical 이상 이동·추가idle 미복귀·focus/값 보존과 error glyph fullclip을 검사했다. 기존 invalid→valid/marker/caret/disabled 회귀도 통과. source 불변·Job 자연 종료 확인. 최신 Candidate/native 시각 인수는 다음이다.

- 최신 Windows 전체 검사19505와 독립 대조11125 exit0: 43 harness, 1,163 성공/0 실패/30 ignored. fmt·clippy·test·release 통과. 증거: docs/validation/evidence/windows-msvc-height-error-context-whole-20261012/root-review.json.
- 실제 private GUI Light48583/Dark75790 및 독립 대조23104/82971 exit0. first/restart에서 입력·label·오류 설명의 본문 bounds, 0→1080 유효성 복구, 원 프로세스와 자연 종료를 검사했다. 독립 JSON을 각 evidence에 보존했다. root가 직접 본 first Light/Dark 캡처에서는 오류 전문이 footer 위에 보인다. restart 픽셀 직접 검토는 아직 남아 있으며 전체 UI/물리 IME 인수로 확대하지 않는다.

- root가 restart Light/Dark PNG도 직접 확인했다. first/restart 네 캡처 모두 입력·label·오류 전문이 footer 위에 보이며 테마별 root-scoped-pixel-review.json에 원 PNG SHA를 기록했다. 이 작업의 좁은 오류 문맥 인수는 완료했다. 전체 UI/UX·물리 입력·모든 화면 배율은 완료로 주장하지 않는다.

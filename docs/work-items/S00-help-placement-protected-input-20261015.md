# S00: 입력을 가리지 않는 도움말 배치

- 상태: prepared, scratch 후보 작성 중
- 분류: UX-fix
- 우선순위: P0
- 요구사항: R07/R08/R13, S00 UI 실제 시각 인수
- 담당/수정 범위: field_reveal은 B/help-placement-v1만 담당한다. Root는 ui_components.rs/accessible.rs 공유 인터페이스·통합·최종 검증을 담당한다.
- 선행 작업: v28 고정 설명이 상태와 버튼을 덮는 Root 직접 픽셀 관측, 기존 Help Escape/focus 계약

## 사용자 결과

도움말을 읽으면서 관련 입력칸과 찾기 동작을 확인할 수 있도록 배치를 개선한다. 설명이 완전히 들어갈 공간이 없는 작은 화면의 한계를 숨기지 않는다.

## 계약과 범위

기존 help 호출·논리 ID·벡터 정보 아이콘·32px 클릭 영역을 유지한다. 도움말 버튼 배치와 표시를 두 단계로 분리하고 executable_field가 실제 input/browse rect를 전달하는 내부 인터페이스를 검토한다. 공개 egui Popup API를 사용한다. worker/DB/schema/의존성/lock 변경은 없다. 새로운 물음표 버튼이나 별도 상시 설명 줄을 늘리지 않는다. 필수 오류/보존 안내는 본문에 남긴다.

## 인수 조건

fit 가능한 viewport에서 실제 Context popup rect와 보호 입력/browse rect가 겹치지 않아야 한다. 기존 pinned/Escape/hover/focus/disabled 및 owner ID를 유지한다. 불가능한 작은 공간에서는 읽기/닫기를 유지하고 fallback을 명시한다. Root가 새 source/실행 파일의 private GUI 캡처를 직접 검토한다. 단위 기하 검사를 물리 OS/전체 UI 인수로 대체하지 않는다.

## 검증과 handoff

현재 scratch 준비 단계다. 제품 통합/검사/시각 성공을 주장하지 않는다. Root가 선택 시험·strict clippy·전체 빌드 및 private GUI를 조정한다. Source와 Cargo 작업 중 동시 수정을 금지한다.

Root 실제 RED는 compile 성공 뒤 새 Context 0 PASS/2 FAIL로 입력 겹침과 긴 popup overflow를 재현했다. GREEN-v1은 도움말11·경로문맥2·찾기배치1·이력12가 실제 통과했고 source 및 S 후보 불변을 확인했다. 다만 Harness가 inner 입력폭에서 좌우24px 여백을 제품과 다르게 계산하여 보호controls가16px 넓었다. 제품 계산은 기존대로 올바르며 이 시험 전제 차이를 보존한 새 v2에서 제품과 같은 폭/실제controls 완전 포함 조건을 추가 검증한다. 원 v1 성공을 전체 UI 시각 인수로 확대하지 않는다.


v2는 제품과 같은 입력폭 및 매 frame controls의 원content/viewport 완전 포함을 검사하여 실제 Context2 PASS/0 FAIL, 최종 strict workspace/all-targets clippy0을 확인했다. production은 v1과 byte동일하다. Root가 History와 함께 관련2파일을 Live 채택했고 새 whole29/실제 private Help popup 검수가 진행된다.

Root 새 whole29 완료: fmt/strict workspace all-targets clippy/test/release 실제0, 1,613 PASS/0 FAIL/42 ignored·43harness·source674이며 원로그/current3PE/lock을 독립 대조했다. CPU10,321,920B(+6,144B), CLI978,432B, installer4,432,896B다. native 화면/최신 ZIP은 후속 진행이며 전체P0로 확대하지 않는다.

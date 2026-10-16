# S00: 진단 입력 포커스 일관성

- 상태: in-progress
- 분류: UI
- 우선순위: P0
- 요구사항: R08, R13
- 담당/수정 범위: diagnostic_inputs — /tmp 후보; root — crates/app/src/accessible_diagnostics.rs 통합, 이 문서
- 선행 작업: source-first UI 전체·native 검사 종료 후 source 변경

## 사용자 결과

진단 multiline 입력의 포커스를 공통 singleline과 같은1.5px accent로 구별한다.

## 계약과 범위

DESIGN.md의 입력 focus 계약과 현재 single viewport frame을 유지한다. 전역 selection.stroke·caret·문서 스크롤·보고서 동작을 변경하지 않는다. 에이전트는 프로젝트 파일을 수정하지 않는다.

## 인수 조건

실제 Panel.show에서 Light/Dark focus·blur·disabled 상태의 정확한 viewport stroke를 검사한다. 내용 선택과 단일 테두리를 보존한다. native raster는 별도 확인한다.

## 검증과 handoff

읽기 감사에서 multiline focus가 전역1px foreground, singleline은1.5px accent인 코드 불일치를 확인했다. 현재 후보 준비 중이며 제품 변경·시험 통과를 주장하지 않는다.

- root는 후보의 실제 Panel.show regression만 먼저 통합했다. RED70699 exit101: actual stationary frame 폭1.0/기대1.5로 실패했다. compile/collector 오류가 아니고 source 불변 및 원 Job 자연 종료를 확인했다.
- enabled focus에만1.5px Light accent-solid/Dark accent를 적용했다. blur/disabled의 기존 interaction stroke와 caret/selection/document/scroll을 보존했다. 새 dependency·schema·lock 변경 없음. GREEN 검사 진행 중이며 native raster 및 전체 검사는 아직 남아 있다.
- GREEN38414 exit0: 두 UI harness 각각 진단17회귀 통과. 실제 Panel.show의 Light/Dark focus·blur·disabled exact viewport stroke와 입력 차단, 기존 caret/scroll/report 회귀 포함. source 불변 및 Job 자연 종료 확인. 네이티브 그림 검수는 아직 별도다.
- 전체67133은 Clippy가 새 테스트의 nested if를 collapsible_if로 거절해 exit1했다. 제품 오류나 성공으로 기록하지 않으며 원 실패를 보존했다. root가 let-chain으로 수정하고 input-focus-error-whole-v2 새 전체 검사를 실행했다. 동작 assertion을 약화하지 않았다.

- input-focus-error-whole-v2 전체28685 및 독립84941 exit0: fmt/clippy/test/release,43 harness/1161성공/0실패/30ignored와 원로그/source/artifact 검증 통과. CPU9,709,568바이트 SHA `b489e5932b227ee91f39eeeed345a5a493dffcf16a60c42f8882cacb5b58cb53`; source-first 대비1,024바이트 증가. 의존성/schema/lock 변경 없음.

- native Light2470/Dark33117 시작. 실제 empty/focus/blur/typed 상태·readonly preview·두 보고서 보존·원 fixture/source/privacy/자연 종료를 검사한다. disabled 시각 상태를 강제로 만들지 않으며 미검증 범위를 유지한다.

- native Light2470/Dark33117 exit0 및 독립20863/62715 exit0: 원 focus ACK·exact 한글24자·두 보고서 미적용 보존·privacy/DB/테마/source/CPU·자연 종료/Job0 검토 통과. fixture 최초 source→compile provenance는 여전히false, 전체 S10은 아니다.
- root 직접6PNG 검토: typed-unapplied는 단일 파란 outline·끝 caret를 확인했으나 focused-empty는 logical provider focus=true에도 회색 outline이었다. blur 캡처는 저장 동작으로 스크롤돼 편집기 전체 테두리가 보이지 않았다. 따라서 focus-empty 및 blur 시각 인수는 보류, root-scoped-pixel-review.json에 partial 기록했다. private HWND window focus와 logical UIA focus/실제 paint의 차이를 별도로 진단한다. 성공한 provider 회귀를 완전한 raster 인수로 바꾸지 않는다.

- source 감사에서 UIA 논리 focus와 Response.has_focus의 window input focus는 별도이며 typed Fill은 소유 WM_SETFOCUS를 전달하지만 빈 focus는 UIA만 사용함을 확인했다. 원인 확정으로 쓰지 않는다. baseline을 보존한 원 HWND/TID query·bounded owned focus event·빈 extra capture 비교를 새 Light95537/Dark28987에 실행했다. OS focus 획득·physical input 인수로 주장하지 않는다.

- window-input Light95537/Dark28987 및 독립24593/8928 exit0. 기존4단계와 추가5단계·원 HWND/TID query·bounded owned WM_SETFOCUS receipt·보고서/DB/privacy·자연 종료 검토 통과. root는 새 빈 capture2장을 직접 봤으나 outline은 여전히gray라 시각 인수 보류를 유지한다. WM_SETFOCUS 단독으로 해결된다고 주장하지 않는다. winit은active와focused를 별도로 합치며 실제 RawInput.focused 관측 없이 제품 원인을 확정하지 않는다.

- 최신 active-focus v3 private GUI Light33034/Dark45069 및 독립 대조6782/4596 exit0. 독립 JSON을 각 evidence에 보존했다. 원 HWND에 제한한 WM_NCACTIVATE→WM_SETFOCUS 이후 root가 직접 본 Light/Dark 빈 입력 캡처에 단일 파란 테두리와 시작 caret이 보인다. 합성 소유 창 이벤트 검증이며 실제 OS 포커스 획득·물리 IME 성공으로 표현하지 않는다. 전체 blur 테두리와 disabled native 픽셀은 남는다.

- diagnostic_inputs에 인접 버튼의 순수 focus로 빈 입력 전체 blur 테두리를 유지할 수 있는 native 시험 후보를 할당했다. 담당 범위는 /tmp 후보만이며 root가 프로젝트 통합·실제 실행을 책임진다. 실행 여부·결과를 후보 준비와 구별한다.

- 최신1165 전체 소스에 고정한 adjacent-blur native Light87995/Dark6320을 실행했다. 정제 후 반영 버튼의 순수 SetFocus이며 Apply를 실행하지 않는다. 담당 diagnostic_inputs가 /tmp 독립 reviewer 후보를 준비하고 root가 실제 원본 로그와 PNG를 통합 검증한다. source/Cargo/vendor는 검증 동안 고정한다.

- adjacent blur native Light87995/Dark6320 exit0. root는 두 blurred-empty PNG를 직접 확인했다. 빈 editor 네 테두리와 인접 버튼이 본문에 온전히 보이고 파란 focus outline이 정상 회색으로 돌아왔다. 각 evidence의 root-scoped-pixel-review.json에 원 PNG SHA와 좁은 판정을 보존했다.
- 독립 reviewer41812/15926은 exit2, active-focus route/no extra logical Focus 코드 구간 대조 실패. 원 JSON을 보존하고 diagnostic_inputs에 실제 source 구간 경계 확인을 요청했다. native 성공과 독립 대조 실패를 구별하며 아직 전체 인수를 주장하지 않는다.

- 독립 v2 40925/26897 exit0. 원 reviewer의 끝 marker가 없어진 탓에 active 단계 뒤의 blur/typed SetFocus까지 잘못 포함한 것을 확인했다. 실제 active receipt 완료를 정확한 구간 경계로 사용하고 원 메시지 순서·추가 focus 금지·후속 blur 별도 검증을 유지했다. 실제 양테마 독립 JSON을 evidence에 보존했다. focus/blur의 좁은 인수 증거는 통과했으며 disabled native와 물리 OS 입력은 여전히 별도다.

# S02: 파일 선택에서 미리보기와 저장 결과로 이어지는 흐름

- 상태: in-progress
- 분류: feature
- 우선순위: P1
- 요구사항: R04, R05, R08, R11
- 담당/수정 범위: root interface 조정; 후속 담당 app/accessible_library.rs, media_workspace.rs, core 정책 및 engine local export
- 선행 작업: S00-media-workspace-interaction, 현재 Windows exact-source whole/GUI 통합

## 사용자 결과

파일 선택에서 미리보기와 저장 결과로 이어지는 흐름.

## 계약과 범위

목록의 bounded 행 선택/Enter→read-only preview를 먼저 연결한다. 로컬 export는 명시 destination과 원본 보존·안전한 publication을 갖춘 별도 수직 slice로 진행한다. 지금 drop은 preview/range only이며 성공한 저장으로 표현하지 않는다. Rust native, source/worker/core 분리와 원 프로세스·파일 소유권을 유지한다. 공유 인터페이스/schema/Cargo는 root 조정이다. 아직 구현되지 않은 제안이다.

## 인수 조건

임시 라이브러리 원 행·잘못된 경로·foreign 기존 output·취소·원본 유지·늦은 요청을 시험한다. 선택 자체를 trust/deletion authority로 승격하지 않는다. 실제 합성 저장 결과·폴더 도달을 최종 slice에서 확인한다.

## 검증과 handoff

현재 holistic 감사로 제안만 기록했다. 실제 실행·완료는 없다. 사용자 UI/UX·물리IME/reader/OS DPI·전체P0는 이 기록으로 인수하지 않는다.

2026-10-11 사용자 전체 진행 지시에 따라 구현 착수. 담당/공유 계약과 통합은 P0-quality-completion-integration.md에서 조정한다. Git 불가로 하위 초안은/tmp에 분리하며root가현재 파일에 통합한다.

2026-10-11 목록 행 선택과 선택 영상 확인/Enter를 read-only preview에 연결했다. 실제 production panel RawInput/AccessKit2회귀는 Windows CPU에서 PASS(`windows-msvc-library-preview-flow-20261011`): take-once, disabled/busy/CloseWait 거절, fixture 원 bytes/DB 미생성 보존. 현재 연결은 UI 회귀 인수이며 실제 영상 local preview/export와 current private native 검증은 별도로 진행한다.

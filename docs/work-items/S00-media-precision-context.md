# S00: 긴 영상의 정밀 구간 조작

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R11
- 담당/수정 범위: root 예정: app/media_timeline.rs, media_workspace.rs, catalog.rs, core/media_selection.rs, CONTEXT.md, ADR
- 선행 작업: S00-media-workspace-interaction, 현재 Windows exact-source whole/GUI 통합

## 사용자 결과

긴 영상의 정밀 구간 조작.

## 계약과 범위

전체 영상 context와 선택 주변 visible window를 분리한다. 현재100ms 키보드·시간 입력을 유지하며 확대/전체 보기와 겹친 handles의 선택을 명료하게 한다. Rust native, source/worker/core 분리와 원 프로세스·파일 소유권을 유지한다. 공유 인터페이스/schema/Cargo는 root 조정이다. 아직 구현되지 않은 제안이다.

## 인수 조건

8시간/7일 도메인 fixture에서 짧은 구간을 pointer/Tab/keyboard/numeric으로 선택하고 구간 불변 조건·원본 보존·실제 짧은 합성 frame seek를 검증한다. 8시간 현실 대기를 필요 조건으로 삼지 않는다.

## 검증과 handoff

현재 holistic 감사로 제안만 기록했다. 실제 실행·완료는 없다. 사용자 UI/UX·물리IME/reader/OS DPI·전체P0는 이 기록으로 인수하지 않는다.

2026-10-11 사용자 전체 진행 지시에 따라 구현 착수. 담당/공유 계약과 통합은 P0-quality-completion-integration.md에서 조정한다. Git 불가로 하위 초안은/tmp에 분리하며root가현재 파일에 통합한다.

2026-10-11 전체 overview/선택 주변 확대/전체 보기·상하 분리 handle·offscreen pointer 거절을 통합했다. actual show의8시간/7일 RawInput 회귀가 최초 Windows whole에서 PASS했다. 숫자/keyboard/AccessKit는 전체 duration을 유지하며 playhead는 range 편집과 분리한다. current private native 시각/합성 seek 인수 전이며 전체P0 완료가 아니다.

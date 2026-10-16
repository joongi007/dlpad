# S00: 문맥에 맞는 하단 행동과 공간

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R01, R08, R11
- 담당/수정 범위: root 예정: app/accessible.rs, catalog.rs, ui_components.rs, DESIGN.md와 native GUI observer
- 선행 작업: S00-media-workspace-interaction, 현재 Windows exact-source whole/GUI 통합

## 사용자 결과

문맥에 맞는 하단 행동과 공간.

## 계약과 범위

200%에서 보조 행동이 두 줄을 소비하는 footer를 현재 작업·복구 상태에 맞춰 간결하게 한다. 자동 확인 안내는 이미 수정됐으며 그 수정이 없다고 재보고하지 않는다. Rust native, source/worker/core 분리와 원 프로세스·파일 소유권을 유지한다. 공유 인터페이스/schema/Cargo는 root 조정이다. 아직 구현되지 않은 제안이다.

## 인수 조건

800/850/960 및150/200의 primary/error/복구/재개 상태에서 overlap 없이 타임라인·시간 필드 도달과 키보드 접근을 실제 PNG로 확인한다. 기존 복구/취소 정책을 보존한다.

## 검증과 handoff

현재 holistic 감사로 제안만 기록했다. 실제 실행·완료는 없다. 사용자 UI/UX·물리IME/reader/OS DPI·전체P0는 이 기록으로 인수하지 않는다.

2026-10-11 사용자 전체 진행 지시에 따라 구현 착수. 담당/공유 계약과 통합은 P0-quality-completion-integration.md에서 조정한다. Git 불가로 하위 초안은/tmp에 분리하며root가현재 파일에 통합한다.

# S00: 도구 화면 본문 폭과 스크롤바 정렬

- 상태: in-progress
- 분류: quality
- 우선순위: P0
- 요구사항: R05, R08
- 담당/수정 범위: 주 담당 crates/app/src/accessible.rs와 이 문서
- 선행 작업: current-ui-placeholder-affordance의 실제200% 도구3종 캡처 검토

## 사용자 결과

도구 선택에 따라 본문 스크롤바가 좌우로 이동하지 않고 같은 본문 경계를 사용한다.

## 계약과 범위

최상위 tools-content ScrollArea의 auto_shrink를 두 축에서 비활성화한다. 내부 조작의 치수와 설치·취소 gate, 단일 스크롤과 one-shot focus 공개를 유지한다. 다른 화면과 schema/Cargo/의존성은 변경하지 않는다.

## 인수 조건

- 실제 Windows Light/Dark 및200%에서3개 도구의 스크롤바가 동일한 본문 오른쪽 경계에 있다.
- 설치 버튼과 버전 도움말에 스크롤로 접근하고 고정 상태줄을 침범하지 않는다.
- fmt·clippy·test·release 및 문서 검사를 실행한다. 이전 snapshot을 변경 후 검증으로 표현하지 않는다.

## 검증과 handoff

원본200% 세 캡처를 직접 열어 전체 설치 버튼 접근과 도구별 스크롤바 가로 위치 차이를 확인했다. root-light-200-tools-actions-pixel-review.json에 근거를 기록했다. 수정 반영 이후 검증은 진행 중이며 전체 UI/UX 인수는 별도다.

2026-10-12 실제 Windows whole 검증은 fmt·clippy·test·release 자연 종료 0, 1,139개 성공/실패0/ignored30이다. 주 담당이 원본 로그·당시 소스·실행 파일 해시를 독립 대조했다. 증거는 ../../validation/evidence/windows-msvc-tools-body-width-whole-20261012/root-review.json이다. 문서 검사 917개 통과. 수정 후 도구3종의 실제 Light/Dark200% 스크롤바 픽셀 인수는 아직 남아 있어 작업 상태를 완료로 바꾸지 않는다.

## 최신 scoped 검증 — 2026-10-12

Windows corrected whole1143/0/30 및 ordinary149 raw/provider 인수와 Dark200 focused11을 완료했다. Dark200 첫Light→선택Dark/재시작Dark, readonlySQLiteDark/integrityok/SHA전후, 세도구 설치영역 직접PNG 검토를 확인했다. 증거: [whole](../validation/evidence/windows-msvc-tool-action-navigation-corrected-whole-20261012/root-review.json), [Dark200 provider/DB](../validation/evidence/current-ui-tools-dark200-focused-v2-20261012/root-independent-provider-sqlite-review.json), [세영역픽셀](../validation/evidence/current-ui-tools-dark200-focused-v2-20261012/root-three-tools-pixel-review.json). v1noop/alias검토실패 원본 보존. 전체contents/물리hover/실제설치/userUIUX 인수가 아니므로 전체작업완료로확대하지 않는다. 동일1143 소스의 최신 패키지 실제5파일 설치·재검증 및 현재27 native ID 대조를 완료했다. [설치 기록](S11-current-ui-tool-action-package-20261012.md). 후속 Library 수정은 이 ZIP에 포함되지 않는다.

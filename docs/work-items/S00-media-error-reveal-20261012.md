# S00: 새 영상 실패 안내 공개

- 상태: implementing
- 분류: bug
- 우선순위: P0
- 요구사항: R04, R08, R13
- 담당/수정 범위: 주 담당; crates/app/src/media_workspace.rs, accessible.rs의 실제 영상 회귀
- 선행 작업: 합성 영상 확대 시험의 실패 재현

## 사용자 결과

저장 설정으로 스크롤한 뒤 새 영상 준비가 실패해도 오류 안내가 한 번 보인다. 이후 사용자 스크롤을 끌어오지 않는다.

## 계약과 범위

Rust native UI 유지. 실제 owned fixture에서 800px/150% 실패 안내가 이전 스크롤 위에 숨는 것을 원본 paint 로그로 확인했다. 새 error 또는 새 source의 같은 error에만 공개하고, 같은 frame의 재pass에서 의도를 유지한다. 포커스를 변경하지 않는다. 의존성·schema 변경 없음.

## 인수 조건

실제 준비 영상→저장 위치 focus/scroll→존재하지 않는 합성 영상→오류 ACK 후 전체 error glyph가 body clip에 표시. 이후 wheel은 공개 요청을 재발생시키지 않는다. 원본 영상·작업 종료를 보존한다. 800/960 및 확대 Context와 actual native screenshot을 구분한다.

## 검증과 handoff

windows-msvc-ready-error-geometry-diagnostic-20261012에서 960/824는 안내 paint 통과, 800/150%는 고급 옵션 영역만 보이는 상태로 실패했다. 원로그 보존. 수정 후 실제 회귀·전체 검사·캡처 필요.

새 소스 오류 안내는 windows-msvc-ready-canvas-inset-scoped-20261012의 정확1개 실제 합성 영상 회귀에서 네 가지 화면 조건 모두 full glyph paint로 PASS했다. 사용자 wheel 보존과 native screenshot은 아직 미검증이므로 작업 상태는 implementing 유지.

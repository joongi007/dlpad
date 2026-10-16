# S05: 작은 창 로컬 영상 열기 버튼과 결과 공개

- 상태: candidate / actual Context verification in progress
- 분류: bug
- 우선순위: P0
- 요구사항: R03, R05, R08
- 담당/수정 범위: search_reveal scratch media_detail_open.rs/media_workspace.rs; Root S·Live통합/검증/문서.
- 선행 작업: source31/필수도구guidance후보, 원 media native v3의 실제200% 결과 안내 가림.

## 사용자 결과

로컬 영상 준비 후 열기 버튼, 명시 열기 요청 결과를 한 번 공개한다. 이후 수동스크롤은 되돌리지 않는다.

## 계약과 범위

원32px·focus·Prepared원pin·worker·OS handoff/automation거절은 유지한다. 새one-shot bool/frame만 사용하며 daemon/의존성/schema변경0. URL canvas 자동 공개는 유지하고 로컬은 detailaction 공개를 우선한다. 같은 오류의 새 요청은 새 결과 episode로 공개한다.

## 인수 조건

실제 Context Light/Dark·800x600상당100/200%에서 전체원문galley/clip/footer·원toolbarfocus·반복오류·sourceclear·수동scroll을 검사한다. 현PE private native로 버튼과 결과glyph를 별도 검토한다. 외부사용자viewer/chooser·physicalhover는 미검증이다.

## 검증과 handoff

child rustfmt0, Cargo/native0. Root가 원 feature후보 byte일치를 확인하고 S에2파일을 stage했다. 실제 회귀3개/strictclippy는 실행중이며 Live/패키지31 미포함이다.

Root Windows actual3회귀는2PASS1FAIL이다. 최초action row의 LIBRARY_OPEN_DEFAULT 전체glyph가 actual body/paint clip 안이라는 검사가 실패했다. 반복결과·원focus/수동scroll/sourceclear2개는 통과했지만 통합완료가 아니며 scale/실제rect를 추가관측해 원인을 진단한다. 원v1/로그는 보존하고 strictclippy단계는 실행되지 않았다.

진단v2 실제1회귀는200% 최초frame0 body4976×4886,이후actualbody340×122/offset0/버튼Textshape0을 관측해 실패했다. Root는 잠긴egui0.33.3 context.rs begin_pass가 pendingzoom에서 직전InputState contentrect에ratio를곱해 RawInput.screen_rect를대체함을 직접확인했다. 새View의 미초기화 기본10000×10000가원인이며 is_sizing_pass라고 단정하지 않는다. 실제viewport빈passseed후원scale을적용하는 test setup 수정 후보를 준비한다. product runtime·실제glyph/clip조건은완화하지않는다.

원viewportseed를바로잡은v3는원3회귀모두통과했다. Root최종unfilteredapp490PASS/0FAIL/6ignored에포함되며원production동작후보v1은동일하다. actualprivate/현PE통합은아직대기다.

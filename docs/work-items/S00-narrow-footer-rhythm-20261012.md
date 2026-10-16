# S00: 좁은 화면 상태줄 간격

- 상태: in-progress
- 분류: bug
- 우선순위: P0
- 요구사항: R01, R08
- 담당/수정 범위: 주 담당 crates/app/src/accessible.rs 또는 accessible_metrics.rs의 실제 원인 callsite와 관련 실제 paint 회귀, 본 문서. diagnostic_cache는 /tmp/dlpad-footer-narrow-spacing-audit만 작성한다.
- 선행 작업: S00-subtitle-control-consistency-20261012의800200 캡처. 현재 source 고정 native 시험 종료 후에만 제품 변경.

## 사용자 결과

좁은 화면의 상태가 두 줄로 바뀌어도 과도한 줄 간 여백 없이 핵심 정보와 작업면을 유지한다.

## 계약과 범위

DESIGN.md의 간격 scale·상태 위계·고정 동작/본문 clip 규칙을 따른다. 현재800200 Light/Dark 실제 PNG에서 큰 줄 간 여백을 관찰했다. 오류·여유 공간·지원하지 않는 관측 안내를 숨기지 않는다. 제품 원인을 확인하기 전에 단순 높이 강제/clip/문구 삭제로 해결하지 않는다.

## 인수 조건

실제 Candidate800/960과100/150/200에서 한 줄/두 줄 상태의 전체glyph·본문과footer 비겹침을 검증한다. 작은외곽viewport에 상태를 강제로 clip하지 않는다. narrow PNG를 직접 확인하고 raw 입력/OS 범위를 구분한다.

## 검증과 handoff

현재는 원인 코드 감사와 후보 준비 중이다. source 변경·회귀·새 native 검증은 아직 미실행이다. 부모 자막 조작의 PASS를 상태줄 디자인 최종 인수로 확대하지 않는다.

원인 후보: egui horizontal_wrapped initial row height가 전역 interact_size32를 받으며 item gap8과 합쳐 두 줄 baseline 간격 약40logical/80physical이 된다. 읽기 전용 status scope만 actualBodyfont lineheight와 itemgap4를 적용하는 후보를 준비했다. native source 고정 시험과 독립 검토가 끝난 뒤 actual Candidate800600@200의 두 metric glyph 간 gap·전체clip 회귀를 먼저 통합하고 수정 전 RED session85777을 실행했다. 아직 terminal 결과/제품 수정/성공을 주장하지 않는다.

첫 RED 시도85777은 test 모듈을 bounded_font_tests에 삽입한 root 통합 오류와 PanelState::load의 현재 API 불일치로 compile exit101이었다. 제품 spacing RED가 아니다. 원 로그를 보존했고 실제 tests 모듈과 load(&Context,Id)로 고친 v2 session72522를 실행했다. production status spacing은 아직 수정하지 않았다.

교정된 RED72522은 actual Candidate 회귀에서 실패했다: free glyph bottom244.5, network glyph top272.5로 실제gap28logical(<=8 요구 위반). fixture의 두번째줄 전제가 실제성립했고 compile 실패와 구분했다. status scope의 interact height를 actualBodyheight로, row gap을4로 교정해 동일 GREEN54561을 실행했다. 버튼/입력/UI thread/worker/state 정책은 변경하지 않았다. 새 의존성·Cargo·schema·파일I/O/캐시 비용 증가가 없다. native 화면과 전체fmt/clippy/test/release는 아직 후속이다.

GREEN54561은 actual exit0이며 실제 app 두harness에서 동일 회귀가 각각1회 성공했다. 원 raw/source-before-after/private naturalJob0를 root-review.json으로 좁게 검토했다. UI가 사용하는 실제 glyph 사이 간격<=8과 전체footer/clip을 확인했으며 새 native pixels/모든상태/전체UI 인수는 아직 아니다.

품질 포커스 변경과 함께 whole11957 통합 검사를 실행했다. 새 native800200 후보는 /tmp/dlpad-footer-quality-private-probe에서 별도 준비 중이며 원 fixed source/proof와 actual PID/Focus/glyph/footer를 대조한다.

통합 whole11957의 fmt·clippy·test는 actual exit0이며43harness1151/0/30을 확인했다. release terminal은 아직 대기 중이다. source를 유지하며 동일 handle을 관측하고, 원 전체 결과와 새 binary hash를 독립 대조한 뒤 native 시험을 실행한다.

통합11957과 독립3744 모두actualexit0로 종료했다. raw4/current source/43harness1151/0/30/GNU·lock 보존을 확인했고 CPU9,692,672B SHA a67e5a0c61bf42d6a7b29983ef4890b823cb320e347293ba456ce1c7f0f0ed76이다. 이전 자막 후보보다2,048B 증가했으며 새 의존성·캐시는 없다. 실행 비용은 파일 크기만으로 증명하지 않는다. 동일 source의 native Dark42146/Light85429를 별도 ownedroot/desktop에서 실행했다. 현재 실제 화면·OS Focus·단계/종료 결과는 미인수다.

새800200 Light/Dark 품질Focus PNG를 직접 보았으며 하단 두 줄이 텍스트 행 간격으로 가까워지고 본문과 겹치지 않는 것을 확인했다. 이는 첫 두 frame의 좁은 시각 확인이며 전체 상태/배율/조작 완료는 아니다.

native Dark42146/Light85429 모두 actualexit0로 종료했고 독립91533/52455도exit0이다. 원 source/CPU·각50PNG 범위·실제Focus ACK·label/control 전체bounds·지표 사각형/관측간격·first/restart theme DB·자연 app/Job0/private cleanup를 대조했다. root는 각 first/restart의 quality-focus PNG(합계4)를 직접 보아 제목·컨트롤·focus outline·상태줄 줄 간격·footer 비겹침을 scoped 인수했다. 모든100PNG 직접 픽셀/모든배율·상태/물리입력·전체UIUX를 인수한 것은 아니다. 현재 설치 ZIP은 수정 전 후보이며 새 패키징은 후속이다.

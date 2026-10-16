# S00: 품질 선택 포커스 공개 재현

- 상태: in-progress
- 분류: bug
- 우선순위: P0
- 요구사항: R08
- 담당/수정 범위: 주 담당 crates/app/src/accessible.rs의 실제 Candidate 회귀 및 원인 확인 후 accessible_options.rs, 본 문서. 하위 에이전트는 제품 파일을 수정하지 않는다.
- 선행 작업: 공통 UI 코드 감사, S00-narrow-footer-rhythm-20261012

## 사용자 결과

좁은 고배율 화면에서 품질 선택에 포커스를 주면 선택값과 조작 전체가 본문 가시 영역에 들어오는지 확인하고 실패하면 교정한다.

## 계약과 범위

고정 footer를 포함한 실제 Candidate를 사용한다. 테스트 전용 큰 Options viewport나 scroll offset 강제 주입으로 대체하지 않는다. 코드에 별도 reveal가 없다는 사실만으로 제품 버그라고 판정하지 않는다.

## 인수 조건

actual Combo를 role/value로 정확히 찾고 같은ID의 포커스·전체 glyph/paint clip·footer 비겹침을 검사한다. target 탐색 실패와 실제 숨은 포커스 실패를 구분한다. Context 결과를 native OS/물리 키보드 인수로 확대하지 않는다.

## 검증과 handoff

/tmp/dlpad-quality-focus-reproduction 후보를 읽고 실제 tests 모듈에 통합한다. PanelState 현재 API를 root에서 확인해 적용한다. 실제 실행 전 성공·실패는 미확정이다.

RED79249은 actual exit101이었다. unique Combo와 sameID focus 확인 후 rect y552..584가 footer y252..300 밖에 남는 것을 재현했다. target 탐색이나 컴파일 실패가 아니다. 품질 label과 Combo response를 유지하고 gained_focus 때만 union rect를 parent scroll에 공개하도록 수정했다. 기존 ID·메뉴 닫기·disabled·품질 값은 유지하며 새 cache/I/O/의존성이나 매 frame 강제 스크롤은 추가하지 않았다. GREEN93238을 실행 중이다.

GREEN93238은 actual exit0이며 app 두harness에서 새 품질Focus 회귀가 각각1회 성공했다. 원 raw/source/private naturalJob0를 root-review.json으로 대조했다. 현재 새 whole11957은 footer와 품질 두 변경 및 회귀를 통합한 fmt/clippy/test/release를 실행 중이다. Context source의 성공을 기존 설치 ZIP/OS 물리입력/새 native pixels에 소급하지 않는다.

통합 whole11957의 fmt·clippy·test는 actual exit0이며43harness1151/0/30을 확인했다. release terminal은 아직 대기 중이다. source를 유지하며 동일 handle을 관측하고, 원 전체 결과와 새 binary hash를 독립 대조한 뒤 native 시험을 실행한다.

통합11957과 독립3744 모두actualexit0로 종료했다. raw4/current source/43harness1151/0/30/GNU·lock 보존을 확인했고 CPU9,692,672B SHA a67e5a0c61bf42d6a7b29983ef4890b823cb320e347293ba456ce1c7f0f0ed76이다. 이전 자막 후보보다2,048B 증가했으며 새 의존성·캐시는 없다. 실행 비용은 파일 크기만으로 증명하지 않는다. 동일 source의 native Dark42146/Light85429를 별도 ownedroot/desktop에서 실행했다. 현재 실제 화면·OS Focus·단계/종료 결과는 미인수다.

주 담당이 새 Light/Dark first/quality-focus.png를 직접 확인했다. 제목·Combo 전문·focus outline이 보이고 footer와 겹치지 않는다. 원PNG SHA와 좁은 판정은 각 root-first-quality-footer-pixel-review.json에 기록했다. native 전체 조작·재시작·종료의 독립 검토는 아직 대기 중이다.

native Dark42146/Light85429 모두 actualexit0로 종료했고 독립91533/52455도exit0이다. 원 source/CPU·각50PNG 범위·실제Focus ACK·label/control 전체bounds·지표 사각형/관측간격·first/restart theme DB·자연 app/Job0/private cleanup를 대조했다. root는 각 first/restart의 quality-focus PNG(합계4)를 직접 보아 제목·컨트롤·focus outline·상태줄 줄 간격·footer 비겹침을 scoped 인수했다. 모든100PNG 직접 픽셀/모든배율·상태/물리입력·전체UIUX를 인수한 것은 아니다. 현재 설치 ZIP은 수정 전 후보이며 새 패키징은 후속이다.

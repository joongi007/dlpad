# UI: 좁은 화면의 보호된 도움말 높이 제한 후보

- 상태: proposed — 구현 후보만 준비, Root 실제 실행 전
- 분류: bug
- 우선순위: P0 UI 검수 후속
- 요구사항: R08, 사용자 UI/UX 추가 인수 조건
- 담당/수정 범위: field_reveal; scratch help-small-popup-fix-v1의 ui_components.rs 파생 파일만
- 선행 작업: source30 닫힘 상태 개선 유지; protected native v3 실제 800×600/200% Deno 겹침 PNG

## 사용자 결과

입력칸 바깥에 설명을 읽을 만한 공간이 있으면 도움말을 그 공간에 배치하고 긴 본문은 도움말 안에서 스크롤한다. 원 입력칸과 찾아보기·도움말 버튼이 가려지지 않아 사용 중인 설정과 설명을 함께 확인한다.

## 계약과 범위

DESIGN.md·채택된 Rust 네이티브 계약을 유지한다. 외부 help/show_help 인터페이스, owner/pinned/scroll 논리 ID, 기존 source30 HelpDismiss 상태기계는 그대로다. 실제 viewport와 owner+input+browse anchor에서 위·아래 공간을 산출한다. 현재 popup Frame 여백/테두리, 그림자 양의 위·아래 여유, 기본 간격 4pt를 차감하고 선택 방향과 동일한 높이로 ScrollArea를 제한한다. 제목 유무 및 실제 폰트 행높이로 최소 읽기 공간(본문 두 행과 pinned 제목 행)을 판단한다. 인접 footer/다른 설정을 모두 안 가리는 범위로 확대하지 않는다.

가로 너비 정책과 원문은 유지한다. 위·아래 모두 최소 읽기 공간이 없으면 기존 읽기/닫기 fallback을 유지하며 이 경우 보호 입력 비가림을 주장하지 않는다. 좌우 배치 최적화는 범위 밖이다.

## 인수 조건

- 실제 Context의 400×300 logical scene, 보호 행 y91..163(아래 여유) 및 y191..263(위 여유), 100/200%와 Light/Dark에서 원 Deno 본문을 사용한다. 좁힌 실제 input+browse는 원 content/viewport 내부여야 한다.
- 실제 focused 설명 및 pinned Area rect가 owner/input/browse와 교차하지 않는다. pinned 첫 표시 이후 3개 관측 프레임에서 actual frame과 실제 shadow.as_shape의 blur penumbra도 보호 controls/owner와 교차하지 않는다.
- 실제 MouseWheel만으로 원 본문 마지막 glyph 행 전체가 paint clip 안으로 들어온다. 텍스트 생략·수동 offset·가짜 focus 사용 금지.
- Escape 후 6개 실제 프레임에 pinned/automatic 본문이 없고 원 owner focus/ID가 유지된다. Enter 명시 재열기는 같은 popup ID로 성공한다. 원 경로 내용은 바뀌지 않는다.
- 기존 protected focus/hover/pinned, 극소공간 readable fallback, source30 window-focus/stationary-popup regressions를 모두 유지한다.

## 검증과 handoff

실행한 것은 Rust 1.94 rustfmt뿐이며 종료 0이다. Cargo/GUI/native는 실행하지 않았다. RED semantic 실패, GREEN 통과, strict clippy 및 새 source proof/native PNG는 Root가 확인해야 한다. 예상 성공 수나 hash를 미리 기록하지 않는다.

비용: 의존성/네트워크/상주 작업/새 UI 없음. 도움말 표시 경로에서 상수 개의 사각형·스타일 계산과 실제 폰트 행높이 조회를 추가한다. 본문과 기존 bounded ScrollArea를 재사용하며 텍스트 복사/무한 캐시/대량 할당을 추가하지 않는다. 정확한 PE 크기/실행 비용은 후속 실제 빌드 전 미측정이다.

Root가기존S00-help-placement의실제FFmpeg/Deno200%가림근거로후속실험을채택했다. S에서RED검사를실행중이며재현성공/수정완료는아직주장하지않는다.

Root실제RED는compile성공후0PASS/1FAIL이다. 원Deno설명frame[[8,0],[373.6,171]]이실제입력+browse[[8,131],[392,163]]를가렸다. 이어S GREEN의동일시험은1PASS/0FAIL이며상하·Light/Dark·100/200%의frame/shadow비가림·실제wheel끝문장·Escape/같은ID재열기를확인했다. 기존회귀/strict/full/native재검증은아직남아있다. 원RED실패를보존한다.

Root가실제S검사·원소스674/현재12파일·lock불변을대조해source31에채택했다. source675의새Windows전체검사실행중이며현재사용자v30 ZIP과구별한다. 격리최종strictworkspace/all-targetsclippy는0이다.

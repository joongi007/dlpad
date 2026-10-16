# S00: 진단 저장 결과의 조작 문맥

- 상태: in-progress
- 분류: UX
- 우선순위: P0
- 요구사항: R08, R13
- 담당/수정 범위: root — crates/app/src/accessible_diagnostics.rs, catalog.rs(필요한 문구), 이 문서
- 선행 작업: 파일 선택창 source 고정 GUI/독립 검토 종료

## 사용자 결과

저장한 결과와 실패 이유를 저장 버튼 옆 문맥에서 바로 확인한다. 선택한 긴 경로도 전체를 확인할 수 있다.

## 계약과 범위

현재 Light 실제 markdown-explicit-save.png에서는 저장 조작 뒤 완료 안내가 아래 로그 편집/MPD 영역 뒤에 있어 보이지 않는다. provider label 존재로 가시성을 대신하지 않는다. 저장·첨부·로그 작업의 결과를 발생한 조작 가까이에 배치하며 기존 작업 worker·create-only·privacy·논리ID·포커스 정책을 유지한다. 파일 선택 직후 긴 경로가 잘리는 경우 전체 경로 확인을 제공한다. 결과 때문에 focus를 강제로 옮기거나 idle마다 scroll을 되돌리지 않는다. 새 DB/schema/Cargo 변경은 없다.

## 인수 조건

저장 성공·기존 파일 거절은 실제 Save 조작과 같은 viewport에서 전체 문구가 읽힌다. MPD/로그 오류가 저장 오류처럼 표시되지 않는다. 상태 삽입에도 입력ID·편집초안·선택·후속사용자 스크롤이 유지된다. Light/Dark 및 작은 창/확대 실제 캡처로 확인한다. 단위/provider 통과만으로 시각 완료를 주장하지 않는다.

## 검증과 handoff

actual Light v7의 draft/save PNG를 root가 직접 확인한 근거로 기록했다. 아직 구현/실행 완료가 아니며 전체 UI/UX 인수도 아니다.

- 하위 후보 준비는 모델 capacity 오류로 종료되어 root가 직접 구현했다. 임시 FeedbackScope로 저장/MPD/로그/일반 결과를 해당조작그룹에 한 번 표시하고 진단경로에 기존폴더와같은 full path hover를 추가했다. 원상태문구·worker·create-only·입력ID는 유지한다. pinned rustfmt 통과, 기능/실제paint/확대검증은 아직 남는다.

- 기존 진단 cluster56692 exit0/source 보존: 새 안내 위치와fullpath hover 통합 뒤 진단·취소·기존파일보존·입력geometry·포커스·parent scroll 회귀가 통과했다. 이 결과만으로 새 성공안내의paint/확대가시성을인수하지않으며 최신whole과실제저장PNG로다음검증한다.

- 최신 whole88798 exit0와 독립23548 exit0: Windows fmt/clippy/test/release,1193/0/30·원source/log/artifact 대조 통과. CPU9,760,768B SHA9ef62a74e05a3e4cd8fdc2fc0c51608d1d3c35f1cb0f9f7dc03d1dbdabb64336으로 같은크기지만원1193 picker바이너리와다른SHA다. 새Cargo/lock변경없음. actual Markdown Light5140을새whole/CPU로실행해savedlabel의버튼주변가시성을검수한다. 작은창/확대와상태그룹오류회귀는미완료다.

- 실제 Markdown Light5140/Dark23268 및 JSON Light19994/Dark9012는 모두 exit0. 최신1193 source/CPU를 사용했으며 independent Markdown40010/40635와 JSON 별도 reviewer도 exit0이다. 각각 evidence/root-independent-review.json에 원 로그·도구·테마·readonly DB·create-only 보존 대조를 보관했다. root는 Dark Markdown 및 Light/Dark JSON의 actual explicit-save PNG에서 성공 문구가 저장 버튼 아래 같은 viewport에 보이는 것을 직접 확인했다. 이 인수는960×640/100% 범위이며 작은 창/200%, 오류 문맥·취소·MPD·fullpath hover의 실제 조작은 여전히 미검증이다. 전체 UI/UX 또는 최신 설치본 완료를 뜻하지 않는다.

- actual800×640 창에서 Settings의200%를 실제 선택하고 다운로드 탭으로 돌아와 저장하는 격리 native 후보를 /tmp/dlpad-diagnostic-feedback-scaled-native-20261012에 준비했다. Light54263/Dark80398 실행 중이며 완료/paint 가시성 판정은 아직 하지 않았다. 원source·CPU·도구·private Job guard를 유지한다.

- actual800640/app200% Light54263/Dark80398 exit0. root가 두 markdown-explicit-save.png를 직접 확인했고 저장 버튼과 전체 성공 안내가 같은body viewport에서 읽힌다. top 경로 필드는 스크롤로 일부clip되어있으며 fullpath hover 인수로 확대하지 않는다. root-scoped-pixel-review.json에 각 원PNG SHA와 좁은scope를 기록했다. 독립37365/64717 진행 중. 기존파일 실패 안내의 전체가시성·로그/MPD 문맥은 아직 남는다.

- 독립37365/64717도 exit0: actual200% settings ACK·800640 원기하·소스/whole/bytes/raw/readonly DB/create-only/currentownerchain을 별도로 대조했다. evidence/root-independent-review.json에 보관했으며 픽셀판정은 별도root-scoped-pixel-review.json의 범위만 적용한다.

- 기존파일거절의actual200% terminal.png Light/Dark도root가직접확인했다. 저장버튼과전체두줄실패안내가같은body안에보이고footer와겹치지않는다. 이전독립byte/SHA보존근거와함께root-existing-file-refusal-pixel-review.json에원PNG SHA를기록했다. 이로써200% 저장성공·기존파일거절가시성은좁게인수했으나로그/MPD문맥·fullpath hover·전체UI는미완료다.

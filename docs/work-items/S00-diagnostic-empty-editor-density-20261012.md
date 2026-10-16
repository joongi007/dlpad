# S00: 진단 빈 편집기 밀도

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R13
- 담당/수정 범위: 주 담당 crates/app/src/accessible_diagnostics.rs와 실제 Panel 회귀, 본 기록. 하위 에이전트는 /tmp 후보만 작성했다.
- 선행 작업: S00-quality-focus-reproduction-20261012, 기존 진단 입력·정제·저장 검증

## 사용자 결과

사용 전 빈 로그 편집기가 불필요한 공간을 차지하지 않고, 편집을 시작하면 충분한 높이를 제공한다.

## 계약과 범위

기존 editor/scroll ID, char·byte 상한, caret/selection, 정제·Copy/Save/MPD/pagination·worker ACK를 유지한다. 먼저 실제 Panel 회귀로 기존120px 공간을 기록한다. 포커스 이탈에 따른 축소가 화면을 흔들면 세션 중 확장 유지 정책을 검토한다. 큰 폼 전체를 새로운 중첩 ScrollArea로 감싸지 않는다.

## 인수 조건

빈 compact64·Focus120·typing·blur·ID/caret·보고서 미적용 보존을 실제 Panel/부모clip에서 확인한다. 실패 경로 및 기존 정제·저장·pagination 회귀를 재실행하고 실제 Candidate native 화면으로 통합 검증한다. Panel 회귀만으로 전체 UX 완료를 주장하지 않는다.

## 검증과 handoff

후보의 현재 egui API와 실제 모듈 위치를 root가 검토하고 수정 전 RED를 실행한다. source 변경은 원 설치 검증 및 현재native ID27 종료 후 수행한다. 현재 설치 ZIP은 이전 검증 snapshot으로 유지한다.

RED34696은actualexit101이었다. actual inner viewport y414..534로120높이가 확인되어 빈64 요구를 위반했다. compilation/target탐색 실패와 구분한다. 패널 수명에만 유지하는 transient edit_expanded(bool)을 추가했다. 새 패널은false이며 실제Focus/ScrollIntoView 요청·현재Focus·기존내용이 있으면true로 전환하고 빈blur에도확장을유지한다. UI상태는 serialize/DB/core domain에추가하지않으며 paneldrop시끝난다. 목표는사용전밀도와편집중안정성이다. 새string/schema/Cargo/I/O/cache 추가없고 bool하나의작은UI상태비용만추가한다. 동일 회귀에emptyblur확장유지까지추가한 GREEN34027을 실행했다. native/전체정제·저장 인수는 아직후속이다.

GREEN34027은actualexit0이며 app 두harness에서 새 회귀가각각1회 성공했다. 원 raw/source 보존은root-review.json에좁게검토했다. 통합18150은fmt/clippy/test actualexit0·43harness1153/0/30이며release는대기중이다. 실제Candidate native 후보는 /tmp/dlpad-diagnostic-density-native에서준비중이고, TextEdit documentbounds와ScrollArea viewport를구별해관측한다. 기존metadata 실패보고서·privacy/export/DB/원cookie·sentinel 보존·naturalJob0를다시대조할예정이며현재source로성공한것이라고아직표시하지않는다.

통합18150과 독립83505가모두actualexit0로종료했다. raw4/source/artifact/43harness1153/0/30/GNU·lock보존을확인했다. CPU9,693,184B SHA38f93a7133d5c050d0e56ee3113e49fb8a03fb0b9c6fe66eebf0cbe191d38ebd이며이전후보대비512B 증가했다. 새 의존성은없고런타임비용은크기만으로완료주장하지않는다. 일반release native35057에서stress를unset하고원Rust metadata-failure fixture를사용해editor실제입력/보고서불변/정제/DB/원본/naturalJob0를시험중이다. viewport/documentbounds를구별하고지금은native성공을미주장한다. 독립담당은 /tmp/dlpad-diagnostic-density-native-independent만준비한다.

native35057은exit2로실패했다. 원GUI result는 editable multiline Fill 기대24/실제20의합성입력readback불일치이며actualvalue가다른문자로기록됐다. UIAsetter는0문자로미적용되었고이후ownedF24/WM_CHAR route였다. sourceliteral의UTF8SHA와기대SHA일치로단순소스encoding가설은낮아진다. 제품버그/IME인수/밀도native전체성공을주장하지않는다. observer기본cookie inputslug재사용으로이전cookie영수증을같은디렉터리에서덮어쓴결함도발견하여freshownedinputsubdir로분리한다. 원failure를보존하고 /tmp/dlpad-diagnostic-density-native-v2에서sender/대기/예전성공route를최소재현한다.

문자전송경계가설을좁혔다. originalDllImport PostMessage는CharSet/EntryPoint가없고 합U+D569·성U+C131의하위바이트69/31이actual'i1'과일치한다. locked winit handler의F24/WM_CHAR text조합경로를독립읽기했으며Unicode전달값을복구하는분기가없다. [Microsoft WM_CHAR](https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-char)는Unicode창UTF16 codeunit을규정한다. 이는원인추론이며확정이아니다. v2의focus/ASCIIprobe변수와분리해v3는원Fill·timing·same24draft를유지하고freshinputreceipt와PostMessageW명시만교정해비교한다. productsource변경은없다.

v3 observer는원Fill·same24한글문자열·F24/timing을유지했고EntryPoint=PostMessageW/ExactSpelling/Unicode및원HWND IsWindowUnicode검증을추가했다. 이전영수증덮어쓰기는별도freshownedinput디렉터리로예방한다. root는samecurrentwhole/CPU/fixturepin으로native57241을실행했다. 실패원본과v2를보존하며actualreadback이통과하기전원인확정/제품입력완료를미주장한다.

v3 native57241은C#compile CS0030(System.Windows.Rect→double cast)로앱실행전에exit2였다. Unicode가설의actualinput 결과로세지않는다. 원compileFAIL을보존했고ManagedTextPatternRange.GetBoundingRectangles의실제Rect[]를X/Y/W/H로정규화하는v4를읽어적용했다. Empty/비유한사각형은unsupported/null로표시하고fakecaret0좌표를만들지않는다. v4 native7228을samecurrentproof/CPU/sameUnicode24draft/90worker로실행중이다. 제품코드변경은없다.

v4 native7228은한글expected24/actual24·SHAe1ec35c7034872a3dee7a48f8b559f6dc1589be666b4f3978268710fd22f75aa 일치까지확인했다. root가typed-unapplied-editor.png의한글·끝커서를직접보았다. InputA/W경계교정의좁은성공이며전체run은90초worker상한으로FAIL이다. 실제trace첫export58.7초/둘째export70.3초/JSON74.9초/경로입력중90.4초를기록했다. 동일tool/host상한을유지하면서UI관측기만2export측정업무량(약98초+여유20)에맞춘120초v5로재시험한다. 제품타임아웃을늘리거나강제cleanup을성공으로바꾸지않는다. 원FAIL과scopedreadback증거를별도보존했다.

### v5 전체 흐름 검증 — 2026-10-12

- root 실제 collector session22415 exit0: 첫 저장·한글 미반영 초안·두 번째 저장·자연 종료까지97.537초. 앱 exit0, 원 Job active0, private desktop 닫힘, 사용자 input desktop 동일. 제품 timeout 변경 없음.
- 독립 검토 session83265 exit0; 원본 검토 결과는 `../validation/evidence/windows-msvc-diagnostic-density-normal-private-v5-20261012/root-independent-native-review.json`. 이전 v1/v3/v4 실패는 보존.
- root가 실제 typed-unapplied-editor.png를 직접 확인: 한글 초안과 끝 커서가 보이며 하단 테두리가 두 겹으로 보이는 시각 문제를 확인했다. 전체 UI/UX 완료 또는 물리 IME 검증으로 확대하지 않는다.
- `python3 scripts/check_docs.py` session45293 exit0:932문서/114 hash-matched notices. 이 단락 추가 전 실행 결과이며 다음 문서 검사를 남긴다.

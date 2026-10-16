# S04: 관리 runtime UI 통합 후 독립 감사

- 상태: done
- 분류: review
- 우선순위: P0
- 요구사항: R03, R05, R08, R11
- 담당/수정 범위: metrics_recovery — 이 문서만
- 선행 작업: root metadata inventory/verified Use/typed apply 통합, 별도 실제 GUI 검증

## 검토 snapshot과 정상 불변

accessible_tools.rs SHAf3e8307bb39b34f3d733458ae1ddb72d52b1c4028c0b1b9c04546cec49d1de7f를 읽었다. root recorded_package_for_manifest는 metadata lookup이고 선택 worker가 실제 select_verified_deno/ffmpeg를 수행하므로 목록 비용과사용증거가 분리됐다. runtime Use는 active DB와metadata 목록의exactversion/target을 찾고 다시 verifiedselection후 typed Selected를 반환한다. accessible.rs apply_managed_tool는 ToolId에따라 tool/deno/ffmpeg를정확반영하며 closewait/worker중에는적용을거절한다.

tool변경은busy때disabled이고 staleinventory/selected/error를지운뒤Refresh를보낸다. 단일commandqueue/busy prevents 같은worker의oldtool결과를새tab에섞는경로다. sameframe switch후 enabled변수가이전값인경계에서도submit 자체busy검사가추가command를거절한다. refresh실패는기존inventory를보존하나 switch직전에비워서다른tool inventory를노출하지않는다. Select/Remove manifesttool mismatch가거절되고 활성행버튼disabled와engineActiveVersion재확인이이중보호한다. 공급runtimeInstalldisabled와실제workerpending거절이일치한다.

## 구체적 표시 문제

show는모든tool에TOOL_HELP/TOOL_LICENSE를공통출력한다. 현재catalog는yt-dlp고정버전/HTTPS·SHA·version뒤activation과GPL3·THIRD_PARTY_LICENSES 설명이다. Deno/FFmpeg탭에서도이설명이동시에보여선택runtime의license와activation정책으로오해될수있다. runtimepending안내자체는맞지만genericlicense로처리하면MIT/LGPL3와구별되지않는다. 최소제품후속은yt-dlp때만기존두labels를표시하고runtime에는일반앱소유/외부보존과각선택package license/공급gate 중앙문구를사용하는것이다. 임의UIliteral이나공급채택승격은금지한다.

## 가시성·키보드 미검증 경계

toolsPanel은CentralPanel에직접show되고outerScrollArea가없다. inventory만max_height320스크롤이며그아래activepath와Use가있다. 긴active Unicodepath wrapping·status/error/recovered/preserved labels가겹치면960×640 또는800×600에서Use가viewport아래에놓일가능성이있다. 이는코드상후보이며현재실제clip재현을주장하지않는다. 실제GUI에서Use의UIAIsOffscreen/bounds-clientintersection/PNG를검사하고Tabfocus로도달후실제visible인지확인한다. 문제발견시wholetoolpage outer scroll 또는 inventoryavailableheight를최소후보로측정하며UIA 존재만으로PASS확대하지않는다.

## 남은 수명 조건

UIbusy상호배제는현재앱의tool삭제를막고activeengineguard가활성package제거를막는다. 다른manager가선택active를바꾸는중이전runtime사용·삭제경계는별도engineexecutionlease정책에따르며현재UI통과로완료라고하지않는다. runtimeinstaller는미채택공급disabled를유지하고existingverifiedpackage만select/use/remove허용한다.

## 검증과 handoff

읽기감사와문서만작성했다. Rust/Cargo/runner/GUI실행·sourcefreeze변경은없다. 실제GUI원시result는root별도통합근거다. concretecatalog표시혼동은root에보고했고scrollclip은실제인수후판단할후보다.

## 주 담당 후속 수정

root가 runtime 탭에 yt-dlp전용help/license가 표시되는혼동을수정했다. 중앙catalog의runtime관리·외부보존설명과공급/라이선스확인중안내만표시하고yt-dlp탭은기존설명을유지한다. catalog.rs/accessibile_tools.rs만수정하며실제GUI는새release로검증한다. select-use/wrong-version이전button-retry는refresh중disabledaction을수집기가너무빨리click하여exit1;기존원시보존및수집기boundedEnabled대기후재시험한다.

# S00: 파일 목록의 키보드 전체 경로 공개

- 상태: in-progress
- 분류: UX
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: root — accessible_library.rs와 필요시 ui_components.rs/catalog.rs, 이 기록과 실제 검증; diagnostic_cache — /tmp 후보만
- 선행 작업: 목록 가상화·128행 page·안정 path ID/Arrow·Tab one-shot reveal와 UI 감사

## 사용자 결과

마우스 없이 긴 파일명 행에 포커스를 주고 전체 경로를 확인하며 선택된 행과 포커스 상태를 구별한다.

## 계약과 범위

DESIGN/GTMUX의 hover와 keyboard focus 모두 도움 공개 기준을 적용한다. rows 가상화·page 상한·core/DB/scan worker와 Enter preview 행동은 변경하지 않는다. 경로는 bounded UI 안에 표시하고 새 권한이나 파일 소유권을 의미하지 않는다. 제품source는 활성 GUI/독립 검증 종료 전 수정하지 않는다.

## 인수 조건

포인터 없는 실제 Context focus에서 전체 경로가 공개되고 focusoff에서 닫힌다. 긴 Unicode path에서도 viewport 밖으로 겹치지 않고 기존 Tab/Arrow/wheel/selected/Enter 및 row ID를 보존한다. Light/Dark800200 실제 populated 캡처와 조작을 별도로 확인한다.

## 검증과 handoff

현재 후보 준비 단계다. 읽기 감사의 성공이나 빈 목록 캡처를 populated 키보드 인수로 사용하지 않는다.

- 원 focusTooltip 후보는 긴 경로 하단을 키보드로 스크롤하는 route가 미완성이라 채택하지 않았다. 후속 `/tmp/dlpad-library-keyboard-detail-20261012`는 가상행 밖 단일 action과 bounded readonly 상세/PageUp·Down·Home·End/Esc 원row복귀를 제안한다. root가 hunk를 읽었으며 기존 행의 Enterpreview를 바꾸지 않는 조건과 실제 Panel multipass/resize/사라진row복귀/clip 회귀가 필요하다. 아직 source 통합·컴파일·native 완료가 아니다.

- root가 단일 action/readonly 상세 후보를 검토한 뒤 helper 회귀만으로 채택하지 않고 실제 LibraryPanel의 key선택→open→Page/Home/End→close→materialized ID/focus/clip·선택·preview 보존 회귀를 /tmp에 추가 준비하도록 했다. 재관측/페이지변경/사라진row/resize close와 배경 keyboard route를 별도 실패 경로로 포함한다. 제품source는 현재 GUI 검증 중이라 동결한다.

- 모든이전GUI/독립handle terminal후 root가 현재library sourceSHA를감사기준과대조하고 단일action/readonly 키보드상세와 actual전체Panel2회귀를통합했다. 원virtual128/stride/행ID/scanDB/preview변경없고 catalog4문구추가, Cargo변경없음. Space선택→상세→Page/Home/End→close→원row복귀와행삭제/페이지교체/resize를실제 Context로검증한다. 아직compile/native미완료다.

- 첫34872의 파일명red는 단계이름일뿐이며 실제결과exit0/기본+CPU각2개회귀통과. root가원guard/current source와실제4test수를독립대조했다. actualPanel 경로endpoint의galley source/glyph clip·readonly/선택·rowArc·previewNone·keyboardreturn·staleid/resize 처리를확인했다. 실제Windowsfont glyph/physicalIME/OSscreenreader/nativePNG 인수는아니다. 전체82307 실행중이며 기존목록/폼 회귀와whole/native를이어간다.

- 전체82307은fmt통과/clippy의collapsible_if·needless_bool2건으로exit1. root가같은동작의조건식으로정리하고 bool직접반환으로수정했다. 첫실패로그보존, source/Cargo/schema 정책변경없음. 새v2 전체검사를 실행한다.

- 전체v2 2103와 독립58131 exit0:43harness/1201성공/0실패/30ignored, 원로그/source/artifact대조. CPU9,779,200B/SHA `4e4c6eb61910eb2a8ceb454118a4ffb94e8a1512cc27004e8328d1c720620e63`, 이전숫자폭snapshot보다10,240B증가, 신규crate/feature/Cargo.lock변경없음. 새실제library native후보준비중이며physical/native시각인수미완료.

- release_preparation은 /tmp에 실제library native/reviewer후보만담당한다. 합성긴한글/emoji파일을선택하고키보드상세·endpoint·close원row복귀를Light/Dark800200에서관찰하며, source/whole/CPU/fixture/nativeparent·leafID/privacy/원data/desktop/natural guards를유지한다. 프로젝트/native/Cargo실행은root만하고 아직새actualPNG인수없음.

- 실제library11511 exit1: 행Space선택predicate 실패, 자연Job0는확인되었지만전체workflow실패보존. root가 원DetailKey가keydown/up만보내고Space WM_CHAR·owned focus메시지가없음을확인했다. v2는원HWND/PID/creation보호아래Space down-char-up envelope와실제선택 Toggle/focus/원·현재ID계측을추가해 입력전달가설을시험한다. Click/Invoke 대체없음/제품수정없음. 원scope·128files·nativeIDs·privacy·desktop·timeout상한유지.

- nativev2 56174 exit1. Space계측에서rowfocustrue이지만TogglePatternavailablefalse를확인했다. 원eguiButton::selectable는시각selected만그리고WidgetInfo::labeled(Button)를내보내며library가선택상태를추가하지않는source를root확인했다. 모든native/독립handleterminal후실제Panel의미선택False→Space선택True 접근성회귀를먼저추가해RED를확인한다. 기존role/ID/label/선택정책을유지한정보전달수정예정이다. Dark800200두scope와합산2537은PASS이며역사1201snapshot근거로보존한다.

- 선택RED77664 exit101에서actual unselected node.toggled=None 대SomeFalse를재현했다. root가row response에실제selected의ToggledTrue/False를추가해Windowsprovider에전달한다. role/ID/label/32px/virtual128/선택policy/renderer외관/core/DB/Cargo변경없음. 원실패보존후green실행.

- 선택green53826 exit0/기본+CPU각2회귀. root 원guard/currentmap/4actualtest와False→SpaceTrue상태검증을독립대조했다. 새전체검사와Windows TogglePattern 실제관찰을이어간다. old1201 UI고급옵션Light960100/Dark800200의숫자+상세2scopePASS는고정snapshot으로보존하며현재수정에소급하지않는다.

- 최신 선택 상태 수정의 전체 검사69051과 독립85268 모두 exit0: 43 harness, 1201 성공/0 실패/30 ignored. CPU 9,779,712B/SHA `24d6a65974cc0bcb39e136f14c8cb35170ee4fdd966e54aed6b1ec8b334dc132`. 문서27004는984개 PASS. 최신 native24312는 분리 가상 화면에서 실제 TogglePattern·상세 창·원행 복귀를 시험 중이며 아직 완료 근거가 아니다. 기존 conventional 경로 fixture의 PageDown 실제 이동 여부는 별도로 감사하며 Context 긴 경로 시험을 OS 스크롤 완료로 확대하지 않는다.

- 최신 native24312 exit1: 동일 원행 ID/focus와 TogglePattern available=true/On을 실제 확인했다. 다음 단계에서 TextPattern.GetVisibleRanges가 유효 범위 조건을 만족하지 못해 전체 상세 인수는 실패했다. 실패 로그·원자료를 보존하고 v3 시험 도구에 범위 수/실제 label geometry/캡처만 추가하며 인수 조건은 완화하지 않는다.

- root source 조사: vendored accesskit_windows/src/node.rs의 GetVisibleRanges는 현재 구현에서 무조건 null을 반환한다. 따라서 현재 provider에서 해당 API의 가시 범위 성공은 입증 불가능하다. 제품 텍스트 잘림 여부와 구분하여 캡처/DocumentRange 실제 bounds를 별도 검증해야 하며, 기존 실패를 PASS로 바꾸거나 API 지원 완료로 주장하지 않는다. vendor 변경은 이 UI 작업에 포함하지 않는다.

- 계측 native49976 exit1에서 range_count=0/label_offscreen=false를 확인했고 root가 실제 첫 Light800200 캡처를 열었다. 전체 경로와 닫기 버튼이 별도 영역에 보이며 일부 emoji glyph는 네모로 표시된다. 전체 경로가 한 화면에 들어오는 fixture이므로 실제 스크롤 이동 인수가 아니다. 후속 DocumentRange 시험은 전체 경로의 실제 bounds와 캡처를 확인하는 별도 범위이며 visible-ranges 지원 완료를 주장하지 않는다.

- DocumentRange 시험50782는 제품 실행 전 C# 시험 도구의 TextPatternRange namespace 누락으로 compile 실패했다. root의 시험 코드 오류이며 실패 로그를 보존했다. 새 v2는 DocumentRange의 실제 반환형을 추론하는 배열로 수정하여 동일 인수를 재실행한다.

- DocumentRange v2의 실제 실행47698은 닫기라는 이름의 Windows 창 버튼과 modal 버튼을 전역 조회해 중복 실패했다. 시험 도구의 범위 문제이며 원 실패를 보존한다. 새 v3는 원 PID의 실제 제목 아래에 있는 활성 modal-body 닫기만 유일하게 선택한다. 텍스트 전체 bounds·원행 ID·키보드·소유 프로세스 조건은 유지한다.

- DocumentRange v3 native4577 exit0, Light→Dark→restartDark의 3 phase/2 close route를 실제 실행했다. root는 restartDark 상세와 원행 복귀 PNG를 직접 확인했으며 path/close 분리와 선택 focus 표시를 확인했다. 독립39188은 WSL에서 소유 media 디렉터리 열거가 EIO로 실패해 완료되지 않았다(재읽기에서도 EIO). 원 native before/after128 manifest와 source는 보존하며 전체 독립 PASS를 주장하지 않는다. 별도 Windows 읽기 검증이 필요하다.

- root가 /tmp long-path 후보의 owned Win32 create-only/ACL/noReparse/명시128file·8directory/endpoint actual readback/실제 PageDown y이동 조건을 검토했다. TextPatternRange namespace 누락을 실행 전에 발견해 원 후보를 보존하고 v2에 System.Windows.Automation.Text import를 추가했다. 최신 안내 배치 whole1251 검증 후 새 source로 실행 예정이며 native 성공으로 기록하지 않는다.

- Windows 읽기 재조회65079 및 역사적 독립19649 exit0: 원128 namespace/nativeID/3byte SHA/원parent/DB integrity128·SHA/sentinel 보존을 재확인했다. 원 DocumentRange native의 historical scope를 현재 source와 구분해 검토했다. WSL EIO 실패는 보존하며 current source GUI 인수로 소급하지 않는다. 최신 안내/emoji source의 실제 긴경로 시험을 별도로 시작한다.

- 긴경로77142 exit1: 소유8directory/128files 실제scan 및 원행 Space선택을 진행했지만 modal 경로의 actual Name이 원 Win32 경로와 일치하는 조회 단계에서 실패했다. 원 실패를 보존하고 v3에 실제 Text 노드 이름과 캡처만 추가한다. prefix 추정/Name 주입/인수 완화는 하지 않는다.

- 계측80096 exit1: actual path는 앱에 전달한 원 short-root 철자 D:\project를 유지한955문자 경로이며, Win32 canonical fixture는 D:\Project extended 경로다. root가 첫 실제 overflow PNG를 확인했다. v4는 원 environment media 아래 exact leaf 이름의 유일한 actual Text를 찾고, 그 표시 경로를 native 재개방해 원 parent/leaf ID·SHA와 일치해야 인수한다. casefold/추정 prefix 치환/텍스트 주입 없이 실제 경로 소유권을 검증한다.

- 최신 long73792 exit1에서 actual conventional955문자 경로를 CreateFileW에 extended transport 없이 보내 Win32 PATH_NOT_FOUND(3)가 발생했다. 시험 도구 오류이며 제품 scan은 이미 같은 파일을 읽었다. v5는 원 owned media 아래 absolute/noADS/norelative 경로에만 Win32 extended transport를 적용하고 표시 문자열을 보존한다. native 원 parent/leaf ID·SHA 대조와 스크롤 조건은 그대로다.

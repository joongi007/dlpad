# S00: 앱·설치 화면 UI/UX 전수 점검과 공통 컴포넌트 통일

- 상태: in-progress
- 분류: UX
- 우선순위: P0
- 요구사항: R01, R02, R05, R06, R07, R08, R11, R13
- 담당/수정 범위: root — ui_tokens/ui_theme/ui_components, accessible.rs, 두main module wiring, catalog/공유문서/통합/native검증
- 선행 작업: 사용자 최신 설치본 디자인 거절, 기존 기능·포커스 국소 검증

## 사용자 결과

앱과 설치 화면의 버튼·입력·선택·hover/pressed/focus/disabled/error, 글꼴과 행높이·간격·본문/스크롤 경계를 동일한 디자인 규칙으로 제공한다. 작은 기능 성공을 전체 디자인 완성으로 보고하지 않는다.

## 계약과 범위

GTMUX 실제 tokens/Button와 PRODUCT/DESIGN의 compact/명료/차분함을 적용한다. Rust네이티브/기존worker/소유권·취소·journal/API/schema/lock을 유지한다. 새pure ui_tokens 계약은 md32/min64/padding12,sm24/min44/padding8,radius6,body/button13,small12,heading16,title20,section24/controlgap8/labelgap6이다. 앱은egui, 설치는기존Win32control의 접근성·IME·키보드 semantics를 유지하고 동일role/state/치수를 적용한다. root공통버튼API는 secondary(ui,label:implIntoWidgetText,enabled), secondary_sized(ui,label,enabled,Vec2), ghost(ui,label,enabled), danger(ui,label,enabled)→Response이며기존primary/choice/select/checkbox/TextInput API는유지한다. Response/원ID/handler/labelled_by/focus정책을보존한다.

## 병렬 담당

installer_batch는 installer_main.rs/installer_catalog.rs만독점구현하고 공유token사용·UIAIDs/worker/설치정책을보존한다. app-audit 담당은 accessible_options/metrics/settings/tools/tool_preferences/library/history/diagnostics/plan/preview,media_timeline/media_workspace/local_export_ui의 화면내공통조작·폰트·행/간격을독점구현한다. root는이파일들을동시에수정하지않고공통프리미티브및accessible.rs만수정한다. performance담당은 /tmpreadonly 진단후보만작성하며루트source/native/Cargo/child를변경·실행하지않는다. Gitinvalid를보존하므로분리프로젝트checkout재구성은하지않는다.

## 인수 조건

전수inventory에화면/조작/state/문제/수정/actual검증증거를기록한다. Light/Dark/System,800/960폭,앱100/150/200%,hover/pressed/focus/selection/disabled/error/loading,한글·긴값·빈/실패·스크롤을실제가상화면에서확인한다. 단위테스트가픽셀/physicalIME/reader인수를대체하지않는다. installer일반/긴path5payload/journal/ID보존도재검증한다. 변경동작·파일·실제명령·미검증·남은결정을기록하고root가원로그/source/pins/대표PNG를독립대조한다.

## 검증과 handoff

현재최신v3ZIP/CPU776d17c…는기능국소검증본이며사용자가디자인을거절했다. full UI/UX인수미완료를유지하고new수정본은별도source/whole/GUI/설치출력에연결한다. CI예비export/hosted/publish는실행하지않고개발잔여는독립병렬로준비한다. Idle최신18창측정은CPU예산실패이며원FAIL을보존하고진단담당에서원시계측을분리한다.


공유계약 보완: root가Win32_Graphics_Dwm feature를기존app windows-sys0.61 dependency에추가한다. 새crate/version/lock변경은없고설치창의supported per-window darkframe API에만사용한다. 시스템설정변경이나OS-wide테마전환이아니다. 정확binary크기/빌드비용은actualwhole후기록하며Win11전용attribute지원은[Microsoft공식문서](https://learn.microsoft.com/windows/win32/api/dwmapi/ne-dwmapi-dwmwindowattribute)에연결한다. 지원실패에설치/worker정책을변경하지않으며프레임Dark완료를실측없이주장하지않는다.

secondary/primary/ghost/danger는nativeeguiButton의입력/Role/ID를유지하고mouse-down과keyboardfocusfill을구분한다. focus는clip안의얇은stroke로,disabled에는hover반응없이중립surface를사용한다. choice는선택시fontweight를바꾸지않고worker잠금중에도선택surface를보존한다. checkbox와combo는같은enabledcursor/focusedge 규칙을사용하며fullglyph/OSprovider실제검증은남는다.

정정: egui RichText.strong은 이 경로에서 글꼴 굵기/폭 변경이 아닌 색상 강조이며 명시적 color가 우선한다. 불필요 강조 제거를 폭 변화 해결로 주장하지 않는다. 실제 폰트 불일치는 설치의 최소16px과 앱 body13, timeline11px과 공통small12px 등이다.

통합 진행 기록: 첫 whole는 Win32 import/attribute 타입 오류로 clippy 실패, v2는 공통 색상 연결의 dead_code로 실패, v3는 새 installer 테스트의 tuple struct 초기화/Clone 오류로 실패했다. 모두 원 로그와 자연 Job 종료를 보존하고 실제 수정 뒤 fresh whole-v4를 실행 중이다. 계획/파일별 rustfmt를 통합 성공으로 집계하지 않는다. 문서 검사1002개 PASS는 추가 문서 갱신 전 snapshot의 결과다.

실제 회귀: whole-v5의 200% 진단 보고서 Copy glyph가 body clip을 벗어났다. root가 형식 선택과 Copy를 한 wrapped toolbar로 통합했고 동일 strict glyph 검사로 whole-v6 재검증 중이다. worker 잠금의 selected surface 보존은 이전 transparent 기대값을 대체하며 pointer 비활성·선택·ID/geometry 회귀는 유지한다. theme미초기화 fixture도 실제 startup configure_context를 적용했다. 현재 test 원로그는43harness 1251/0/30이지만 release terminal까지 전체 성공으로 승격하지 않는다.

실제 Light100 초기 PNG를 root가 직접 검토해 navigation/footer/actionbar의 좌우16px와 본문24px 불일치를 발견했다. 기존 가상 시험과 패키지 작업이 원source pin을 마칠 때까지 변경하지 않고, 다음 source batch에서 세 bar의 수평 margin을 PAGE_MARGIN24로 통일한 뒤 새 strict 검사/캡처를 실행한다. current whole-v6(1251/0/30)은 이 결함 수정 전 snapshot이다.

후속 실제 결함 수정: source고정jobs 종료뒤 root가현재 baseSHA를대조하고 installer-only 후보를채택했다. nativeEDIT WS_BORDER/customNC 중복을제거하고유일한border/inset을사용한다. Combo는표준OWNERDRAWFIXED/HASSTRINGS·DRAW/MEASURE 및nativeheight4회bounded보정으로provider/문자/키보드를유지한다. native약속/픽셀/높이회귀는새 whole/가상설치에따라판정하며적용만으로해결완료표시하지않는다.

현재 통합 whole-v8: fmt/clippy/test/release 및 root 독립 대조 PASS, 1253/0/30. CPU c69a1e4c… 9,824,256B, installer b1fc12ef… 4,350,464B, CLI bb5976d8… 978,432B. Cargo.lock/version 변경 없음. 앱 실제 Light100/Dark200 다섯 화면 캡처를 검토했으나 기본 Windows 메시지 pointer pressed 픽셀 시험은 여전히 FAIL이다. synthetic egui fixture의 실제 렌더 상태 검증과 OS 입력 검증을 분리한다. 새 설치 패키지 v6는 최신 whole-v8 source 고정으로 생성/검토 중이다.

후속 병렬 담당(현재 모두 프로젝트 read-only): installer_batch는 /tmp installer popup/577UTF16 helper preflight; advanced_units는 /tmp synthetic pointer 9state helper와 feature build current proof preflight; windows_auto는 /tmp 현재 UI 미통합 후속 inventory. root는 패키지/실제 가상 설치/원 로그·PNG·소유권 통합 검토와 공유 source만 담당한다. repaint generation 후보는 /tmp에 있으며 아직 채택하지 않았다.

최신 v6 package: 16,664,245B SHA ae19c1b9e7bf7fcf5a6839638a95a246b554d8be9269b4399d157d423b462b78. graph258/86/258·archive/5payload/binding independent review PASS. popup수집기는C#compile성공 및18조합×3scroll54capture뒤 데스크톱이름 확인에서FAIL(Win32 0); 설치동작까지 성공한 것으로집계하지않는다. root가Light/Dark960100및Dark640200대표PNG를확인했고54stateJSON의visible expected-role-height mismatch는0이었다(직전v5Combo불일치54개). 원FAIL/ownedcleanup을보존하고가상popup bound proof를진단한다. 긴경로설치및독립canonicalID root검토는별도PASS.

실제 popup-v4: 8상태에서 지원되는 SelectionItem.Select와caption/원index/RuntimeID 복원, 일반5파일설치·verify·자연close0/Job0 PASS. root가8popup PNG 전부를직접보았고원로그/소스/pins 및현재27native canonicalID를별도로대조했다. v1/v2/v3수집FAIL은보존한다: foreign GetThreadDesktop가NULL/error5, ComboLBox부모가정상desktopwindow인것을관측했다. exactcreated/currentdesktop·nativeComboBoxInfo mapping·원PID/thread/creation·64bounded HWND namespace대조를유지해실행했다.

Native dropdown29logical/closed32는선택field가listitem보다작지않아야하는[Windows 계약](https://learn.microsoft.com/en-us/windows/win32/controls/cb-setitemheight)에맞춘chrome차이다. /tmp32row실험후보는아직채택하지않으며현재검증된동작을유지한다. 전체UI높이완전동일완료로표현하지않고이유를기록한다. root다음sourcebatch에서는checked checkbox에도PRIMARY_HOVER/PRIMARY_PRESSED를사용하고disabled checked는secondarysurface로선택을보존했다. BM_GETCHECK/provider/native입력과중앙font/worker/설치정책은유지하며실제새native state검증은후속이다.

다음 source batch첫whole은신규옵션회귀2개 field_reassign_with_default lint로clippyFAIL했다. root가테스트struct초기화를수정했고새whole-v2를실행한다. 원FAIL/raw/privateJob0를보존하며경고억제나인수조건완화는하지않는다. package-v7 helper는/tmp별도경로에만준비하며whole성공전생성하지않는다.

## 2026-10-13 현재 통합과 병렬 담당

root가 파일 행 공통 choice, 네트워크 입력 오류 공개, 첫 관측 안내, default-OFF worker 계측을7파일에통합했다. Windows MSVC whole fmt/clippy/test/release **1278/0/30**,43 harness/655 source/원로그·artifact·lock/GNU원본의독립대조PASS. CPU9836032B로직전F42보다3072B증가, installer4350464B/CLI978432B 동일크기이나 installer SHA는새값이다. 새 dependency/version/Cargo.lock변경없음. [최신 whole](../validation/evidence/windows-msvc-ui-feedback-library-worker-whole-20261013/root-review.json).

병렬담당은 windows_auto=/tmp 새 native 입력/오류PNGcollector, installer_batch=/tmp v8 package/설치검수helper, advanced_units=/tmp52property진단tracecollector로분리한다. 제품파일/source/Cargo/schema는root만통합하며native/Cargo실행도root가소유권·sourcefreeze·비용측정간섭을조정한다. 후보보고를실제성공으로쓰지않는다. 이전v7의긴경로577/598UTF16/5파일/독립27ID검증과세checkbox200mssettledblue는rootactualPASS이지만새v8산출물의GUI성공으로소급하지않는다.

현재8ab7 GUI nativecompile/layout2operation·63PNG·Light→Dark저장후재시작/원소스·pins·3SQLite immutable-read integrity·자연0/Job0/private보존을root독립확인했다. collector는기대파일명gui/result.json과실제settings-result.json불일치로FAIL이며원FAIL을보존하고native성공scope를별도검토했다. 팀이실제63PNG를view_image로전부확인하고해시/경로63unique·누락0을대조했다. root는대표화면과별도9상태/네트워크오류PNG를독립직접확인한다. 200%설정의footer줄바꿈에따른본문높이/배율상자partial·테마label문맥은후속확인항목이며keyboardfocus값false를geometry성공으로대체하지않는다. [native별도검토](../validation/evidence/current-ui-feedback-library-all-screens-20261013/root-review.json). 전체UI/물리/reader인수는아니다.

v8 ZIP16676021B/SHA bd2a9c7866217f38f5bd5e1eef807bfe130779a6d7b2254abba3c138c9a50f72 생성·streamedarchive/currentwhole/graph258notice검토PASS. 최신v8의3checkboxsettledblue/8popup선택복원/5파일설치·verify·원source7/journal보존 및독립27nativeID원검토PASS. 이전54matrix반복은없으며v7의고정source범위로유지한다. [설치검토](../validation/evidence/windows-msvc-ui-cross-surface-v8-20261013-popup-settled/root-review.json), [독립27ID](../validation/evidence/windows-msvc-ui-cross-surface-v8-20261013-popup-settled-independent-ids/root-review.json).

### 최신 v9 실제 설치·독립 검토

v9 ZIP16,676,533B/630ec9db6b2174ad0f5394658455bf954651a3655b6ee6e075b209859724dd79. f15 CPU9,836,544B, whole-v2 actual1284/0/30/source655. graph258/86/258·선택notice누락0·bounded7archive/5payload/3binary/PE/proof/lock 대조PASS이며 whole-lock superset누락43·서명/공지완결/정식배포 미완료이다. 실제일반 설치5파일/원7source/journal/8popup/체크206207208settled 표시/root PNG검토/독립27ID PASS. 실제긴설치577target/598exe·원20nativeID/6parent/5payload/journal읽기보존/MTA/naturalexit0/Job0/입력desktop유지·독립27canonical 대조PASS.

[일반 설치](../validation/evidence/windows-msvc-ui-cross-surface-v9-20261013-popup-settled/root-review.json), [긴 설치](../validation/evidence/windows-msvc-ui-cross-surface-v9-20261013-long-install/root-review.json), [긴 경로 독립 ID](../validation/evidence/windows-msvc-ui-cross-surface-v9-20261013-long-install-independent-ids/root-review.json). 주 담당이 long installed/wrapped-selection/verified 및6체크·popup PNG를 직접보았다. 읽기전용 UIA 지원 범위이며 clipboard/물리IME/reader/OSDPI/전체GUI·crashcommit·역사소유권·삭제권한 인수가아니다. 첫root long 감사helper는과거빈초기위치 계약/schema 기대에실패했고원script를남겼다. 실제최신default hint를소유target으로덮어쓴다는계약과int nativecount20/6을읽고후속정확scope검토가통과했다. 현재v9 label3 Dark200 GUI는본문크기실패를보존하고별도compacterror후속통합중이다.

### compact-error v10 인수

whole1284/0/30/root raw/source655/artifact/lock 대조PASS, CPU0093b3b7…9,836,544B/installerbfe2dfa6…4,350,464B/CLIbb5976d8…978,432B. v10 ZIP16,676,533B/8b93e3470780f43a87b10e90e415609be1a155f58517f828f7d049a6f42bff98. 실제일반5파일/8popup/3settled checkbox와독립27ID, 실제긴577/598UTF16/원20objects·6parents 및독립27ID·journal읽기 보존·MTA/naturalapp0Job0 인수PASS. Root는원raw/current655/pins독립검토 및9popup/checkbox/long대표 PNG를직접확인했다. [일반](../validation/evidence/windows-msvc-ui-cross-surface-v10-20261013-popup-settled/root-review.json), [긴 설치](../validation/evidence/windows-msvc-ui-cross-surface-v10-20261013-long-install/root-review.json), [독립 긴 ID](../validation/evidence/windows-msvc-ui-cross-surface-v10-20261013-long-install-independent-ids/root-review.json). 설치 후 실행은후속candidate로아직v10에포함되지않으며 fullGUI/physical/cleanOS/upgrade·rollback/crashcommit/배포 미완료이다.


## 현재 v13 전수 화면 갱신

최종 source656/1,297·0·30 전체 검사와 원로그·artifact 대조를 완료했다. root는 /tmp/dlpad-completion-copy-app63-collector-20261013으로 current-completion-copy-app63-20261013의 실제63화면 수집을 시작했다. Windows 기본CPU cd8fe0fe…와 exact 전체 proof를 사용한다. 원v8 observer/63목록은 동일하며 잘못된 top-level receipt 파일명만 settings-result.json으로 바로잡았다. 기존 원FAIL은 보존했다. source·원3artifact/GNU·lock·readonly3DB·private MTA/Job0·PNG상한 guard를 유지한다. 실제 결과 및63픽셀 검수는 진행 중이며 전체 물리 UI 인수가 아니다. 설치 완료 단계 인수는 S00-installer-completion-copy-20261013 담당이다.

## v13 최신 전수 검수와 병렬 후속

최종 code/source656 whole fmt·strict clippy·test·release1,297/0/30, ZIP16,725,685B와3실행 파일을 독립 대조했다. 공통 선택 Light의 실제 paint 대비 회귀는 RED를 보존한 뒤 수정본에서 통과했으며 설치 완료 안내·Run 수명과 변조 거부를 실제 재검증했다. 앱63개 PNG는 전체 경로·SHA를 대조하고 팀이 직접 보았다. 현재 normal/long 각각27 native ID, 설치 완료 Light100/Dark200/경로 편집/재검증/실패 화면과 actual 설치 앱·긴 journal 화면도 root가 직접 확인했다. 원compile/GDI·과거 pointer FAIL은 유지한다. [63개 화면](../validation/evidence/current-completion-copy-app63-20261013/root-team-visual-review.json), [정상 설치](../validation/evidence/windows-msvc-ui-cross-surface-v13-20261013-completion-happy-v3/root-review.json), [실패 거부](../validation/evidence/windows-msvc-ui-cross-surface-v13-20261013-completion-negative-v3/root-review.json).

현재 병렬 담당은 root=실제 실행·공유 source·독립 통합, advanced_units=/tmp 성능 원자료/DB 감사, installer_batch=/tmp 최신 checkbox/popup collector와 설치 문서 후보, windows_auto=/tmp 확대 메뉴/설정 코드 경로·warmup 진단 감사다. 프로젝트 파일과 Cargo는 root만 수정하며 실제 성능은 다른 native 실행과 겹치지 않았다.

확대 Settings 일부 갤러리에서 메뉴 두 개만 보인 발견 사항은 남긴다. fresh Settings-only 및 원 Dark 페이지/창크기/URL warmup을 추가한 좁은 재현에서는 root가 본150/200 대표 화면에 다섯 메뉴가 나타나 아직 원인을 확정하지 않았다. 200%에서 앞선 unfocused Theme 라벨의 scroll 밖 위치는 active Scale 전체 조작의 가림과 다르다. PrintWindow 및 관측 자체의 영향은 미확인이고 IsOffscreen 공급자 미지원은 unknown으로 기록한다. 이번 재현을 원 갤러리 결함 해결로 표현하지 않는다. 물리 pointer/IME/reader/OS DPI·실제 사용자 평가는 별도 인수다.

후속 관찰 정정: 위 ‘확대 메뉴 두 개’ 메모는 root와 원검수 담당이 정확한 원 두 이미지를 다시 직접 열어 다섯 메뉴를 확인했으므로 철회한다. 원보고·이미지는 보존하며 제품 메뉴 결함 잔여로 집계하지 않는다. 앞선 Theme 라벨의 scroll 밖 위치와 활성 Scale 전체 공개를 구분한다. 별도 초기 진단의 footer 두 줄/배율 하단 일시 가림은 [후속 작업](S00-scale-viewport-reflow-20261013.md)에서 추적한다. [정정 증거](../validation/evidence/current-completion-copy-app63-20261013/root-visual-correction-review.json).


v14 통합: whole1,305/0/30와 source656/3PE·63개 팀직접열람·archive·변조거부·27ID를 독립 대조했다. 지연footer 배율 공개는 별도 scoped5회귀로 수정했다. 설치 내부native0/Job0와 외부143 미확인 종료를 분리 보존한다. 전체 물리UI/UX 인수와 이 넓은 작업은 계속 미완료다. [배율 작업](S00-scale-viewport-reflow-20261013.md).


v15 통합: 공통Browse·filledcommandfocus의 실제RED/GREEN과whole1,307/0/33·별도GDI3/0/0, current63팀직접열람·archive·directinstall/run0·tamperrefusal·27ID를 검증했다. 전체UI 인수는 계속미완료이고 Dark200 하단명령scroll 실제검수는 별도진행한다. History/Tools rowaction stableidentity, activeRemove 사유와keyboard도움말은 source-only후속이며 실제오작동/PASS를단정하지않는다. [포커스](S00-installer-filled-command-focus-20261013.md), [Browse](S00-folder-browse-command-consistency-20261013.md), [200%하단](S00-installer-200-action-scroll-20261013.md).


v16 배포후보1,309/0/33/source656은원3검사+같은소스release복구의네actual명령으로독립검토했다. 원143/graphpreflight/패키징wrapper143을보존하고한번의연속whole로표현하지않는다. current63팀직접검수·ZIP16,726,709B·directnormal/negativeouter0·독립27ID·Dark200하단5명령actualscroll/root+팀pixels8를검증했다. 이어source-onlyHistory/Tools위험을실제ContextRED로확인하여수정했다. 최종기록8/tool17회귀PASS,새whole는진행중이다. keyboardhover-help gap은actualRED지만cross-widget arbitration검증전에제품helper채택을보류하고정확testedsource를보존했다. 전체물리UI/P0인수는미완료다.


v18 실제 네명령1327/0/33/source657·43harness, 새 설치ZIP16,741,557B 및 정상/변조/private0/현재27ID·앱63/설치8 직접 검수를 통합했다. 도움말/진단 입력/SQL binding scoped 인수는 완료했지만 full UI/P0는 미완료다. 새 distinct-commit Settings actualFAIL을 보존하고 최소 vendor 및 기존 회귀 강화 작업을 병렬 준비·시험 중이다. 제품vendor의 채택은 아직 하지 않았다.

# 미디어 작업면과 상호작용 계약 제안

범위: S00-media-workspace-interaction. 실제 gtmux source와 보관 workspace-dark/appearance-settings PNG, 직접 본 dlpad close-safe library800/settings800/150settings/200download 및960source를 비교했다. 이 문서는 구현 승인이나 UI 인수 완료가 아니라 root용 설계안이다. 프로젝트/native/Cargo 실행 없음.

## 원인 우선순위

1. **작업 대상이 화면에 없다.** 현재 첫 화면은 URL·저장 위치·품질·여러 설명과 옵션이다. 사용자는 무엇을 저장하는지, 영상 길이/내용/선택 구간을 확인하기 전에 다운로드 시작을 누른다. execution-plan-preview는 argv 설명이며 영상 미리보기와 다르다. 색상/정렬 교정만으로 media workspace가 되지 않는다.
2. **행동과 상태의 의미가 분산됐다.** 다운로드 페이지의 준비 전 시작/재개/이전 작업 다시 확인, tools페이지의 설치, library의 적용/새로고침을 사용자가 연결해야 한다. prerequisites는 해당 행동 근처에서 다음 한 행동으로 이어져야 한다.
3. **공간이 업무보다 form을 따른다.** 빈 중앙공간, 각label아래세로설명, 계속노출되는보조옵션이 시선을 소비한다. gtmux는선택된객체를작업면에두고toolbar/context/inspector로제어를분리한다. dlpad의경우객체는video이며그주변range/저장설정/결과가맞다.
4. **interaction states가전역style에묶였다.** ui_theme의active는focus와pointerdown공유, explicitprimary.fill은hover/pressedfill차이를덮을수있다. selected와focus는별도개념으로유지해야한다. 실제상태PNG와코드gate가필요하다.
5. **스크롤 경계가낮은학습비용을제공하지못한다.**200%에서initial저장위치아래대부분숨는다. focusscroll만으로는마우스사용자에게아래내용존재를알리지못한다. 작업면/inspector별스크롤과고정주동작의규칙이필요하다.

## gtmux에서 실제 확인한 구조

참조: styles/tokens.css/global.css; ui/Button.svelte/Input.svelte/IconButton.svelte/Tooltip.svelte; chrome/Titlebar.svelte/SettingsOverlay.svelte; toolbar/Toolbar2.svelte; sidebar/FileTreeView.svelte. 웹코드/localhost를앱에옮기는제안은아니다.

- chrome: titlebar44/toolbar56,toolbar좌context·중앙36glyph그룹·우측history; padding12,gap6/그룹2,divider22. OS titlebar를쓰는dlpad에는clientbrand추가하지않는다.
- Button: primary/secondary/ghost/danger각역할,24/32높이,radius6. hover는variant별명도/표면변경,disabled.5+notallowedcursor. IconButton24/32,hoverneutral표면,dangerhover만dangeraccent. 원CSS에모든pressed/busy/error가명시됐다고과장하면안된다. 해당부족상태는dlpad의명시계약으로설계한다.
- Input: label10/muted,gap6,input32,padding12,strongborder1,focusoutline2+offset1; errorredborder+inlineerror. placeholder는label대체아님. dlpad13본문은한국어접근성목적변경이며원11px를기계복제하지않는다.
- selected: toolbaractiveblue; tree aria-selected이며currentselection행과hover를분리. Settings는listselection+rowdivider,넓을때label설명왼쪽/현재control오른쪽,좁을때stack.
- scroll: SettingsOverlay outer overflowhidden/sidebar/content 각각overflow-yauto; global8pxthumb와hover변화. dlpad에custom웹scrollbar를복제하지말고nativeegui visibletrack/clip/scrollownership을분명히한다.
- keyboard: modal focusable목록 수집/초기focus/복귀지원코드. tree는selection·focus·keyboardstate를갖는다. dlpad에는nativeAccessKit role/value/selection 및실제Tab/arrow인수가필요하다. ARIA를Rust에직접옮기지않는다.
- loading/error: tree loadingDirs/request lifecycle과명시rowloading/error문구; 빈목록을무조건빈공간으로그리지않는다. screenshot의작업면은상태마커+본문+toolbar가함께있어현재대상을볼수있다.

## 최소 사용자 수직 흐름

**입력 → 준비 → 내용확인 → 구간선택 → 저장 → 결과확인**을한workspace에서완료한다. 다운로드목록/파일목록/설정은보조navigation이며현재draft를버리지않는다.

1. 상단sourcebar `[URL 또는 로컬영상][준비]`와명시drop영역. Enter는준비,아직저장아님. `Drop`은파일선택만;앱외부파일수정/삭제/실행권한없음. URL문법검사와실제콘텐츠확인성공을분리한다. Local은ffprobe로format/duration/size/streams확인;unsupported는원본보존+행동가능설명.
2. 준비중에는현재source+단계텍스트+취소를표시한다. 진행률모르면indeterminate에'확인중'만,추측percent금지. 입력변경/supersede는requestgeneration증가,원worker취소,늦은응답은반영0. 이전preview는새source와같은것처럼노출하지말고명시적으로이전내용/dim또는제거.
3. 성공은중앙preview(영상still/image,제목,길이,해상도,source종류)와아래timeline. 준비실패는같은영역에무엇/왜/다음행동(도구선택/재시도/다른source)표시. 원격preview는auth/cookie/영상다운로드권한과다르며외부전송/bytes/timebound계약이별도로확정돼야한다. 실제원격영상미다운로드라면'내용확인미완료'를유지한다.
4. timeline은한선택range와현재playhead를분리한다. default전체구간; start/endhandles를drag,선택band+두시간필드동기화. handlefocus시←→작은step/Shift큰step/HomeEnd경계,정확step은metadata기준계약으로정한다. start<end<=duration checked,invalid입력은마지막validrange를유지하며저장disabled+inline원인. 전체선택버튼으로복귀. 클릭playhead를range끝으로오해하지않게shape/label분리. 처음부터실시간playback/전체framecache를약속하지말고boundedstillseek가최소기능이다.
5. 오른쪽inspector는저장폴더+browse,품질/포맷요약,구간길이,예상용량(실제모르면미확인),필요고급설정.960logical에서preview+inspector,800/zoom에서inspector아래stack;preview가최소폭못지키면column전환. 상시wideform/navsidebar는없다.
6. 저장은validpreparedsource/range/destination/tool조건충족뒤에만primary한개. 실행중preview선택사항은snapshot으로작업에고정하고현재draft편집이진행작업조건을바꾸지않는다. 작업row는준비/수신/변환/검증상태·실제progress·중단을보인다. 실패는단계와재시도가능여부를표시,완료는원artifact+폴더열기/결과내용확인으로이어진다. 임의path발견/삭제권한/자동앱실행을추가하지않는다.

## 컴포넌트별 실행 계약

| 컴포넌트 | 상태와 feedback | 키보드/스크롤 |
|---|---|---|
| navigation/tab | hover neutral, selected persist accent tint+문자, pressed strongerneutral,focus2pxoutline. 현재draft보존 | Tab→Enter/Space;보조페이지후돌아오면draft/scroll복원 |
| primary prepare/save | hover명도변경,pressed별도fill,focusoutline,disabled이유를옆에, busy취소로전환 | 준비Enter는sourcebar에서만,저장전체앱Enter단축키없음 |
| text/path | label+focusborder,dirty/valid/error별문구, 선택은accenttint,placeholder선택적 | 기존cursor/selection보존;IME Enter조합중액션0; browse취소input보존 |
| preview | Empty교습,Loading bounded,Ready미디어정보,Error행동,Stale명시 | 현재focus빼앗지않음;프레임완료repaint는실제변화때만 |
| timeline handles | hover hitarea,presseddrag,selectedband,focusring,invalidinline | Slider/범위 role+시간값 AccessKit;수치입력과handle왕복동일 |
| inspector | subsection구분,고급접기,행별오류;긴도구법적문구상세 | 자체verticalscroll;focusrect전부보이게;사용자wheelhoverowner만 |
| job/result list | hover/selected분리,단계badge+텍스트,empty/loading/error/partial명시 | boundedvirtualrows;선택 유지;append가사용자scroll강제0 |
| status strip | 전체health/실제disk/network한줄wrap,필수taskerror는row로 | content을덮지않음;200%lineheight/clipping계약 |

spacing4/8/12/16/24,radius6,헤딩16/본문13,색token사용. 실제hover/pointercursor는InteractiveHand;readonly텍스트Text;disabledDefault. focus와pressed가eguiactive동일이어도컴포넌트response.has_focus/is_pointer_button_down_on으로renderer역할을분리한다. explicitfill버튼은response.hovered/down에따른fill을계산하거나공통helper에넣는다. defaultstyle만설정했다고interactive인수통과로쓰지않는다.

## UI/core/service seam

Core draft: source_revision, prepared_revision, selected_range, destination,validationstatus. 서비스는boundedrequestID+cancel+deadline+resultchannelcap1,metadata/stillframebytes/decodedpixels상한과maximuminflight1. `PreparedMedia`는검사결과snapshot이지원sourceimmutable/trusted/삭제authority아님. durationinvalid/seekerror/lateframe/wrongID를state전이로다룬다. root가공유schema와한worker합성방식을결정;기존engineownedprocess계약재사용. texture교체시oldrelease/framecache상한,UIthreadffprobe/ffmpeg/파일읽기0.

## 의미있는 인수

합성색변화영상으로prepare→특정seekstill→range수치/handles→실제구간artifact내용→reopen확인;uniform색은시작구간검증부족. URL실패/도구없음/취소/늦은응답/Drop비영상/durationunknown/oversizeframe원본preserve. 독립hover/pressed/focus/selected/disabledPNG같은control비교;실제pointer와keyboarddrag/scroll경계(부모이중scroll없음).800/960/150/200에서현재작업·주동작·취소도달. privateGUIfailedaction을PNG정상으로승격0. docs/whole/새budget/패키지인수는별도,physicalIME/reader/cleanOS/useracceptancefalse유지.

전체목표는화면에내용을보이게하고선택→저장결과를추적가능하게하는것이다. 색상·몇개margin교정이나새component개수로완료를판정하지않는다.

## 적용 상태 (2026-10-07)

미디어 작업면, 상태 버튼, bounded preview, 구간/숫자 필드 동기화, focus scroll, primary contrast를 구현했다. 전부 완성이라는 판정은 아니다. 긴 영상 정밀 탐색·200% footer·라이브러리 연결은 각각 docs/work-items/S00-media-precision-context.md, S00-media-context-footer.md, S02-library-media-flow.md에 제안으로 기록했다. 실제 사이트 추출·물리 pointer/OLE drop·IME·reader·OS DPI와 최종 사용자 UI/UX 인수는 미검증이다.

## 2026-10-08 정렬·밀도 재검토

[새 Light 캡처](../validation/evidence/windows-msvc-ui-polish-layout-final-20261008/native/first/initial-download-full-960.png), [Dark 캡처](../validation/evidence/windows-msvc-ui-polish-layout-final-20261008/native/first/selected-download-full-960.png)를 root와 독립 담당자가 직접 봤다. GTMUX의 기준선·일관된 제어 높이·약한 배경 구분을 따라16px외곽,32px제어,24px열간격,고정저장열과품질ComboBox로정리했다. 내부중복브랜드는돌려놓지않았다.

63개privatePNG는Light→Dark→restart와800/960·앱150/200배율의레이아웃/도달시험증거다. 물리mouse·OS DPI·IME인수는별도다. 일부캡처에서URL상단이보이지않는것은focus/scroll후관측이며초기상태전체소실로확정하지않는다. 향후스크롤복귀위치,준비안내중복,preview아래여백과설정밀도균형,좁은저장경로의전체위치확인수단을더다듬어야한다. 이개선을사용자최종디자인인수로표현하지않는다.

## 2026-10-08 실제 공개 영상·입력 정렬 감사

root와 독립 담당이 치지직 실제 준비/재생 및 설정·목록·도구 PNG를 직접 봤다. [준비 화면](../validation/evidence/windows-msvc-public-chzzk-refresh-20261008/native/gui/prepared.png)에는 빌려온 냥시기·04:36:40.000과 방송 준비중 영상이 표시된다. [유튜브 실패 화면](../validation/evidence/windows-msvc-public-youtube-20261008/native/gui/prepare-failed-or-deadline.png)은 actual muxed-only 형식 선택 결함을 드러냈다. 이 이미지는 수정 전 후보의 증거다.

입력 12px 여백/중앙 정렬과 Segoe 우선 글꼴로 경로 backslash와 기준선이 개선됐다. 실제 UIA/WM_CHAR readback과 단위 좌표 회귀를 물리 입력 완료로 표현하지 않는다. 개선 우선순위는 긴 영상의 정밀 시간 입력 가시성, 원인별 실패 안내, 좁은 경로의 전체 위치 확인, 도구 현재 사용 상태 위계다. 이후 실제 두 URL의 필드·품질·고급 옵션·준비·짧은 재생/정지 재시험은 아래 최종 범위에서 통과했다.

## 최종 통합 검증 (2026-10-08)

현재 source610의 Windows MSVC 전체 검사 `windows-msvc-public-ui-playback-clippy-fixed-20261008`에서 fmt/Clippy/test/release 모두 exit0, 41 harness 합계 **953 passed / 0 failed / 17 ignored**다. ignored 공개망 시험은 별도 명시 실행에서 실제 production engine의 0/2/4초 연속 batch 각각12frame을 검증해1passed/0failed였다. 이전 FAIL·RED와 수정 후 GREEN 로그를 보존했다. 의존성·Cargo.lock·공유 schema 변경 없음.

두 실제 공개 URL의 최종 private GUI `windows-msvc-public-youtube-playback-final-20261008`, `windows-msvc-public-chzzk-playback-final-20261008`에서 준비·짧은 무음 미리보기·재생/일시정지 Invoke·1/2초 시간 입력 readback·오디오 품질 선택·고급 펼침/접힘·preview 취소 동작·4페이지 탐색·자연 종료/Job0를 확인했다. root가 원 PNG를 직접 검토했고 원 collector REVIEW_REQUIRED와 별도로 root-review.json에 scoped 판정을 기록했다. YouTube 길이는01:35:22.000, 치지직은04:36:40.000이므로 짧은 영상만 시험한 것이 아니다. 전체 길이 재생·전체 다운로드·실제 cut·취소 terminal ACK·물리 pointer/IME를 검증한 것은 아니다.

최종 layout63PNG(800/960,Light/Dark/restart,앱150/200%), RawInput19PNG·실제 FFmpeg 합성 색 변화, 실제 Windows 폴더 선택 두 흐름을 재검증했다. 라이브러리 선택은 재실행 후 root/1행3B 복원과 DB integrity를 확인했고 다운로드 폴더 선택은 draft 변경 범위다. 비용은 합성1만 파일 목록 idle 3×60초/18창에서 최대 single-core CPU0.786260%·sampled whole-Job WS72,925,184B(69.55MiB), 각 DB1만행·원합성3만파일 보존이다. 재생/다운로드 비용·quiet host·물리 첫 화면 지연은 측정하지 않았다.

남은 인수: 최종 사용자 디자인 평가, 긴 영상 정밀 제어 가시성,200% 밀도/스크롤 맥락, canonical 경로 표시 정돈, library 미디어 연결/로컬 export, 물리 IME/reader/OS DPI, preview managed lease/cache, 서명·clean OS·정식 설치/updater·전체 법적 공지 및 전체P0. 현재 공개 두 URL의 scoped 성공을 모든 사이트 호환으로 일반화하지 않는다.

입력과 playback action의 논리 ID를 고정해 상태 안내 추가에도 focus/selection/AccessKit 동작을 유지한다. 품질 메뉴는 현재값 재선택과 접근성/키보드 선택에서도 명시적으로 닫힌다. 준비 완료 후 프레임 로딩 안내와 데이터/용량/도구 실패 안내를 구분했다. 네이티브 Rust 제약이 이 상호작용 결함을 남겨둘 이유가 아니며 실제 helper RED/GREEN과 private PNG로 교정했다.

## 네트워크 관측 입력 누락과 교정 (2026-10-08)

공통style를수정했어도직접TextEdit.singleline 호출까지공통helper의Center/margin12/ID규격이자동적용되는것은아니다. 실제네트워크칸의상단glyph를원PNG에서확인했고actualpanel inset RED를작성해공통helper로교정했다. 새native첫/재실행키편집/적용/배율시험과whole957/0/17을확인했다. docs/development/input-coverage-audit.md는각입력의실제수행근거와누락을표시하며,진단singleline3개를완료로처리하지않는다. 전체화면캡처수나공통helper단위회귀만으로각제품입력의전수인수를보고하지않는것이재발방지규칙이다.

## 2026-10-12 현재 구현과 인수 구분

URL 준비·미리보기, 파일 drop 입력, 구간 손잡이·시간 입력은 구현돼 있다. 남은 항목은 현재 source의 전체 화면과 상호작용 인수이며 이 기능들이 미구현이라는 뜻이 아니다. 과거 공개 영상, 합성 loopback, Context RawInput, private UIA, 물리 입력은 서로 다른 증거다. 기존149/Dark11/Library35를 모든 화면의 최신 시각 인수로 합산하지 않는다.

현재 코드 감사에서 자막NONE와 나머지 언어의 선택 컴포넌트, 자동 자막 Checkbox가 같은 화면의 공통 규칙과 달랐다. [자막 일관성 작업](../work-items/S00-subtitle-control-consistency-20261012.md)에서 공통 choice/checkbox로 통합했으며 Windows whole과 실제 화면 검증을 진행한다. 품질 ComboBox 포커스 공개 누락은 코드 감사에서 발견했으나 최신 실제 offscreen 실패를 재현하지 않았으므로 확정 기능 결함으로 보고하지 않는다.

일반 release 진단 화면과 별도 DEBUG timeout 대기 화면을 직접 보았다. [두 PNG의 SHA·시각 판정](../validation/evidence/ui-current-priority-pixel-review-20261012.json)은 저장 입력 focus·커서와 대기 안내에만 적용한다. 진단은 이미 본문 한 scroll 안에 있고 긴 multiline/readonly preview에는 별도 viewport가 필요하다. 전체 form을 다시 nested scroll로 감싸지 않는다. unused 빈 로그 editor의 공간을 줄이는 후보는 아직 미채택이며 focus 확장·축소가 화면을 흔드는지 먼저 검증해야 한다.

전체 UI/UX 인수는 진행 중이다. 다음 검증은 현재 source의 선택·호버·포커스·disabled와 좁은 고배율 레이아웃, 준비 영상의 track·시간 입력·footer, 실제 사용자 조작의 누락 범위다.

# 제품 입력 coverage 읽기 감사

이 문서의 원953/957 표는 과거 snapshot이다. 2026-10-11 교정과 실제 scoped 인수는 마지막 현재 증거 표를 우선한다. 과거 공개 walkthrough와 최신 CPU의 전수 입력 인수를 구분한다.

2026-10-08 읽기 전용 감사의 원 표는953후보기준이다. 아래 네트워크 교정 결과는 현재957후보에서 별도로 확인했다. crates/app/src 전체 TextEdit/helper callsite를검색했으며 감사 담당은 프로젝트/native를수정하지않았다. '실제'는privateGUI/ownedWM 입력 또는actualContext이며물리입력과분리한다. currentCPU기준과legacy/spike를분리한다.

| 원953 CPU 입력 | 실제구현 | 확인된입력증거 | 저장/적용인수·누락 |
|---|---|---|---|
| 다운로드URL | accessible.rs shared text_input_with_id | 공개URL UIA/WM_CHAR readback·자동prepare, shared caret/ID 회귀 | 실제사이트metadata와preview결과별도;물리pointer아님 |
| 저장폴더/파일목록폴더 | path_picker.rs shared identified | layout32px800/960·folderpicker결과·RawInput경로행, shared회귀 | 경로apply/nativefolder기존증거별도;새font직접클릭전체필드인수아님 |
| yt-dlp/FFmpeg/Deno 실행경로 | accessible.rs field shared identified | 관리Use/재실행복원·사용자custom보존, 공개toolpins | 외부경로전필드실제클릭/타이핑/저장동일coverage라고단정0 |
| 범위시작/끝 | accessible_options.rs field shared identified | firstcharID REDGREEN·공개1–2초입력/readback | 실제자르기/저장결과와별개;최신whole회귀PASS |
| 품질height/고급숫자옵션 | accessible_options.rs shared identified | 공개고급펼침/값입력·품질popup REDGREEN | 모든숫자필드각각클릭/검증/다운로드영향의원증거추적필요 |
| 설정한글시험 | accessible.rs shared identified | 기본Unicode readback/별도featureIMM, shareddirectedselectioncancel | 현재기본binaryphysicalIME0;feature증거를기본인수로소급0 |
| **네트워크interface 설정** | **accessible_metrics.rs408 원TextEdit.singleline, charlimit64/min32/hint, 기본margin·정렬·autoID** | invalid/empty/stop단위정책회귀. 현재layout은설정접기항목수집;접기만으로내부입력인수0 | **shared12px/center/stableID 적용누락**. 실제클릭/한글·영문/가로scroll/readback/Apply/잘못된값오류인수확인못함. apply_interface는runtime selection+changed만,Store/settings저장호출없음:재실행지속저장완료주장0 |
| 진단log index | accessible_diagnostics.rs276 원singleline char20 | 기존large-report/diagnostic행focus및action회귀 | shared32/center/12px/논리ID미적용;전필드클릭/타이핑coverage없음 |
| 진단edited log | 同305 원multiline | 기존report정제편집/대형preview경로 | multiline는32px단일행정책대상이아님. edit선택·caret/scroll별도 |
| 진단MPD첨부경로 | 同316 원singleline | 기존첨부/정제engine+UI일부증거 | sharedgeometry/논리ID미적용;현재font/nativeinput개별인수필요 |
| 진단export경로 | 同368 원singleline explicit diagnostic-export-path ID | S10JSON/Markdown실제export/readonly원DB+파일확인 | ID는안정적이지만sharedcenter/margin미적용. 실제export성공과입력caret시각인수분리 |
| 보고서preview | accessible.rs730/diagnostics429 원multiline readonly | S10대형236page/first-next-lasttext·export | 사용자편집입력아님,32px규격을그대로적용하면안됨 |

## 다른entrypoints

main.rs는iced text_input들(URL/root/tool/quality/range/diagnostic/IME/interface)로현재CPUsharedhelper대상이아니다. egui_spike.rs URL/IME원TextEdit도alternative실험entry다. installer_main은Win32EDIT101–104로별도nativefont/IME/browse/worker계약이다. 이들까지이번CPU953whole/63PNG가전입력인수했다고표현하지않는다. 테스트모듈TextEdit는제품callsite아님.

## 원 감사의 수정/검증 제안 (네트워크는 아래 완료)

네트워크input을sharedidentified로옮기고label/fixedlogicalID/charlimit64/enabledgate를보존해야한다. invaliderror문구삽입후ID/focus를보존하는반례회귀,Apply유효값·invalid원selectionpreserve·emptydisable·CloseWaitinert를원Context/실제privateGUI로검증한다. 저장기능을추가하지않는다면'이실행에서관측할interface선택'으로정확히표현한다. 디스크저장은새계약/실패경로없이지속성으로주장하지않는다.

진단singleline3개도같은geometry와논리ID를적용하되MAX_INPUT_BYTES/4등기존bounds·label·ancestorScrollAccessKit경로를보존하고실제export/첨부실패preserve를재검증해야한다. multiline와readonlypreview는독립규격으로남긴다.

원953 감사 결론: 당시 시험은sharedhelper핵심경로/개별사용흐름인수이며제품모든input전수인수가아니다. 당시 네트워크와진단singleline이확정누락이었다. 물리입력/reader/IME/DPI/useracceptance미확인은유지한다.

## 네트워크 교정 진행

S00-network-observation-input은실제show_settings RED(12px inset FAIL)→공통hint/ID helper교정→Windows2회귀PASS까지확인했다. actualnativeclick/키편집/적용/배율/새빌드전체검사는별도현재수행중이다. 앞선표의네트워크누락은953후보기준이며현재진행을모든입력인수완료로확대하지않는다.

네트워크 수정본 실제 Windows private GUI가첫/재실행에서클릭·Home/End/Backspace·Tab→적용·유효/무효/빈값readback/적용과150/200배율을통과했다. 원collector와root의scoped판정은 windows-msvc-network-input-native-20261008에있다. whole957/0/17/610source와원RED/GREEN을보존한다. 당시 진단singleline3개는S10-diagnostic-singleline-input-consistency.md후속이었다. 2026-10-11 실제 교정 결과는 아래와 같으며 물리 포인터 전수 클릭으로 확대하지 않는다.


## 2026-10-11 현재 교정과 실제 인수 범위

[S00 현재 입력 인수표 작업](../work-items/S00-current-input-coverage-20261011.md)은 원 raw 결과와 root-review를 대조한다. 최신 plan 시험의 CPU SHA256은 `b8434e702ab0282af5f406471bc4eb360d3f2cef429a3fa54eac4fd1e90c7d6f`다. S10 실제 GUI의 CPU는 앞선 `e241510c83fb35a506b52282b044afb32c71c5560d98d724774f7746486c403e`다. 각 증거의 source snapshot과 binary pin을 유지하며 앞선 GUI 결과를 b843 CPU에서 다시 실행한 것으로 쓰지 않는다.

| 입력/동작 | 현재 확인한 실제 scoped 증거 | 남은 범위 |
|---|---|---|
| 진단 log index·MPD 경로·export 경로 singleline | 공통 identified32px helper 교정 및 진단controls의 중복scroll 제거. [S10 raw](../validation/evidence/windows-msvc-s10-unclipped-final-ui-20261011/gui-single/result.json)는 960×640, edit scenario, three singleline native readback, cookieclear, scenario assertion, original Job0/private desktop를 확인한다. [root-review](../validation/evidence/windows-msvc-s10-unclipped-final-ui-20261011/root-review.json)는 세32px 실제PNG, report edit/privacy, failed DB attempt, sourcecookies/sentinel SHA보존을 PASS_SCOPED로 확인했다. | root 기록상 log/export는 exact-private-provider SetFocus, MPD는 already-focused-or-pointer-unconfirmed다. 물리pointer/IME/reader 및 S10fullsuite는 false다. actual 시각·readback 범위를 전수물리클릭으로 확대하지 않는다. |
| 고급 숫자3개 | b843 [raw](../validation/evidence/windows-msvc-option-plan-native-scroll-20261011/native/option-plan-result.json)의 six numeric receipts: 속도0→1048576, 재시도21→0, 조각동시성0→1. 각 invalid→valid·exactprivateprovider focus·32px/readback assertion 및 help labels를 [root-review](../validation/evidence/windows-msvc-option-plan-native-scroll-20261011/root-review.json)가 PASS_SCOPED로 검토했다. | physicalpointer와 actual option download effect는 false다. 3개 valid값 전체 범위/경계값을 모두 시험했다는 뜻이 아니다. |
| 고급 help | 같은 raw/help_labels_verified 및 root six_numeric_and_exact_help_argv_reviewed. 기본값/예·충돌·설치버전의 exact문구를 실제 UIA로 확인했다. | 모든 help 항목의 pointer/reader/배율·reflow 전수 인수는 별도다. |
| readonly 실행 계획 | 같은 raw의 plan_scroll_route=ScrollItemPattern.ScrollIntoView, bounded viewport geometry, readonly text preserved, plan_flags_verified_once. root actual_plan_png_glyphs_visible_reviewed=true. limit-rate1048576/retries0/concurrent-fragments1가 각각한번이며 기본retries중복없음을 실제planJSON으로 확인했다. | readonly는 focus할수없는 정상규격이다. 전체396px TextEdit를 bounded200px viewport에 모두노출시키는 계약이 아니다. native scroll후146px content intersection과 실제glyph PNG를 검토했다. 실제다운로드옵션효과·physicalpointer는 미검증이다. |
| custom yt-dlp/FFmpeg/Deno 경로 | [현재 private GUI root 검토](../validation/evidence/windows-msvc-current-custom-tool-input-third-20261011/root-review.json): 합성tool3개·한글긴absolute/relative/복원9readback32px, navigation유지,3relativeplan거절/복원, FFmpeg/Deno literalargv, 같은data 재실행 초기환경값 복귀. root가3개복원PNG glyph·raw natural0/Job0·SHA를 검토했다. | 실제 외부tool버전/다운로드효과·physicalpointer/IME·managedUse충돌은 미검증이다. manual값은 재실행에 저장되지 않으며 저장성공을 주장하지 않는다. 첫두helperFAIL을 보존했고 deadline/focus절차를 교정한 세 번째만 인수했다. |
| 진단 edited log multiline caret | 현재 whole1,015/0/28의 실제 private GUI에서 입력9건·Home/End/Delete/선택 교체·LINE-C 선택·ROW35 표시·invalid index 원본 보존·정제/반영과 두 DB를 root 검토했다. 실제 galley cursor/clip 및 exact ScrollIntoView 회귀 RED/GREEN도 통과했다. [인수](../work-items/S00-current-diagnostic-multiline-input-20261011.md). | collapsed provider caret geometry는 미지원이다. 물리IME/포인터·reader·모든 disabled/action 조합 전수 검증은 남는다. multiline/readonlypreview는 별도 규격이다. |

[plan outer collector](../validation/evidence/windows-msvc-option-plan-native-scroll-20261011/result.json)는 source_unchanged=true 및 original CPU/helper/config/source pins를 유지한 REVIEW_REQUIRED이고, scoped승인은 별도 root-review다. 두 시험 모두 사용자desktop fallback/clipboard를 사용하지 않았다. plan 시험은 synthetic fixture.invalid URL과 owned synthetic tool/data를 사용했다. synthetic tool이 preview metadata argv를 받을수있지만 외부network/account/download 구현은 없으며 actual_download_verified=false다.

과거 공개 YouTube/CHZZK stable-input·playback-final·popup-final 20261008은 시간입력1–2초/readback·Audio품질선택·고급펼침을 확인했다. 고급펼침만으로 숫자입력을 인수한 것은 아니다. 이름이 walkthrough-final인20261008 YouTube시험의 rawFAIL(expected12/actual1·forcedcleanup)은 성공으로 쓰지 않는다. 위 신규고급3숫자/help/plan proof가 해당 누락을 좁게 보완하며 과거CPU와현재b843의scope를 유지한다.

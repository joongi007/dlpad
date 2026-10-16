# S00: 고급 옵션 정보 위계와 화면 리듬

- 상태: in-progress
- 분류: UX
- 우선순위: P0
- 요구사항: R07, R08, R13
- 담당/수정 범위: root — crates/app/src/accessible_options.rs와 필요한 중앙 catalog, 이 기록 및 검증 근거; release_preparation — /tmp 후보만
- 선행 작업: 고급 입력 validity 전체1195 검사와 실제 first/restart 캡처

## 사용자 결과

고급 옵션에서 필요한 입력과 오류를 빠르게 찾고, 기본값·예시·설치 정보의 반복으로 화면을 불필요하게 길게 읽지 않는다.

## 계약과 범위

DESIGN의 입력 옆 오류와 단계적 공개, PRODUCT의 컴팩트하고 명료한 화면을 적용한다. root가 직접 본 dark first 재시도21, dark restart 속도0, light restart 동시성1 캡처에서 옵션마다 여러 설명 줄이 반복되는 현재 배치를 확인했다. 도움 내용을 삭제하거나 hover에만 숨기지 않는다. 입력 ID·32px 높이·core validation·plan·AccessKit invalid/description과 keyboard focus 계약을 유지한다. 제품 소스는 활성 native 시험이 끝날 때까지 동결한다. 공유 schema/Cargo 변경 없음.

## 인수 조건

기본값·예시와 입력 목적을 읽을 수 있고 추가 상세 설명은 키보드로 접근 가능하다. 오류는 해당 입력 바로 옆에 유지한다. 800/960 폭과 확대200%, Light/Dark에서 실제 입력·교정·스크롤·focus·restart를 확인하며 overlap/clip을 성공으로 인정하지 않는다. 유효/무효 입력의 기존 실제 plan/provider 회귀를 유지한다.

## 검증과 handoff

현재는 직접 본3개 캡처를 근거로 후보를 준비하는 단계다. 소스 구현·새 픽셀 검증·전체 UI/UX 인수는 미완료이며 기존 숫자 오류 시험의 성공을 레이아웃 개선 완료로 사용하지 않는다.

- root source 대조: accessible.rs1946은 width>=760에서 preview 유무와 무관하게 오른쪽400px 설정+왼쪽 preview 영역을 만든다. 직접 본 고급입력 캡처의 큰 빈 왼쪽 영역은 이 조건과 일치한다. preview 상태별 화면 구성 검토를 추가하며, hunk는 /tmp만 준비하고 active native 종료 전 제품source 변경하지 않는다. 사용자 미디어 대신 fixture의 empty/preparing/ready/failure 조건으로 검증한다.

- 후보 `/tmp/dlpad-advanced-option-density-20261012`를 root가 읽었다. 오류를 입력 바로 아래로 옮기고 요약·기본값·예시·충돌은 유지한다. 실제 동일한 설치 버전 안내만 한 번 표시하며 영향 설명은 키보드 접근 가능한 상세에 둔다. 별도 layout hunk는 준비된 미디어/busy/closing에서만 넓은2열을 유지하고 초기/오류는 stack으로 한다. parent에 의존하는 path ID·focus와 긴 full-width 입력의 검증이 필요하므로 아직 layout hunk는 채택하지 않았다. 실제 Context 회귀 후보를 추가 준비 중이며 Rust/Cargo 통합 완료를 주장하지 않는다.

- 양테마MPD native와독립검토 terminal후 root가 options 설명 hunk와 중앙catalog1문구만 통합했다. layout hunk는 아직 보류한다. 오류는 입력 직후/필수help는 항상/영향은 keyboard disclosure/동일version만 한번이며 shared ID/core/validation/Cargo는 변경하지 않았다. 실제 회귀와 새 전체 검증은 아직 미완료다.

- 통합 options/catalog rustfmt 통과. 기존 실제 폼의 invalid→valid→empty/ID/focus/plan 회귀를 새 evidence `windows-msvc-advanced-help-density-regression-20261012`에서77715로 실행 중이다. 회귀 실행 동안 제품source를 동결한다. 상세 열기/닫기와 화면 구조 추가 회귀는 /tmp에서 준비하며 아직 새 사용자 화면 인수는 미완료다.

- 실제 기존 폼 회귀77715 exit0/source unchanged. root 원로그 guard와 현재 source map을 대조했다. invalid→valid→empty/입력 ID·focus·32px·danger stroke·disabled·buildplan 회귀 통과 범위이며 펼침/접힘 및 새 native 픽셀은 아직 미검증이다.

- root가 실제 DownloadForm.show에서 고급 열기→필수 안내/공유ID→속도0 오류의 입력 직후 위치→상세3개 Expand/Collapse→disabled 입력 무반응→유효1048576/plan 보존을 검사하는 회귀를 통합했다. Light/Dark Context를 사용하며4000px 높이는 논리 회귀용일 뿐 실제 viewport/pixels 증거가 아니다. rustfmt 통과 후32050 실행 중.

- disclosure32050 compile 통과/test 실패(exit101): 펼침 애니메이션 직후 기준 frame에서 summary가 아직 표시되지 않았다. root가 첫 actualClick 뒤16frame을 기다리도록 고치고 표시·validation 조건을 유지해 재실행한다. 원 실패 로그 보존.

- v2 disclosure36594도 동일 basic summary 검사에서 exit101. 앞선 애니메이션 원인 설명은 가설이며 확인되지 않았다. 원v1/v2 로그를 보존하고 v3 실제 접근성 labels 계측36632로 제품 문구 누락과 테스트 관측 문제를 구분한다. 실패 상태를 통과로 기록하지 않는다.

- v3 계측에서 labels에는 버튼만 있었다. vendor/egui/src/response.rs884..885는 Role::Label의 텍스트를 value로 제공한다. 원 회귀가 label을 조회한 시험 오류를 확인했다. 필수 안내와 오류 Label의 exact value를 조회하도록 수정하고 버튼 label 조회/표시 위치/validation 조건은 유지한다. 제품 코드를 시험 통과용으로 변경하지 않았다. v1/v2/v3 실패 증거 보존.

- v4 disclosure1332 exit0. root 원로그/현재 source map/실제 default+CPU2회 테스트 성공을 독립 대조했다. actual Form 기본 안내·오류의 입력 인접 위치·상세3개 펼침/접힘·disabled·유효값/plan 보존 통과. 새 native 픽셀/전체 UI 인수는 미완료다.

- root가 준비된 미디어/busy/closing에서만 넓은2열을 쓰는 layout 조건과 실제 idle 경로 입력→local 미리보기 준비→오류 전환에서 path ID/draft/focus 보존 회귀를 통합했다. path ID 불변은 아직 확인 전이며 회귀 실패를 보존한 뒤 수정한다. 기존 texture/영상비율/footer/bodyclip 기준을 유지한다.

- layout38713 exit0는 실제 대상2개가 ignored여서 인수 증거가 아니다. root가 원 로그에서0실행/2ignored를 확인했고 이를 성공으로 세지 않는다. 프로젝트 소유 합성 red-blue.mkv와 고정FFmpeg를 넣고 --ignored로 실제 CPU preview/geometry 회귀를 별도 fresh evidence에서 실행한다.

- 실제 preview49695 exit101: idle→local prepared에서 저장 경로의 실제 input ID가 변경되는 회귀를 재현했다. path_picker.rs를 root 담당 범위에 추가하고 기존 library/default parent ID는 유지하면서 download만 명시적 고정ID를 전달하는 folder_field_with_id를 도입했다. 레이아웃 부모 변경이 동일 input의 편집 상태를 바꾸지 않게 한다. source/schema/Cargo 영향 없음. 원 실패 증거 보존 후 fresh green 재검증.

- green75842 exit101: ID/초안/논리focus 검사는 넘어갔지만 좁은폭의 focused folder가 고정footer 위에 완전히 보이지 않는 조건에서 실패했다. 기존 gained_focus만으로는 이미 focus인 입력의 위치 변경을 감지하지 못한다. root가 workspace 상태 전환 때만 focused path를 한 번 reveal하는 요청을 추가해 반복 wheel 강제복귀를 피하고93891 재검증 중이다. 원 두 실패를 보존하며 시각 인수는 미완료다.

- v2/v3 실제 회귀도 동일 footer visibility에서 실패했다. 회귀는 준비 전부터 경로focus를 유지한 채 준비 후 명시적 AccessKitFocus를 같은ID로 다시 보내는데, gained_focus는 같은ID에 false이므로 표시 요청을 놓친다. root는 실제 명시적 Focus action을 별도 읽어 has_focus인 입력을 reveal하며 일반 wheel에는 영향을 주지 않는다. 준비/busy/closing 상태는 각각 기록한다. 다음 fresh v4에서 기존 전체 geometry 조건을 유지해 검증한다.

- v4 실제 회귀32217은960/824의 focused path visibility를 통과한 뒤800150 preview deadline에서 실패했다. 자동 path reveal이 새 preview 표시와 경쟁해 영상이 본문 밖으로 이동한 원geometry를 확인했다. state-transition 자동reveal을 제거하고 준비 완료의 기존preview reveal을 우선 유지한다. inputID/draft/논리focus는 유지하며 명시적 Focus 요청 시에만 같은ID 입력을 표시한다. 이로써 state별 자동scroll 경쟁을 없애고 v5에서 원 texture/bodyclip/경로focus 조건 전체를 검증한다.

- v5 실제64288 exit0/1pass/0ignored. root raw guard/current source/합성 fixture·FFmpeg SHA를 독립 검증했다.960100/824100/800150/800200 actual previewtexture/영상비율/bodyclip/footer와 idle→ready→error 경로ID·초안·논리focus, 명시Focus 전체field 표시 회귀 통과. 실제 native 픽셀/physical 입력/전체 UI 인수는 아직 미완료다. 전체 검증을 이어간다.

- 전체69540와 독립48092 exit0:43harness/1197성공/0실패/30ignored, 원fmt/clippy/test/release로그/source651map/artifact 대조 통과. CPU9,768,448B/SHA `16ca21b97d08cdd0233642a95996535292727f67cc5dce465531aa40ee26f97f`. MPD문구 snapshot CPU보다6,144B 증가, 의존성/Cargo.lock 변경 없음. 실제 native 캡처 후보 준비 중이며 전체 UI/UX/새 설치본 인수 미완료.

- 새1197 whole/CPU를 원 v5 native producer에 공급해 Light9378/Dark를 각각 독립 private desktop·fresh evidence·합성root로 실행한다. 원12숫자 phase/24COMproperties/exactinline/focus/bodyclip/restart/readonly DB0job/natural 종료·원90worker/500host 조건을 유지한다. 상세 펼침과800200는 별도 후보이며 이번960100 시험으로 완료 처리하지 않는다. source freeze 유지.

- root가 새1197 source의 Light first retry invalid/concurrency corrected와 Dark retry corrected3PNG를 직접 확인했다. 빈 왼쪽 제거와 입력 직후 오류는 보이나 숫자input이 약900px로 길게 늘어나는 시각 문제를 확인했다. 실행중 원native/독립검토 종료 전 수정하지 않고 숫자 input만 최대360logical 제안 hunk를 준비한다. 설명/오류/sibling parent/ID/32px/core는 유지한다. 전체workflow/전체UI는 아직 미인수다.

- 새1197 native Light9378/Dark10266와 독립52099/54513 모두exit0. 각12숫자phase/24nativeCOMproperties·원fieldID/value/inline scope/body bounds/restart/0jobs/원fixture/versiononly/natural 확인. root가 first/restart 대표6PNG를 직접 확인한 범위에서 오류 아래표시·수정 후 해제·빈 왼쪽제거를 확인했다. 모든 physicalfocus/IME/모든픽셀 인수는 아니다. 모든 native/독립handle terminal후 numericinput만 최대360logical 제한을 root 통합해 지나치게 긴 입력폭을 줄였다. parent/ID/help/오류위치는 유지하며 field_response default폭은 기존대로다. 새 전체검사/native/800200 상세검증을 이어간다.

- 숫자폭v2 회귀48739 exit0. 실제 외곽 폭360상한과 invalid→valid→empty/ID/focus/plan/상세 펼침·접힘 회귀가 기본+CPU4회 통과했고 root 원guard/currentmap/실제test수를 대조했다. 새 whole을 fresh evidence `windows-msvc-advanced-numeric-width-whole-20261012`에서 실행하며 제품source 동결 유지.

- native 상한 기록 정정: root가 원 v5 observer를 직접 다시 읽어 UIA worker Join180000ms, cleanup Join5000ms를 확인했다. 앞서 원90worker라고 적은 문장은 잘못이며 해당 원 시험은180초/host500초였다. 결과·원소스·로그는 그대로 보존한다. 새 details 후보는 root가 요청한90초로 축소하며 원v5를 변경하거나 과거시험을90초 검증으로 소급하지 않는다.

- 숫자폭 whole13949 exit0. 독립1197/30 대조를 시작했다. 새 details 후보의 root 사전C#compile92791 exit0/source copy SHA 일치이며 actual GUI는 아직 미실행. producer9/independent9 순수helper검사도 root 직접 실행해 통과했다. 문서50476 PASS983이며 실제 확대/상세 인수는 남는다.

- 숫자폭 전체13949와 독립53522 exit0:1197/0/30; CPU9,768,960B/SHA `afa6acd3c3ceced28a051ca9d6251851bd55dc28f9e4c3be8af8cad53d0b2e13`, 이전layout snapshot보다512B 증가. 새details Light960100/Dark800200를 독립private desktop/fresh evidence에서 먼저 실행한다. 원12숫자phase/24COMproperties+3상세first/restart 상태/값/ID/focus/본문geometry/capture를 유지하고 worker90초/host500초로 제한한다. source동결 유지하며 아직 actual인수 미완료다.

- details Light18605/Dark48054 모두exit1: first 숫자6phase 후 상세에서 value/ID/focus combined predicate 실패. native 자연Job0 종료는 확인됐지만 전체workflow 실패를 보존한다. 아직 어느 항목이 달라졌는지 증거가 없으므로 새v2 producer에 펼침 직후 실제값/원·현재ID/field·header focus를 기록하는 계측만 추가해 구분한다. 제품수정 없이 새 evidence로 재현한다.

- details 계측v2 78701 exit1에서 actual값1048576/원·현재runtimeID 동일, fieldfocusfalse/headerfocustrue를 root 직접 읽어 확인했다. Toggle가 조작한header에focus를 주는 정상 동작을 원시험이 field자동focus 유지로 잘못 기대했다. 제품수정 없이 새v3 producer/reviewer에서 실제headerfocus와 명시적field복귀ACK를 각각 검증하도록 교정한다. 원FAIL/계측은 보존하며 단일관측을 전체workflow 인수로 확대하지 않는다.

- root가 실패한details 원run의 Light960100 재시도21/Dark800200 속도0 numeric PNG를 직접 확인했다. 입력폭 제한과32logical높이·입력 아래 전체 오류표시는 이2캡처의 좁은 시각 근거다. failed-run-numeric-pixel-review에 PNG SHA와 실제provider폭/높이를 기록했다. 이후detail 실패 때문에 전체workflow/restart/전체UI를 통과로 쓰지 않는다.

- root가 v3 producer13/독립reader10 순수회귀를 직접 실행해 통과했다. 실제 Toggle 뒤headerfocus/field unfocused와 값·ID 보존, 명시field.SetFocus 복귀는 별도 returned JSON/PNG로 요구한다. 원numeric12/COM24/3상세/privacy/natural/90worker500host 계약을 유지하며 Light960100/Dark800200 fresh v3를 실행한다. 제품 수정 없음/source동결 유지.

- v3 Light19376/Dark28252은 UIA worker90초 timeout으로 exit1. first numeric6phase 후 첫상세를실제로열었고 Light는field복귀도기록했지만 전체실패다. 원Fill이 각char마다150ms pump하여 긴합성tool경로에 인위대기를누적하는 source를root확인했다. v4는 glyph별35ms로줄이고 high/low UTF16pair를같은F24 envelope에넣는다. 원1024상한/최종exactreadback/40boundedpoll/PID소유/12numeric·24COM/3details/privacy/natural/90worker500host를그대로유지한다. 전체시간상한 증가없음/제품수정없음. 원v1/v2/v3보존.

- v4 Light93573/Dark74704도worker90초에서exit1. 더많은상세·복귀기록은생겼지만 전체실패를보존한다. 단순대기감소만으로원인을확정하지않고 v5는 초기화/숫자6단계/상세3개의 begin/end elapsed를 bounded 원checkpoint에추가한다. 같은90초/전체조건으로 Light960100만직렬실행하여 단계별비용과동시시험영향을구분한다. 제품source/검증조건/상한 변경없음.

- v5 단독 계측: 초기화25.0초, 숫자6단계약47.9초 후상세시작73.48초. Root가원Find의RawView256node순회와각node CurrentName/Role 반복조회source를확인했다. v6는같은RawView/256node·queue상한에서Each-node Name/ControlType만CacheRequest(Element,Full)로함께조회하고 선택target의CurrentName/Role은다시확인한다. 값/ID/focus/COM/geometry/exactinline/privacy/natural/deadline90초는유지한다. APIcache는후보실행으로검증하며 제품source 변경없음.

- v5 7265 terminal exit1/worker90초이며 단계계측근거는 보존한다. v6 producer 순수회귀13개를 root 직접 실행해 통과했고 실제29131은실행중이다. CacheRequest는각Element의Name/Role만가져오고RawView·Full현재접근·256상한·최종target CurrentName/Role ACK를유지한다. 완료/속도개선을아직주장하지않는다.

- 최신library1201whole을사용하여 분리v7 numeric/details Light960100를각각독립private Desktop/합성root에서시작했다. numeric은12phase/24COM/오류와수정·재시작, details는6실제corrected/12COM/6상세·headerfocus/명시field복귀를모두요구한다. 두PASS의동일whole/source/CPU/fixture/theme/viewport합산검토까지완료해야이범위인수다. 원90worker500host 유지/source동결.

- v7 Light numeric79871/details36010 및 합산55758 exit0. 원12numeric/24COM와별도6corrected/12COM/6상세/복귀를 모두같은1201whole/source/CPU/fixture/theme/viewport로독립대조했다. first/restart/원데이터·0jobs·자연Job0검증유지. root가first속도상세open과동시성field복귀2PNG를직접본범위에서설명은읽히지만physicalfocus/caret/fullUI인수는아니다. Dark800200 두scope9126/15148실행중.

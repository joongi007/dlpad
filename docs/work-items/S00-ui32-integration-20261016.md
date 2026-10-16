# S00: P0 화면 접근성과 필수 도구 복귀 통합

- 상태: in_progress / scratch candidates, Live31 frozen
- 분류: integration
- 우선순위: P0
- 요구사항: R03, R05, R07, R08, R10, R11, R13
- 담당/수정 범위: Root가 accessible.rs, accessible_tools.rs, accessible_tool_preferences.rs, accessible_options.rs, media_workspace.rs, media_detail_open.rs, accessible_history.rs, ui_components.rs, catalog.rs를 단독 통합한다. child는 서로 분리된 scratch만 수정한다.
- 선행 작업: 실제 source31 whole1636/0/source675·패키지·private 재설치 완료; source31 feature critical section terminal 및 기본3 exact restoration.

## 사용자 결과

200% 작은 창에서 이력 안내·영상 상세 동작/결과·긴 도움말을 읽고, 필요한 도구 설정으로 이동했다가 URL·구간을 보존한 채 작업으로 복귀한다.

## 계약과 범위

History의 실제 viewport와 egui paint overscan을 구분한다. pinned Help는 원owner/열림/overflow일 때만 PageUp/Down/Home/End를 처리한다. media detail은 새 source/새 결과의 실제 rect를 한번 공개한다. 도구 설정이동은 실제설치와 구분하고 자동설치/자동retry를 만들지 않는다. 원 worker/native pin/취소·timeout/cache/CloseWait 불변조건을 유지한다. DB/schema/Cargo/lock/의존성/feature 변경0. P1/P2와 network updater는 범위 밖이다.

## 인수 조건

의미 있는 원 실패를 보존하고 actual Context/Windows tests·원문glyph/clip·focus/IDs·수동scroll·draft·실패/취소 경로를 검증한다. 수정전후 root-owned S에만 stage하고 strictclippy 이후 Live로 통합한다. 이후 fmt/strictclippy/workspace test/release 네 명령과 Root 원로그·현source/3PE/lock 대조, actual private GUI, 검수패키지 CRC/SHA·설치→제거→같은 경로 재설치를 검증한다. 물리Windows/외부viewer/cleanOS 검증을 synthetic 시험으로 대체하지 않는다.

## 검증과 handoff

현재 Help RED1PASS1FAIL→GREEN2PASS0FAIL, History 실제Main 원5px/잔여3px 실패→v5 GREEN1PASS, 원앱 회귀 known label1skip 명시486PASS/6ignored 및 strictworkspaceclippy0이다. 필수도구9개8PASS1FAIL은 Label.value와label 혼동을 조사중이며 미완료다. 영상상세공개3개2PASS1FAIL은 최초actionrow 실제glyph/clip을 조사중이다. 이 후보들은 v31 ZIP에 포함되지 않는다. source31 성능feature 검증동안 S/Live를 동결하고 Cargo/native 중복을 피한다.

후속 실제검증: source31 feature4/60초 진단을 종료하고 기본3 exact복구를 Root확인했다. UI최종후보v1은 unfiltered489PASS/1FAIL/6ignored, media3회귀모두PASS였다. 유일실패는 도구사유107logical행을110paintclip에맞는다고판정한200%검사다. v4는실제ScrollAreaOutput.inner_rect를직접관측하고paintclip과분리해더강한실제가시범위를사용한다. 원사유/action각glyph도달과fit행joint조건은유지하며모든app/strict를다시검증중이다. 초기relativepin setup실패는S변경전에발생했고새명시경로namespace에서다시실행했다.

## 제품 소스 통합

Root가490PASS/0FAIL/6ignored·strictworkspace/alltargetsclippy0 및원 Live/S/hash를 독립확인하고 원9파일을 backup한뒤 source32로 통합했다.675→675 entries, Cargo.lock/의존성/schema변경0. receipt는 root-ui32-integration-v1/receipt.json이다. 실제 `windows_msvc_workspace_probe.py --configuration windows-msvc-ui-feedback-v31-whole-20261016/configuration.json --output windows-msvc-ui-feedback-v32-whole-20261016 --reuse-owned-target`를 실행중이다. fmt와strictclippy는0,workspace test/release는진행중이며전체완료/실제GUI/새설치본인수로아직표현하지않는다. 최신검수ZIP은여전히v31이다.

Root 실제whole32 네명령0/43harness/1,668PASS/0FAIL/42ignored/source675/3PE·lock대조완료다. CPU10,360,320B(+9,216B/v31),CLI978,432B,설치기4,432,896B다. 현재privateHelp와read-onlygraph를서로분리된output에서실행하며원Live/release를동결한다. native/새ZIP/설치 인수는아직후속이다.

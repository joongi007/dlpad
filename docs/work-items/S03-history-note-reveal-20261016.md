# S03: 작은 창 검색 결과의 재개 안내 전체 노출

- 상태: in_progress / 실제 native 실패 보존, Context 재현 후보 준비
- 분류: bug
- 우선순위: P0
- 요구사항: R07/R08/R13
- 담당/수정 범위: field_reveal scratch accessible_history.rs; Root S실행·통합·실제 native 검증
- 선행 작업: production fixture native v2 원100%6쿼리PASS/200%Failed ID 하단5pxFAIL 보존

## 사용자 결과

작업이력 검색 후 실패 상태의 재개 안내와 실패 원인 안내·재개 버튼이 작은 창의 실제 스크롤 영역 안에 노출된다.

## 계약과 범위

ScrollArea의 확장 paint clip 대신 실제 viewport로 첫 검색결과 전체 group의 노출을 판단한다. 원 ID/라벨/문구·Resume32px·한번 reveal·focus 유지·wheel/no-resnap·worker/schema는 유지한다. 전체 그룹이 물리적으로 들어갈 때만 전체 노출하며 큰 그룹은 기존 ID 공개를 유지한다.

## 인수 조건

기존 native 원조건과 actual Context 352×117.5/125logical·100/200%·버전유무8조건에서4원Label provider rect/glyph와Resume32를 실제 inner_rect로 확인한다. 설정실패와semantic실패를 구별하고 기준을 낮추지 않는다. Context 통과 후 current PE로 실패했던 native를 다시 검증한다.

## 검증과 handoff

현재 scratch rustfmt0만이며 Cargo/native미실행. 실제0/FAIL은 후속 실행 후 기록한다. 제품/원증거를 바꾸지 않고 Root가 실행 순서를 조정한다.

Root실제v1 RED는1PASS/0FAIL로8조건모두원viewport안에맞았으며native5px실패를재현하지못했다. 이결과를수정근거로승격하지않고GREEN채택을보류했다. S History는원source30로복구했다. native실패원본은보존하고새v2에서실제Main중첩viewport/폰트/관측범위를대조한다.

실제v2 RED는Cancelled note1px초과로0PASS1FAIL이다. 하지만합성footer가긴RECOVERY_CHECKING을써실제native의짧은RECOVERY_DONE보다viewport가16logical작았다. 이를원native동일재현으로기록하지않고Root전용v3에서새testmodule의두상태참조만실제문구로바꿨다. 높이/offset/인수조건은그대로이며v3 RED실행중이다.

Root v3 실제RED는원native와같은Failed ID 하단230.5logical/inner228=5physical초과를재현했다. correctedMain의Cancelled ID/URL은통과후Failed에서실패했다. 같은GREEN은하단229.5로여전히1.5logical초과해실패했다. 조건을완화하지않고egui center가paint clip overscan을사용하는것을확인한v4후보로원요청좌표를보정할예정이다. S의실제offset/원Label rect를직접수정하지않는다. 원실패둘다보존한다.

Root v4 RED/GREEN은 새100% wheel 회귀가 실제 overflow가 없는 조건에서 이동을 요구해 먼저 실패했다. v5는 실제 ScrollAreaOutput.content_size를 관측해 overflow 전제를 바로잡았으며200조건의 overflow는 반드시 요구한다. v5 RED는 Failed note229.5>228을 실제 재현했고 현재 GREEN/기존회귀/strictclippy를 실행중이다. 원 라벨·높이·물리5px 기준은 유지한다.

Root v5 GREEN은 실제 Windows test0/1PASS이다. Failed note 하단228=실제inner228로 맞고,200실제overflow 및 사용자wheel offset229→223·6frame재스냅없음을 확인했다. 새 source31위 후보 단계이며 기존History/전체검사·현PE native는 아직 대기다. 원결과는 managed-tool-isolated-ui32-candidate-v1-20261016/test-0.log에 있다. 같은 통합 실행의 다음 필수도구9회귀는8PASS1FAIL(known rejection label 미관측)이어서 전체 실행은 실패로 보존한다.

## current32 native 재검수
# R08: current32 작업 이력 작은 창 native 재검수

- 상태: proposed — 실행 후보 준비
- 분류: validation
- 우선순위: P0
- 담당/수정 범위: search_reveal; history-current32-native-v1 scratch만
- 선행: actual current32 whole4/Root sibling review/default3PE freeze 및 새 production repository fixture build

## 사용자 결과

현재32의 viewport/overscan 수정이 실제 Windows 작은800×600/100·200%에서 Failed 긴 안내와 재개 버튼을 온전히 보여주는지 다시 검수한다. v30의200% Failed 안내5px 잘림 실패를 성공으로 바꾸지 않는다.

## 계약과 범위

기존 history-compact-native-v2 observer/production repository Rust fixture/seeder는 byte-identical이다. interface contract history-compact-private-v1/fixture_revision2 및6query×2scene/원 ID·URL·selection·focus/Label 원identity/Resume32px·Failed-only enabled/Resume실행0/14PNG를 유지한다. namespace만 새로 만들고 Root review는 exact sibling root-review.json으로 좁혀 동적 source/harness/current3PE를 검증한다. 현재 source675는 Root actual 사실이며 helper에 미래 또는 예상 count를 상수로 고정하지 않는다. source32 Context 통과는 native 인수로 바뀌지 않는다.

## 인수 조건

생산 API create_job/stored_options/validated_resume/history로 유효 options/snapshot을 생성하고 Python은 synthetic terminal/Attempt/Artifact만 바꾸며 원 snapshot/options byte불변을 확인한다. 현재 whole/source/path/실제 fixture lock/exe/build로그가 모두 묶여야 GUI를 시작한다. resume/외부download0, synthetics만, 원140media SHA/ID·DB readonly 검증·원input/private desktop/자연app0/Job0를 유지한다. 새200% 실패면 원 geometry/PNG를 보존하고 fakefocus/wheel/조건완화0이다.

## 검증과 handoff

Root가 Cargo/native 전부 terminal 후 build-argv-template.json→gui-argv-template.json 두 단계를 순차 실행한다. 새 scratch lock만 offline generate/registry subset audit→release --locked --offline로 실제 빌드한다. 공유 Cargo/lock은 변경하지 않는다. 원 v30 fixture proof를 current32로 승격하지 않는다. 각 build1800초2op·GUI compile45/전체685초 상한 등 기존 cap 유지. 준비 중 Python AST7 및 byte동일/pins 정적대조만 수행, compile/native/Cargo0. glyph/hover/physical IME/reader/OS DPI/cleanOS/전체P0 인수는 별도다.

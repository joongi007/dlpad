# S00: 배율 선택 뒤 조작 문맥 유지

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R06, R08
- 담당/수정 범위: 주 담당 통합; diagnostic_inputs는 /tmp 후보만. crates/app/src/accessible_settings.rs, 이 문서
- 선행 작업: S00-current-five-screen-ui-polish-20261011

## 사용자 결과

배율을 변경한 뒤에도 선택한 배율 조작과 설명에 접근한다. 다른 설정에 포커스를 옮기면 해당 행을 필요한 만큼만 공개한다.

## 계약과 범위

DESIGN의 설정 문맥 공개 규칙을 따른다. 기존 Combo ID, 테마 저장 worker, disabled, popup, CloseWait를 유지한다. 선택 뒤 zoom 재배치는 기존 gained_focus와 구별해 검사한다. 새 의존성·schema 변경은 없다.

## 인수 조건

실제 Appearance.show와 AccessKit 선택에서 200% 재배치 뒤 선택 상자의 bounds·clip·focus를 확인한다. 기존 테마 행 회귀를 유지하고 800/960 실제 GUI에서 선택 직후와 후속 포커스 화면을 구분한다. 단위 검사로 OS 화면 검증을 대신하지 않는다.

## 검증과 handoff

현재 200% 선택 직후 캡처에서 배율 조작이 viewport 아래에 있다. 당시 focus receipt가 없어 원인은 미확정이다. /tmp 읽기 진단과 후보 준비를 진행하며 native 실행·프로젝트 통합은 주 담당만 수행한다.

### 실제 Windows RED 확인

`windows-msvc-scale-footer-ui-red-20261012/test.log`에서 회귀 테스트 실패를 확인했다. 화면 배율 설명 rect가 200% 재배치 후 viewport 밖으로 벗어남(960px). 컴파일 성공, 27 passed / 2 failed / 2 ignored; 소유 Job natural-zero=True, forced=False, input desktop unchanged. GREEN 통합 및 화면 검증은 아직 진행 전이다.

### 첫 통합 검증 결과

`windows-msvc-scale-footer-library-ui-whole-20261012`: fmt 및 strict clippy 통과. workspace test는 첫 UI harness 228 passed / 2 failed / 3 ignored에서 중단되어 release build는 실행되지 않았다. 배율 선택 회귀와 기존 테마 문맥 회귀는 통과했다. footer 좁은48px wrap 및 library wheel 안내 도달 회귀는 실패하여 원인 조사 중이다. 전체 통과 또는 UI 완료로 주장하지 않는다. 소유 Job 자연 종료/정리 및 input desktop unchanged 확인.

### v2 workspace test 종료 확인

원본 `windows-msvc-scale-footer-library-ui-whole-v2-20261012/test.log`를 주 담당이 집계하여43 harness / 1,111 passed / 0 failed / 28 ignored를 확인했다. 테스트 Job natural-zero/forced=False/input desktop unchanged. release build는 아직 진행 중이므로 전체 검증 완료로 기록하지 않는다. 교정된 footer/library RED와 GREEN 테스트 본문은 rustfmt 후 바이트 동일성을 독립 확인했다.

### v2 전체 검증 완료

`windows-msvc-scale-footer-library-ui-whole-v2-20261012/root-review.json`에서43 harness / 1,111 passed / 0 failed / 28 ignored, fmt·strict clippy·locked CPU workspace test·release build의 실제4명령 및 current source/hash/artifact를 주 담당이 독립 확인했다. CPU 9,670,144 bytes SHA256 `252b7bbaf3a476e574af5925ca392c3680f891a8f5c6b25024d40be88562c92f`; Cargo workspace/lock/schema 변경 없음. `current-ui-scale-selected-context-layout-20261012` 가상 화면 실행 중. 사용자 시각 인수·설치본 갱신은 미완료다.

### 첫 native 관측 실패 및 v2 실행

첫 `current-ui-scale-selected-context-layout-20261012`은 library800 기존 예약폭 기대값 불일치에서 FAIL. C# compile 성공, observer Job natural-zero/forced=False; 내부 앱은 실패 처리 과정에서 owned Job termination으로 정리되어 자연 종료 성공으로 주장하지 않는다. actual button right772=800−16−12 및 ScrollArea inner/solidbar 설정을 근거로 library800 예약 폭만12로 보정했다. 기존32px/16px/fullrectangle 및 library960/download 기대값은 유지했다. pure9 주 담당 재실행 PASS. 새 `current-ui-scale-selected-context-layout-v2-20261012` 실행 중.

### 실제 800×600·200% 캡처 반증

주 담당이 `current-ui-scale-selected-context-layout-v2-20261012/gui/first/scale-selected/initial-zoom-200-800.png`을 직접 검토했다. Scale label/help는 보이지만 control bounds386..450의 아래쪽이 본문 끝408 이후 잘리고 footer 시작433과 겹친다. observer의 effective_clip_bottom_assumed=Int32.MaxValue 창 전체 containment는 실제 scroll parent clipping을 증명하지 못한다. 따라서 전체 회귀 GREEN을 native 배율 UX 해결로 확장하지 않는다. 실제 동적 footer 포함 Context 재현 및 생산 코드 추가 수정 필요. 960 및 restart 캡처도 계속 확인하며 기존 PNG/receipt를 보존한다.

960×640·200%의 동일 immediate-selection PNG도 주 담당이 직접 검토했다. Scale label/help/200% control은 본문 안에 온전히 보인다. 800의 실패를960 성공으로 덮지 않으며 두 크기 조건을 유지한다.

### 실제 앱 footer 포함 회귀 준비

`/tmp/dlpad-scale-real-footer-clip/regression.rs`는 실제 Candidate.show_ui 및 원래 TopBottomPanel/metrics.status를 사용해800×600/960×640에서200% Combo 선택 뒤 최종 painter clip과 control 배경·label/help rect containment를 확인하는 후보이다. 고정100px 시험이나 client bounds 추정으로 대체하지 않는다. 아직 실행하지 않았으며 GUI source freeze 종료 뒤 RED 검증할 예정이다.

### 실제 RED 결과

`windows-msvc-real-footer-scale-virtual-keyboard-red-20261012/test.log`: 컴파일 성공,31 passed / 2 failed / 2 ignored. 실제 Candidate800×600 Scale control [16,203..176,235]이 실제 painter clip bottom203을 벗어난다. library Tab은 path4 가상화 경계에서 다음행 focus ID를 얻지 못했다. Arrow 검증은 Tab 실패로 도달하지 못했으므로 통과로 주장하지 않는다. 소유 Job natural-zero/forced=False/input desktop unchanged. 설정과 목록 수정 후보를 파일별 담당으로 분리해 준비 중이다.

### 실제 viewport 안정화 후보 통합

선택한 zoom이 적용되고 서로 다른 두 frame에서 actual clip_rect가 같아진 뒤 한 번만 공개한다. 선택 origin frame의 재실행에서는 pending 중단/예산/viewport 처리를 유예한다. 이후 새 pointer/scroll/Theme focus/disabled에는 요청을 취소하며8frame 상한으로 무한 repaint를 막는다. focus 강제 이동·worker revision·popup ID는 유지했다. actual pointer popup 선택+request_discard multipass 회귀와 실제 Candidate footer 회귀를 함께 Windows 실행한다. 아직 GREEN 전이다.

### 실제 footer GREEN

`windows-msvc-real-viewport-scale-multipass-keyboard-20261012/test.log`: actual Candidate800×600/960×640 fullcontrol/background painter clip 회귀가 통과했고 기존 Theme/Scale 문맥 회귀도 통과했다. pointer multipass 신규 회귀는 scale100→200 선택 자체가 실패하여 원인 조사 중이다. 전체 첫 UI harness33 passed/2 failed/2 ignored; Arrow clip도 아직 실패. 시각 캡처 재검증과 전체4검사는 미완료다.

### 실제 전체 검증의 잔여 실패

`windows-msvc-real-viewport-keyboard-ui-whole-20261012`: fmt·strict clippy 통과, 첫 UI harness233 passed / 1 failed / 3 ignored. 실제 footer 및Tab/Arrow128 회귀는 통과했다. pointer 신규 시험은 클릭 전 popup200 glyph/row fully painted 검사에서 실패했다. 초기 Scale 조작이 scroll viewport 밖에 있는지/접근성 Click이 focus 공개를 건너뛰었는지 조사 중이다. release build는 실행되지 않았으며 전체 검증 완료로 주장하지 않는다. Job natural-zero/forced=False/input desktop unchanged.

### pointer 시험의 실제 가시성 전제 보완

생산 코드 변경 없이 Scale control에 실제 AccessKit Focus를 주고12frame 안정화 뒤 control background가 painter clip 안에 완전히 있는지 검사한다. 그 뒤 popup 안정화/hover/press/release+forced multipass를 실행한다. popup glyph/row fullclip·scale200·pending 생존/상한 조건은 유지했다. 실패하면 실제200% textclip/glyph 및 node rect를 출력한다. `windows-msvc-pointer-scale-visible-popup-multipass-20261012` 실행 중이며 아직 결과 미확정이다.

### popup 내부 스크롤 실제 좌표

`windows-msvc-pointer-scale-visible-popup-multipass-20261012/test.log`: control 공개 전제는 통과했다. 200% node621,315..751,347/glyph323.5..338.5는 popup 내부 ScrollArea clip112..318보다 아래에 있다. egui ComboBox::show_ui는 기본 combo_height의 독립 ScrollArea를 사용하고6개 scale 옵션은32px+간격으로 그 높이를 넘는다. 따라서 실제 popup 안에서 작은 wheel로 마지막 항목을 공개한 뒤 pointer를 클릭하는 시험 순서를 추가한다. clip 조건이나 옵션 높이를 늘려 실패를 숨기지 않는다.

### 실제 popup wheel 회귀 시작

실제200% text painter clip 안에 완전히 들어오는 popup option의 중심을 anchor로 사용해 PointerMoved 뒤 MouseWheel Point−40을 전달한다. 최대12step/8settleframe 상한 안에서 fresh200% full glyph/row paint를 확인한 뒤 press/release와 forced multipass를 실행한다. 직접 scroll state나 popup 기본 높이는 수정하지 않는다. wheel_steps>0/scale200/pending 생존 및 후속 종료 조건은 유지한다. `windows-msvc-pointer-scale-popup-wheel-multipass-20261012` 실행 중.

### 실제 popup wheel·pointer·multipass GREEN

`windows-msvc-pointer-scale-popup-wheel-multipass-20261012/root-review.json`: 실제 bounded popup wheel로200%를 공개하고 press/release·forced multipass·pending 생존 및 후속 종료 회귀가 두 UI harness에서2 passed / 0 failed다. 주 담당이 original raw Job guard/current source snapshot과 독립 확인했다. production을 시험 통과용으로 수정하지 않았다. physical pointer/IME 검증은 아니다. 전체4검사 `windows-msvc-real-viewport-keyboard-ui-whole-v2-20261012` 실행 중이며 actual800/960 캡처 재검증 및 설치본 갱신은 남아 있다.

### 最新 전체 검증과 실제 화면 재검증 시작

주 담당이 `windows-msvc-real-viewport-keyboard-ui-whole-v2-20261012`의 원본 logs/Job guard/current source 및 artifact pins를 독립 검토했다. fmt·strict clippy·locked CPU workspace test·release의 정확한4명령 계약 통과,43 harness / 1,119 passed / 0 failed / 28 ignored. CPU 9,672,192 bytes SHA256 `b4902683093d64b1b32c97713761128e3ae9cb85ad86cf91178cb63bffbabe64`. 실제 화면 또는 UI 완성 인수로 확대하지 않는다. 새 `current-ui-real-viewport-keyboard-layout-20261012` 가상 화면 관측을 시작했으며800/960 배율 캡처 직접 검토 및 최신 설치본 갱신은 남아 있다.

첫 새 화면 collector는 `compile.log`의 WSL `UtilBindVsockAnyPort: socket failed 1`로 Windows host 실행 전에 종료됐다. 앱/observer 컴파일 성공이나 GUI 결과로 주장하지 않는다. 실패 output을 보존하고 승인된 Windows 실행 환경에서 fresh `current-ui-real-viewport-keyboard-layout-v2-20261012`를 시작했다. 사용자 visible desktop fallback은 없다.

### 新 캡처 직접 검토:800·960 배율 control

`current-ui-real-viewport-keyboard-layout-v2-20261012/root-scale-pixel-review.json`: 주 담당이 실제800×600/960×640·200% immediate-selection PNG를 직접 보았다. 두 화면 모두 label/help/200% selector 전체가 footer 위 본문 안에 있으며 이전800 control clipping은 관찰되지 않는다. PNG별 SHA 보존. 전체 collector 종료/raw 검토 및 다른 화면의 hover/input/selection 시각 인수는 진행 중이며 UI 완료/physical pointer/user acceptance로 확대하지 않는다.

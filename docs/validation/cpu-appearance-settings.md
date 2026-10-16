# CPU 테마·배율 모듈 구현 및 검증 범위

2026-10-03, R06/S02. 신규 accessible_settings.rs는AppearanceSettings::load_initial(data)/apply_to_context(ctx)/show/poll/busy/request_stop/stopped를제공한다. 기존 theme.sqlite settings('theme')의serde "System"/"Light"/"Dark" 문자열과legacytheme.json최대4096bytes를호환한다. DB값이있으면우선하며값이없으면legacyfallback,둘다없으면System이다. malformed/oversize/readerror는System과한국어APPEARANCE_FAILED를반환하고startup자동수정/파일생성없음.

startup읽기는root가제공한Store::read_setting_readonly(path,key,max_bytes)의read-only/no-migration 단일SQLbounded4096바이트값을사용한다. legacy는Take4097로초과를검출한다. 경로component/파일/SQLite-sidecar의knownsymlink와nonregular를거절한다. 이검사가ancestorTOCTOU나공격자가동시에파일을교체하는모든race를막는다고주장하지않는다. 기존Store::open에는WAL설정이없지만migration은있다(이전초안의WAL추정은정정). load_initial은window/eventloop전에만blocking호출하며Store5초busytimeout이첫표시를지연할수있다. 실제firstpaint비용은후속관측게이트다.

사용자는System/Light/Dark를선택하고75/100/125/150/175/200%의session-only배율을적용한다. 테마만기존themeSQLite에저장하며scale을새schema/setting으로저장하지않는다. apply_to_context는root가첫frame전에호출한다. egui_winit0.33.3 State::new가초기nativewindowtheme를RawInput.system_theme에설정하고ThemeChanged도갱신하므로기존hosteventforward+ctxSystemPreference가OS변경반영계약이다. explicitLight/Dark는OS변경보다우선한다. nativeOS themeunknown이면eguiSystemfallback이며실제3OS첫frame정확성은미검증이다. 표시된6개scale선택지를egui/OS모든keyboardzoom경로의절대상한으로확대하지않는다.

저장은singleworker/command/result각channelcapacity1,acceptedlatesttheme한개/revision으로관리한다. UI선택은즉시context에적용하되이전revision의완료가최신visibletheme/error를덮지못한다. 빠른변경은active저장뒤마지막latest만저장한다. request_stop은새선택을막고이미acceptedactive+latest를drain한뒤workerstopack한다. UIthreadjoin/Dropblocking없고workerouterpanic은RAIIstopack+fixederror,writeoperationpanic은fixedfailure응답이다. SQLite단일호출중즉시취소가아니며5초busyboundary를기다린다. ordinaryDrop은stopflag를설정하므로gracefulCloseWait와abruptdrop의선택보존을혼동하지않는다.

저장직전existinginvalid/oversizedDB값은readonlypreflight에서거절하여자동repair하지않는다. Store::openmigration후set_setting실패의보존범위는기존theme선택값이며전체SQLite파일bytes가항상같다고보장하지않는다. malformedDB파일/oversizedlegacy읽기실패에서bytes보존은전용회귀로검증한다. 권한/DBbusy 실패는UIfixederror로표시하고기존자료를자동삭제하지않는다.

작성한11개회귀: legacy/DBpriority와startupno-write, malformedoversizelegacySystem+bytes보존, corruptedoversizedDBload/write거절,rapidlatestcoalescing,stoplatestdrain,stalecompletionvisiblechoice보존,workerpanicack,finite75..200catalog,initialcontext175%/Systemnativechanges/explicitoverride,Unixsymlink/sidecar거절,Unixdeniedreadbytes보존. permissionbypass(root/capability)환경은denial성공을주장하지않는다. 테스트fixture는독립AtomicU64prefix+AlreadyExists최대256retry로소유temp만생성/정리한다. rootengine의readonlyDBexclusive5초busy/bytebound/Unicode회귀는실제Windows에서통과했으며finalsymlink회귀는cfgUnix라Windows에없다.

담당파일 `rustfmt --edition 2024 crates/app/src/accessible_settings.rs` exit0만실행했다. rootsharedmod/entry/Candidate연결완료. 실제LinuxCPUentry38tests(appearance11포함)/CPUClippy17.08초/fmt가통과했다. WindowsCPU전체clippy/test/release도188passed1ignored/exit0이며appearance9회귀가양binary통과했다. 실제theme/scaleGUI/첫paintcost/nativeThemeChanged/한글IME좌표변화/reader/WindowsmacOSLinux현재binary인수는미검증이다. 제품기존파일/engine/Cargo/schema/catalog는owner가수정하지않았다.

최신원시Windows검증은windows-cpu-appearance.md 및238개inputmanifest/rawlogs/result. 첫통합Clippy의permissionmode0표기오류를0o0으로고쳐재검증했다. 실제GUI인수는별도수집중이며이workitem전체GUI완료를주장하지않는다.

## 비포인터 선택 후 메뉴 닫힘 수정

실제 Linux AT-SPI 시험에서 200% 선택값은 반영되지만 메뉴가 남고 이어지는 Tab의 focus가 없는 실패가 관측되었다. egui 0.33.3의 Popup 기본 close_click은 pointer.any_click에 의존하지만 Response::clicked는 AccessKit의 FAKE_PRIMARY_CLICKED도 포함한다. 테마와 배율의 selectable_value 응답이 clicked일 때 ui.close()를 호출하도록 두 선택 루프를 수정했다. 담당 파일 rustfmt는 exit0이다.

이 수정 소스를 동결해 통합 담당자에게 전달했다. 위 Linux/Windows CLI 성공과 바이너리 SHA는 수정 전 입력에 대한 증거로 유지한다. 수정 후 공통 빌드와 실제 비포인터 GUI의 메뉴 닫힘·Tab focus 검증은 통합 담당자가 다시 수행한다. 추가 egui AccessKit 단위 회귀는 아직 작성하거나 실행하지 않았으며 실제 GUI 검증으로 대체됐다고 표현하지 않는다.

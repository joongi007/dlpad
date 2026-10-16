# ADR-0001: Rust 네이티브 UI

상태: Rust 네이티브·WebView 제외는 Accepted(사용자 요구), Iced/tiny-skia 선택은 Proposed.

## 2026-10-12 Windows 입력 유효성 bridge — 구현 검증 중

실제 private Windows 높이 입력0→1080에서 AccessKit 내부 Invalid/description 회귀는 통과했으나 UIA30103은 양쪽 모두false였다. locked accesskit_windows0.29.2에 유효성 mapping이 없어 원 node data의 invalid를 IsDataValidForForm으로 연결하는 작은 동일 버전 fork를 통합한다. 기존 properties macro의 native dispatch와 값 변경 비교를 함께 사용한다. 내부 consumer/dispatch 회귀는 실제 Windows에서 통과했으며 최종 앱 COM 재검증은 남아 있다.

원 registry archive SHA·정확 commit·파일 inventory·MIT/Apache/Chromium BSD 원문을 vendor와 notice manifest에 보존한다. root Cargo.lock은 해당 crate의 registry source/checksum만 제거하며 버전·dependency 목록을 유지한다. runtime getter와 property 비교 한 개를 추가하며 thread/I/O·새 runtime dependency는 없다. binary 크기와 actual COM 인수는 [작업 기록](../work-items/S00-windows-input-validation-provider-20261012.md)에 따른다. 전체 renderer 선택·physical IME/reader·사용자 UI/UX 인수와 별개이며 Proposed 부분을 완료로 바꾸지 않는다.

맥락: 세 OS, gtmux 디자인, 작은 설치 크기와 메모리, 장시간 다운로드가 필요하다.

결정안: Iced의 CPU 렌더러와 wgpu를 실측하고 단일 backend 중심으로 배포한다. core와 UI를 분리하여 GUI 교체 비용을 제한한다. gtmux의 토큰·상호작용을 이전하고 웹 스택은 이전하지 않는다.

대안: egui/eframe의 도구 UI·반복 갱신 비용, Slint의 UI 문법·라이선스, OS별 UI 구현의 유지 비용을 비교했다. 아직 성능 우열은 실측하지 않았다.

결과: S00의 OS별 입력·접근성·메모리·배포 결과 없이는 Proposed 부분을 Accepted로 바꾸지 않는다. 소프트웨어 렌더링이 배터리 사용량을 증가시킬 가능성도 측정한다.

## 2026-09-30 검증

[S00 실측](../validation/S00.md)에서 Iced tiny-skia와 egui/Glow의 실제 Linux 창을 열었다. Iced wgpu는 같은 WSLg X11 환경에서 surface 초기화 오류가 재현됐다. tiny-skia는 초기 RSS 목표 내에 있지만 접근성 bridge와 한글 조합 입력을 증명하지 못했다. egui는 AccessKit를 포함하지만 RSS 목표를 초과했고 스크린리더 실증은 하지 못했다.

최종 renderer/UI 채택은 계속 Proposed다. 개발 기본 tiny-skia는 실험을 계속할 수 있는 가역적인 선택이다. 네이티브 제약은 Accepted를 유지한다. OS별 IME/접근성·3 OS CI가 없는데 Accepted라고 기록하지 않는다.

## 2026-10-02 S00 재감사와 후보 입력 실험

[현재S00감사](../validation/S00-current-audit.md)는최종채택조건미충족을확인했다. [egui비교](../validation/egui-ime-validation.md)에서는AT-SPI실제tree/한국어label/text를확인했고, rawCommit이도착했으나inactiveIME cursor epoch 때문에연속commit이입력에반영되지않는가설을동일binary opt-in대조로지지했다. fixture-only inactiveCommit직전Enabled주입은exact한글commit/취소/Latin/선택대체를복구했지만inlinepreedit는여전히없다. 일반제품default나최종스택채택에적용하지않았다.

비교nativeGL software경로의10초10000행표본은RSS133636KiB/idle0.0%/881만bytes급이며단일환경표본이다. 실제IME전부/스크린리더/3OSGUI/장시간비용인수를충족했다고쓰지않는다. Xlib reference callback시험으로IMEserver와winit/위젯경로를추가분리한뒤결정한다.

## Linux XIM startless Draw 보정과 제품 실증

Xlib 기준 클라이언트는 Start 없이 Draw/Done을 전달하는 현재 Fcitx 서버를 관측했다. winit0.30.13의 Start 이후에만 Update를 허용하는 조건을 [프로젝트 vendor](../validation/winit-vendor.md)에서 보정했다. nonempty Update가 조합을 시작하며 inactive empty Update는 무시한다. wchar/XFilterEvent 가설은 기준 대조가 배제해 변경하지 않는다.

수정 후 egui 후보와 실제 Iced 제품 양쪽의 private Xvfb 조합 표시·확정·BackSpace취소·영문전환·선택대체가 확인됐다. 주 담당이 제품 preedit.png에서 한[글] 조합과 밑줄, 아래 committed-only mirror의 한을 직접 대조했다. [제품 증거](../validation/iced-current-ime/full/result.json)의 원 collector exit1/legacy pixel proxy false는 보존하고, screenshot 검토 기반 인수 판정을 별도로 기록했다. 이 Linux 입력 수정은 개발 제품에 유지한다. Rust native 제약은 Accepted이고 최종 UI/renderer 선택은 계속 Proposed다. 실제 스크린리더와 다른 OS 입력·접근성·장시간 비용을 이 결과로 대체하지 않는다.

## 접근성 실패에 따른 후속 구현 선택

[private AT-SPI/실제 무음 Orca 대조](../validation/reader-validation.md)에서 Iced 제품은 앱 트리를 게시하지 않았고 egui는 한국어 entry와 텍스트·Tab focus 이벤트를 제공했다. 현재 Iced를 최초 릴리스 최종 UI로 확정하지 않는다. 접근 가능한 egui의 실제 P0 기능 적합성을 검증하기 위해 기존 엔진을 사용하는 optional dlpad-accessible 수직 후보를 구현한다. [작업 계약](../work-items/S00-accessible-download-slice.md). 기존 제품을 제거하거나 모든 화면이 이식됐다고 표현하지 않는다. 추가 runtime dependency는 이미 alternative feature에 잠근 eframe/AccessKit/Glow이며 기본 빌드에는 추가되지 않는다. 후보의 높은 RSS와3OS GUI/전체 reader인수는 별도 게이트다. 최종스택채택은 여전히 미완료다.

## CPU 후보의 기능 통합 방향 (2026-10-03)

Iced의 실제 reader 실패와 Glow의 RSS 예산 초과를 근거로, 기존 egui0.33.3 및 AccessKit를 유지하는 CPU pixel 후보를 다음 기능 통합 대상으로 선택한다. 이는 가역적 구현 방향이며 전체 S00 최종 채택은 Proposed를 유지한다. 기본 Iced entry 교체나 모든 화면 이식 완료를 뜻하지 않는다.

직접 egui-winit/softbuffer host는 AccessKit adapter를 숨긴 창에 연결한 뒤 표시한다. Windows에서 먼저 표시한 기존 순서는 actual GUI panic을 일으켰으며 CLI 시험 성공으로 발견되지 않았다. [수정 CLI 증거](../validation/windows-cpu-hidden-init/result.json)와 실제 GUI 재시험을 별도로 관리한다. 기존 crate의 편의 host는 AccessKit가 없어 사용하지 않는다.

새 Linux hidden-init binary에서 RSS50624KiB(49.4MiB), idle0%의10.099초 표본을 [직접 측정](../validation/accessible-performance/cpu-hidden-init/result.json)했다. 예산을 변경해 통과시킨 결과가 아니다. renderer0.0.2/constify/strength_reduce 다운로드 합계505827B, MIT OR Apache2, 선택feature std만 사용해Rayon을 추가하지 않았다. 정확한 버전·SHA·용량은 [pin 검증](../validation/software-renderer-pin.json), lock 재해석의 기존 windows-sys dependency edge 변경은 [lock 대조](../validation/cpu-lock-review.json)를 따른다.

전체 화면 기능 parity·테마·숨김/복귀·완전한 reader 사용성·모든 OS 입력/패키지·장시간 비용은 남는다. [후속 계약](../work-items/S00-cpu-stack-direction.md)을 따른다.

## CPU 설정 후보의 접근성 선택과 메뉴 수명

System/Light/Dark 및75–200%session배율을 기존egui후보에 이식한다. 시작 읽기는 창 생성 전 readonly bounded Store로 수행하며 저장은capacity1 worker/latest선택을 사용한다. 종료는 이미 승인된 선택을drain한 뒤stopped ack를 기다린다. 이는 최종renderer 채택을 뜻하지 않는다. [통합 기록](../work-items/S02-cpu-settings-integration.md).

실제 private AT-SPI선택은 값 변경을 확인했으나 배율팝업이 남았다. locked egui0.33.3 Popup의 자동닫기는 pointer.any_click이고 Response.clicked는 AccessKit/키보드fakeclick을 포함한다. 메뉴항목의 clicked에서 Ui::close를 명시 호출하는 좁은 수정으로 경로를 맞춘다. 소스 계약과 실제키보드/팝업닫힘은 별도 검증이며, 수집기의 staticlabel오선택 및 WM없는Xvfb focus미설정 실패도 보존한다.

설정후속에서Light재실행선택버튼이19px인실제좌표를발견했다. egui의Light/Dark는별도style이므로all_styles_mut로두theme의interact_size32를맞추었으며실제3테마높이32px를검증했다. [크기 인수](../work-items/S02-theme-control-sizing.md). TextEdit의별도min_size(현재21px)는후속P0검증항목으로남겼다.

## 2026-10-05 Windows CPU 후보의 캐시 설정

현재 후보는 기존 egui_software_backend 0.0.2의 `with_caching(false)`를 사용한다. 같은 source snapshot에서 software_host.rs 설정 한 파일만 바꾼 대조와 Windows 전체 fmt/Clippy/test/release를 통과했다. 파일 1만 개를 표시한 fresh 3회 표본의 working set은 약75.57/70.58/70.54MiB, single-core CPU는 약0.31%로 원80MiB/1% 기준을 통과했다. 캐시 사용 baseline의 첫 표본80MiB 초과도 보존했다. 새 의존성·버전·lock 변경은 없다. [측정과 좁은 채택 범위](../work-items/S00-renderer-canvas-cost.md)를 따른다.

이는 후보의 가역적인 설정 채택이다. 실제 한글 화면·숨김 텍스처 회귀와 대형 진단 페이지 동작을 검증했지만, IME·reader·first paint·장시간 GUI 등 S00 전체 인수가 남아 최종 UI/renderer 선정의 Proposed 상태를 유지한다. 최초 릴리스는 사용자 지시에 따라 Windows 우선이며 Linux/macOS 신규 출시 인수는 후속이다.

## 2026-10-05 IME 취소 transaction의 좁은 채택

Windows owner context의 실제 IMM cancel은 nativeemptyCommit, commit은 nonemptyCommit을 전달했다. 동일binarycancel의원UIA/PNG한글잔류FAIL을보존했다. 원egui0.33.3의실제Context회귀4pass/5fail을확인했고동일회귀는state/builder두파일의패치에서9pass로바뀌었다. 원선택·cursor·undo복원이필요해현재egui0.33.3원cache를vendor/egui로고정했다. [실제근거와인수](../work-items/S00-egui-ime-transaction.md)를따른다.

이는UI버퍼의boundedephemeraltransaction수정으로채택한다. 원upstreamcommitlicense와sourcefileSHA를보존하고version/새의존성은바꾸지않았다. Cargo.lock은eguiregistry source/checksum만제거했다. 원text2MiB+expectedpost2MiB상한과Arc참조clone,serde/debug생략을사용한다. 실제nativeGUI재인수·defaultrelease와용량측정은진행중이다. 최종UI/renderer의Proposed·물리한글키IME/reader/장시간GUI게이트는유지한다.

## Windows 초기 테마 프레임과 창 표시 순서 (2026-10-05)

CPU host는숨김창에AccessKit를먼저연결하고실제첫themedbuffer의present성공뒤최초show를수행하도록조정했다. 실패/zero-size초기frame은가시화하지않는다. 기본OFFfirst-paint-fixture가원UIthread/HWND의첫present/visibility/bufferRGB/ctx와nativeSystemtheme를기록하여Light/Dark/System3case와원cleanup을실증했다. [좁은인수](../work-items/S00-windows-first-visible-theme.md). GDI초기샘플은오래된/가려진화면일수있어actualbuffer의첫receipt와구분한다. compositor scanout/실제OS테마변경/reader/기본제품최신전체복원은후속이다. 이는초기프레임정책의좁은구현결정이며최종UI/renderer Proposed상태를변경하지않는다.

## CPU host의 지표 최신성 기한 분리 (2026-10-06, 구현·실험 진행중)

actual wake33 계측에서 일반 callback 기한 반복이 관측됐고 source의 metrics stale 예약을 별도 host 기한으로 분리하는 후보를 구현했다. 동일 DisplayKey는 bounded try_lock으로 timestamp만 갱신한다. stale3초를 완화하지 않고 Unknown 후 동일값 복귀는1초 retry로 확인한다. 일반 repaint/Close ACK 기한과 OR 결합하고 원 queued redraw는 유지하며 실제 성공 present만 표시키를 확정한다. eframe Candidate는 기존 egui 예약을 유지한다. Windows 전체4검사와790/0/13 회귀는 통과했지만 실제 worker 중단/복귀·current 기본18구간 비용·숨김복귀 인수 전 성능 해결이나 최종 UI 스택 채택으로 표현하지 않는다. 새 dependency/worker/schema/lock 변경이 없고 기본 CPU는2,048B 증가했다. [작업](../work-items/S00-metrics-freshness-native-timer.md).

## 대기 worker의 종료 통지 (2026-10-06, 실험 진행중)

현재 default18구간의17PASS/1FAIL을 보존하고 metrics50ms·appearance100ms idle timeout을 기존capacity1 채널의 StopWake로 교체하는 실험을 진행한다. atomic Stop 우선·try_send 비차단·Full 원accepted입력보존·disconnection/원Ack를 유지한다. appearance의 accepted-latest 저장drain과 원Drop abort는 구분한다. metrics1초 관측기한/완료timestamp·UI Stop100ms·stale3초는 바꾸지 않는다. mailbox poison이 통지없이 atomic만set하던 경로는 기존원채널 수신 assertion으로 실제 red(Cargo101)를 재현하고 동일stop_worker를 재사용해 교정한다. 정상freshnesscheck에는선택명령/I/O가없으며 terminal poison 오류만bounded Stop통지를한다. 이변경의최신whole/native18 검증전 성능해결·최종스택채택으로표현하지않는다. newdependency/thread/schema/lock변경없음. [작업](../work-items/S00-windows-current-idle-worker-budget.md).

## 2026-10-12 Windows 관측 연결 — 검증 중

기존 하나의 metrics worker가 stateful NetworkObserver를 소유하고 선택 revision 변경에서 baseline과 native binding을 초기화한다. Windows는 기존 windows-sys0.61.2의 IpHelper/Ndis feature를 사용하며 새 crate/version/worker를 도입하지 않는다. UI thread에서 OS counter API를 호출하지 않는다. LUID/GUID epoch, down/reset/time gap을 넘는 rate 계산은 거절한다. 전체 OS table allocation 후256행 traversal cap이며 사전 메모리 상한을 보장하지 않는다. API 자체의 강제 취소·시간 제한도 보장하지 않는다. 실제 비용과 제품 UI 인수는 진행 중이며 기본 자동 선택은 후속 연결이다.

Windows 자동 관측 통합은 기존 windows-sys에 WinSock feature를 추가하여 local route 구조를 사용한다. 새 crate/thread/schema와 lock 변경은 없다. route 후보4096행·family/LUID 조회64쌍과 인터페이스 목록256개로 제한한다. OS 전체 allocation 이전 상한 및 강제 API 취소는 보장하지 않는다. IPv4 우선 정책은 다운로드 목적지의 실제 route 선택으로 표현하지 않는다. 자동 선택 binary 크기·idle 비용·실제 GUI는 통합 검증 뒤 기록한다.

2026-10-13 통합 whole1227/0/30에서 CPU 실행파일9,817,088B다. 수동 counter whole9,783,808B 대비 자동 네트워크+속도 단위+UI 개선 묶음33,280B 증가(약0.34%). 자동 counter만의 비용으로 분리한 측정은 아니다. 실제 제품 healthy auto/list/off/manual-missing/return private GUI는 통과했으며 idle CPU/RSS·route 변경은 별도 미측정이다.

## CPU host callback generation 검증 — 2026-10-13

locked eframe0.33.3의 현재 pass==요청 pass 또는 현재 pass==요청 pass+1 규칙을 단일 ROOT host에 적용한다. 두 슬롯은 generation별 earliest deadline을 보존하여 stale 요청을 fresh deadline으로 재표시하지 않는다. 소비자는 Context→queue 순서로 접근하고 callback은 queue만 접근한다. 실제 Context의 settling·cache hit·worker 경합·다른viewport·bounded eviction 회귀7개를 통합했으며 Windows검사와기본18구간비용은검증중이다. 이 국소실험은전체스택/성능원인확정/최종ADR채택을뜻하지않는다.


## 2026-10-13 IME 확정 anchor의 좁은 채택

서로 다른 조합·확정 문자열로 시험해 실제 Settings 역방향 선택에서 확정이 건너뛰는 오류를 확인했다. preview range를 저장·반환에 일치시키고 성공 취소 시 original anchor도 복원한다. 기존 취소 transaction·외부 변경/선택 이동 거부·크기 상한은 유지한다. 첫 후보의 split Disabled/Commit 회귀는 인수를 거부하고 보존했으며 두 번째 후보는4필터12/0, Settings100/200·Light/Dark·양방향8fixture와 이동2fixture를 통과했다. [작업·원 실패](../work-items/S00-settings-ime-positive-commit-20261013.md).

이는 가역적인 UI buffer 패치 채택이며 새 상태·schema·dependency/Cargo.lock 변경이 없다. 실제 전체 Windows build·새 설치본 인수는 후속 실행으로 확인하고, 물리 OS IME/reader/최종 renderer와 전체 S00/P0 상태는 그대로 미완료다.


## Windows 파일 드래그 등록 실패 대응 (2026-10-14, 후보 검증 중)

긴 경로의 소유 private 설치에서 앱 초기 exit101을 재현했고 원 stderr는 winit의 RegisterDragDrop S_OK assertion에서 HRESULT0x8001ffff를 기록했다. 장경로 자체와 Windows 내부 오류의 인과를 확정한 것은 아니다. 실패한 API를 정상 등록으로 표시하지 않고 선호 OLE 경로를 유지하며 해당 창에만 native WM_DROPFILES를 허용하는 후보를 통합한다. 원 API 성공 초기화의 S_OK/S_FALSE는 한 번씩 OleUninitialize로 균형을 맞춘다. 등록 성공만 revoke하며 실패·이미 등록 상태를 다른 등록 대상의 해제 권한으로 해석하지 않는다. WM_DESTROY는 공유 WindowData의 RefCell에서 registration을 꺼내 borrow를 해제한 뒤 정리하여 OLE 재진입 동안 borrow를 유지하지 않는다.

OLE 매체는 ReleaseStgMedium, WM_DROPFILES의 전달된 HDROP는 DragFinish RAII로 서로 다른 소유권에 따라 정리한다. 파일 후보 전체는128개/경로32767 UTF-16 단위/더 큰 UTF-16·UTF-8 payload 합4MiB 내에서 먼저 검증한 뒤 기존 DroppedFile 경로로 전달한다. 파일 identity·미디어·네트워크·실행 정책은 기존 worker의 재검증을 따른다. fallback은 놓기 전 HoveredFile을 제공하지 않아 CPU host가 별도 세션 안내를 표시하며 현재 오류·복구·진행률을 변경하지 않는다.

새 crate/version/feature/lock/thread/timer는 없다. 창별 작은 registration과64KiB 이하 scratch buffer, 최대4MiB path payload에 컨테이너 비용이 더해진다. API 강제 취소·COM teardown hard deadline·실제 물리 드래그 성공은 보장하지 않는다. 원 IUnknown QueryInterface의 upstream unimplemented 경로는 이 좁은 수정에서 변경하지 않았다. 제품 최신 전체 빌드·원 긴 경로 HWND/자연 종료·실제 메시지 fixture와 크기/비용 검증 전 전체 S00나 릴리스 인수 완료로 쓰지 않는다. [실제 실패와 후보 작업](../work-items/S11-long-path-app-startup-diagnostic-20261014.md).

## 2026-10-14 Windows 현재 후보 검증

현재Windows 개발설치후보는egui/AccessKit/CPU software host를사용한다. Iced/tiny-skia 방향은이전실험기록이며현재CPU후보의검증으로그선택을확정하지않는다. v24 전체1538/0/42·native선택24/설치88원화면·로컬CI15/8Cargo/7capture를독립검토했다. 기본세60초CPU평균0.676534/0.546358/0.624538%와memory18구간은충족했으며원counter로평균을검산했다. 추가strict10초CPU3구간FAIL을보존하며UI/layout/present인과는후속이다. CPUbinary10,243,072B,신규dependency/schema/Cargo.lock변경없다. 물리IME/reader/OSDPI·사용자UI인수는나중에수행하며 최종S00/renderer Proposed를Accepted로바꾸지않는다. [현재근거](../validation/P0-remaining-20261011.md).

## 2026-10-15 native 문자열 값 변경 요청

Windows bridge는 UIA ValuePattern.SetValue를 AccessKit Action::SetValue의 문자열 데이터로 전달하지만 기존 TextEdit는 이를 처리하지 않았다. 실제 private URL readback 실패를 보존하고 Context 9개 회귀의 원 구현 2 통과/7 실패를 재현했다. 같은 vendor 버전의 builder에서 대상 문자열 요청을 처리하는 좁은 수정을 채택한다. unfocused 입력도 자신의 요청만 처리하며 다른 focused 입력의 keyboard를 소비하거나 포커스를 가져오지 않는다. enabled mutable 입력에만 SetValue를 광고한다. readonly/disabled/noninteractive와 잘못된 target/data·2MiB 초과 값은 거절한다. 기존 문자수 제한·단일행 규칙·undo와 조합 중 외부 변경 정책을 유지한다.

수정 후 Context 9개가 실제 Windows에서 통과했다. 전체 검사·실제 COM 재검증은 진행 중이며 물리 IME/reader 및 최종 renderer 채택의 Proposed 상태는 유지한다. 새 의존성·lock 변경과 상시 I/O는 없다. 대상 변경 시 입력 길이에 비례하는 상한 있는 일시 문자열 하나가 추가되며 binary 비용은 새 release에서 실측한다. [작업 기록](../work-items/S00-native-text-value-action-20261015.md).

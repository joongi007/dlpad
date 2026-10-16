# S00: 지표 최신성 확인과 전체 화면 렌더링 분리

- 상태: done (typed freshness/current worker scope; 최종 스택·전체 P0와 분리)
- 분류: performance
- 우선순위: P0
- 요구사항: R01, R08, R10, R11
- 담당/수정 범위: root — 공유 API·accessible.rs·software_host.rs·frame_phase_fixture.rs·통합/Windows 검증; metrics_recovery — /tmp accessible_metrics.rs 최신성/표시 snapshot·실패 회귀 초안; diagnostic_cache — /tmp 고정 HWND observer/collector 계측 계약 초안; soak_recovery_audit — 독립 stale/coincident/hidden/ACK 계약 검토
- 선행 작업: wake33 actual6구간·원파일/DB 및 기본/진단 전체 검증. 일반 GUI callback transport와 metrics semantic 원인 확정은 구분한다.

## 사용자 결과

같은 지표값의 최신 여부는 화면 전체를 다시 그리지 않고 확인한다. 오래된 관측·실패·새 표시값은 기존처럼 화면에 반영하고 종료/백그라운드/다른 이벤트를 유지한다.

## 계약과 범위

Rust native software host만 명시적으로 host freshness mode에 연결하며 다른 Candidate host의 기존 egui 예약은 유지한다. 지표 mailbox는 기존 bounded try_lock과 latest-only Sample/DisplayKey를 사용하고 정상 timer 확인은 외부 I/O나 선택 명령을 전송하지 않는다. terminal mailbox poison은 기존 atomic Stop 뒤 bounded 비차단 StopWake를 보내 idle worker 종료 확인이 늦어지지 않게 한다. 일반 egui/window/access/Close/ACK repaint 및 동시 기한은 절대로 소비하거나 취소하지 않는다. freshness 기한은 별도이며 host wait는 일반 예약과의 최소값이다. 동일 표시값이면 metrics-only 확인만 수행한다. 기존 stale3초·worker1초·pending/stop100ms는 완화하지 않는다.

공유 후보 API: MetricsPanel.use_host_freshness_timer(), freshness_deadline(now)->Option<Instant>, check_freshness(now)->bool(redraw), frame_presented(). 실제 status 렌더 시 만든 bounded pending 표시키만 성공 frame 뒤 last-presented로 commit한다. timestamp는 key와 분리하며 mailbox.key를 재사용한다. unknown 후 같은 값으로 worker가 복귀해도 재표시되도록 건강한 worker의 unknown에는 bounded 1초 freshness retry를 유지한다. try_lock 경합은 원 expiry를 연장하지 않는다. poison/failed/revision/availability 변화는 redraw이며 worker terminal 뒤 불필요한 timer를 종료한다. root가 Candidate bridge와 두 deadline 수명을 조정한다. 공유 schema/Cargo/lock 변경 없음.

## 인수 조건

동일값 최신 sample은 freshness timer에서 full frame 없이 재예약된다. stall은 기존3초 경계에 unknown, 이후 같은값 복귀도 fresh 표시된다. busy/poison/stale/revision·WouldBlock과 오래된 snapshot·실제 frame commit을 회귀로 확인한다. generic+freshness 동시 due는 항상 redraw, 이미 queued Redraw는 유지, hidden close/Stop ACK는 기존 경로다. 기본 Windows 전체4검사 및 실제 단독 private GUI·18개10초 CPU/메모리 구간과 목록Next/worker완료/CloseWait 회귀를 확인한다. 성능 개선은 실제실험후채택하며 원FAIL/진단오버헤드를 보존한다.

## 검증과 handoff

현재 계측 결과를 근거로 구현 초안을 준비하는 단계다. metrics 원인 확정/성능 해결/전체P0 완료 주장이 아니다. root는 현재 producer검증 입력을 보존하고 초안을 /tmp에서 병렬 준비한다.

root는새API/module원SHA와stagedSHA를대조해6파일을통합했다: accessible_metrics/accessible/host/phasefixture와원observer/collector. Host는별도metrics_deadline을저장하고실제timercheck 또는전체frame후에만재예약한다. 일반기한도동시에due면기존redraw를유지하며hidden metrics-only 변경은pendingdirty로보존하고generichiddenClose/ACK paint는유지한다. hidden/zero-size/present실패에서는표시키를commit하지않는다. Probe36 fixedscalar의 MetricsChecks와Unchanged또는Redraws는단일checkedupdate+generation으로게시해합계일관성을유지한다. module8개실패회귀초안/AST와원schema36대조를진행했고Windowsfmt/전체4검사·실제기능/성능검증은진행중이다. 새thread/I/O/schema/dependency/lock변경없음.

첫실제전체검사는fmtPASS/ClippyFAIL로보존했다. 동일Candidate가CPU feature와함께 eframe bin/test에도컴파일되므로CPU호스트용bridge4메서드가그호스트에서미사용경고였다. root는4메서드만별도CPU impl에묶어 eframe의기존egui예약을유지한다는reason으로좁은dead_code허용을선택했다. 다른Clippy경고나전체모듈검사를완화하지않는다. 독립검토에서hidden/non-drawable 전체paint가Unknown retry를다시미룰수있는계약경계를보강해기존별도deadline을유지하고None일때만arm한다. 새fmt/전체검증을수행하며기존실패snapshot과구분한다.

현재 correctedsource의 독립 읽기 검토는 새 구체 결함을 찾지 않았고 hidden deadline 보존·generic OR·성공 present commit을 확인했다. 실제 corrected Windows fmt/Clippy/test는 exit0이며 release와 전체 proof 인수는 진행중이다. 후속 실제 중단·복구 관측은 root 담당 scripts/windows_metrics_stall_probe.cs 및 scripts/windows_metrics_stall_collect.py로 분리한다. diagnostic_cache는 /tmp 초안만 담당하며 root가 게시·컴파일·실행한다. 원 PID/TID/creation/description을 검증한 앱 소유 metrics thread만 잠시 정지하고 finally에서 자기 suspend 증가분을 복구한다. 이미 정지된 thread는 자기 증가분을 undo한 후 FAIL한다. 측정 불가와 fresh 복귀를 실제 UIA/PNG로 관측하며 표시값이 바뀌면 INCONCLUSIVE로 보존한다. private Job 자연 종료·worker drain·input 불변·36 scalar 정확한 합계/기한/source SHA가 인수 조건이다. 아직 native 시험을 수행하지 않았으며 hidden 전이 인수로 대체하지 않는다.

corrected 기본 Windows 전체4검사는 실제 exit0/root raw4/source/artifact 검토 PASS다. 790passed/0failed/13ignored이며 새8회귀는 CPU/accessible 두 harness에서16회 실제 통과했다. CPU91217a805cb88ecc67d280615aafacbffe43c756bcb1084c1d0a4db16e0da6ea/8,711,680B는 직전8,709,632B 대비2,048B 증가, CLI97217440…/841,728B는 동일하다. 새 dependency/thread/I/O/schema/lock 변경은 없다. source579는 항목 수로 정확 SHA map을 evidence/windows-msvc-metrics-freshness-corrected-default-workspace-20261006-continue/result.json에 보관한다. 별도 feature36/actual stale 복귀/current 기본18구간/hidden 회귀 인수는 진행중이다.

현재 binary의 기존 background 숨김/복귀 회귀를 root 담당 scripts/windows_msvc_background_native_runner.py adapter로 재사용한다. soak_recovery_audit는 /tmp 초안만 담당했다. native schema/source는 기존 private background observer/합성worker를 그대로 사용하고 현재 MSVC whole/raw4/current CPU SHA·원PID/creation/HWND·동일worker heartbeat·자연Job0·private/input복원·raw부호/pin불변을 인수한다. root가 PNG·readonlyDB를 추가 검토해야 완료이며 hiddenCloseACK정확기한 및 same-value stale 시험으로 확대하지 않는다. 아직 native 미실행이다.

별도 feature36 Windows 전체4검사 실제808/0/13·CPUe53c87d2…8,739,840B root PASS다. 실제 stall 최초 compile은 C# CS0165(Require의 short-circuit out 변수가 뒤에서 확정된다고 컴파일러가 판단하지 못함)로 exit1 FAIL 보존했다. GUI는 실행하지 않았다. root가 해당 local4개를0으로 초기화해 readonly 원 identity검사를 유지한 채 fresh evidence에서 재컴파일/실행한다. raw forcedFalse/Job0/input동일은 유지됐지만 compile 성공으로 쓰지 않는다.

교정 실제 native stall은 baseline321.8GiB→측정불가→같은321.8GiB로 복귀, resume약1017ms·원thread복원·workerdrain·naturalJob0·input/private복원을 관측했다. 다만 Python collector가 Windows codepage의 한국어 stdout를 UTF8로 decode하여 UnicodeDecodeError로 최종FAIL했다. 원FAIL을 보존하고 ASCII raw terminal byte guard로 교정해 새 evidence 전체실행을 재수행한다. native 관측과 wrapper 인수는 구분하며 현재 전체완료로 쓰지 않는다.

byteguard 교정 실제stall 수집은 exit0·DIAGNOSTIC_ONLY·same_value_acceptance=true이며 root/독립 raw·원source/artifact/GNU핀·36counter 대조와 root PNG를 확인했다. 이번 원값321.7GiB→측정불가→동일321.7GiB, resume1015ms, MetricsChecks7=Unchanged5+Redraws2, original metrics thread복원·workerdrain·자연Job0·input/private복원. fixture10000/DB root전체검토는 남아 있으며 기본제품 CPU/hidden/physical 인수로 확대하지 않는다. 첫 compileFAIL와UTF8 wrapperFAIL은 그대로 보존한다. 현재 current feature의단독60초/6구간을 실행하며 후속 기본3×60초/18구간 및background회귀를 순서대로 수행한다.

current 기본18구간이 실행되는 동안 source를동결한다. metrics_recovery의 후속 /tmp 초안범위는 실제 Host due dispatch의 private DueTarget/HostDue 추출과 arm_if_absent, 실제MetricsPanel/StopACK를연결한 coincident/hidden 회귀다. 담당projectfiles software_host.rs/accessible_metrics.rs는 root만통합하며 genericpublicschema/Cargo 변경은없다. 단순OR조건식복제시험을피하고 actualpending표시키/ACK효과/재시도supplier호출을검증한다. 초안은 컴파일/native 미수행이며 root가 현재실험후채택·통합을결정한다. source/hash변경이생기면전체4검사와currentbinary실제검증을다시수행하며 현재PASS를소급하지않는다.

current 기본 source의 Next128·수동목록갱신·invalid-root 보존 회귀는 root 담당 scripts/windows_msvc_library_loaded_native_probe.py로 재검증한다. soak_recovery_audit는 /tmp 자체컴파일형 초안만 담당했다. 현재 MSVC whole/raw4+exact CPU·고정 원 C#→bounded복사→실제csc/rawguard·ownprivate native수명을 확인하고 원10000fixture의의도된add/rename/delete외byte보존·readonlyDB root/count/first129/integrity·root PNG를 인수한다. CloseWait 전용feature는 별도proof가필요해이adapter 인수로대체하지않는다. source/compiledhelper/collector/원wholeproof pin전후검증을수행하며 아직 native미실행이다.

최종worker/poison교정source579의전체4검사811/0/13 및진단829/0/13, 실제같은값복귀321.4GiB→Unknown→321.4GiB/1065ms, 기본18구간18PASS/최대CPU0.781324%와30k합성/3DB보존을root/독립인수했다. 같은current기본binary의숨김복귀·Next128/manual/invalidroot와별도CloseWait release3656ms/timeout20009ms·원ACK/rollback/naturalcleanup도인수했다. 초기rawcompile/UTF8wrapper/CPUbudget실패는보존한다. 새readonly설치source581는후속별도whole진행중이며이source579 정확map의GUI 인수를소급하지않는다. 실제physical IME/reader/DPI/최종stack는별도남는다.

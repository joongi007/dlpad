# S00: 격리 GUI main thread 프레임 비용 계측

- 상태: in-progress
- 분류: diagnosis
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — feature·소프트웨어 host 통합·실제 검증; metrics_recovery — /tmp frame phase module/host patch 초안; diagnostic_cache — /tmp C# observer의 원 HWND 숫자 관측; soak_recovery_audit — 계측 계약 독립 검토
- 선행 작업: source577 CPU5946의 실제 18구간 비용 실패와 원 thread counter 분리

## 사용자 결과

성능 기준을 넘은 구간의 UI/layout·래스터·present·프레임 외부 비용을 구분하여 근거 있는 수정을 선택한다.

## 계약과 범위

새 dependency/Cargo.lock/schema 없이 기존 defaultOFF first-paint-fixture feature를 계측 build gate로 재사용한다. 기존 first-present inherited3caps/4096mapping/ACK protocol은 바꾸지 않는다. 별도 명시 환경 opt-in과 원 HWND/현재 thread·private desktop 확인을 모두 충족할 때만 숫자 properties를 발행한다. 경로·입력 텍스트·stdout·사용자 desktop 조작을 하지 않는다. 기본 feature OFF 오버헤드는 없다.

후보 파일은 crates/app/src/frame_phase_fixture.rs, software_host.rs 및 accessible_cpu_main.rs의 cfg module 연결과 새 scripts/windows_frame_phase_cost_probe.cs/collector다. 공용 feature 추가가 필요하면 root가 정확한 비용과 대안을 검토하며 자동으로 추가하지 않는다. 숫자 counters·CPU/wall·cachehit/frame수는 상한과 overflow/queryfailure 표식을 갖고 generation 전후 일치로 관측한다. zero는 +1 encoding하여 누락과 구분한다. 원 HWND 숫자를 pointer·프로세스 제어 권한으로 사용하지 않는다.

## 인수 조건

계측 없는 기본 artifact의 기존 proof는 보존한다. feature build는 별도 immutable artifact와 source/config/원nativeJob proof를 기록하고 default 비용 인수로 쓰지 않는다. 최소60초/6구간 관측의 phase합·원main counter·cache/frame수와 non-atomic boundary 한계를 공개한다. absent/odd/torn/overflow/CPU조회 실패는 계측 실패로 남긴다. 기존 resize/숨김/테마/첫present protocol과 defaultOFF 동작을 확인한다. 타이머나 CPU/메모리 기준을 완화하지 않는다.

## 검증과 handoff

가설은 반복 프레임의 UI/AccessKit 비용, 첫 구간의 래스터 cachemiss, frame 밖 native 이벤트 비용이다. DisplayKey의 반올림 억제와 show_rows virtualization은 이미 존재하므로 중복 구현하지 않는다. 원source577 실패는 main78.125ms/library46.875ms/churnfalse, Job1.247473%이고 메모리/정리는 통과했다. 계측 초안은 아직 실행 전이다.

root는 scripts/windows_msvc_frame_phase_build.py를 구현했다. 기존 MSVC configuration을 재사용하고 feature만 cpu,first-paint-fixture로 지정하며 별도 fresh target-frame-phase-20261006에 fmt/clippy/test/release를 수행하도록 했다. 기본 MSVC CPU/CLI·GNU CPU/CLI·Cargo.lock·configuration SHA/bytes를 전후 대조하고 진단 exe를 immutable copy한다. 해당 producer의 Python syntax/--help는 통과했으나 native build는 아직 실행 전이다. 새 target 기존8.5GiB workspace cache를 재사용하지 않아 build 시간과 임시 디스크 사용이 늘어날 수 있으며 프로젝트D드라이브 가용331GiB를 확인했다. 원 기본 바이너리 성능 인수로 사용하지 않는다. 문서 검사824개 PASS.

root는 원 SHA와 exact diff를 대조한 뒤 frame_phase_fixture.rs·software_host.rs·accessible_cpu_main.rs와 C# observer/Python collector를 통합했다. CPU query는 같은 UI thread capability에서 읽고 stage CPU 및 total/wall·cachehits·early return counter를 원 HWND 고정12숫자 properties로 발행한다. 정상 종료 직전 final sample을 사용해 present 뒤 cache bookkeeping도 포함한다. PresentCpu100ns는 실제 present와 후처리를 합친 값이며 optional 첫present hook이 활성화된 경우 그 비용도 포함한다. hidden counter는 숨김뿐 아니라 오류 등 모든 미완료 frame을 뜻한다. 별도 standalone typed 실패 테스트5개는 아직 native feature test 실행 전이다. Cargo.lock/dependency/feature 추가는 없다.

실제 Windows 포맷 적용은 exit0·원Job자연0·private/input 복원을 확인했다. source_unchanged=false는 포맷 변경 자체이며 format-check PASS로 바꾸지 않았다. default CPU feature 전체4검사를 실행 중이다. 독립 검토 의견에 따라 producer/collector가 원 실제4명령·workspace·feature·MSVC target·locked·clippy strict 인수와 exact source를 검증하도록 보강했다. 기본4검사 및 계측4검사 후에만60초 진단을 실행한다.

기본 feature 실제 전체4검사와 root/독립 검토 PASS: source578 exact map·760passed/0failed/13ignored, 기본CPU16b4d0e…8,709,632B·CLI a0bb2ab8…795,136B, 원GNU/lock 보존. [원검증](../validation/evidence/windows-msvc-frame-phase-default-workspace-20261006-continue/result.json). CPU는 크기 동일/SHA 변경이며 이전 source577 비용 결과를 새 binary에 소급하지 않는다. defaultOFF 계측의5개 시험·FFI·실제GUI는 별도feature build/60초 인수 전이다. 협조 Stop을 정리 전에 게시하고 worker 미회수 시 원cap을 유지하며 공유 mutable 결과를 직렬화하지 않는 observer 실패 경로도 root가 통합했다.

새 observer의 실제 Windows C# 컴파일은 exit0·원Job자연0·source불변·private/input 복원 PASS다([컴파일 기록](../validation/evidence/windows-frame-phase-observer-compile-20261006-continue/result.json)). 이는 실제 observer 실행·Stop timeout 경로·60초 측정 검증이 아니다. 기존 계측 defaultOFF feature 전체 MSVC 검사를 별도 target에서 수행 중이다. 비용 시험 중에는 다른 dlpad native probe를 실행하지 않는다.

원 source577 실패의 독립 보충 검토에서30,000 합성파일 이름·3B payload01,02,03와readonly DB3×10k exactpath/size/present·integrity/전후SHA 보존을 확인했다. 원wrapper는 FAIL/source_after미발행 그대로다. 최초577 source/pin 읽기는 mismatch0였으나 이후578 통합과 순차 관측 시간 차이를 공개하므로 현재 source578==577으로 주장하지 않는다. [보충 원검토](../validation/evidence/windows-msvc-source577-current-gui-cost-20261006-continue/independent-review.json). frame CPU는 시작clock조회 뒤부터 정상 종료 전sample까지이며 초기 query와 numeric property 게시 비용은 누락될 수 있다. 따라서 main counter-frame counter 차이를 전부 native event 비용으로 단정하지 않는다. Present 단계는 cache bookkeeping·후처리를 포함한다.

별도 feature 실제 fmt/clippy/test가 통과했고 release가 진행 중이다. actual test 집계770passed/0failed/13ignored이며 frame bound·산술 overflow·phase합 불일치·실패/숨김 분리·부분 성공 방지의5개 새테스트가 모두 실제 Windows에서 통과했다. 첫present feature의 기존 시험도 포함되어 기본760 집계와 차이가10개다. 전체4검사 완료나 GUI 측정 완료로 아직 표현하지 않는다.

계측 feature 실제전체4검사·root/독립 검토 PASS: source578, test770/0/13, CPU27ee32c2…8,733,184B. 기본CPU 대비23,552B 증가한 진단용 artifact이며 defaultfeatureOFF 기본CPU16b4d0e…를 보존했다. 새runtime dependency/Cargo.lock 변경은 없다. [원4검사](../validation/evidence/windows-msvc-frame-phase-feature-workspace-20261006-continue/result.json). 진단 observer 실제60초/6구간 원Job·privateGUI 수집을 시작했고 terminal 후 counters·readonlyDB·합성payload와 immutable pins를 root/독립 대조할 예정이다. GUI 인수는 아직 미완료다.

첫 실제60초 계측은 DIAGNOSTIC_ONLY로 자연exit0·원Job0·private/input복원됐다. 원6snapshot/delta를 root가 재계산했다: frame48/cachehit43, 첫10초23frame/18hit·wall14,087µs, 이후 각5frame/5hit·wall2,730–3,057µs. CPU phase와frame total은 원counter에서6구간 모두0인 반면 main thread는 일부구간15.625ms를 기록했다. 짧은frame 분해능 한계와 query/postpublish 제외 때문에 CPU없음·전부프레임밖 원인으로 단정하지 않는다. [원진단](../validation/evidence/windows-msvc-frame-phase-cost-real-20261006-continue/collector-result.json). 예산 초과 재현/제품비용 인수/원인 해결은 미확정이다.

다음 계측은 단계wall과동일원thread QueryThreadCycleTime의boundedcycles를 추가한다. cycles를경과초로변환하지 않는 Microsoft API 계약을 지킨다. 원입력/한계·defaultOFF·고정window·예산은 유지한다. 이전진단 증거는 보존한다. producer에는 완료한 원target의marker/evidence를 검증하는 명시적--reuse-owned-target만 추가하여 새 전체빌드 때 기본app·원targetmarker·GNU/lock보존을 검증한다.

첫진단의 root/독립 숫자·10,000합성파일·readonly DB·initialsource578/pin 대조를 완료했다. 숫자 진단 범위만 인수하고 작업 전체는 in-progress다. 독립 검토에서 다음cycle/wall collector의 단계 합계·window continuity 가드 누락을 발견해 root가 보강한다. Rust cycle API는 windows-sys0.61.2의 Threading namespace가 아니라 WindowsProgramming에 있으므로 새feature를 추가하지 않고 kernel32의정확signature extern을 사용한다. frame 사이에도 동일원thread cycle 누적단조검사를 유지한다. 새코드의실제4검사와재측정 전에는 보강완료·원인수정을주장하지 않는다.

cycle/wall 보강 후 실제 기본4검사·feature4검사와 root/독립 검토 PASS다. 파일수578은 같아도 module hash가 바뀐 새 snapshot이다. 기본760/0/13·CPU16b4d0e…8,709,632B·CLIa0bb2ab8…795,136B는 보존했고, feature775/0/13·계측10개 exactok·CPUcaeedf2d…8,734,208B로 기본보다24,576B 큰 진단 artifact를 따로 보관했다. 원GNU/lock/완료targetmarker/config·immutable 첫feature artifact도 대조했다. 새로운60초 cycleGUI 측정을 실행 중이고 기준CPU/memory나window범위는 바꾸지 않았다.

새cycle60초도 root/독립 숫자·원10kpayload·readonlyDB·source578initialpins 대조를 완료했다. frame48/hit43·framewall합27,494µs, UI11,870/tess2,683/paint2,887/present9,986µs·framecycles103,201,084를 기록했다. 이후cachehit5frames/10초 구간은 paintwall0µs지만nonzero cycles가 있어 '0경과값=0작업'이 아님을 실제관측했다. 원frame CPU 합31.25ms·Job최대0.780599074%, 자연exit0/Job0·private/input복원이다. [cycle원진단](../validation/evidence/windows-msvc-frame-cycles-cost-real-20261006-continue/collector-result.json). 순차·동일hardware가변의계측표본이며 cycles를시간으로환산하지 않는다. 기존1.247473% 실패 재현·원인수정·기본artifact18창성능 인수는 여전히별도다. 반복frame의metrics freshness예약을 read-only가설로검토하며 지원기한을 제거하거나예산을완화하지않는다.

후속 병렬 담당: metrics_recovery는 동일 화면의 반복 frame wake를 원 timer/event 이유와 연결하는 bounded/defaultOFF scalar 계측 초안을 /tmp에서 검토한다. root는 공유 HWND property protocol·software_host·frame_phase_fixture 통합을 전담한다. 기존 stale3초·repaint·접근성 정책을 변경하지 않으며 source native 계측 전 원인 확정이나 성능 해결로 표현하지 않는다. 설치 전체 검증 중에는 프로젝트 Rust 파일을 수정하지 않는다.

후속 wake 원인 계측4파일(Rust module/host·C#·Python)은 /tmp/dlpad-frame-wake-stage에 exactdiff·원SHA/stagedSHA로준비됐고 독립읽기검토를받았다. 총33개 scalar property로기존21개와고정12개 bounded100만 wakecount를원generation+1프로토콜로묶는다. Root/Callback/Action의earliestdeadline·tie·Action override와Window/AccessKit/Redraw/Close경로를구분하고 Destroyed는repaintcount에서제외한다. 이는semantic metrics caller확정이나성능해결이아니며 publication마다33개SetProp의계측비용이있다. 현재설치native검증의source를보존하므로프로젝트Rust/C#계측통합·실제compile/60초진단은아직미실행이다.

wake4파일 root통합/Windows실제fmt 후 기본전체4검사 root/독립PASS(774/0/13)다. CPU984226c2…8,709,632B·CLI97217440…841,728B, exactsource579map/GNU/lock 보존. 진단feature 첫전체4명령(raw792/0/13)은통과했으나 마지막source/default/GNU/lock/config보존에서wrapperFAIL이다. root가기본/진단target이별도라는사실만으로동시실행했으나진단producer는기본CPU보존SHA를고정하는의존성이있었다. 기본release가CPUa6d9223a…→984226c2…로바꾼것을거절한원FAIL을보존하고기본완료뒤fresh출력의진단검증을다시실행한다. 이실패를성공으로고치거나보존검사를완화하지않는다. 신규schema/dependency/lock변경0; feature계측오버헤드는기본성능인수와분리한다.

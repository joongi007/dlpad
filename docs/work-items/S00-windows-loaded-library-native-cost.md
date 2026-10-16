# S00: Windows 합성 10k loaded-library steady-idle 비용

- 상태: in-progress (독립 staging)
- 분류: validation
- 우선순위: P0
- 요구사항: R03, R06, R11, R13
- 담당/수정 범위: metrics_recovery — `/tmp/dlpad-windows-library-cost-probe.cs`, 이 문서만
- 선행 작업: 원 Windows watcher/loaded-library GUI 및 DB 인수, 최신 frozen CPU binary

## 사용자 결과

합성 10k 폴더가 실제 목록에 열린 상태에서 fresh3회 각10초 steady-idle 비용을 확인한다. empty-library 기존 비용과 구분하며 loaded scan 자체 비용 또는 최종 스택 채택 완료로 확대하지 않는다.

## 계약과 범위

기존 windows_idle_native_cost_probe.cs의 원 Job 구성원/HANDLE/creation/image/parent/native accounting CPU 및 합산 WS/private sampler를 복사한다. 처음 recovery ready를 확인한 뒤 파일 목록을 열고 실제 media00000/로컬 파일 label과 원 root Edit/Apply 활성화를 확인한다.100ms×10회 안정 관측과2초 준비 후 샘플을 시작한다. initial row 하나만으로 scan완료를 판단하지 않는다. 다만 native watcher 구독/백그라운드 scan in-flight를 GUI에서 직접 표시하지 않으므로 controls 안정은 직접관측이고 전체 scan actor 무동작 증명은 아니다. root의 actual read-only DB10k 대조가 후속필수다.

원90초 worker/10초 sampler/128 samples 및80MiB/CPU1%를 유지한다. UIA/PNG/WM_PAINT는 비용구간 밖에서만 호출한다. whole owned Job CPU와 pinned 모든 구성원의 WS/private합을 기록하고 역할 미확인은 REVIEW_REQUIRED다. OS WorkingSet trim/budget 상향은 없다.3fresh 자체 protected SIDroot/data/media에3byte 합성파일만 만들며 실제Documents/media/계정/클립보드/globalinput/visiblefallback은 없다. quiet_host_verified=false이며 다른nativeprobe 동시실행 없음은 root 운영조건이다.

## 인수 조건

root fresh compile→3회 loaded actualUIA/한글PNG/원binarysourceSHA→원 Job 전체 비용→자연exit0/Job0/desktop close/input 불변. 각 fixture10k DB/scopedroot 및 syntheticpayload는 root독립postreview로 확인한다. 실패 raw와 budget 초과는 그대로 유지한다. 데이터/fixture는 검토를 위해 보존하고 자동삭제0이다.

## 검증과 handoff

현재 native/Cargo/프로젝트Rust/scripts 수정은0이다. root 직렬 native실행 전에 staged SHA를 인계한다.

### Staging READY

원 sampler source SHA256 `11d5810a66c78b21572c3b2022e33c811415ea837e778fbc5a93622b931be09c`; staged `/tmp/dlpad-windows-library-cost-probe.cs` SHA256 `0112ea6c48aed7069fa732f1a3acd5fa842bfa4b6b6148eb247f5c4148359688`,39379B. 초기 create-new10k 파일은 비용구간 밖에서 만들고 각File을 닫으며 비용 측정을 포함하지 않는다. library 준비는 최대30초 row+enabled 관측 후1초 안정상태로 제한한다. 준비/end PNG와 UIA는 전체샘플 밖이다. 기존5인자 CLI를 그대로 유지하며 마지막은 --no-concurrent-dlpad-probes다. first observed glyph capture는 현재 loaded 준비 후의 첫 collector 캡처이고 physical first paint가 아니다.

root watcher actual native/DB/PNG PASS는 독립 선행근거이며 새 비용 수집기의 성공은 아직 없다. C# compile/native실행0, source정적검토만 수행했다. 추가 native는 root 직렬실행으로 인계한다.

## root 실제 측정: 기준 초과

SHA검토 후 scripts/windows_library_cost_probe.cs를 통합하고 compile PASS, 원native3round terminalFAIL을확인했다. 각10초 wholeownedJob(app1) 결과는 round1 WS84668416 bytes/CPU0.31155%, round2WS79466496/CPU1.08394%, round3WS79773696/CPU1.24777%이다. 원80MiB/1%기준을유지해 각각memory/CPU/CPU 초과로BUDGET_FAIL이며 모두자연exit0/Job0/privateDesktop닫힘/input불변이다. windows-library-loaded-cost-20261004-resume/root-review.json에기록했다. hostquiet미검증/DB10k독립후검토는남으며 원인은아직확정되지않았다. 가설은 watch 초기화·추가스캔 겹침, 변경없는목록주기repaint, 스캔·폰트·renderer잔류allocation이다. 다음검증은구간별scan/repaint를구분하며budget상향/WorkingSettrim을하지않는다.

## 실제 3회 비용 실패와 독립 진단 초안

[원 loaded 비용](../validation/evidence/windows-library-loaded-cost-20261004-resume/gui/result.json)은3회 모두 BUDGET_FAIL이다. CPU0.31154937/1.08393937/1.24777096%, WS84,668,416/79,466,496/79,773,696B로 첫 회 메모리 초과 및 두 후속 CPU 초과를 그대로 보존한다. 기준을 높이거나 첫 실패를 삭제하지 않는다.

별도 `/tmp/dlpad-windows-library-cost-diagnostic.cs`를 만들었다.10초 비용 구간에서5개의 약2초 창을 기록하고 original app CPU와 whole Job accounting CPU 차이 및 원 app GetProcessIoCounters(6개 counter)의 창 전후를 저장한다. 실제 monotonic windowduration으로 계산하고 마지막 창은10초 경계에 맞춘다. I/O는 app 범위이며 wholeJob I/O로 확대하지 않는다. counters 실패/identity 변동은 실패다. UIA/paint/screenshot은 비용창 밖이고 새 thread는 없다.80MiB/1% 및 worker90초/native host360초 계약은 바꾸지 않는다.

원2초 준비를 기본 비교군으로 보존하고 새 명시 CLI 마지막 인자 `--warmup-seconds=2` 또는 `--warmup-seconds=10`만 허용한다. 다른5인자는 기존계약과 같다. 결과에는 diagnostic_variant 및 diagnostic_warmup_seconds를 기록한다. source읽기로 initial watch 후속scan과30초 주기성 가능성을 조사할 수 있지만 창별 CPU/I/O만으로 실제 causal stack을 확정하지 않는다.

진단 staging SHA256 `0066c9242044c0d68eb3bdce62e6c6bd70195d9d135df4e6f5dc41a06052b6c2`. 아직 C# compile/native실행0이다. root가 별도 fresh 비교를 수행하며 원 sample3FAIL은 그대로 유지한다. 프로젝트 source 변경0.

## 첫 cold WS 잔여 실패의 읽기 감사

root 최신 비용은 CPU0.46482/0.93230/0.78064%로 모두1% 아래지만 첫 WS84,467,712B만80MiB를 초과한다. 후속79,474,688/79,187,968B는 통과했다. 첫 baseline84,668,416→84,344,832→84,467,712B의 반복 초과를 보존한다. private peak약49MiB와 WS를 혼동하지 않으며 실제 shared량을 WS-private로 계산하지 않는다.

가설1: 실제 renderer 캐시 canvas. software_host.rs359은 기본 cache=true painter를 생성하고 upstream renderer는 Canvas.data(width×height×4)를 보유한 뒤 softbuffer로 blit한다.960x640에서2,457,600B의 추가 소유 canvas가 source상 확인된다. with_caching(false) 대조는 private/WS 감소와 repaintCPU 증가를 예측하므로 exactPNG/스크롤/resize/hidden복귀/1%비용까지 검증해야 한다. 현 제품은 바꾸지 않았다.

가설2: 폰트/atlas 복제. epaint0.33.3 text/fonts.rs208은 owned FontData bytes.clone을 FontVec에 전달한다. 원13,459,196B가 parser에 복제되는 경로가 확인되며 atlas도 egui와 swizzled renderer 저장소가 있다. 원 font allocation 개선으로 이 parser복제까지 제거한 것은 아니다. dimensions/capacity 값 계측에서 중복이 예상되나 첫cold만의 차이를 이 사실만으로 확정하지 않는다. 한글폰트 제거/static leak/권한 없는 mmap 수명 확장은 제안하지 않는다.

가설3: cold mapped/shared resident. UIA/provider/이미지 mapping의 resident 증가가 privatecommit 증가 없이 WS에 나타날 수 있다. 원 app QueryWorkingSet/Ex의 aggregate 공유/비공유 resident를 ready/start/end에 기록하면 구분할 수 있다. foreign 프로세스/주소출력 없이 own HANDLE만 사용하고 같은 UIA/PNG lifecycle을 유지해야 한다. shared resident도 원80MiB기준에서 빼지 않는다.

독립 `/tmp/dlpad-loaded-memory-hypotheses.md` SHA256 `ccb1ca02b186d258a650ede4c9e656b11f1246b0b996b0b0387d55fcf085ae92`. 읽은 software_host SHA `e0dd2e404067ee8a613111c1670c73536cb48a91afcf2db2f77187231faef06b`, accessible SHA `4f90a583cb61bd2f57725b2599afdba83544b4da5790e4affb3f27e61296b594`. native/Rust/Cargo추가변경0. root DBred16251 진행 중이며 실험은 후속조정한다.

2026-10-05 최신CPUdfae에기존loadedcollector를동일고정5argv로실행한다. root전용/tmp wrapper,3회소유10k×3Bfixture이며빈화면시험/이전CPU결과와구분한다. 다른native성능시험없음·exactsource/binarypin·private화면·naturalJob0·원DB/files독립검토를유지하고1%/80MiB를완화하지않는다.

최신loaded 첫3회실측에서CPU1.5552/0.7756/0.3105%, sumWS70,221,824/70,533,120/70,582,272B다. 첫회차CPU1%예산 초과로전체FAIL이며단독좋은2회차를선택해PASS로바꾸지않는다. source/binary원SHA·naturalJob0/private정리정상,원결과보존. watchworker/rootmetadata/30초reconcile와UIpager/repaint에대해원threadCPU계측가설을검토한다. 새계측collector scripts/windows_library_thread_cost_probe.cs는root통합예정이고하위는/tmp초안만담당한다. 제품/Cargo변경전재현·원인instrumentation을우선한다.

root가thread 계측초안을diff검토하고원processcreation·QUERY_LIMITED_INFORMATION·GetProcessIdOfThread/GetThreadTimes핀·원PI mainTid만명시분류·최대256threads/churn미확인·원Job기준유지를확인했다. baseline은Job보다앞/terminal은뒤여서소량관측비용포함을표시한다. 추가pin목록상한을강화하고기존고정3×10sec/nativecleanup으로actual재현한다. budget및scanready정책변경0. 독립검토가원3×10k×3B+DBpresent10k/integrity/source535를확인했다.

첫thread계측은C#compiler의short-circuit out값 definite-assignment CS0165로compileFAIL했다. 원Jobnatural0/desktop정리증거를보존했다. root가constructor출력변수초기화만교정했으며실제identity/타임조회Require는유지한다. fresh계측을재실행한다.

thread변수교정 actual3회10sec는0.3116/0.3113/0.1555%·원24/24/25threads 안정으로PASS했으나원1.5552% failure의원인해결로취급하지않는다. 원watch30초와겹치도록새60sec collector를rootdiff검토해게시했다. 3×60초/최대750표본, fixed비중복10초6window의원JobCPUticks/duration/WS를모두검증하며한window라도1%/80MiB초과면실패,좋은60초평균으로실패를덮지않는다. readiness/stabilization불변,worker150초/outer470초로유한상한유지;제품Rust/Cargo변경0.

# S01: 주기 worker CPU counter에 따른 부채 대기

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: root core library_budget.rs·engine thread_cpu.rs/lib.rs·app accessible_library.rs·공통 문서·native 증거; metrics_recovery /tmp pure정책 초안; diagnostic_cache /tmp Windows counter reader 초안
- 선행 작업: 2ms/500ms와1500µs/500ms 후보 whole Windows 및 GUI CPU 비교

## 사용자 결과

wall 작업 시간만으로 제어되지 않은 CPU 구간을 실제 주기 worker thread의 CPU 사용량에 따라 분산한다. 명시 명령·실제 변경·복구·Stop 우선순위는 유지한다.

## 계약과 범위

core는 actual cumulative100ns ticks와 monotonic 시간만 받아0.4% thread 예산 초과분을100ms 이하 조각으로 기다린다. engine은 Windows 원 caller thread ID/creation을 검증한 GetThreadTimes reader를 제공하고다른 thread이동을막는다. UI worker가100ms이하 sampling/checkpoint/기존cause와명령보관을조정한다. 독립 OS/SQLite 호출과 counter 해상도의 overshoot는hard quota가아니다. 원180초기한/30초요청/GUI1%기준·fullfresh메타데이터/rollback은유지한다. Cargo/dependency/schema/추가thread0. 다른OS는wall정책만의미검증후속이며Windows확인으로표현하지않는다.

## 인수 조건

가상CPU/시간 비율·quantized overshoot·no initialcredit·backwards counter·overflow·Wait100ms상한 실패회귀. 실제Windows caller counter/creationguard. 현재whole fmt/clippy/test/release, 실제Next/Stop/완료≤180 및GUI60초3회 원avg/strict10s/메모리. 새counter부하/완료지연을실측하고단독threadcounter를wholeJobCPU와혼동하지않는다.

## 검증과 handoff

1500µs후보 avg0.52034/0.52011/0.38997%, max10s1.09140/1.09192/0.77913%로2회strictFAIL. wall만으로P0완료를선언하지않는다. core 순수4회귀·engine reader 원identity/overflow/actualWindows 시험 및 !Send/!Sync compile-fail 문서시험을 통합했다. app은 Store 열기 전 baseline/100ms sampling/cached pause/전체 남은기한에 따른 wait상한을 연결했다. 실제 Windows 전체 검증 중이며 GUI/Next/Stop/완료 인수는 미완료다. 누적 idle 여유가 있어 sliding10초 quota를 보장하지 않는다.

### 실제 전체 검증

Windows 소유 CLI host에서 cpu feature의 fmt --all -- --check, clippy --workspace --all-targets --locked -- -D warnings, test --workspace --locked, build --workspace --release --locked 네 명령을 실제 통과했다. 전체source543개와현재CPUb2bd9d584810eb0f4d614ad27a414dd76857867fa64cfb059c09882a2a3853fe/7915008B의root대조PASS, 네명령naturalJob0/forcedfalse/privateclosed/inputunchanged도확인했다. core31/engine354PASS7ignored 및enginecompile-fail2PASS 포함이며 ignored는성공으로승격하지않는다. reader/policy의하위안전감사2개는차단결함없음이지만root실행검증과구분한다. GUI와최신functional인수는현재미완료다.

### GUI 原3회 결과

현재 b2bd9d58 source543의GUI60초3회 avg0.54632785/0.33807028/0.67676638%, max10초0.78021076/0.77962059/1.24870845%로round3첫창FAIL/전체FAIL을유지한다. 메모리3회PASS,원계산식18창·cleanup·identity독립감사도완료했다. 첫창은30초주기요청전이며phase원인을단정하지않고S00-gui-idle-window-attribution으로후속계측한다. 같은후보의defaultOFFfixture Clippy/앱tests는통과했으며최신완료/Next/Stop은아직미검증이다.

### 실제 최신 완료 인수 실패

표준CLI --case completion을Windows격리소유Job에서실행했다. defaultOFFfixtureCPU83e0aafa/source543은actualperiodicfirstINSERT/4caps/baselinecopy까지도달했으나180초commit신호가없어FAIL이다. [원결과](../validation/evidence/windows-periodic-thread-cpu-completion-20261006-continue/result.json). outerhostnatural0과innerapp강제소유Jobcleanup을구분하며innernatural완료로표현하지않는다. deadline/priority원인을아직단정하지않고단독threadCPU/phase실측후속및동일후보Next/Stop인수를이어간다. 원CPU/180초기한완화0.

### 지연 재현과 command 구분

같은fixtureCPU83e0aafa의표준NextCLI는관측firstINSERT후2.040초page2/원10000행exact/10000payload·CAP·sentinel/source·compiledinputbinding·자연Job0를통과했다. [Next](../validation/evidence/windows-periodic-thread-cpu-next-20261006-continue/result.json). StopCLI는45초내첫INSERT미관측으로선행조건FAIL이며Stop동작성공이나회귀로단정하지않는다. [Stop원실패](../validation/evidence/windows-periodic-thread-cpu-stop-20261006-continue/result.json).

분리된privateCargo의actualcore/engineclock·WatchSession·connectedidlechannel를사용한current1500µs/0.4%단독scan도180.008초에budget-deadlineCancelled로재현했다. CPU0.671875초/checkpoint69753/watchpoll1490/CPUquery1491,마지막128행과10000payload·원sampleID2개보존이다. 단독phase는GUI/명령인수가아니며9phase/profile비용을포함한다. 같은기한/CPU정책의2ms비교를직렬실행중이고root가productCargo/lock/schema를변경하지않았다. 추가가상반례로CPUdebt대기중wallactive창이먼저소비될수있음을확인한초안이있으나실제원인확정/수정검증은아직이다.

### 2ms/0.4% 후보 선택

privateCargo단독실험원두mode결과에서current1500µs는180.008초/CPU0.671875초/69753callbacks/1490polls로deadlineCancelledFAIL,experimental2ms는146.459초/CPU0.578125초/81428callbacks/1194polls로완료PASS다. mainCargo/lock/productCPU는원입력과불변이었다. 2ms작업폭만제품후보에반영하고0.4%/500ms/100ms/180초와GUI기준은유지한다. source543새hash의전체·nativefunctional·GUI인수는다시실행한다. CPUfirst가상반례/별도privatecomposed실험디렉터리는준비만했고실행·제품채택하지않았다.

### 현재 2ms 전체 검증

Windows cpu feature fmt/clippy/test/release 네 명령의 root-review가PASS다. source543/currentCPU71ec7e489a1e0d7f0ae4301c179353a1ccdf4a0c37d63cd35e1c345121c16c94와원Job/sourceguard를대조했다. 새GUI3회는평균모두1%미만이나round1 첫10초1.090614%로원strict기준FAIL이며S00후속원thread조사를진행한다. 현재2ms defaultOFFfixture featureClippy/앱시험을새evidence에서실행중이며최신완료/Next/Stop은미검증이다.

### 2ms 실제 완료 인수 통과

표준CLI --case completion의actualperiodicfirstINSERT/원4eventcap/원nativebinding과123825ms commit을확인했다. root runner가全10000 baseline/final행 exact/readonly integrity/原10000payload·CAP·foreignsentinel 보존을실행하고자연exit0/Job0/privateclosed/inputunchanged와최종source/compiledpins를대조해PASS했다. [원결과](../validation/evidence/windows-periodic-thread-cpu-2ms-completion-20261006-continue/result.json), [root대조](../validation/evidence/windows-periodic-thread-cpu-2ms-completion-20261006-continue/root-completion-review.json). fixtureCPU03eaeffc0ccdb547d83af1ccd4e94b2ede4854e785943e198ef02b71c3078f5b이며기본CPU71ec와구분한다. allnative ID 재열기0/GUI CPU 인수아님/Next와Stop 후속은현재진행중이다.

현재2ms의Next 표준CLI도PASS: actualfirstINSERT후2039ms의page2, 전체10000baseline/final행exact, 10000payload/CAP/sentinel, 고정native입력binding, source/compiledpins, naturalJob0/privateclosed/inputunchanged를root runner로확인했다. [원Next](../validation/evidence/windows-periodic-thread-cpu-2ms-next-20261006-continue/result.json)。시간은DBcopy등을포함한관측gate후상한이며정확한Invoke latency가아니다. Stop/CloseWait는후속실행중이다.

현재2ms Stop의45초firstINSERT 관측선행조건도FAIL이다. 原innerapp強制Jobcleanup과기존실패를보존한다. 별도collector만75초bounded선행관측및실제wait시간field를추가했고同03eaeffc fixturePE/source543에서다시검증중이다. 제품180초scan/Next5초/StopACK5초/CPU1% 기준변경0. Sourceguard측정이끝날때까지제품Rust변경0이며frame reuse초안통합은보류한다.

75초bounded관측의2ms Stop는33739ms에첫INSERT를관측하고제어비활성·release전ACKfalse20회·release후workerACK·commitfalse·자연Job0와원10000행/payload·CAP·sentinel보존/sourcecompiledbinding검증을통과했다. [원Stop](../validation/evidence/windows-periodic-thread-cpu-2ms-stop-observer-20261006-continue/result.json). 이실측은45초보다짧으므로이전45초실패가관측기한때문이었다고확정하지않는다. 原FAIL과미확정원인은보존하고新PASS의동작범위만인정한다. 제품기준변경0. 후속renderer실험으로software_host.rs가변경되어이상의current2ms 기능결과는그이전소스증거다.

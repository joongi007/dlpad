# S01: Windows 인터페이스 관측 연결

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R01, R08
- 담당/수정 범위: diagnostic_cache — /tmp/dlpad-windows-interface-counter-20261012/ prototype; release_preparation — /tmp/dlpad-windows-counter-integration-20261012/ 설계; root — 향후 crates/engine/src/metrics.rs·metrics_worker.rs와 UI 설정 통합, 공유 Cargo/lock 조정, 이 문서
- 선행 작업: 현재 진단 UI 통합 인수와 공유 관측 인터페이스 확정

## 사용자 결과

Windows에서 선택한 NIC의 실제 수신·송신 속도를 작업 다운로드 속도와 구별해 본다.

## 계약과 범위

실제 GetIfTable2/GetIfEntry2/FreeMibTable을 worker에서 사용한다. 원 LUID/InterfaceGuid와 alias·counter reset/down/stale를 다루고 같은 이름의 NIC 재생성을 새 baseline으로 처리한다. Linux 동작을 유지한다. IP/MAC·payload·비밀을 수집하지 않는다. windows-sys0.61.2의 IpHelper/Ndis feature 두 개는 root가 비용과 함께 조정하며 하위 에이전트는 Cargo/schema를 쓰지 않는다.

## 인수 조건

Windows 실제 API 결과·Unicode alias·일관된 counter 및 reset/down/stale 실패 경로를 검증한다. UI가 긴 조회로 막히지 않고 stop ACK·idle 비용·bounded 출력/선택 오류를 계측한다. 기존 UI64자와 Windows UTF16 alias256자 차이를 명시적으로 처리한다. 지원 성공을 zero/미지원 표시로 대체하지 않는다.

## 검증과 handoff

prototype과 읽기 설계 후보가 /tmp에 있다. 실제 NetworkSample 계약은 core가 아니라 engine metrics.rs에 있으며 alias만 비교하는 rate 계산은 NIC identity gap이 있다. OS 테이블 allocation 전 상한과 동기 API의 시간 상한은 아직 보장하지 않았고 실제 비용/stop ACK 인수도 미실행이다. 프로젝트 구현·새 Cargo feature·native 관측은 아직 수행하지 않았으며 Windows 지원 완료로 주장하지 않는다.

- UI native 검증과 독립적인 /tmp engine 후보 준비를 diagnostic_inputs에 요청했으나 model capacity 오류로 해당 turn이 종료됐다. engine 후보 완료/실제 작업 중으로 계산하지 않는다. root가 이후 재배정하거나 직접 진행한다. 프로젝트/Cargo/lock/native 변경은 없고 R01 제품 구현은 여전히 미완료다.

- 후속병렬준비담당release_preparation에/tmp/dlpad-windows-counter-engine-candidate-20261012의현재engine계약을유지하는NIC구현/실패회귀후보준비를요청했다. 프로젝트/Cargo/lock/native0이며root가공유feature/통합을조정한다. baseline/파일분담확인후진행하고제품Windows관측구현완료로해석하지않는다.

- root가원Counter의TableRAII/256row traversalcap/LUID+GUID epoch/원sample schema유지후보를읽었다. UI소스동결과독립된standaloneWindowsAPI실험후보를release_preparation에/tmp전용으로준비하도록위임했다. 실제GetIfAPI/latency/1초query/stopACK/실패관측을root가새project-ownedtemporarycrate에서실행하기위한harness이며mainCargo/lock/제품지원완료는아직변경하지않는다. IP/MAC/payload/GUID로그와외부트래픽0계약을유지한다.

- standaloneNIC harness준비turn은modelcapacity오류로종료됐다. /tmp부분Cargo/src를완료나실행중으로계산하지않으며원engine후보는보존했다. root가완료된후보를기준으로독립harness준비를직접이어가거나재배정한다. 제품관측지원여전히미완료.

- root가capacity종료후/tmp부분harness의Counter/read-onlyAPI/sync1mailbox·stop후drain/ACK·122sample/16alias·120초상한/noIP·MAC·payload·GUID/no실트래픽scope를직접읽고새project-owned .local-tools/windows/tmp/dlpad-counter-native-harness-owned-20261012에원3파일/SHA·owner marker를스테이지했다. 제품workspace/Cargo.lock변경0. 원MSVC환경에독립targetdir를써offline generate-lockfile95056실행중이며컴파일/API/비용은아직미실행이다.

- 원캐시독립lock95056 exit0/isolatedbuild59952 exit0:windows-link0.2.1/windows-sys0.61.2+2NIC feature와실험crate의WindowsMSVCrelease컴파일을통과했다. 제품소스/lock/source snapshot불변. 원실험failuretests51399와API10초관측55962를별도로실행중이며실제API/worker종료인수는아직결과확인전이다.

- standalone nativetest51399 exit0/6failurepolicyPASS(sourcepreserved). 실제API run55962 exit1:InvalidData/native_codeNone만반환해실패stage미관측. 원raw/ownedJob자연종료와실험source보존. typedstage/원rowlayout·count/UTF16termination·decode/aliaslen/duplicate·cap를개인정보없이관측하는새instrumentation후보를위임했다. rowpolicy완화/실패skip이나Windows지원주장을하지않는다.

- root가새typedtrace의bounded64+overflow1/no원alias/IP/MAC/GUID출력/원row검증불변·원manifest동일을검토해독립stage/정확old standalone lock을복사했다. tracebuild7626 exit0/sourcepreserved. 실제3초trace와6개회귀를시작했으며실패원인/제품지원은미확정이다. 원harness/stage/log보존. mainworkspace/Cargo.lock변경0.

- traceactual24208 exit1/6tests63686 exit0:generatedtargetrow1352/align8/tableoffset8/alias28·cap257,GetIfTable2success/code0/NumEntries31·row17에서eligiblelist16cap에 도달했다. table1/free1/entry0으로원API tablefree확인. 이는experiment aliases목록16상한이고제품Counter.lookup의256row조건과다르다. root가새독립observationstage에서aliaslist도256으로맞췄으며row검증/duplicate/UTF16/identity는유지한다. 추가최대240aliases·최대256UTF16문자에따른bounded추가heap은약240KiB문자payload+Stringoverhead이며제품worker추가비용으로소급하지않는다. 원fail/log/stage보존/제품source/Cargo.lock변경0. 새build실행중후실제10초관측재검증한다.

- 보정isolatedbuild41616/native6tests83844/actualrun76166 모두exit0. 실제eligible21/10066ms/11samples/errors0·GetTable2calls2(max878300ns)/Entry2calls10(max199500ns)/Free2·workerjoined/stopACK0ms관측. short독립API/정책실험만통과했고제품worker/UI지원·실제idle비용·상한·reset실OS인수는남는다. 원16cap실패보존/제품Cargo.lock변경0.

- root 독립 검토: 원 build/test/run guard와 각 역사 source 전후·서로 map 일치, 소유 harness source pins,6policy tests, 실제10,066ms/11samples, GetIfTable2/GetIfEntry2/FreeMibTable2/10/2회·오류0·workerjoin/stopACK0ms를 대조해 observation-run/root-independent-review.json에 기록했다. 현재exe SHA는 대조했으나 과거 compile-to-binary provenance는 미증명이다. 제품 UI/R01 지원·OS API 강제 취소 상한·idle 비용은 별도이며 완료로 주장하지 않는다.

- root가 최신 source SHA를 원후보와 대조해 engine NetworkObserver/native Counter 및 기존 sampling worker/UI 연결을 통합했다. 수정 파일은 engine metrics.rs/metrics/windows_counter.rs/Cargo.toml, app accessible_metrics.rs/catalog.rs/accessible.rs 회귀와 문서다. 공유 공개 sample/rate/schema 및 Cargo.lock은 보존하며 Windows API feature2개만 추가했다. 기존 Linux 정책은 유지한다. Windows지원 후 미지원 문구를 요구하던 Candidate 회귀의 대상 OS를 수정했다. 아직 제품 build/native/비용 및 자동 선택 UX는 미완료다.

- 관련 실행92076 exit0: 68성공/0실패/1ignored. root가 원 raw의6 Windows Counter 정책 시험과 실수, 자연 종료·source 보존·Cargo.lock SHA를 확인했다. 실제 제품 counter/GUI/자동 인터페이스 선택·저장 완료는 아직 입증하지 않았다.

- 통합 전체78175는 fmt 통과 후 clippy에서 미지원 OS 회귀용 edit_for_test가 Windows에서 unused임을 발견해 exit101이었다. 원 실패를 보존하고 해당 helper도 실제 소비자인 미지원 OS 회귀와 동일 cfg로 제한했다. 경고 억제 없이 테스트 대상 OS를 맞췄으며 v2 통합 검증을 이어간다.

- v2 통합36006 및 독립70204 exit0: 1205성공/0실패/30ignored. CPU9,783,808B/SHA `4d7619721ef3d81204533ac788dfbf193bae1e94ff183cad6f759b0fb786f5ae`. 고급 옵션 v2보다3,072B 증가, 신규 crate/version 없음/Cargo.lock SHA 유지. 실제 제품 counter와 GUI 및 자동 선택 인수는 남는다.

# S10: 실제 보고서 저장 worker CloseWait 결정적 fixture

- 상태: done (default OFF fixture feature의 Windows private GUI CloseWait 인수 완료)
- 분류: test
- 우선순위: P0
- 요구사항: R08, R10, R11
- 담당/수정 범위: soak_recovery_audit; 본 문서와 `/tmp` report save/barrier 초안만
- 선행 작업: 실제 private GUI report 저장 인수, 원 diagnostics worker stop ACK

## 사용자 결과

원 export 파일을 실제로 연 저장 worker가 아직 종료하지 않았을 때 닫기 요청이 앱을 종료시키지 않고, 해제 후 worker ACK를 받은 뒤에만 종료한다.

## 계약과 범위

고정 fixture command와 원 observer가 생성한 native event capability만 사용한다. 환경변수·사용자 파일로 barrier 활성화하지 않고 private desktop/exact handle inheritance를 유지한다. 제품 저장 정책/create_new/2MiB 상한/원 파일 정리는 그대로이다. 프로젝트 Rust/Cargo/scripts 및 native 실행은 통합 담당 소유다.

## 인수 조건

실제 create_new 원 파일 handle 확인 뒤 started, Close→StopExit 후 release 전 원 process HANDLE 미종료/worker stopped false/CloseWait UI 확인, release 후 실제 write+sync 및 worker ACK·앱 자연 exit0·export privacy/hash 확인. fixture wait는 유한 timeout이고 취소·timeout도 worker 종료 ACK를 요구한다. 빠른 저장을 race로 관측한 것을 인수로 대체하지 않는다.

## 검증과 handoff

최소 API 및 임시 구현, 실제 시험/미검증 구분을 기록한다.

## 현재 코드와 공백

`accessible.rs`는 diagnostics busy 중 닫기를 CancelClose하고 확인창을 표시한다. 중단 후 종료는 stop intent를 보내고 diagnostics.stopped까지 CloseWait한다. `accessible_diagnostics.rs` worker는 현재 save_report I/O를 마친 뒤 stop을 확인하며 StopAck가 stopped를 release-store한다. 기존 빠른 합성 export/순수 worker 테스트만으로 실제 파일 handle 획득 후 CloseWait가 유지됨을 결정적으로 증명하지 못한다.

## 최소 전달 제안

원 private observer가 manual-reset event 3개(started/release/worker ACK)를 생성하고 exact HANDLE_LIST duplicate만 앱에 상속한다. 명시 `--fixture --fixture-report-closewait`와 primitive handle 3개를 함께 요구하고 제품 환경변수·사용자 파일 활성화는 없다. 초기 dedup/상속 flag/type ResetEvent 검증 후 상속을 해제하고 RAII를 소유한다. GUI/다운로드/tool에는 이벤트가 재상속되지 않는다.

engine의 save_report는 noop callback인 기존 경로로 유지하고 explicit fixture callback은 원 create_new 파일을 연 직후만 받는다. 신규 barrier는 실제 DISK/regular/non-reparse/nlink1/size0 원 handle 뒤 started를 signal하고 release를 worker에서 최대20초 기다린다. stop intent로 이 대기를 조기 PASS하지 않는다. timeout은 고정 오류와 기존 원 partial 보존/cleanup 정책을 따르며 성공 export 인수가 아니다.

앱 Signals에 optional Arc barrier, fixture constructor, worker save branch의 callback 연결만 필요하다. native ACK는 worker loop가 반환한 StopAck::drop에서 stopped.store **앞**에 signal해야 한다. 반대 순서는 UI가 stopped를 먼저 보고 프로세스를 끝내 native ACK를 잃을 수 있다. ACK는 thread join 증명이 아니라 기존 worker 종료 acknowledgment와 같은 경계이며 export write/sync/hash 인수를 따로 요구한다.

## 임시 산출물과 잔여

`/tmp/dlpad-report-closewait-save-draft.rs` SHA `2e6d4b3c2a002ecacb69e823b4e6cf85550034ade79846646082de653e10bb1b`, event 모듈 `/tmp/dlpad-report-closewait-event-draft.rs` SHA `ce86420f8e253a5891bccb48afa92bcdb52213ab450d9a039c0ae0713a87cb8a`. 두 소스 rustfmt exit0. 프로젝트 Rust/Cargo/scripts/native 실행은 하지 않았다. engine callback의 노출 및 fixture-only compile 조건은 root 최종 결정 전 제안이다.

root 잔여 실제 시험: started+새 export size0 원파일 확인 → owned HWND Close/StopExit → 최소 관측창 동안 original process HANDLE 미종료·ACK unsignaled 및 private CloseWait 화면 → 원 release event → ACK 및 자연 process exit0 → export parse/privacy/hash/원 sentinel 보존. release 없이 timeout case도 failure raw 및 ACK/원 partial 정책을 따로 검증한다. 실제 native CloseWait PASS는 아직 주장하지 않는다.

## Feature 격리와 완전 통합 사본

후속 `/tmp/dlpad-report-closewait-integration/`은 현재 diagnostics 원 SHA `d18ae052d4c1c445e86fd447ade0b132a4f4226e690beea0779d8fc8523084d6`에서 시작했다. 기본 OFF `report-closewait-fixture` app/engine feature 제안을 포함한다. engine callback 공개 진입은 feature가 켜졌을 때만 컴파일하고 기본 save는 noop 경로다. 앱은 feature+Windows에서만 명시 CLI를 parsing하며 guardian entry 이후/GUI 이전에 optional process-local capability를 등록한다. panel constructor는 optional capability만 복사하므로 Candidate/UI 생성자 시그니처 변경은 없다. 추가 의존성 버전은 이미 workspace가 쓰는 windows-sys0.61이며 optional Windows fixture feature에서만 활성화한다. 실제 lockfile/크기/빌드 비용 확인은 root 담당이고 기본 runtime 비용 증가를 주장하지 않는다.

ResetEvent는 EVENT **타입만** 검증한다. manual-reset semantics는 원 observer CreateEvent 및 exact HANDLE_LIST 생성 증거가 담당하며 helper가 이를 독립 확인했다고 주장하지 않는다. 검증 중 이미 인수한 valid inherited duplicate는 즉시 File RAII로 소유하고 후속 오류에서 닫는다. 기존·invalid·중복 handle을 다시 소유하거나 전역 이벤트 이름/PID를 찾지 않는다.

준비된 meaningful feature 회귀: 실제 create_new 후 callback timeout 오류의 원 partial 정책 및 existing 파일·callback 미호출 보존, 실제 native release event 짧은 timeout이 worker ACK를 발생시키지 않음, 실제 diagnostics worker가 원 export 파일을 연 뒤 stop을 받아도 release 전 ACK/stopped가 false이고 release 이후 write/sync 및 native ACK/worker join이 확인됨. 회귀 소스는 아직 실행하지 않았다.

| 임시 파일 | 원 SHA | 수정 SHA |
| --- | --- | --- |
| reports.rs | 7a52a4103a62e2981fdac3b285f6e28924aca475eaf051ff6ea1b4554694b139 | 7fff6b061ab63e34f51f4dfd3d49b5dfbd1a22f255b2ea8f51d32388fd4a025b |
| accessible_diagnostics.rs | d18ae052d4c1c445e86fd447ade0b132a4f4226e690beea0779d8fc8523084d6 | da5ed92d99a329bbdf5e44464dca4760ed7847427717b7e83755181c3ca7ba9e |
| accessible_cpu_main.rs | 4d8b2f8ab44e3d841cb7168e189118a0c70a48020669ef36f4fd16a642df0e34 | e9738ed2f291b6082df08156f394c7652a1ddba9bdbf729a191ca19c08303890 |
| accessible_main.rs | 934be7439ae3dc518ede8ae80cd660c8defac410a0c163244808265bb05086ca | 3758dccabbdd4ac6d045401451c407ce8a3e9aef4b0ea85b86b884f1c7dd0a62 |
| app-Cargo.toml | c5c7e00b7b163e84b33e6b428eeff023f21c77cc015b5e1595ec200ac4f5992f | e47555d8bc5b5eaff853c54141812361152d928f7434c5f1b453b3b5158a715f |
| engine-Cargo.toml | d25de03d98bc9396d95c9bb1085be9b233ce33123a5f364848c67ba9b1136deb | ee23df9e1489179fd077dbc549e964094ddfe56dc7e66cbce5150b2980982b17 |
| report_closewait_fixture.rs | 신규 | a859798e895a9c6cc1eca6b4e65c5f04bfa7f5d5b82015cddf645c7a25dbbfc6 |

모든 원 source SHA 불변을 manifest로 확인했다. `rustfmt --edition 2024 --config skip_children=true /tmp/dlpad-report-closewait-integration/*.rs` exit0. 프로젝트 Rust/Cargo/scripts 게시는 하지 않았고 actual compile/native 결과는 root 통합 전 미검증이다.

후속 root의 Windows GUI subsystem 속성 추가 후 CPU main 사본만 새 원본으로 rebase했다. 새 `#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]`를 보존하며 위 표/manifest의 CPU SHA를 갱신했다. 원 root 속성을 옛 사본으로 덮어쓰지 않는다.

root가 manifest와 맞는 통합 사본을 제품에 게시했고 default OFF feature의 Windows feature 회귀를 session95347에서 실행 중이라고 전달했다. 본 문서 작성 시 아직 terminal 결과는 받지 않았으며 Cargo.lock 변경 없음도 root 관측이다. child는 해당 프로젝트 코드를 수정하거나 Cargo를 실행하지 않았다.

후속 root 결과 정정: 최초 feature 회귀는 `--locked`에서 신규 app → 기존 windows-sys dependency edge의 lock 갱신 필요로 **컴파일 전에 실패**했다. offline fresh resolver는 별개의 기존 yoke yanked 제약으로 실패했고 기존 lock을 보존했다. root가 기존 lock 기반 offline check와 explicit Win32_Security feature 연결을 진행 중이다. 버전 변경이나 회귀 실행 성공은 아직 없으며 위 임시 Cargo SHA는 제안 사본의 해시로, root의 추가 명시 feature와 실제 lock 검증 결과는 별도다.

## 실제 private observer 사본

`/tmp/dlpad-report-closewait-observer/windows_s10_report_closewait_probe.cs` SHA `a21c819392d29c04b916e27d6673c543bb64cf29392cc28bd2750021b77e159e`와 runner SHA `6e4e0c5ba4696dc2081bcfea899ef211a03d332f12ddfec769d48d485fe2e31f`를 준비했다. 원 `scripts/windows_s10_report_save_probe.cs` SHA `9c4c2962213d26c4a4dca1f0c5972c6ea3f02131b006dc7ea20b6a02b11856b4` 불변을 확인했다. Python compile exit0이고 C#/native 실행은 하지 않았다.

observer는 원 anonymous manual-reset 이벤트 3개를 생성하고 원 non-inheritable/duplicate inheritable flags를 조회한다. STARTUPINFOEX exact3 HANDLE_LIST와 suspended app → 원 Job assignment → Resume 순서를 사용한다. started 이후 original export FILE_ID_INFO 128bit 및 size0를 보유하고 owned WM_CLOSE/실제 StopExit 클릭·disabled 상태/PNG 및 20×50ms original process alive·ACK false를 검사한다. release 이후 ACK → natural exit0 → 원 export identity/parse/privacy/hash 및 natural Job0를 요구한다. 강제 Job 정리 뒤 0을 natural0으로 바꾸지 않는다.

timeout case는 release 이벤트를 보내지 않고 ACK/자연 종료까지 유한 대기한다. 18초 이상 관측과 원 0byte partial 유지로 기대 실패를 별도 기록하며 `closewait_inflight_verified=false`, `expected_timeout_failure_verified=true`다. 성공 보고서 저장으로 PASS하지 않는다. 두 case 모두 root DB/image 독립 인수 전 REVIEW_REQUIRED다.

실행 계약은 프로젝트 cwd에서 임시 Python runner에 fresh evidence output, immutable feature binary/SHA, synthetic fixture binary/SHA, 임시 observer source 경로를 명시하는 것이다. runner는 자체 build 없이 root가 준비한 immutable binary만 copy/pin하고 기존 hidden Windows CLI host로 observer compile/native 실행을 조정한다. 사용자 desktop/global input/clipboard·기존 script·프로젝트 Rust 변경은 없다.

## 실제 feature 회귀와 관측기 계약 교정

root의 `windows-closewait-feature-regression-20261004T133915Z` 실제 terminal PASS를 수신하고 [독립 검토](../validation/evidence/windows-closewait-feature-regression-20261004T133915Z/root-review.json) 및 원 test.json의 passed/source_unchanged를 확인했다. actual_opened_save_blocks_stop_ack_until_original_release는 CPU/accessible 두 대상에서, actual_release_event_timeout_does_not_ack_worker와 callback_failure_uses_original_partial_policy_and_preserves_existing도 통과했다. locked 검증은 기존 lock을 보존한 offline check로 app → windows-sys0.61.2 dependency edge만 추가한 뒤 통과했으며 버전 변경은 없다고 root가 독립 대조했다. 이는 실제 feature/unit worker 인수이고 private GUI 저장중 닫기 인수는 아직 아니다.

root 검토로 최초 observer 사본에 기존 save collector의 8argv/scenario choices가 남아 있고 runner loop가 release/timeout 대신 single/all을 보내는 오류를 발견했다. 그 사본은 실행·게시하지 않았으며 준비 완료 보고 범위를 교정했다. 최종 C# Main은 **7argv**(binary/SHA/fixture/SHA/output/failuremode/release 또는 timeout)만 받고 관계없는 save scenario를 제거했다. runner는 `--scenario release|timeout|both`(기본 both), 실제 loop/operation/output/raw scenario를 동일하게 사용한다. source 원 경로 및 모든 ancestor symlink를 resolve 이전에 검사하고 ROOT/tmp 범위로 제한한다.

최종 임시 observer SHA `ca2e8ef45b797a9dfef08c5180ddfe0ed8d4e257fa2263d5b08f93b5ae35fa38`, runner SHA `cf5587d7a041cf21e2408164cab25a2afff41be66b68bceda009029e67f8c15d`. Python compile 및 bounded static argv/source 계약 assertions만 통과했다. C# compile/native GUI 실행은 미검증이며 root의 원 증거 검토를 기다린다. 앞의 임시 observer SHA는 최초 미실행 사본의 이력이다.

## 최종 실제 Windows private GUI 인수

앞 절의 대기·미검증 문구는 각 준비 시점의 이력이다. root가 교정한 관측기를 게시하고 현재 source와 immutable exe/collector SHA를 대조한 뒤 실행한 [최종 독립 검토](../validation/evidence/windows-report-closewait-native-20261004T134851Z/root-review.json)에서 release/timeout 두 case를 인수했다. 수집기 원 result의 REVIEW_REQUIRED는 그대로 보존했고, 최종 판정은 root의 별도 DB/file/image 검토로 완료했다. 최초 lock/observer 계약 실패와 이후 StopAck nested-if clippy 실패도 삭제하지 않았다. root는 nested-if를 let-if로 교정한 새 소스로 실제 GUI를 재검증했다.

[release 원 결과](../validation/evidence/windows-report-closewait-native-20261004T134851Z/gui-release/result.json)는 원 파일 open/size0, exact inherited event 3개 및 원 manual-reset 생성, 닫기 확인창과 StopExit 후 20회 process alive/ACK false/size0 관측을 통과했다. release 이후 worker ACK와 자연 exit0, 자연 Job active0, private desktop 회수를 확인했다. 실제 export는 971bytes, SHA `21cff1e9ec3cf77b28a5236b7968388320756a986729e8e4d5db93ce7e62a3b9`, 전체 관측 6054ms다. root는 CloseWait PNG와 metadata report 7항목, JSON privacy를 독립 확인했다.

[timeout 원 결과](../validation/evidence/windows-report-closewait-native-20261004T134851Z/gui-timeout/result.json)는 release를 보내지 않고 19706ms 뒤 worker ACK/자연 exit0/자연 Job0를 확인했다. 원 export는 기존 Windows 실패 정책에 따른 0byte partial로 남았으며 성공 저장으로 인수하지 않았다. 두 case 모두 Failed DB 상태 불변, artifact/cookie 부재, 합성 secret 비노출, version/metadata tool 호출, 원 합성 source/sentinel 보존 및 input desktop 불변을 root가 확인했다. 실제 앱 SHA는 두 case 모두 `17a45cd5840a5a77ebbbeefde3fb8a34b277e5867a9d7d251f037433a3ed0e86`다.

최종 범위는 명시 fixture feature를 켠 Windows CPU 앱의 실제 report worker 저장중 CloseWait 및 timeout 실패 경계다. feature는 기본 OFF이며 기존 windows-sys 버전을 재사용하고 lock 변경은 app 의존 edge 1개뿐임을 root가 검증했다. 새로운 제품 정책·환경변수 활성화·일반 출시 동작 인수를 추가하지 않는다. clipboard, IME, 일반 S10 저장 시나리오와 다른 OS는 별도다. child는 문서만 갱신했으며 최종 Cargo/native 실행과 독립 인수는 root 담당이었다.

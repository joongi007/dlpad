# S00: Windows CPU 최신 후보의 원 native steady-idle 비용

- 상태: done (Windows 합성 empty-library steady-idle 범위)
- 분류: validation
- 우선순위: P0
- 요구사항: R06, R08, R11, R13
- 담당/수정 범위: metrics_recovery — 신규 `/tmp/dlpad-windows-idle-native-cost.cs`, 이 문서만
- 선행 작업: 최신 immutable CPU release, root frozen source manifest, 실제 격리 startup ready

## 사용자 결과

3회 fresh private Desktop 실행에서 startup ready와 실제 원 frame를 확인한 뒤 동일 10초 구간의 original-process CPU/WorkingSet/private bytes를 측정한다. 최종 스택/IME/reader 채택 완료를 주장하지 않는다.

## 계약과 범위

기존 `windows_managed_recovery_gui_observer.cs`의 private Desktop/원 HANDLE+creation/HWND/owned Job/PrintWindow/UIA/cleanup 도우미만 복제한다. 기존 파일은 수정하지 않는다. PowerShell/policy bypass/global input/clipboard/visible fallback/user Documents/실미디어/도구 실행 없음. 원 download button enabled와 recovery ready/cancel 부재, Job app1을 확인하고 안정화 2초 뒤 idle을 측정한다.

원 GetProcessTimes kernel+user 차이/Stopwatch monotonic 실제 구간으로 single-core CPU를 계산한다. WorkingSetSize/PrivateUsage/OS PeakWorkingSetSize/표본 최대 WS를 명확히 구분한다. idle80MiB/1% 목표를 유지하고 실패 raw를 저장한다. 실제 실행은 root가 다른 dlpad probe 동시 실행 없음 가정을 확인한 뒤 fresh C# compile 및 native 3round를 수행한다. source/exe SHA와 argv는 원시 결과에 기록한다. 전체 제품 source frozen manifest는 root outer collector의 before/after 검증 대상이다.

## 인수 조건

원 binary SHA/원 source hash 고정, 각 fresh data/media/temp root 원 ownership proof, 실제 PNG/UIA ready, 원 PID/creation match, 10초 정확 duration·단일코어 CPU/메모리 각값, 자연 WM_CLOSE/Job0/input Desktop 불변. glyph pixel proxy는 direct PNG 검토와 함께 해석하며 창 생성만으로 ready를 판정하지 않는다. 주기 sample 최대와 OS process lifetime peak를 혼동하지 않는다.

## 검증과 handoff

staging 구현 진행이며 프로젝트 Rust/Cargo/scripts/native 실행은 0이다. 기존 S10 GUI source freeze를 유지한다. 상태/결과는 새 collector 실제 실행 후 root가 통합한다.

## staged 구현 인계

신규 C# source는 기존 ManagedRecoveryGui observer의 Win32 declarations/원 HWND·creation 검사/WM_GETOBJECT·bounded UIA tree/PrintWindow/JSON 도우미를 읽기 복제했다. recovery manifest·cookie bytes 조사는 복제하지 않고 새 idle runner를 구현했다. 원본 observer 및 프로젝트 scripts는 수정하지 않았다.

CLI: `CPUexe expected64hexSHA freshoutput collectorSource.cs --no-concurrent-dlpad-probes`. 운영자는 다른 dlpad 소유 probe가 동시에 실행되지 않는 조건만 확인한다. 전체 호스트가 조용하다는 관측은 하지 않는다. source file와 compiled collector, 원 immutable app의 SHA를 기록하고 app/source 전후를 비교한다. 전체 제품 input manifest는 outer root collector에서 별도로 비교해야 하므로 staged result의 `product_source_manifest_checked_by_outer_root_collector`는 false다. 근거가 아직 없는 확인을 true로 쓰지 않는다.

각 round는 OS temp 하위 GUID namespace에 신규 SID private data/media 및 고정 ownership cap를 생성한다. 사용자 Documents나 existing cache를 선택하지 않고 `DLPAD_TEST_DATA`/`DLPAD_LIBRARY_ROOT`의 새 소유 위치와 `--fixture`를 사용한다. download start는 누르지 않는다. 원 CreateProcess HANDLE을 suspended 상태에서 own Job에 바인딩하고 실제 FILETIME/원 HWND를 대조한다. private MTA thread가 ready UIA 및 header glyph pixel proxy PNG를 확보하고 start enabled/cancel absent·recovery pending/error 부재를 확인한다. 2초 안정화 뒤 조건을 다시 확인한다.

idle 구간에는 UIA·WM_PAINT·screenshot을 호출하지 않는다. 원 GetProcessTimes 시작/끝 차이를 monotonic 10초 구간으로 나누며 core 수로 나누지 않는다. memory 100 ms 목표·128 sample 상한과 정확 duration을 저장한다. resident/private는 각각 sampled maximum으로80MiB 목표를 비교하고 OS process lifetime peak WS는 별도 field로 보고한다. startup transient peak를 idle memory 한계 판정으로 혼동하지 않는다. 실제 UIA 준비부터 측정 시작까지 시간을 별도로 기록하며 first observed glyph capture는 physical first-paint latency 측정이 아니라고 표시한다.

끝에는 원 ready 조건과 PNG를 다시 검사하고 WM_CLOSE exit0/자연 Job ActiveProcesses0/input Desktop 불변을 필수로 요구한다. 오류 또는 budget fail도 round/native-result.json에 남긴다. active Job 강제 정리는 실패 처리를 유지하며 success로 승격하지 않는다. MTA worker는 60초, 실패 이후5초 재join이고 root outer host watchdog도 필요하다. private Desktop close 실패/worker 미종료는 실패다. JSON은 round64KiB/aggregate262KiB로 제한하고 3회 fresh 결과를 수집한다. 원 temp fixture는 root independent 검토를 위해 보존하며 자동 재귀삭제는 없다.

## 실제 검증 상태

source `/tmp/dlpad-windows-idle-native-cost.cs` 28562 B/SHA256 `0c6c7e1a3224c04cfe940c615fadaefdcf0539f53ea80b6394e6bae0e424f67b`다. C# compile/native GUI/비용 실측은 아직 실행하지 않았다. 새 API declaration/원 helper 패턴을 읽기 대조했을 뿐 native 성공을 주장하지 않는다. 프로젝트 Rust/Cargo/scripts 수정은 0이다.

root의 fresh compile references는 UIAutomationClient/UIAutomationTypes/WindowsBase/System.Drawing/System.Web.Extensions 및 기본 framework assemblies다. root는 freeze binary/source hash와 다른 dlpad probe 동시 실행 없음 전제를 확인하고 existing private CLI host의 원 Job/watchdog로 실행해야 한다. actual ready/idle-end PNG 직접 검토, root outer source before/after 대조, 측정 duration/수치 검토 후 scoped 결과를 기록한다. CPU목표 실패나 peak 증가 시 budget을 조정하지 않고 실제 비용을 먼저 보고한다. IME/reader/200%·actual OS theme·장시간·최종 스택 채택은 남는다.

## 호스트 부하 문맥 보강

`quiet_host_verified=false`와 `external_host_load_measured=false`를 유지한다. `GetSystemTimes`를 10초 구간 전후 한 번씩 호출하여 합산 CPU busy 비율을 기록한다. 이 값은 dlpad와 OS까지 포함한 aggregate 문맥이며 다른 앱만의 부하를 분리하거나 조용한 호스트를 입증하지 않는다. 프로세스 열거 및 다른 앱 정보 수집은 없다. processor group이 하나이고 유효한 차이를 얻은 경우만 `whole_host_aggregate_load_measured=true`다. 여러 group은 호출 group으로 범위를 제한한다. API 실패나 비단조/잘못된 차이는 실패 코드와 null로 남기며 0%로 바꾸지 않는다.

동시 dlpad probe 제외는 운영 조건이지 관측 결과가 아니다. CPU/RSS는 해당 구간 앱 관측치이며 최종 스택 전체 성능 확정으로 확대하지 않는다. 보강된 C#은 아직 컴파일·실행하지 않았다. root가 native 결과·PNG·전체 source snapshot을 통합 검증한다.

## root 통합

원sourceSHA0c6c7e1a3224c04cfe940c615fadaefdcf0539f53ea80b6394e6bae0e424f67b를대조해scripts/windows_idle_native_cost_probe.cs에게시했다. root는입력거절뒤기존output에failure파일을쓰지않도록outputOwned gate를추가했다. 첫nativecompile/3freshactualGUI·originalCPU/메모리측정과PNG/source독립검증을root가담당한다. 현재다른dlpad소유nativeprobe는모두terminal이며3round동안동시probe를실행하지않는다. unrelatedhost quiet/externalload 분해미검증표시를유지한다.

## 초기 ACL 실패와 수정 초안

root의 [첫 native 실패](../validation/evidence/windows-cpu-idle-native-20261004T130853Z/gui/failure.json)는 GUI 진입 전 `System.UnauthorizedAccessException`이다. 원 collector는 `Directory.CreateDirectory` 후 `Directory.SetAccessControl`을 호출했다. 따라서 3회 GUI idle 수치는 아직 확보하지 못했으며 실패를 성능 인수로 쓰지 않는다. 원 source SHA는 `953c4ed244e0d44f2e69ea90b9a29340a6495493fcef4188550bf6e6e6ab30e7`, 앱 SHA는 `73d517da7f8d7f6d069cda649a0558adb3331bca9c8ddfb0ca4eb273b59d9649`다.

수정은 `/tmp/dlpad-windows-idle-native-cost.cs`만 staging했다. 기존 S10 collector의 생성 시 보안 설명자 패턴을 재사용하여 `CreateDirectoryW`에 현재 SID owner와 protected 단일 FullControl ACE를 전달한다. fresh 경로만 허용하고 생성 후 원 SID/보호 DACL/정확한 단일 비상속 ACE를 읽기 대조한다. 기존 경로의 ACL 수정과 실패 시 삭제 또는 visible fallback은 없다. 원자적인 것은 신규 디렉터리에 초기 보안 설명자를 적용하는 생성 호출이며 이후 readback까지 외부 경합 전체를 원자적이라고 주장하지 않는다.

실패 기록은 root가 추가한 `outputOwned` gate를 보존한다. fixture 생성 단계(root/data/media/validated)와 현재 자체 임시 root를 기록하여 생성 후 readback 실패의 잔여 위치를 구분한다. 생성 실패에는 고정 메시지와 native 오류 코드만 추가한다. 실패 root는 보존하며 성공으로 승격하지 않는다. 아직 C# 컴파일·native 재시험하지 않았다.

staged SHA256 `18089e874b3123dd1e896e69d6f98176e44bbe179e2c5c9157fbb7b0156e5be1`, 30987 B. 프로젝트 collector/Rust/Cargo는 변경하지 않았다. root의 다음 fresh compile과 private GUI 3회 실측을 기다린다.

## Job 두 구성원 실패와 후속 계측 초안

root의 `windows-idle-initial-acl-20261004T132056Z`는 세 회 모두 root/data/media ACL readback 및 GUI ready/glyph까지 도달했으나 비용 표본 전에 active=2 조건에서 실패했다. 준비 시간 325–511ms다. 원 결과는 `owned_job_naturally_empty=false`, `owned_job_terminated=true`이며 자연 종료가 아니라 소유 Job 강제 정리를 수행했다. 강제 정리 이후 desktop close/input 불변만 확인되었다. idle 비용 측정 성공으로 쓰지 않는다. Windows register_self_guardian은 OnceLock 경로 등록일 뿐 프로세스를 시작하지 않으므로 두 번째 프로세스를 상주 guardian으로 단정하지 않는다.

후속 `/tmp` 초안은 원 Job의 최대16개 PID 목록만 얻고 각 원 HANDLE을 열어 IsProcessInJob/creation/live/image와 native parent를 확인한다. 원 app PID는 원 HANDLE GetProcessId로 대조한다. 동일 immutable app image의 직접 자식만 허용하고 다른 executable/부모/구성원 변동은 거절한다. 조회 실패는 실패이며 PID 숫자로 종료하지 않는다. child의 command 역할은 아직 확인하지 않았으므로 역할 미확인 표기를 유지하며 두 구성원이면 REVIEW_REQUIRED로 남긴다.

10초 동안 전체 Job accounting CPU 차이와 모든 pinned 구성원의 WS/private 합을 측정하고 app CPU는 따로 기록한다. 공유 페이지는 WS 합에 중복될 수 있다. 80MiB/1%는 전체 Job의 보수적 비용 비교에 적용하며 app-only 수치로 전체 제품 예산을 통과시키지 않는다. 현재 후속 비용 경로의 시스템 합산 부하 관측은 false다. 기존 app-only 함수는 초안에 남아 있으나 호출하지 않는다. C# 컴파일/native 실행은 아직 없고 root가 fresh 검증한다.

staged source SHA256 `14a57566ef1a2ae0ad593be59bf42358b079f05a26bc8618f91066da7d66f8fd`, 37168B. 프로젝트 source/Rust/Cargo 변경 없음.

### 정책 거절 이전 원시 증거 보존

후속 초안은 native member query 성공 직후 원 round result의 bounded `job_member_observations`에 PID/creation/image/parent/is_original_app를 먼저 남긴다. 그 뒤 executable/parent 정책을 검사하므로 foreign 구성원 거절도 실제 두 번째 프로세스 증거를 보존한다. 일부 HANDLE 획득 후 실패는 기존 catch에서 모든 획득 HANDLE을 닫는다. 원 app 역할은 creation 동등이 아니라 원 HANDLE GetProcessId 동등으로 구분한다. 새 staged SHA256 `f7097224b23f7fca4aefc9c6d12740cddb7434f09b24359f2c1cff7017d5beca`,37562B. 컴파일/native 미실행이다.

## 실제 구성원 및 GUI subsystem 이후 비용

`windows-idle-job-members-20261004T132612Z` 첫 compile은 CS0170(kernel/user definite assignment)로 실패했으며 GUI 실행 전이다. root는 Accounting before/after 초기화만 교정했다. 다음 native 세 회의 추가 구성원은 실제 `C:\Windows\System32\conhost.exe` 원 app 직접 자식이었다. guardian으로 판정하지 않았으며 image 정책 거절로 idle 표본은 없었다. 원 foreign reject를 유지했다.

root의 GUI subsystem 적용 후 [실제 3회 결과](../validation/evidence/windows-idle-gui-subsystem-20261004T133036Z/gui/result.json) 앱 SHA는 `cf38ef92240e529073fb934b7794f17927488098c9251dce89c43c0c62d38887`다. 세 회 모두 원 app 하나만 Job에 있었고 자연 exit0/Job0/desktop close를 확인했다. CPU는 각각 0.311186%,0.466028%,0.931822%; 최대 WS 합은 85,266,432/79,921,152/79,515,648 B; 최대 private 합은 49,852,416/50,339,840/49,979,392 B다. 첫 WS가 80MiB(83,886,080B)보다 1,380,352B 커서 첫 회 BUDGET_FAIL 및 전체 FAIL을 유지한다. 전체 호스트 quiet/외부 앱만의 부하 측정은 하지 않았다.

### 메모리 개선 후보 읽기 조사

`accessible.rs::font_bytes`는 metadata로 32MiB 한계를 확인한 후 File.take(...).read_to_end를 빈 Vec에 읽는다. 현재 Windows malgun.ttf 원 파일 크기는 13,459,196B다. Vec의 실제 retained capacity는 아직 측정하지 않았으며 증가 정책에 따라 필요 이상 용량이 남을 수 있다. 최소 후보는 검증한 bounded metadata 길이로 try_reserve_exact 후 읽고 최종 길이를 다시 확인하거나 반환 직전 shrink_to_fit하는 것이다. 이것은 실제 소유 font bytes 용량을 줄이는 방향이며 OS WorkingSet trim이 아니다. 다만 allocator 실제 절감/재할당 peak/폰트 parser 복제 여부는 계측 전이므로 1.38MiB 절감을 보장하지 않는다.

`software_host.rs`는 원 frame960×640×4B(약2.34MiB)를 softbuffer에서 받아 직접 renderer에 빌려주며 앱 코드의 별도 전체 frame 복사는 보이지 않는다. resize는 dimensions 변경 때만 수행한다. 따라서 근거 없이 renderer frame을 먼저 줄이거나 한글 font/fallback을 제거하지 않는다. private 약50MiB와 WS 약76–81MiB 차이는 공유 image/UIA/provider 페이지도 포함할 수 있으며 현재 정보만으로 font가 주원인이라고 확정할 수 없다. 다음 인수는 원 font len/capacity·texture dimensions/보존용량의 값만 수집하고 동일 freeze/binary의 fresh3회 비용/한글 PNG를 비교하는 것이다. UIA activation 동일성도 유지한다. 프로젝트 Rust 변경 및 새 native 실행은 하지 않았다.

## 최종 통합 및 scoped 인수

root가 staging SHA를 대조하여 최소 font 읽기 변경을 통합했다. `windows-closewait-feature-regression-20261004T133915Z` 실제 Windows feature workspace에서 bounded_font_tests의 short chunks/성장·축소/oversize·empty 시험이 CPU 및 accessible 대상 모두 통과했다. `windows-font-closewait-default-workspace-20261004T134018Z`의 default fmt/clippy/test/release 네 명령도 통과했고 root가 원 snapshot SHA를 독립 대조했다.

[font 개선 후 fresh3회 결과](../validation/evidence/windows-idle-font-exact-20261004T134258Z/gui/result.json)는 root의 원 source/binary/collector identity 확인과 한글 PNG glyph 검토를 포함한다. CPU는 0.31135286/0.31060837/0.30969038%, sampled WS는82,173,952/76,140,544/76,206,080B, private는46,870,528/46,735,360/46,739,456B다. 세 회 모두80MiB/1%를 충족하고 자연 exit0/Job0/desktop close/input 불변을 확인했다. 이 수직 인수 범위에서 done으로 마감한다.

기존 첫81.3MiB WS 초과 및 전체 FAIL은 그대로 보존한다. 기존 font capacity를 실측하지 않았으므로 절감 전체의 인과를 allocator capacity 하나로 확정하지 않는다. 새 수치는 합성 empty-library·준비 후10초·fresh3회 관측이며 quiet_host_verified=false, 외부 호스트 부하 분해 미검증이다. 실제 IME/reader/장시간GUI/영상 실행/최종 스택 채택 완료로 확대하지 않는다. 현재 source freeze를 유지하며 근거 없는 추가 메모리 변경은 하지 않는다.

2026-10-05 최신CPUdfae315d/7,871,488B의 이전성능snapshot 차이를 확인했다. 기존bounded private desktop collector의 동일고정args로 empty3×10초를 재측정한다. 현재S10native와동시실행하지않고 종료후 시작하며 loaded10k와8h/quietHost증거로확대하지않는다. root는고정 /tmp wrapper로compile·원source/binarySHA·전체Job·naturalcleanup을검증한다.

최신 empty3회 실제PASS, CPU0.1556/0.4664/0.1553%, sumWS72,687,616/66,150,400/65,990,656B, 각10초 이상, 원member1/SHA535/naturalJob0/private정리 확인. root/independent-review.json에원자료검증을기록했다. quietHost/장시간과loadedgate는별도다.

# S04: Windows 관리 Deno 환경 정책 전달 계약 감사

- 상태: done (읽기 계약 감사; 제품 구현·EJS 인수 미완료)
- 분류: review
- 우선순위: P0
- 요구사항: R04, R05, R08, R13
- 담당: metrics_recovery
- 수정 파일: 이 문서만
- 선행 작업: Windows 공식 Deno opt-out 비용 실험, 관리 실행/paired 원 Job 연결

## 범위와 읽기 근거

README→PRODUCT→CONTEXT→REQUIREMENTS의 R04/R05/R08/R13→ADR0003 및 INTEGRATIONS, Deno source/실제 비용 작업을 읽고 현재 process/config/capture/download 경로를 조사했다. Rust/Cargo/schema/기존 문서/환경/도구·GUI 실행은 변경하지 않는다. Windows managed runtime 정책만 제안하며 외부 Deno와 사용자 cache 소유권을 유지한다. 공식 opt-out 존재, collector 실험과 제품 구현을 구분한다.

[공식 Node 호환 문서](https://docs.deno.com/runtime/fundamentals/node/#tools-that-spawn-node)는 실제 Node가 PATH에 없을 때 cache의 node shim을 사용하고 DENO_DISABLE_NODE_SHIM=1로 끌 수 있다고 안내한다. [고정 Deno source](https://github.com/denoland/deno/blob/0c071246a412575e07423263404a5d13e7ed6aa2/cli/node_compat_shim.rs)의 로컬 확보·hardlink 분기는 [기존 조사](S04-windows-deno-offline-native.md)에 있다. 이번 web 재조회는 고정 GitHub source cache miss였고 공식 문서의 opt-out 안내는 직접 확인했다. Node shim opt-out은 Deno의 node: built-in 지원 전체를 끄는 정책으로 단정하지 않는다. 실제 bundled EJS가 native node child를 요구하는지 별도 인수가 필요하다.

[공식 EJS 안내](https://github.com/yt-dlp/yt-dlp/wiki/EJS)는 공식 standalone에 solver가 포함된다고 설명한다. 이것은 이번 제품의 offline challenge 실행 성공 증거가 아니다. --remote-components를 새로 허용하지 않고 기존 pinned standalone+typed --js-runtimes 경로를 유지한다.

## 현재 제품 전달 경로와 결함 경계

| 코드 위치 | 실제 현재 동작 | 변경 요구 |
| --- | --- | --- |
| process.rs ProcessSpec:189 | executable/args/timeout/max_log_bytes 네 필드, 환경 정책 없음 | 프로세스별 typed 실행 정책을 전달할 공유 계약 필요 |
| process/windows.rs prepare:471 | 원 Job/HANDLE_LIST로 CreateProcessW, environment=NULL | NULL 상속을 이 정책의 보증으로 쓰지 말고 실제 tool에 bounded Unicode environment block 적용 |
| process/windows.rs helper spawn:543 | 원 helper도 environment=NULL | 정책은 config로 전달하고 helper 자신의 전역 set_var 없이 실제 tool spawn에 적용 |
| windows_managed_guardian_config.rs / windows_paired_guardian_config.rs:173 | argv/timeouts/원 기록만 serialize; process_spec 복원 시 env 없음 | 두 독립 helper config 모두 optional typed policy를 bounded/deny_unknown으로 보존해야 함 |
| windows_managed_capture.rs:151 / windows_paired_capture.rs:169 | ProcessSpec에서 argv만 config로 옮김 | policy 누락 없이 복사; cookie-only 경로가 필요한 경우도 명시 처리 |
| windows_managed_guardian.rs:150 / windows_paired_guardian.rs:181 | PreparedChild::prepare_with_stdio(spec) 뒤 원 Job gate | prepare 전에 policy/cache 원 소유 proof 확인, 동일 원 Job/Bound/receipt 순서는 그대로 |
| deno_runtime.rs inspect_with_managed:278 | 선택 Deno --version, session managed 존재 시 관리 helper; 그 외 일반 run_metadata | 선택 Deno가 실제 managed인지 별도로 판별해야 함. managed Some은 Deno 소유권 증거가 아님 |
| download.rs:694 및 metadata:733, checked download:1220 | SelectedDeno verify 후 yt-dlp에 literal deno 경로; metadata/download는 managed 또는 paired helper 실행 | 같은 정책을 yt-dlp 부모에 넣어 그 Deno 자식이 상속하도록 전달해야 함 |
| tools/deno_version.rs:71 | 관리 Deno --version 선택은 일반 process::run_metadata | 선택 단계도 typed 정책 사용 가능하나 -version 하나의 성공으로 run/EJS 경로를 대체하지 않음 |

읽은 SHA: process.rs `36a2f8053d53b4fbd34b27d41e0459fe7b700cb507ded516f8548d0238102da3`, windows.rs `e0896683b891cd3a8dd945afa2870d49ce0dda458c6dab78440bac232d55cd06`, deno_runtime.rs `51f88a3043dff21dd2b7c88370b547d3d8595a99ef1906fb38fd209c647c8157`, download.rs `a072d0a79bb0c30e86f2ed20ba6b46d3bedae9532f6e7484dccaa19399a0026d`. 이 snapshot 이후 root 수정이 있다면 현재 코드를 다시 대조한다.

app accessible.rs:1074는 yt-dlp/FFmpeg/Deno 어느 하나라도 managed root 안에 있으면 하나의 ManagedExecution을 획득한다. 따라서 managed yt-dlp+외부 Deno도 session.managed=Some이 된다. deno_runtime의 Some 분기만으로 외부 Deno에 cache 이동/opt-out 정책을 적용하면 소유권 경계가 바뀐다. ToolManager의 원 package/active 또는 explicit managed version native pins와 선택 파일 ID를 대조해 **Deno 자체**가 관리 구성요소일 때만 정책을 만들어야 한다. 문자열 prefix만 새로운 cache 삭제 권한으로 사용하지 않는다.

## 최소 공유 인터페이스 제안

root가 선택할 계약은 실행 전용 `RuntimeEnvironmentPolicy::WindowsManagedDeno { owned_cache, policy_version:1 }`다. arbitrary env map 대신 key/value를 제품에서 고정한다. cache 필드는 별도 원 소유 capability를 검증한 뒤 config에 bounded 절대 UTF-16 경로로 전달하며 config path만으로 소유권을 새로 만들지 않는다. enum은 Debug redacted, fixed error, nonzero policy version/지원 enum/절대 경로/noParent/NUL/길이 검사를 갖는다.

ProcessSpec에 optional field를 추가하는 안은 기존 literal constructors 전체 수정이 필요하다. 공개 API 호환을 우선하면 기존 ProcessSpec을 유지하고 내부 `run_*_with_runtime(spec,...,Option<&RuntimeEnvironment>)`와 PreparedChild의 내부 prepare_with_environment를 추가할 수 있다. 어느 안이든 root가 공통 호출부/guard config를 함께 연결하고 이전 wrapper None을 보존해야 한다. 전역 env 변경이나 thread-local 추적은 채택하지 않는다.

정책 생성 위치는 DownloadSession의 execution-only context 또는 SelectedDeno의 명시 owned-policy capability다. 관리 Deno inspect/version에 먼저 적용하고, 같은 capability를 metadata 및 download의 yt-dlp ProcessSpec/실행 context로 넘긴다. 관리 yt-dlp든 외부 yt-dlp든 **검증된 관리 Deno를 실제 실행할 예정**이면 tool의 Deno child가 받을 policy가 필요하다. 이 조건에서 외부 yt-dlp를 관리 소유로 바꾸지는 않는다. 반대로 외부 Deno·Deno None에는 관리 DENO_DIR를 강제하지 않는다.

Windows block은 기존 필수 OS 환경을 원 실행 시 bounded snapshot으로 만들고, 대소문자 무시 key 비교로 DENO_DIR/DENO_DISABLE_NODE_SHIM의 모든 ambient 중복을 제거한 뒤 `DENO_DISABLE_NODE_SHIM=1`, 실제 owned DENO_DIR를 정확 한 번씩 넣는다. 상속된 false/외부 DENO_DIR를 권한으로 사용하지 않는다. 변수 count/각 key·value/UTF-16 합32767이하와 마지막 double-NUL, NUL·invalid UTF-16·중복/unsupported key 거절을 시험한다. CreateProcessW에는 CREATE_UNICODE_ENVIRONMENT와 실제 block pointer가 필요하다. block은 spawn 동안 살아 있고 secret/value를 로그/argv/일반 DB로 출력하지 않는다. DENO_NO_UPDATE_CHECK/NO_PROMPT 추가는 별도 승인된 fixed policy로 검증할 수 있지만 이번 opt-out 요구를 임의 전역 env allowlist 개편으로 확대하지 않는다.

설정 snapshot에는 기존 deno 경로 선택만 유지한다. 실제 cache 경로와 lease/환경 capability를 StoredOptions version1에 저장하지 않는다. 고정 제품 안전 정책 version1은 재개 시 새 worker가 재검증해서 생성한다. 환경 정책의 historical version 표시가 필요하면 기존 Attempt component/진단의 fixed 정책 설명을 활용할 root 결정 사항으로 남기며 이 감사만으로 DB schema를 추가하지 않는다. 사용자가 arbitrary opt-out 값을 저장하거나 raw advanced override로 바꾸는 경로는 없다.

## DENO_DIR 소유권과 비용

cache는 사용자 기존 DENO_DIR, 외부 Deno cache, 설치 package component 디렉터리를 사용하지 않는다. 앱 data 아래 별도 runtime-cache/deno/<ownednonce>를 create-only로 만들고 current SID protected ACL/원 directory FileId/marker/고정 allowlist를 갖춘다. managed-tools 설치 root 안에 cache를 만들면 package strict entries와 mutation gate에 충돌하므로 분리한다. 새 worker capability로 원 cache를 잡고 같은 원 Tool Job의 실행·terminal 확인까지 소유권을 유지한다. cookie receipt나 PID 부재만으로 cache 삭제를 허용하지 않는다.

opt-out 없이 node shim hardlink가 원 exe nlink1 정책에 충돌한 [실제 실패](S04-windows-deno-offline-native.md)를 근거로 nlink 검증을 완화하지 않는다. 기존 사용자가 만든 hardlink를 자동 제거하지 않고 원 component를 보존/오류로 처리한다. opt-out 후 actual native nlink1·FileId·SHA/managed postverify를 새 fixture로 확인한다.

32MiB cache는 기존 실험의 비용 인수 상한이며 현재 API로 Deno 실행 중 filesystem 쓰기를 hard cap할 수 있다는 뜻은 아니다. 원 cache 전후 bounded inventory/bytes/entries, node.exe 부재와 실제 비용을 측정하고 초과는 release gate 실패로 보존한다. 예4096entries/32MiB postflight 정책을 구현할 경우 이를 실시간 저장 quota라고 표시하지 않는다. 정리는 terminal 확인·원 identity/ACL/no-reparse를 재검증한 앱 소유 파일만 대상으로 하며 unknown/교체/Busy/cleanup 미확인은 보존한다. 사용자 source/cache를 삭제하거나 다른 볼륨으로 강제 이동하는 대안은 없다.

## Linux·외부 실행 호환과 파일 담당 분할

Linux guardian command는 Command 환경을 상속하고 user_service environment()/gated_environment()는 launcher 환경 snapshot을 별도로 만든다. Windows 전용 enum은 cfgWindows policy만 받아 현재 Linux를 무변경으로 유지하는 것이 가장 좁다. 후속 OS 정책을 추가할 때 spec에만 env를 넣고 service Environment에 안 옮기는 누락을 별도 시험한다. 지금 Linux 새 제품 인수는 수행하지 않는다.

- root 공유 조정: process.rs typed optional API, windows.rs 실제 environment block, managed/paired/cookie config+capture+runtime 복원, tools execution_lease 전달. 한 파일의 동시 수정은 금지한다.
- 별도 독립 모듈 후보: runtime_environment.rs 순수 fixed block encoder와 malformed/중복/Unicode tests; owned Deno cache capability 모듈. 공유 declaration/Cargo는 root가 조정하며 새 dependency는 필요 없다.
- download 담당: deno_runtime.rs의 정확 관리 분류 및 download.rs의 version→metadata→download context 전달. app은 현재 options.denopath UI/snapshot을 유지하며 정책 누락 오류만 중앙 catalog로 처리한다.
- fixture 담당: Windows 실제 관리/paired helper에서 합성 yt-dlp가 실제 Deno 자식을 생성하게 하고 key값/read/write/nlink/FileId/cache beforeafter를 관측한다. 환경 노출 출력은 합성 fixture만 사용하며 인증 raw 출력 omission은 유지한다.

## 실제 인수 조건과 미완료 사항

관리 Deno actual --version만이 아니라 run 및 pinned standalone bundled EJS challenge를 offline fixture로 실행해야 한다. node child shim을 요구하는 케이스의 실패를 숨기지 않고 호환성 판정한다. 네트워크 solver 다운로드/remote-components는0이다. 실제 사이트 동작은 별도 승인된 익명 사이트 시험과 구분한다.

관리/paired metadata와 download의 원 yt-dlp→Deno child 환경이 같은 policy1이고 ambient DENO_DIR 또는 DENO_DISABLE_NODE_SHIM=0으로 덮이지 않는지 확인한다. 외부 Deno 사용은 기존 사용자 cache/source와 전후 동일을 확인한다. 준비·취소·timeout·app crash·terminal 미확인 시 원 cache와 설치 package를 보존하고 정상 원 Job terminal 뒤만 정리/원 package 재검증한다. managed-only/cookie-only/paired/nonmanaged wrapper의 None 호환 회귀가 필요하다.

이번 작업은 코드 읽기와 공식 문서 조회·계약 기록만 수행했다. 제품 환경 전파·cache capability·EJS 호환·공급 채택 또는 전체 P0 완료가 아니다. opt-out 실제 비용 실험은 다른 담당의 독립 evidence이며 그 결과 전에는 성공으로 기록하지 않는다.

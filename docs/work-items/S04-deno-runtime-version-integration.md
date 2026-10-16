# S04: Deno runtime exact version와 실행 연결

- 상태: proposed
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R03, R07, R11
- 담당/수정 범위: metrics_recovery — 이 문서만; root 후속 API/source 조정
- 선행 작업: 정확 공식 archive/component/notice 공급 증거 및 package fullset 검증

## 사용자 결과

관리 Deno의 실제 정확 버전을 확인한 뒤 명시 선택하고, metadata/download argv에서 선택된 원 runtime만 사용한다. runtime 정보를 Attempt와 실패 보고서로 전달하며 실행 시 원격 solver 갱신을 추가하지 않는다.

## 현재 코드 조사

core DownloadOptions에는 ffmpeg Option<PathBuf>, subtitles와 typed numeric advanced가 있다. 기존 AdvancedOption.validate는 limit-rate/retries/concurrent-fragments만 허용하므로 js-runtimes/remote-components를 raw 옵션으로 우회할 수 없다. ffmpeg 경로 검증·stored JSON·plan literal argv 및 engine canonical selection/version process/component 저장 패턴을 최소 확장 기준으로 쓴다. download_attempt_components는 component/tool_path/tool_version의 기존 generic table이므로 Deno 기록에 새 table이 필요하지 않다. 현재 ffmpeg_version 조회/Execution 결과 및 diagnostics 소비는 component별 노출을 별도 연결해야 한다.

관리 package select_package는 fullset SHA 검증 후 active DB만 바꾸고 runtime 실행 버전 증거는 없다. archive 설치도 inactive 유지다. direct yt-dlp install은 owned ProcessSpec로 --version을 검증한 뒤 publish/active하므로 해당 process 정책을 runtime version worker에서 재사용한다. Deno에는 yt-dlp의 --ignore-config/--no-plugin-dirs를 붙이지 않고 정확 argv[--version]만 전달한다.

## 최소 exact version 계약

Deno --version은 실제 세 줄(deno /v8/typescript)을 출력하므로 전체 stdout을 하나의 token으로 비교하지 않는다. bounded stdout 첫 줄에서 정확 'deno <expected version> <known release channel>' 구조를 검증하고 세 줄/line length를 상한으로 읽는다. 서브스트링 expectedversion 포함은 금지한다. 나머지 v8/typescript 줄은 공급 runtime metadata로 bounded 관측할 수 있으나 Deno token 판단과 혼동하지 않는다. 빈/비UTF8/여분 banner/다른버전/nonzero/truncation/timeout/cancel은 failclosed다. 실제 official binary output 형식을 root가 먼저 기록하고 strict parser 계약을 확정한다.

버전검증은 도구 worker에서 fullset/native identity 검증 후 owned process로 수행하고 실행 이후 original FileId/hash를 다시 확인한다. 정확 버전 성공 뒤만 active 선택 transaction을 수행한다. 기존 active는 실패시 유지하고 설치된 새 package는 inactive 보존한다. 실행 중 downloaded version 문자열을 package mutable JSON에 사용자 입력으로 다시 쓰지 않는다.

## 옵션·argv 최소 연결

별도 거대한 schema 대신 기존 DownloadOptions/StoredDownloadOptions에 명시 typed runtime selection을 추가하는 공유 root 변경만 필요하다. ffmpeg처럼 절대 literal PathBuf 한 개와 선택 runtime kind Deno를 고정한다. relative/parent traversal/control/oversized path를 거절한다. plan은 --js-runtimes와 'deno:<literal canonical path>'를 하나의 OsString 값으로 만들고 shell tokenize/colon split으로 Windows drive경로를 손상하지 않는다. 원래 option preview·snapshot equality·resume 검증 및 source URL 최종 위치를 유지한다. raw advanced js-runtimes/remote-components override는 계속 거절한다.

기본 활성 Deno/PATH 탐색을 그대로 두면 명시 선택 이외 runtime 실행을 막는 계약이 약해진다. 원 실행 argv에 --no-js-runtimes 후 선택 Deno만 --js-runtimes로 넣는 policy를 root가 공식 yt-dlp help와 actual fixture로 검증한다. metadata와 download 모두 같은 runtime argv를 사용하고 --version/cookie 없이 runtime 버전 확인한다. path는 snapshot/component 일반 tool evidence이며 cookie secret과 달리 redacted report의 정책에 맞춰 경로 익명화한다.

## Attempt·diagnostic와 네트워크 정책

기존 download_attempt_components에 component='deno', exact canonical tool path, parsed version을 저장해 reopen 후 실패 report가 해당 Attempt의 runtime을 조회하게 한다. 새 DownloadOptions 선택이 기존 Attempt를 덮지 않는다. ffi nonUTF8 경로를 to_string_lossy로 권한 증거처럼 취급하지 않고 기존 OS path identity 정책을 따른다. yt-dlp 공식 standalone bundled EJS solver를 고정 사용하고 --remote-components 활성화/네트워크 solver 다운로드는 도입하지 않는다. 영상 요청 네트워크와 remote component 갱신 금지는 서로 다른 계약이다. DENO_DIR를 도입하면 앱 소유 bounded cache만 사용할 별도 수명 정책이 필요하고 기존 사용자 Deno cache를 삭제하지 않는다.

## 실제 인수와 handoff

정확 세 줄 parser 성공/다른버전/banner/nonzero/timeout/truncated 실패, 원 hash 변경 이후선택 거절/active 유지, 한글·공백·Windows drive path literal argv, raw override 거절, metadata/download 동일 runtime, Attempt 재열람/report version 전달을 검사한다. 실제 Linux/Windows 공식 Deno --version 및 yt-dlp EJS 합성/실사이트 OS gate를 구분한다. 이번은 읽기 문서만 작성했고 Rust/Cargo/프로세스/sourcefreeze를 변경하지 않았다. API·parser 및 공급 채택은 root 후속 구현이다.

## 순수 parser 초안

root 확정 API의 /tmp/dlpad-deno-version-parser.rs를 작성했다. pub(super) verified_deno_version(stdout bytes,expected_version,expected_target)은4096바이트/정확3줄/각256바이트 printable ASCII, 정확 stable release version target 첫 줄, v8/typescript 각각128바이트 이내 공백 없는 token을 요구한다. CRLF는 str.lines로 허용한다. 실제 shape 및 Windows CRLF, 잘못된 prefix/version/target/extra/empty token, nonUTF8/전체·줄 상한/truncated shape 실패3시험을 포함한다. 개별 rustfmt만 수행했으며 프로젝트 Rust/Cargo는 변경·실행하지 않았다. 반환은 고정 ToolError::VersionMismatch다. 프로세스 exit/truncated flag 판단은 별도 root caller 계약이다.

## 통합 담당 선택 API 계약

root가 select_verified_deno(&ArchivePackage,&Arc<AtomicBool>)를 구현한다. worker의 동일root installlock 안에서 원 package/native directory/main file pin을 보유하고 ProcessSpec --version(max4096bytes/10초)을 실행한다. actual exit0/정확3줄 parser 성공, 원 파일/디렉터리ID 및 전체set hash 재확인, 마지막cancel/deadline 확인 후 active DB를 전환한다. 기존 select_package 하위primitive와 제품검증된선택을 구분하며 공개GUI가 unverified Deno 선택을 직접 사용하는경로를 방지하는 후속연결이 필요하다. 공급target은 실제host-compatible exactnative target만 허용한다. runtimeprocess 시작 timeout IO와 Cancelled/TimedOut outcome을 typed toolerror로 매핑하고 이전active를보존한다. schema/Cargo 변경은없다. pureparser 검증과 actualprocess 인수는 별도다.

## 실제 process selector 회귀 작성

root selector 제품 코드는 수정하지 않고 tests에 Linux x86_64 Python 합성 executable의 exact --version argv/3줄 출력을 작성했다.3회귀는 실제 process 성공activation, wrongversion/nonzero/selfmutatinghash 후 prioractive 보존, foreign/cancel/unsupportedtarget 거절 및 원 foreign 바이트 보존을 포함한다. prior fixture는 lower primitive select_package로 준비하고 새 selector 실패가 이전active를 유지함을 확인한다. 제품 source·공유schema/Cargo는 수정하지 않았고 개별rustfmt만 수행했다. Cargo/native 성공은 root actual검증 전 주장하지 않으며 Windows 공식exe는 별도 실증이다.

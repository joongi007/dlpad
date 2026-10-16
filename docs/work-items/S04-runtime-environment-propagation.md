# S04: Windows 관리 Deno 환경의 원 프로세스 전달

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R05, R08, R11
- 담당: root 공유 API·프로세스·helper 통합, metrics_recovery 순수 encoder staging, diagnostic_cache 별도 EJS offline fixture
- 수정 파일: crates/engine/src/runtime_environment.rs, process.rs, process/windows.rs, Windows managed/paired/cookie config·capture 및 ProcessSpec 호출 파일; 본 문서
- 의존성: 공식 Deno optout native PASS, typed 환경 감사, 기존 원 Job/guardian gate

## 사용자 결과

검증된 관리 Deno의 원본 링크 수를 늘리는 node shim 생성을 막고 앱 소유 cache를 사용한다. 설정이 실제 yt-dlp 자식 Deno까지 전달되며 기존 외부 Deno와 사용자 cache는 보존한다.

## 계약과 범위

Windows ProcessSpec의 optional typed 환경 정책과 bounded Unicode block을 추가한다. private 정책 필드를 사용하고 cache 소유 검증 뒤 생성한다. helper handoff는 정책을 누락 없이 복원하며 기본 None는 기존 실행 경로를 유지한다. 관리 yt-dlp 실행 여부만으로 Deno 관리 소유를 추측하지 않는다. root가 Cargo/schema/API를 조정하며 같은 파일의 동시 수정은 금지한다. 새 dependency는 추가하지 않는다. Linux/macOS 신규 인수와 외부 cache 삭제는 범위 밖이다.

## 인수 조건

실제 VersionProbe 및 yt-dlp→Deno 자식에 정책 전달·case-insensitive ambient override 거부·original nlink1/cache 소유/예산을 확인한다. managed-only/paired/cookie-only/None/외부 Deno 호환, 잘못된 policy/cache/UTF16/환경 상한/취소·timeout 실패는 원본을 보존해야 한다. 실제 EJS offline 인수와 실제 사이트를 구분한다. Windows fmt/clippy/test/release 및 원 프로세스/receipt 증거를 root가 검증하며 모듈 작성만으로 기능 완료를 주장하지 않는다.

## 검증과 handoff

기존 ProcessSpec은 환경 필드 없이 실행하며 Windows CreateProcessW가 NULL 환경을 상속한다는 읽기 근거를 확인했다. 구현·통합은 진행 중이다. 공식 Deno offline optout은 실제 PASS이며 제품 전달과 EJS 호환성은 아직 미검증이다.


## 초기 구현과 현재 검증

순수 private-field `WindowsManagedDenoEnvironment` encoder를 root 검토 후 engine에 게시했다. cfgWindows ProcessSpec optional field와 기존34개 literal의 None 기본값을 연결했고 실제 원 Job/stdio CreateProcessW에 bounded Unicode block/CREATE_UNICODE_ENVIRONMENT를 전달한다. 새 dependency/Cargo 변경 없음. 관리·paired·cookie helper는 아직 typed wire 전달을 구현 중이므로 Some 정책을 journal 변경 전에 거부하여 silent loss를 방지한다. 현재 production caller는 None이며 관리 Deno 자동 활성화 완료가 아니다. 소유 cache native proof와 정확 관리 Deno 분류는 여전히 후속이다.

첫 전체검증의 Clippy 표현 오류 및 초기 native regression의 미등록 guardian 오류를 보존했다. 직접 PreparedChild 원 spawn으로 회귀를 교정한 뒤 fmt/clippy는 통과했지만 [Windows 전체 시험](../validation/evidence/windows-runtime-environment-spawn-corrected/result.json)은 환경 인코딩 거부로 신규 실제 자식 회귀1개 실패(271pass/2ignored)하여 release 단계는 미실행이다. 순수 encoder7회귀는 PASS이나 실제 Windows inherited environment가 거부되므로 전체 통합 성공이라고 주장하지 않는다. [빠른 실패 loop](../validation/evidence/windows-runtime-environment-focused/tests.json)도 보존했다.

진단은 원 환경 key/value를 출력하지 않고 drive pseudo key·case duplicate·UTF16/size 분포를 계측한다. 실제 실패 test를 유지하고 원인이 확인된 최소 수정 후 같은 loop/full workspace를 재실행해야 한다. helper handoff 구현은 /tmp staging만 병렬 수행하며 root가 shared schema를 검토 후 통합한다.


## 진단 수정 후 현재 통합 인수

원 Windows environment 분류는 선두 '=' 특수키1개를 확인했고 중복·UTF16·상한 문제는0이었다. 실제 Rust encoder는 InvalidKey였으며 수정 전 synthetic 선두 pseudoentry 회귀와 원 native child 회귀 모두 실패해 [red loop](../validation/evidence/windows-runtime-environment-pseudo-red/tests.json)에 보존했다. Windows inherited pseudoentry의 선두 '='만 허용/보존하고 embedded '='·빈이름·control·중복·NUL·invalidUTF16 거부는 유지했다. [Microsoft CreateProcessW의 특수키 설명](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessw)과 원 category 관측을 대조했으며 실제 특수키 이름·값은 출력하지 않았다. 임시 [DEBUG-dlpad-env] 계측은 현재 source에서 제거했다.

관리/paired/cookie config의 strict nested typed wire optional policy도 root가 original6source SHA와 staged7source SHA를 검토 후 통합했다. old None wire는 필드를 생략하며 invalidversion/unknownfield/path/UTF16/Some roundtrip·native 복원 회귀를 포함한다. 기존 Some unavailable guard는 원 journal 변경 전에 유지하고 production 기본 None를 임의 활성화하지 않았다. 실제 helper 프로세스의 Some 실행 인수와 원 owned cache·관리 Deno 분류 factory가 남는다.

[수정 후 Windows 전체 검증](../validation/evidence/windows-runtime-environment-pseudo-fixed/result.json)과 [root 현재 source/회귀 대조](../validation/evidence/windows-runtime-environment-pseudo-fixed/root-review.json): fmt/clippy/test/release 모두 exit0. 실제 원소유 자식은 한글 cache path와 node shim optout1을 관측했고 원 Job terminal을 확인했다. 성공 source에서 추가 계측0, dependency/Cargo 변경0. 전체 다운로드 또는 EJS 제품 통합 완료로 확대하지 않는다.

[공식 bundle EJS 부분 실제 인수](S04-windows-deno-ejs-offline.md)의 preprocessed API n/sig/error를 root가 원 ZIPapp 멤버 SHA와 native 출력에서 [독립 검토](../validation/evidence/windows-deno-ejs-preprocessed-first/root-review.json)했다. selfauthored preprocessed 입력이며 upstream full player·AST·실제 사이트 인수가 아니다.

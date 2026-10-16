# S04: 공개 관리 runtime 환경 실패 경계 감사

- 상태: done
- 분류: bug
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: soak_recovery_audit; 본 문서 및 `/tmp` 신규 회귀 초안만
- 선행 작업: 관리 Deno environment factory/capture/download 연결

## 사용자 결과

관리 runtime 원 root와 다운로드 실행 원 root 불일치 및 실패 경계에서 실행·정리 증거가 잘못 섞이지 않도록 확인한다.

## 계약과 범위

공개 DownloadSession/ToolManager factory를 기준으로 source mismatch, 다른 managed scope root, journal 전 취소, helper 종료미확인, cache 재획득 실패를 읽기 감사한다. 공유 제품/Cargo/설정/native 실행은 제외한다. 원 binding/native proof·deadline/pending 정책을 완화하지 않는다.

## 인수 조건

각 경계의 실제 코드 보장과 회귀 공백을 구분하고 재현 가능한 결함은 최소 실제 공개 API 회귀 초안으로 제공한다. 실행하지 않은 회귀는 실패 재현 완료로 표현하지 않는다.

## 검증과 handoff

읽기 결과 및 임시 regression source SHA/root 후속 범위를 기록한다.

## 확인된 결과

실질 결함 두 개를 root가 독립 확인했다. `download.rs::execute_inner`는 environment가 있을 때 Deno/managed 존재만 검사하고 원 roots를 대조하지 않는다. managed/paired capture도 binding과 원 cache 자체만 대조하여 A factory의 policy와 B execution scope를 섞을 수 있다. 또한 `EnvironmentAuthority::_package`는 factory 이후 보유만 하고 full package verify를 다시 호출하지 않아 notice/manifest 또는 version 응답을 유지한 main 변조를 원 공급 hash와 재대조하지 않는다. Deno inspector가 실행 시 현재 main bytes를 새 기준으로 잡는 것은 factory 원 package 증명을 대체하지 않는다.

선택 경로 불일치는 `deno_runtime::inspect_with_managed`의 selected_executable 대조로 거절된다. 다만 DownloadService는 그 전에 환경 없는 yt-dlp/FFmpeg version probe를 실행하므로 초기 preflight root/path/fullpackage 검증 없이는 원 journal mutation 전 거절을 보장하지 못한다. root에게 capture와 공개 DownloadSession preflight를 모두 보강할 필요를 전달했다.

초기 취소는 capture ExecutionControl이 journal mutation 전 interrupted_result로 반환하고 cache Mutex 획득도 outcome을 확인한다. helper 종료미확인은 Terminal/EOF/original exitcode를 모두 요구하고 cache를 재획득하거나 pending을 지우지 않는다. strict terminal 후 cache try-reacquire 실패는 journal resolve/reader confirm 이전 unconfirmed이므로 원 Bound/receipt를 보존한다. 마지막 두 경계는 코드 읽기 보장이며 새 environment Some을 포함한 실제 native 실패 회귀는 아직 없다.

## 재현 초안과 잔여

`/tmp/dlpad-managed-environment-failure-regression.rs` SHA `d06d037adfe4f9eacc71e251e45d3b3ca633d281ba328c3f46dfd393b593266a`, rustfmt exit0. 기존 classification 합성 fixture에 삽입할 ignored Windows native 세 회귀를 작성했다. public factory → Aenv+Bscope 또는 factory 이후 LICENSE 변조 → public run_tool 오류 → finish의 pristine 인수를 검사하므로 단순 오류만으로 잘못 PASS하지 않는다. 실제 tool은 존재하지 않는 owned fixture 경로이며 최신 original adjacent guardian 빌드가 선행 조건이다. 세 번째는 public 선행 취소와 pristine finish를 검증한다. Cargo/native 및 프로젝트 Rust는 수정·실행하지 않았다.

root가 실제 red 재현과 origin-root 비교 및 read-only full package 재검증을 통합한다. 기존 ToolManager::new/DB 생성으로 읽기 증명을 재구성하지 않으며, 원 deadline/cancel 이하 검증과 native pin·receipt 정책을 유지한다. 공개 DownloadService 전파, 일부 실패 이후 cache/pending 실제 인수는 별도 후속이다.

## 주 담당 재현과 수정 착수

root가 세 회귀를 프로젝트에 통합해 [실제 Windows red](../validation/evidence/windows-managed-environment-failure-red-20261004T122336Z/result.json)를 실행했다. 컴파일은 성공했고 취소1PASS, foreign root와 notice 변조2FAIL이며 실제 pristine finish 실패를 확인했다.

root 수정 담당 파일은 tools.rs, tools/package_inventory.rs, tools/managed_deno_classification.rs, tools/execution_lease.rs, runtime_environment.rs, download.rs와 세 Windows capture다. 기존 full-set verifier를 원 root HANDLE 기반 read-only 함수로 추출하고 retained package 권한에서 재사용한다. DB 생성/다시 열기와 경로 기반 권한 재생성은 없다. 공개 download preflight는 첫 version probe 전에 선택 경로·원scope root/nativeID·패키지를 확인한다. managed/paired capture는 자체 journal root와 원 cache binding을 다시 대조하며 세 capture는 원 실행 deadline/cancel 이하 retained package 검증을 수행한다. 추가 의존성/Cargo.lock/DB schema 변경 없음. 각 capture의 추가 full component streaming hash 비용이 발생하며 원 64KiB buffer/manifest bytes 상한을 유지한다.

첫 수정 native build는 execution_lease.rs의 file_identity 모듈 한정 누락으로 실패했고 원 로그 windows-managed-environment-failure-green-20261004T122655Z에 보존했다. crate::file_identity로 수정하고 새 helper build/실제 회귀를 재실행 중이다. 현재 수정 green/전체 workspace/public5case는 결과 확인 전이며 앞선 product PASS를 최신 source로 확대하지 않는다.

## 실제 실패 회귀 green

[수정 후 실행](../validation/evidence/windows-managed-environment-failure-corrected-20261004T122749Z/result.json)에서 새 helper build와 명시 public 회귀3개 모두 exit0이다. foreign-root/notice 변조는 이제 Intent 전에 거절하고 pristine finish/reuse 및 원 bytes 보존을 확인했다. root-review.json은 operation source SHA와 현재 source가 같음을 대조했다. 최신 전체 workspace와 public 성공5case 및 DownloadService 전용 preflight 실패 인수는 아직 후속이며 작업은 이 잔여 때문에 열린 상태로 유지한다.

## 최신 통합 인수 확정

Windows public preflight3개는 windows-managed-download-preflight-20261004T123432Z에서 통과했다. [최신 전체4명령](../validation/evidence/windows-managed-download-preflight-workspace-20261004T123512Z/result.json)과 [proof gate 이후 public5case](../validation/evidence/windows-managed-deno-proof-download-20261004T123457Z/result.json)도 모두 PASS이며 root가 현재 source/artifact/각 receipt·owner·pending을 다시 대조했다. 확인된 root 혼합 및 factory후 notice 변조 결함의 수정·native failure/success 통합 검증을 완료했다. helper 종료미확인/cache 재획득 실패의 Some 경로 별도 injection 회귀 및 부모kill/공급/site/GUI는 후속으로 남으며 이 완료 범위와 구분한다.

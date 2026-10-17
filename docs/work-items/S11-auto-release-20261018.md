# S11: 자동 Windows 릴리스 패키징 사전 감사

- 상태: in-progress
- 분류: research
- 우선순위: P1
- 요구사항: R11
- 담당/수정 범위: release_audit; 이 문서만 수정. 제품/워크플로/schema/Cargo/lock 수정 없음.
- 선행 작업: 7819c9e Windows CI 성공; 주 담당의 소개 페이지 작업.

## 사용자 결과

소개 페이지 다음 단계인 자동 Windows 설치 ZIP·portable 릴리스의 실제 기존 구성과 차단 조건을 식별한다.

## 계약과 범위

읽기 전용으로 기존 payload 생성·패키징·CI artifacts·라이선스·runtime·manifest/version/개발 전용 gate를 감사한다. root가 공유 CI·인터페이스·통합을 소유한다. 공개 tag/release/push/비밀 전송은 수행하지 않는다.

## 인수 조건

실제 소스 위치에 근거한 최소 구현안, runtime/NOTICE/배포 readiness 차단, 검증된 CI artifact 재사용과 재빌드 금지 계약, 파일 소유를 주 담당에 전달한다. 미실행 설치 검증을 성공으로 표시하지 않는다.

## 검증과 handoff

기준 source7819c9e 읽기 감사 완료. 제품 코드·실제 릴리스·설치 동작 검증은 이 감사 범위 밖이다. 실행한 기존 회귀는 install payload18PASS, registry source binding13PASS, CI notice graph12PASS, unsigned update candidate3PASS다. Windows 실제 PE/runtime·설치 시험은 실행하지 않았다. `python3 scripts/check_docs.py --source-only`는1119문서 PASS이며 local-only1505링크와 runtime증거는 검증하지 않았다.

## 기존 코드 재사용 범위와 차단

- `scripts/generate_install_payload.py`는 후보 ZIP의 원 binary/notice SHA를 확인해 CPU를 `payload/dlpad.exe`로 명시 매핑하고 manifest를 마지막에 게시한다. 64KiB manifest·64entries·256MiB payload·128MiB exe·16MiB document 상한과 create-only 정책은 재사용할 가치가 있다. 단 version0.1.0과 unsigned development candidate 입력 계약이 고정되어 있다.
- `scripts/package_install_payload.py`는 7개 member의 create-only deterministic ZIP을 만들고 독립 CRC/SHA 재검증·receipt를 수행한다. `development_only=true`, `release_ready=false`, `signed_distribution=false`, `complete_legal_coverage=false`를 반드시 요구한다. 기존 동작을 릴리스 준비 완료로 바꾸지 않는다.
- `scripts/windows_msvc_candidate_package.py`는 `/mnt/d/project/dlpad`, 로컬 evidence/toolchain 경로와 graph258개를 고정한다. hosted CI의 `ci-captures/whole.json` 포맷과 다르므로 자동 릴리스에 그대로 쓰면 안 된다. 후보의 static CRT 요청 표기도 hosted build의 증거가 아니다.
- CI artifact `windows-development-build-inputs-${github.sha}`는 CPU/install/local-install/process-guard4exe, lock/toolchain, 전체 capture·graph 입력을 담지만 LICENSE/NOTICE 수집·runtime 점검을 제공하지 않는다. GPU build는 그 artifact upload 다음이므로 소비자는 native job/workflow 전체 성공도 기다려야 한다.
- Rust `BundleManifest`는 숫자 세 부분 version만 지원하고 installer/journal/registration 계층 여러 곳이 development_only=false를 거부한다. `v0.1.0` tag는 workspace version0.1.0과 일치시켜야 하며 prerelease suffix를 내부 bundle version에 임의 삽입하지 않는다. GitHub draft/prerelease 분류와 내부 development flag는 다른 축이다.
- `validate_update_candidate.py`는 검증된 publisher key/signature가 없는 read-only Python 정책뿐이다. 앱 updater나 안전 교체 권한이 구현된 것으로 설명하지 않는다.

## 설치 payload와 guardian/runtime

`crates/app/src/accessible_cpu_main.rs`는 `guardian_entry` 뒤 `register_self_guardian`을 호출한다. `crates/engine/src/process/windows_guardian_registration.rs`의 등록 분기는 current_exe 자체를 helper로 사용한다. 따라서 제품 CPU payload에 독립 process-guard.exe가 빠진 것은 이 경로의 누락이 아니다. guardexe는 독립 engine 시험의 fallback에 필요하므로 CI artifact에는 유지한다.

현재 hosted workflow와 Cargo profile에는 crt-static 요청이 없다. 원 후보의 `static_crt_build_requested=true`를 복사해서는 안 된다. `windows_candidate_package.pe()`는 실제 AMD64/PE32+/subsystem/일반 import를 읽을 수 있지만 delay import/runtime 존재/최소 OS 검증을 false로 남긴다. 새 packager는 실제 CPU와 installer import를 기록하고 private runtime 공급 또는 static CRT를 선택하기 전 Windows 기본 OS 외 DLL 의존이 있으면 분명한 차단 사유를 남겨야 한다. 재사용 검증 artifact를 받은 뒤 다시 빌드해 dependency를 고치는 것은 허용하지 않는다. 필요 build flag 변경은 Root CI에서 검증한 다음 새 artifact로 해야 한다.

## hosted NOTICE 수집의 최소 자동화

새 collector는 `ci-graph-format.json.resolved_packages`와 원 cargo metadata·Cargo.lock으로 실제 Windows CPU/engine normal+build union을 동적으로 선택한다. 258/259 같은 과거 개수에 고정하지 않는다. 같은 name/version의 서로 다른 source는 모호성 오류다. 각 registry package의 metadata.manifest_path 부모를 원 source로 하고 sibling registry/cache의 exact name-version.crate를 찾는다. 원 archive 전체 SHA가 Cargo.lock checksum과 일치해야 한다. source의 Cargo.toml·선택 notice·font source/notice를 `.cargo-checksum.json`과 대조하고, 원 archive에서도 같은 relative bytes가 존재하는지 streaming 확인한다. checksum JSON이 없으면 archive 검증 자체를 대체하지 않으며 그 사실을 표시한다.

`ci_registry_source_binding.validate`는 이미 source/원crate/lockchecksum/required files의 이 binding과 bounded archive/path/case collision 거절을 제공한다. 선택 notice는 root LICENSE/COPYING/NOTICE, 명시 package.license_file, 그 하위 제한된 notice 경로, `epaint_default_fonts`의4font/4grant를 포함한다. crate source 밖 license_file·symlink/reparse·중복·상한 초과는 거절한다. 최대4096package·package당64선택파일·합계16MiB notice 등 상한을 문서화한다.

path patch/workspace는 tracked checkout 원 bytes를 `whole.source_before/source_after`와 current hash에 연결한다. `vendor/accesskit_windows`의 Chromium notice 등 실제 vendor의 root/subdir notice도 포함한다. registry root에 없는 grant를 보완하는 `docs/licenses/upstream/manifest.json`과 정확 pin된 original notices는 별도 tracked upstream appendix로 함께 수집한다. 기존 `windows_candidate_package.licenses()`는 사용자 cache 위치/Iced cache_binding/workspace0.1.0 고정이 있어 그대로 호출하기보다 새 collector의 명시 root/cache interface로 분리하는 편이 작고 검증 가능하다. 원 notice 없거나 hash 불일치면 패키징 실패이며 선언된 license 문자열만 써서 성공시키지 않는다. 법적 완비·정확 linked inventory·publisher trust는 이 자동화만으로 true가 되지 않는다.

## 제안 파일 소유와 interface

Root: `.github/workflows/ci.yml`의 `workflow_call` 및 tag/release orchestration, Cargo/lock/공유 schema, site 다운로드 계약, 최종 통합. 후속 packager 담당: 신규 `scripts/collect_ci_notices.py`, `scripts/test_collect_ci_notices.py`, `scripts/package_ci_preview.py`, `scripts/test_package_ci_preview.py`와 본 작업 문서. 실제 구현 전 Root가 해당 파일 담당을 확정한다. 기존 개발 packager는 수정하지 않는 방향이다.

제안 CLI:

```text
collect_ci_notices.py --root CHECKOUT --graph GRAPH --metadata METADATA --lock LOCK --cargo-home CARGO_HOME --whole WHOLE --output FRESH_NOTICE_DIR
package_ci_preview.py --root CHECKOUT --inputs ORIGINAL_ARTIFACT_DIR --notices NOTICE_DIR --commit SHA --run-id ID --version X.Y.Z --output FRESH_OUTPUT_DIR
```

packager는 source SHA/run/repository/target·whole source hashes·7exact operation/lifecycle/log pins·binary hash·lock/toolchain·dynamic graph/notice receipt를 함께 확인한다. `windows-development-build-inputs.txt`는 수동 text 참고이며 유일한 authority로 쓰지 않는다. artifact namespace는 upload-artifact의 공통 조상 규칙을 고정해 test fixture로 검증한다.

Root의 같은 SHA reusable CI job outputs에는 `windows_artifact_name`, `windows_artifact_id`, `source_sha`, 가능하면 artifact digest를 내보낸다. caller는 GitHub가 제공한 같은 run/job artifact만 ID로 다운로드하고 성공한 workflow/needs를 기다린다. 다른 PR/fork/workflow_run artifact나 파일명만 같은 arbitrary ZIP을 신뢰하지 않는다. 권한은 compile/docs contents:read, 마지막 draft creation job만 contents:write다. packaging job에는 cargo build가 없으며 검증한 원 CPU/install bytes를 재사용·원본 hash 재검증한다.

파일명 계약은 `dlpad-{Cargo semver}-windows-x86_64-installer.zip`, `dlpad-{Cargo semver}-windows-x86_64-portable.zip`. installer ZIP은 root dlpad-install.exe와 payload/manifest.json 및5payload파일, portable ZIP은 검증 CPU를 dlpad.exe로 매핑하고 LICENSE/NOTICE/README/binding을 함께 제공한다. `SHA256SUMS`, 원 binary/CI/notice binding receipt, 명시 unsigned preview release notes를 첨부한다. 자동 publish·updater trusted manifest는 별도 다음 단계이며 초안 성공으로 제품 자동 업데이트 완료를 주장하지 않는다.

후속 실패 회귀는 tag/version 불일치·다른SHA/run·실패/누락operation·forced cleanup·log/binary/source 변조·graph package 누락·notice/source/archive checksum 변조·delay/runtime 미검증 구분·기존 output 보존·ZIP path/case/size를 포함한다. 실제 hosted artifact와 Windows private 화면 설치/재검사·실행·제거 및 synthetic 사용자 데이터 보존을 Root가 별도로 통합 검증한다.


## 2026-10-18 구현 재개

Root 지시로 신규 collect_ci_notices/package_ci_preview 및 해당 테스트·release.yml 구현 담당을 채택했다. 기존 packager/ci.yml/Cargo/공유 문서는 수정하지 않는다. Root는 workflow_call 및 artifact outputs/ci-notices 포함을 담당한다. 공개 release/tag/push와 비밀 접근은 수행하지 않는다. 검증 후 Root가 독립 통합한다.


## 구현 결과와 Root 통합 인계

신규6파일만 작성했다: `scripts/collect_ci_notices.py`, `scripts/package_ci_preview.py`, `scripts/test_collect_ci_notices.py`, `scripts/test_package_ci_preview.py`, `.github/workflows/release.yml`, 본 문서. 기존 packager·ci.yml·Cargo·lock·공유문서·Git metadata는 수정하지 않았다. Root가 CI reusable `workflow_call`, `windows_artifact_name/id/source_sha` outputs, native graph 뒤 collector 실행 및 `ci-notices/` artifact 포함과 docs 신규2회귀 실행을 연결한다.

collector는 원 graph4입력 재분류·source before/after/lock pin·registry 원crate/선택source checksum·vendor/workspace tracked grant·tracked upstream 원문과 font4개/원source/notice를 확인한 뒤 bounded create-only `inventory.json`을 마지막에 쓴다. cache/원본을 변경하지 않는다. cargo-home 옵션은 생략 시 CARGO_HOME 또는 사용자 ~/.cargo의 읽기 전용 원cache를 사용한다. 후속 job에 원 source가 없으므로 수집은 native job에서 graph기록/whole후에 실행한다. `ci_whole_orchestrate.after()`가 마지막으로 `folder/whole.json`을 기록하므로 `ci-captures/whole.json`이 실제 입력이다.

packager는4exe 원pins·7capture 원command/worker0/원Job 자연종료/로그stream·원 lifecycle/whole binding·checkout source·원 graph·NOTICE inventory를 검사한다. CPU와 installer 원PE를 읽고 OS DLL allowlist 외 import는 unbundled runtime dependency로 차단한다. MSVC static CRT를 요청하거나 기존 설치를 바꾸지 않는다. delay imports/간접로드/최소 OS/clean OS는 기존PE verifier와 마찬가지로 미검증이며 true로 만들지 않는다. 설치 payload는 기존 BundleManifest5entry와 numeric3part version을 보존하고 기존 개발 installer가 수용하는 development_only=true다. two ZIP은 같은원CPU bytes를 명시 매핑하며 재빌드·외부도구bundle·사용자data수집 없음. 새 runtime/제품dependency/앱 binary크기증가0, CI 임시 NOTICE16MiB·개별exe128MiB·전체ZIP256MiB상한의 복사/hash비용만 추가한다.

workflow는 tag/Cargo numericversion 일치→같은 실행의 reusableCI 전체성공→ID로만 원artifact다운로드→Windows packaging→별도 contents:write의 미공개 prerelease draft생성을 제공한다. branch manual dispatch는 거절하며 기존 draft/release를 overwrite하지 않는다. tag·public publish·Release 실제생성은 여기에서 실행하지 않았다. Root가 download-artifact v4.3.0 pin d3f86a106a0bac45b974a628896c90dbdf5c8093을 GitHub공식ref API로 확인했다. 서명/법적완비/readiness는 false를 유지하며 app updater는 구현하지 않았다.

실행 검증:

- `python3 scripts/test_collect_ci_notices.py`:8PASS. 원 registry tar·lock·checksum JSON·source grant의 실제byte대조, graph/package삭제·원notice변조·원archive변조·경로탈출·누락·기존출력보존을 합성temp에서 시험했다.
- `python3 scripts/test_package_ci_preview.py`:13PASS. 동일format7capture·2ZIP·SHA256SUMS·원binary/notice보존·deterministic hash와 다른SHA/run/version·source/log/lifecycle/binary/NOTICE변조·기존출력보존·member탈출/case충돌을 시험했다. 원lifecycle파일pin을 갱신해도 forced_cleanup=true는 거절한다. PE runtime 분류의VCRUNTIME140.dll거절은 제한된 stub회귀이며 실제hosted PE를 읽은 결과가 아니다.
- `python3 -m py_compile scripts/collect_ci_notices.py scripts/package_ci_preview.py`:PASS.
- `python3 scripts/check_docs.py --source-only`:1119문서PASS, local-only1505링크와runtime증거미검증.
- 현재환경Python3.10에서 첫시도tomllib import실패를 보존하고 새dependency없이 좁은 workspace version/상속license문법의 기존형식을 파싱하도록 보정해 이후 전회귀PASS.
- Linux git status는 Windows-created worktree의 .git gitdir D:/...표기를 해석하지 못해128로 실패했다. Git metadata를 수정하지 않았으며 Root는 기존WindowsGit으로 통합한다.

미검증: 실제hosted selected NOTICE전체·runtime import·자동draft 생성·Windows실제 ZIP설치/검사/실행/제거·cleanOS·전체Rust회귀. 이 감사/구현branch의synthetic format시험은Windows native인수나법적완비가아니다. Root가 독립통합검증한 뒤 작업완료 상태를 결정한다.

## Root 통합 조정

ci.yml에 workflow_call의 artifact name/id/source SHA와 native collector/notice artifact를 연결했다. v태그 standalone CI 중복을 제거하고 reusable 전체 CI를 사용한다. release workflow의 candidate_only 수동 main 실행은 태그·Release 생성 없이 전체 검증과 설치 ZIP·포터블 ZIP을 artifact로 만든다. v태그는 workspace numeric version과 일치해야만 미공개 preview draft로 진행하며 공개는 하지 않는다. Root가 collector8+packager13회귀를 다시 실행해21PASS를 확인했다. 실제 hosted runtime/notice/package 성공은 남는다.

### 로컬 metadata signer 구현 결과

신규 `scripts/sign_app_update.py`, `scripts/test_sign_app_update.py`를 작성했다. Root Rust verifier의 실제14필드 wire contract를 그대로 사용한다. CLI는 `--package`, `--version`, `--sequence`, `--channel preview`, `--key-id`, `--private-key`, `--output`, `--issued-at`, `--expires-at`이며 시각은 UNIX초다. 채널preview만 허용하고 version은 3부분 숫자·선행0금지·최대32문자, sequence는 양의u64, 유효기간은 현재시각을 포함하는 최대31일이다. 설치버전/최소sequence rollback판단은 signer가 알 수 없으므로 Rust 소비자의 pinned policy가 담당한다.

입력 canonical installer ZIP은256MiB streamingSHA/원파일전후identity와7member namespace·manifest schema/5entries·원파일SHA/CRC·preview binding false flags를 확인한다. OpenSSL은 argv만 사용하며10초deadline·각stdout/stderr4KiB·단일소유process 종료/reader종료를 확인한다. PRIVATE KEY bytes를 command argv·로그·출력파일에 기록하지 않는다. 기존 PEM16KiB상한·symlink/reparse거절·POSIX현재owner/0600이하groupworld모드거절과 원키변경전후관측을 적용한다. Windows ACL 보안을 검증했다고 주장하지 않는다.

공개 DER SPKI를 exact Ed25519 prefix/44bytes로 확인해 raw32bytes를 출력한다. exact rawJSON bytes를64byte Ed25519 signature로 만들고 별도OpenSSLverify한뒤 freshdir에 `update-windows-x86_64.json`, newline없는128lowerhex `update-windows-x86_64.json.sig`, `update-public-key.json`을 create-only로 기록한다. 마지막 파일은 {schema:1, algorithm:Ed25519, key_id, public_key_hex}의 공개 bootstrap보조자료이며 candidate안의 공개키를 trustroot로 수용할 수 없다. 패키지서명관측은 Windows Authenticode/운영publisher trust/자동설치 권한이 아니다. 운영키생성·키외부전송·GitHubsigningsecret설정·publish0을 유지했다.

검증: 현재환경OpenSSL3.0.2와Python3.10에서 `python3 scripts/test_sign_app_update.py`12PASS. 원temp0700fixture키만 생성했다. 실제sign/verify·payload변조거절·X25519키거절·키변경/권한/size/symlink거절·package크기/내용/previewflags변조거절·현재시각/최대31일/semver/sequence/channelpolicy·기존출력보존·실제OpenSSLtimeout/outputbound를 검증했다. `python3 -m py_compile scripts/sign_app_update.py scripts/test_sign_app_update.py`PASS. source-only문서1119PASS(local-only1505/runtime미검증)다. NativeWindows OpenSSL 실행·Root Rust간 실제이종검증·운영키/secret및release연결은 미검증/미실행이며 Root통합담당이다. signer는개발도구OpenSSL만 추가요구하고 앱/페이지dependency·runtime·크기증가0이다.


### 서명 도구 외부 프로세스 소유권 보강

Root 검토에서 기존 start_new_session+leader kill만으로는 fork 동반프로세스를 회수하지 못할 수 있음을 지적했다. 지원을 Linux/POSIX `waitid(WNOWAIT)`와 기본SIGCHLD·단독소유reaper 환경으로 제한했다. Windows는 owned Job 구현이 없으므로 키를 읽거나 프로세스를 생성하기 전에 Unsupported로 거절한다. 따라서 Windows PEM ACL 안전성도 지원했다고 주장하지 않는다. 서명은 개발 CI Ubuntu 또는 Linux의 기존 키로 실행하며 Windows 앱 자체는 OpenSSL을 사용하지 않는다.

OpenSSL leader에 poll/wait를 먼저 호출하지 않는다. waitid의WEXITED|WNOHANG|WNOWAIT로 leader원child를 reap하지 않은 채 유지하고 원PGID를 SIGKILL로 정리한 다음 readerEOF/join과 마지막leaderwait를 수행한다. 종료한원leader가zombie로 남아PGID재사용을 막는 동안만 group을 주소로 삼으며 원child관계가사라졌으면 numericPID로 kill을추측하지않는다. reap후killpg호출은0이다. 정상leaderexit도 동반pipechild가남을수있으므로 동일cleanup을 적용한다. OpenSSL은 session을탈출하는자식을만들지않는 신뢰된개발도구라는 좁은범위이며 일반untrustedprocess sandbox·제품guardian의대체가 아니다.

실제fakeOpenSSL fixture가 os.fork로동반child를만들고 gate를기다리도록했다. 시험은child의원pidfd를미리보유한뒤 timeout/stdout상한/leaderexit0 세경로를허용하고 원kernelprocess의pidfd readiness로 child가더이상실행되지않음을확인했다. 추가실제waitid/getpgid관측으로 groupcleanup이 originalleaderreap보다앞임을검증하고 Windows/비기본SIGCHLD의실행전거절도확인했다. 수정후 `python3 scripts/test_sign_app_update.py`17PASS, 원실제OpenSSL Ed25519서명/검증·변조거절12회귀도유지했다. 신규운영키·전역signal설정변경·외부전송·릴리스게시없음.

### Root 실제 PE 통합에서 발견한 수정

새 Windows MSVC CPU/installer 원 PE를 packager.runtime으로 직접 검사하니 OS 기본 UIAutomationCore.dll/Combase.dll이 초기 allowlist에 빠져 거절됐다. 접근성·COM의 OS DLL 두 개를 명시적으로 추가했고 임의 DLL/vcruntime/msvcp 거절은 유지한다. 실제 PE 재검증과 기존13실패회귀를 다시 실행한다. flags만으로 runtime 통과를 추측하거나 하위 fixture PASS를 실제 배포로 집계하지 않았다.


## Root hosted 성공과 후속 후보 ZIP 연결

source32607f76416254f2876f32bbaf5953511f28d355의 [CI39057841587](https://github.com/joongi007/dlpad/actions/runs/39057841587)는 docs/native 모두success다. 신규 원notice 수집·same-run source/lifecycle/graph·GPU compile까지 통과했으며 windows-development-build-inputs 원artifact12110418080/9,329,536B/digest5a57ceeebea3ed3c3b88a645f654022a004d66b242d8566bceea17f7803b6248가 생성됐다. 이source에 ZIP step은 없었으므로 패키징 성공으로 바꾸어 쓰지 않는다.

후속 CI는 GPU compile 뒤 같은 원파일을 재검증해 RUNNER_TEMP의 새폴더에 installer/portable ZIP·SHA256SUMS·binding·notes를 작성하고 windows-preview-candidates-SHA로14일보관한다. actual CPU/installer의 UIAutomationCore/Combase OS DLL만 allowlist에 보강 후 원 PE 검사와 packager13회귀 모두PASS다. [UI Automation OS runtime](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-uiautomationoverview)을 근거로 외부 VC runtime/unknown DLL 거절은 유지한다.

release version gate를 scripts/release_version.py로 분리했다. candidate-only main만 허용하고 tag ref의 candidate-only는 명시거절해 draft/sign으로 전환되지 않는다. 숫자버전/불일치/tag/candidate 실패회귀5PASS. draft는 취소 이후 생성하지 않도록 !cancelled()를 확인한다. optional sign은 repo/org variable DLPAD_UPDATE_SIGNING_ENABLED=true와 release-signing environment의 기존 DLPAD_UPDATE_PRIVATE_KEY_PEM/UPDATE_KEY_ID가 준비된 경우에만 실행한다(key ID 변수명 DLPAD_UPDATE_KEY_ID). 기존키 외부전송/환경설정/운영서명은 아직 수행하지 않았다. signing 실패 시 unsigned draft로 fallback하지 않는다. public-key aid는 제품 trust root가 아니다. 이 구성이 존재한다는 사실과 hosted optional sign/최초draft의 실제 성공은 구분한다.


## 실제 hosted 설치형·포터블 산출물 — source49c3812

[CI39060610796](https://github.com/joongi007/dlpad/actions/runs/39060610796)는 docs/native·GPU·신규ZIP단계까지 전체success, attempt1이다. 원candidate artifact12112071301/11,887,366B/SHAf92a68f51b19705e72923be6adcbabaa76b5b0fe029e2bcd27c411532f5fa7f8를 Root가 공식API로 내려받아 직접대조했다. 기존repo credential은 메모리와 api.github.com 인증에만 사용하고 signedAzure 다운로드에 Authorization을 전달하지 않았다. secret/URL 로그/운영키전송0이다.

InstallerZIP17,193,308B/SHA9ed4360afd65c060401d4c2eb58cd64c5d03a33aecbeaafaf9baed6ad3200dc5, portable12,769,934B/SHAb0bc04f9daff8e6b0fb046b4df8908ca550b354fe040cc52e64eb268570e0b2b. Root가 SHA256SUMS·release-binding identity/source/run/flags·모든CRC·manifest5원hash·installer7/portable6·동일payload를 실제검사해 PASS했다. 원 CPU10,429,952B/SHA4a60a714ea3a0e6a8616c24535ac03903bf3e5651551349a14baedec923f3d67, installer4,423,168B/SHAa2b34c8a1896d7f1d315a8243b84f6ea902640e9987c39da80e408d84638f8a9다. Root로컬CPU와 원hostedCPU는 서로 다른 빌드bytes이며 동일하다고 표시하지 않는다.

검수파일은 .local-tools/update-integration-20261018/hosted-49c/packages, 원검토는 같은폴더상위 package-review.json이다. publicRelease/Pages/태그/운영signing은 아직 수행하지 않았다. 현재installer 실제private 설치smoke는 별도로 검증한다. 과거 full_packaged probe의 empty-target·즉시enabled·과거VERIFIED문구 가정이 현재recommended-default/admission/catalog와 달라 v1~v4실패를 보존했다. 사전원source경로는일치했으며 제품코드오류로고치지않았다. 현재catalog/실제enabled전이/명시GUID임시target에맞춘로컬probe로재실행한다. 그 결과전에는실제설치인수완료로쓰지않는다.


### 원 hosted ZIP의 실제 private 설치·Root 검수

최종 v5 private observer는 current catalog/default/admission 계약에 맞춰 GUID 임시target을 명시했고 실제설치→재검증→WM_CLOSE exit0/Job자연0/강제정리0/inputdesktop불변을 통과했다. Root가 원result JSON과 installed/verified PNG를 직접 열고, 실제bundle/registered의5payload+3record namespace·manifest5원SHA/bytes·journal5원SHA·외부sentinel/forgedjournal부재를 별도Python으로 다시 대조했다. `.local-tools/update-integration-20261018/hosted-49c/private-install-v5/root-review.json`에 기록했다. oldprobe source에 대한 initialblank·readiness·검증문구 드리프트가 원인이었고 제품코드수정0이다. v1~v4 실패와 원강제정리facts도 보존한다. 최종C#derivedsource의 DEBUG instrumentation0이며 임시진단자료는 명시적인 .local-tools 아래에만 남았다.

실제hosted바이너리는원49cartifact로한정하며 Root현재bundle후속코드와 동일source빌드라고 주장하지 않는다. user Documents/실영상·실제설치기본위치쓰기를사용하지않고, 도구선택은unchecked·외부도구설치0으로 유지했다. readonly default경로hint관측은 설치권한이 아니며 변경대상은 GUID임시폴더뿐이다. 정식/cleanOS/Authenticode/자동update 전체인수는 아니다.


Root는 실제49c CI ZIP에서 임시설치한 원 dlpad.exe(SHA4a60a714...)도 별도로실행했다. gui-hosted49-v1의Light/Dark·960/800·200%focus실제스크롤·공개NoRelease조회·restart·두번자연exit0/Job0/강제0/inputdesktop불변을 확인하고 원PNG를직접열었다. raw REVIEW_REQUIRED를변조하지않고 root-review.json으로검수를추가했다. 현재Root의bundle후속소스빌드나물리IME/OSDPI/실제자동교체를검증한것은아니다.

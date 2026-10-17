# S11: 서명 후보 원 reader의 preview 설치 ZIP 읽기 검사

- 상태: done
- 분류: feature
- 우선순위: P1
- 요구사항: R08, R11
- 담당/수정 범위: updater_audit; 신규 crates/engine/src/app_update/bundle.rs, bundle/tests.rs와 본 문서만
- 선행 작업: 49c3812의 signed metadata·StagedPackage 원 reader/native identity; Root 모듈 등록·통합 검증

## 사용자 결과

서명 후보로 저장한 Windows preview installer ZIP을 설치하거나 실행하기 전에 원 File capability로 크기·SHA·ZIP 구조·manifest·모든 member 내용을 제한된 메모리로 읽기 검증한다. 설치/압축 해제/실행/추가 신뢰 권한을 제공하지 않는다.

## 계약과 범위

README→PRODUCT→기존 CONTEXT/요구사항 R08/R11→ADR0005 및 현재 updater 계약을 따른다. 기존 app_install manifest의 bounded parser와 tools/bounded_zip의 local/central·CRC·hash·중복/경로·상한 검사를 재사용/참조한다. rawzip0.5.1와 기존 flate2/sha2/serde만 사용하고 Cargo/lock/lib/export/staging/shared schema를 수정하지 않는다. 새 분리 branch codex/update-bundle-20261018와 지정 base49c3812를 Windows Git으로 생성했으며 기존 worktree/branch를 정리하지 않았다. key/push/publication/subagent0.

## 인수 조건

- StagedPackage 원 native identity와 signed outer size/SHA를 ZIP parsing 전후 대조하며 path 재오픈을 하지 않는다.
- 현재 CI preview의 정확7개 regular member만 허용하고 duplicates/alias/traversal/symlink/encryption/지원하지 않는 method/overlap/local-central 불일치/압축폭탄·크기·CRC/hash 오류를 거절한다.
- manifest version/target·정확5개 payload entry hash 및 binding의 installer/CPU byte/hash와 대조한다.
- cancel/deadline는 chunk 경계에서 지키며 부분/미지 입력은 읽기 실패이고 출력파일/프로세스를 만들지 않는다.
- 공통 parser는 Linux synthetic fixture로 검사하고 실제 stage capability Windows 회귀는 별도 준비·통합 검증한다.

## 검증과 handoff

구현 전 기록. Root가 모듈 등록·실제 workspace/Windows/native 검증과 도메인 문서를 담당한다. 원 reader 접근에 필요한 verify_current/version/sequence/bytes/sha256/reader_mut API가 이미 있으므로 shared accessor 추가는 필요 없다.


## 구현 결과

공개 `inspect_staged_bundle(&mut StagedPackage, cancel, deadline)`는 기존 native capability.verify_current와 원 reader의 signed size/SHA를 전후 대조한다. rawzip::from_seekable은 빌린 원 Read+Seek만 사용하며 package.path()를 source로 재오픈하지 않는다. 결과는 제한된 member/version/sequence/outer digest의 읽기 관측이며 Deserialize/추출/설치/실행·정리 API를 제공하지 않는다. 원 StagedPackage는 호출자에게 그대로 남는다.

현재 CI packager의 정확7개 regular member와5개 payload manifest entry만 허용한다. local/central name/flags/method/size/CRC·descriptor·physical range·directory offset/EOCD를 대조하고 case alias·중복·미지 member·directory·symlink·encryption·미지원 method·범위 overlap·부분 archive를 거절한다. current preview ZIP profile에 없는 ZIP64/extra field/comment/preamble/trailing data도 fail-closed다. 현재 packager가 Unix regular·ZIP_STORED·allowZip64=False로 생성하는 형식에 맞춘 제한이며 미래 다른 ZIP format 지원을 주장하지 않는다.

STORE/DEFLATE는 각각 전체 decompressed size/CRC/SHA와 compressed consumption을 검증한다. 압축 해제 결과의 저장은 없으며 manifest/binding만각64KiB로 유지하고 나머지는64KiB buffer로 hash한다. 실행파일각128MiB, 일반문서각16MiB, 전체 expanded256MiB, signed outer256MiB 상한과 chunk cancel/deadline을 적용한다. 해석 전후 whole-file2회 hash와 각 member stream 검증이 있으므로 file size에 비례하는 읽기 비용이 있고 source 전체를 RAM에 적재하지 않는다.

manifest는 기존 app_install::manifest::BundleManifest::read를 재사용하고 signed version/WindowsMSVC target/development_only 및 정확5개 entry를 추가 대조한다. binding-record의 installer/CPU hashes·mapping·preview readiness flags를 실제 member 관측과 대조한다. signed outer가 current preview임을 검사하는 것이며 provenance bool을 trusted로 고치지 않는다. binding의 다른 CI 증거/법적 완전성 전체를 검증했다거나 Windows Authenticode를 검증했다고 보고하지 않는다.

## 실제 명령과 결과

- Windows Git `worktree add -b codex/update-bundle-20261018 ... 49c3812e5305fd34d26dec6141d2812cad634307`: exit0. 기존 worktree/branch 변경/정리 없음. Git 명령에 GIT_OPTIONAL_LOCKS=0을 WSLENV로 전달했다.
- Rust1.94 rustfmt: 본인2코드파일만 skip_children으로 처리, exit0.
- `/tmp/dlpad-update-bundle-20261018`에 본인2코드파일의 byte-identical 복사와 기존 manifest/metadata/staging/file identity/private capability를 넣은 독립 harness를 생성했다. source workspace Cargo/lock/lib/export는 변경하지 않았다.
- `CARGO_HOME=/mnt/d/project/dlpad/.local-tools/cargo cargo test --offline --manifest-path /tmp/dlpad-update-bundle-20261018/Cargo.toml`: 실제28 PASS/0 FAIL/0 ignored. 신규 bundle10개, 기존 module18개다.
- 같은 harness `cargo clippy --offline --manifest-path ... --all-targets -- -D warnings`: exit0.
- 신규 시험은 Stored/Deflate 정상7member, signed outer 크기/hash, duplicate/alias/traversal/symlink/암호화/method/local-central·CRC·manifest·binding·overlong/truncated/EOF/comment/압축폭탄·취소/timeout/read error·마지막 outer 재해시 변조를 포함한다. 실명·실영상·사용자 Documents를 사용하지 않았다.

## 남은 통합과 미검증

Root가 app_update 모듈에 `pub mod bundle;`을 등록하고 Windows workspace/native을 검증한다. Windows actual StagedPackage 회귀1개(정상/내용불일치2case, 조회 취소 및 final ZIP 보존)는 준비했으며 Linux에서 실행하지 않았다. 실제 GitHub CI installer ZIP과의 byte compatibility/native 전체·설치·실행·운영 서명·자동 rollback은 현재 결과로 인수하지 않는다. 최초 실제 CI ZIP이 준비되면 공통 parser의 read-only compatibility를 별도로 확인한다. 새로운 shared accessor/dependency/schema는 필요 없다.


## Root 실제 통합 — 2026-10-18

Root가 source를 직접 검토하고 허용2파일 SHA(78c75457.../c6276129...)를 대조해 복사, pub mod bundle을 등록했다. 기존 Cargo/lock 변경0이다. 실제 Windows `cargo clippy -p dlpad-engine --all-targets --locked --offline -- -D warnings` 및 `cargo test -p dlpad-engine --lib --locked --offline app_update -- --test-threads=1` 모두exit0, 43PASS/0FAIL. 신규 Windows 원 StagedPackage test1이 정상/내용변조2case·cancel·원 ZIP 보존을 실제 실행했다. Root가 `.local-tools/update-integration-20261018/native-checks-v3/native-tests.log`의 명령/43개 이름/exit와 JSON을 직접 확인했다. natural-zero/final-zero=True, forced=False, private/inputdesktop불변·source불변, 경과38.6초다. Linux 실제 workspace의 app_update37PASS이며 기존media_preview Linux unused_mut 경고는 그대로다. Windows strict lint 성공과 구분한다.

실제49c CI installerZIP은17,193,308B/SHA9ed4360afd65c060401d4c2eb58cd64c5d03a33aecbeaafaf9baed6ad3200dc5다. GitHub원artifact digest f92a68f51b19705e72923be6adcbabaa76b5b0fe029e2bcd27c411532f5fa7f8와 Root의독립 CRC/manifest/memberhash/installer7·portable6 동일payload검사를 먼저 통과했다. 하위 /tmp generic parser 시험1PASS도 확인했으며 서명 metadata가 없는 unsigned 실물을 Windows signed StagedPackage 시험이라고 보고하지 않는다. 압축 해제·새B설치·실행·운영key·activation/rollback은 다음작업이다.


Root도독립harness의실제49cZIP compatibility testcase를직접재실행해1PASS/0FAIL/28filtered를확인했다(원stdout .local-tools/update-integration-20261018/root-actual-zip-parser.log). cargo fmt --all -- --check도exit0다. 하위보고만으로실물호환성결론을내리지않았다.

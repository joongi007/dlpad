# S04: Windows guarded runtime discovery 공개 다운로드 fixture

- 상태: completed (Windows 합성 runtime 전파 인수; 실제 배포 runtime 기능은 후속)
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R07, R08, R11
- 담당/수정 범위: soak_recovery_audit; 신규 examples/windows_managed_runtime_download_fixture.rs와 본 문서만
- 선행 작업: managed public Repository/version discovery/Deno inspect_with_managed 통합

## 사용자 결과

옵션 FFmpeg/Deno 지정 시 관리 guardian의 실제 VersionProbe 전파와 metadata/download literal runtime argv 및 persisted component version을 확인한다.

## 계약과 범위

self exe를 합성 yt-dlp/deno/ffmpeg로 복사하고 버전 argv만 fixed 출력한다. 성공5stages 및 Deno version mismatch 실패를 각 fresh root로 검사한다. cookieNone이며 paired 재검증 아님. 실제 FFmpeg/Deno binaries·영상·계정·네트워크·Cargo/shared source 변경 제외.

## 인수 조건

success Downloaded/artifact+3VersionProbe/Metadata/Download 원 receipt5·finish/reuse, FFmpeg/Deno literalpath meta/down 일치, actual Attemptcomponent versions. mismatch Failed/artifact0·receipt3 확인/cleanup/source sentinel 보존. 실제 compile/native 실행 전 완료 주장 금지.

## 검증과 handoff

source freeze SHA256 `745482621f9812f66d41f609795e5edd87869547aba65ffa80458c3841b00ed4`. CLI `--fixture-run <fresh-result.json> [--scenario success|version-mismatch]`, default success. 원 self source를 managed yt-dlp와 owned runtimes/deno.exe·ffmpeg.exe에 복사하며 TEMP direct-prefix/CAP/sentinel/exact path를 검사한다. 각 version argv는 실제 helper에서 실행되어 합성 Deno2.9.7/MSVC-target/v8/TS 및 FFmpeg7.1-synthetic 출력만 낸다.

success는 원 receipt5의 magic/index/stage byte `[VersionProbe,VersionProbe,VersionProbe,Metadata,Download]`를 관측하고 사본/SHA를 보존한다. 이것을 권한으로 삼지 않고 public finish의 원 records/bound receipts 검증을 필수로 사용한다. metadata/download literal FFmpeg location와 no-js-runtimes/deno:path를 검사하고 같은 path JSON을 보존한다. 공개 반환 version과 실제 SQL Attemptcomponents의 Deno2.9.7/FFmpeg7.1-synthetic, Downloaded/artifact 원 bytes를 확인한다.

version-mismatch는 실제 합성 Deno probe가 2.invalid.7을 출력하여 public DenoDiscovery 오류를 요구한다. Failed/artifact0와 metadata/download 미실행, 원 VersionProbe3 receipt 및 DB의 FFmpeg component만 존재를 확인한 뒤 managed cleanup/reuse와 모든 source/runtime/tool SHA·sentinel 보존을 검사한다. 다른 예상 밖 오류나 정상 Downloaded 결과는 통과하지 않는다.

staging rustfmt 실제 exit0. Cargo/Windows native는 아직 실행하지 않았으며 root compile/private host 통합 대기다. 원 source/DB/artifact/receipt evidence는 owned TEMP root를 보존하여 독립 검토 가능하다. **실제 Deno/FFmpeg runtime 배포 파일/기능/쿠키 paired를 검증한 결과가 아니다.** 다른 source/Cargo/schema/dependency 변경 없음.


## root 통합 검증 (2026-10-04)

격리 Windows GNU host에서 `cargo build -p dlpad-engine --example windows_managed_runtime_download_fixture --locked` 후 `--fixture-run <fresh-result.json> --scenario success`와 `--scenario version-mismatch`를 실제 실행하여 모두 exit0이었다. [원 실행 결과](../validation/evidence/windows-managed-runtime-download-first/result.json)와 [root 독립 검토](../validation/evidence/windows-managed-runtime-download-first/root-review.json)를 보존했다. 성공은 원 receipt5/Downloaded/artifact47B/두 구성요소 버전/동일 runtime argv, 오류는 원 receipt3/Failed/artifact0/metadata·download 미실행을 확인했다. 양쪽 원 managed 기록 부재와 보존된 실행 파일 SHA 및 foreign sentinel을 root가 SQLite/파일에서 독립 대조했다.

최신 소스에서 Windows `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked --features cpu -- -D warnings`, `cargo test --workspace --locked --features cpu`, `cargo build --workspace --release --locked --features cpu` 모두 exit0. [전체 검증](../validation/evidence/windows-managed-runtime-workspace/result.json), [현재 source snapshot 대조](../validation/evidence/windows-managed-runtime-workspace/root-review.json). 후속 workspace 검증이 debug example을 재빌드하므로 보존된 case별 복사본 SHA와 당시 실행 SHA를 대조했고, 현재 바이너리와 동일하다고 주장하지 않는다.

새 제품 의존성·Cargo/schema 변경 없음. 실제 공식 Deno/FFmpeg 기능·공급 채택·GUI 조작·사이트 호환성·전체 P0는 이 합성 인수로 완료하지 않는다.

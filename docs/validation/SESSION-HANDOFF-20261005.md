# 다음 세션 인수인계 — 2026-10-05

사용자는 새 작업을 멈추고 다음 세션에서 이어갈 수 있게 정리하되, 이미 하던 작업은 마무리하라고 지시했다. 최신 개발 패키지의 독립 검증까지 마무리했다. 새 기능 구현이나 장시간 시험은 시작하지 않았다. 하위 에이전트 3개는 모두 완료 상태이고 패키지 실행 프로세스도 terminal exit 0이다. 다음 세션은 명시적인 재개 지시에 따라 개발한다.

## 먼저 읽기

AGENTS.md → README.md → PRODUCT.md → CONTEXT.md → 관련 REQUIREMENTS/ADR. 전체 상태는 [P0-status](P0-status.md), [남은 인수 감사](P0-remaining-audit-20261004.md), docs/ROADMAP.md를 함께 읽는다. Windows 우선, S10 P0 포함. 정상 Git 저장소가 없으며 기존 .git은 보존했다. commit/remote CI를 수행하지 않았다. 경로의 Project/project는 현재 NTFS alias다.

## 이번 세션에서 확인된 결과

- 실제 Windows 격리 GUI에서 한글 IME 취소 후 preedit가 남는 버그를 재현했다. vendor/egui의 두 text_edit 파일에 좁은 transaction 복구를 적용했다. 동일 회귀 9개가 원본에서 4PASS/5FAIL, 수정 후 9PASS였다. native IMM 실제 취소/확정 모두 PASS. 물리 키 입력이나 screen reader 전체 인수는 아니다.
- 최신 기본 CPU Windows GNU fmt/clippy/test/release 4명령 PASS. 근거: `evidence/windows-ime-cancel-current-default-20261005-resume/result.json`. 바이너리 SHA256 `156003ccfe566693ccace15a4d75933b36ba75081473024ffb0969d722f8f547`, 7,856,640 bytes. IME fixture feature는 기본 제품에서 꺼져 있다.
- 최신 10k 합성 목록 GUI 비용 수집 3회 raw PASS. `evidence/windows-ime-cancel-default-loaded-cost-20261005-resume/`의 DB·파일·원 PNG에 대한 최종 root 독립 검토는 아직 남았다. Working set 79,024,128 /73,768,960 /74,297,344 bytes, 측정 CPU 0.3103 /0.1546 /0%. 8시간 GUI나 조용한 물리 호스트 보장은 아니다.
- 최신 개발 ZIP 두 개의 독립 검증 PASS. `evidence/windows-candidate-package-current-licenses-retry-20261005-resume/independent-archive-review.json`. SHA256 `4ce93d25d140779957425a525bb7c7fdb333dd458da245bfc7fd6efdc5eb6fd0`, 각각 12,529,149 bytes/831항목. 내부 exe·원 체크 근거·notice digest·안전 경로·고정 metadata·동일 입력 재현성을 검증했다. manifest의 내장 폰트 표기를 정정하고 정확 upstream notices를 추가했다. **누락 68건, release_ready=false**. 실제 linked license/SBOM·배포 완비가 아니다.
- 이전 90초 packaging timeout 결과와 과거 재현성/IME 실패 증거는 그대로 보존했다. 현재 package retry는 240초 제한으로 성공했다.

## 바로 이어갈 순서

1. 위 최신 default 4체크 및 10k 비용 증거를 주 담당이 현재 source/artifact SHA, 원 DB/합성 파일, PNG, natural exit/owned Job cleanup에 대조하고 root-review를 남긴다. 현재 raw PASS를 독립 전체 인수로 부풀리지 않는다.
2. 작업 기록 `S00-windows-owner-ime-compose.md`, `S00-windows-ime-cancel-retention.md`, `S00-egui-ime-transaction.md`, `S06-windows-license-gap-audit.md`의 좁은 완료/미검증 범위를 최신 근거로 갱신한다. 전체 S00/S06/P0 완료 표시 금지.
3. 다음 독립 두 작업을 병렬 진행할 수 있다: `S00-windows-first-visible-theme.md`와 `S05-windows-real-ffmpeg-cut.md`. 각 문서의 범위/파일 담당/인수 조건을 확정하고 같은 파일의 동시 변경을 피한다. Cargo/schema/lockfile은 주 담당만 조정한다.
4. 이어서 S10 clipboard·정확 경계·접근성, DPI/키보드/실제 IME, crash/sleep/recovery·GUI 장시간, 설치/upgrade/uninstall·보존·의존 DLL, license/CI를 남은 감사와 요구사항에 맞춰 수행한다. 실제 사이트·계정·공개 릴리스는 추가 권한 없이 진행하지 않는다. CI/CD 이후 updater는 사용자가 예고한 후속 범위다.

## 재개에 필요한 구현/검증 지점

IME 계약과 수정: `docs/work-items/S00-egui-ime-transaction.md`, `vendor/egui/DLPAD_PATCH.md`, `vendor/egui/DLPAD_SOURCE.json`, `crates/app/src/ime_cancel_regressions.rs`. 실제 GUI: `scripts/windows_ime_compose_private_native_probe.py` 및 같은 이름 C# observer, `crates/app/src/ime_compose_fixture.rs`. trace 확인으로 Enabled → Preedit 한글 → Preedit empty → Commit empty/correct → Disabled를 관측했다. Disabled만 처리하는 bridge 초안은 채택하지 않았다.

실제 검증 근거: `evidence/windows-ime-cancel-original-fixed-fixture-20261005-resume/` (의미 있는 red), `windows-ime-cancel-vendor-green-20261005-resume/`, `windows-ime-cancel-patched-feature-20261005-resume/`, `windows-ime-compose-cancel-patched-20261005-resume/`, `windows-ime-compose-commit-patched-20261005-resume/`. vendor patch로 Cargo.lock에서 egui registry source/checksum만 제거됐고 의존 버전은 바꾸지 않았다. upstream original .cargo-checksum은 patch 후 checksum이 아니다.

Windows 명령은 `scripts/windows_current_workspace_probe.py`가 bounded owned CLI/Job·private desktop·source snapshot을 수집한다. 기본 CPU 실제 네 명령은 fmt --all -- --check; clippy --workspace --all-targets --locked --features cpu -- -D warnings; test --workspace --locked --features cpu; build --workspace --release --locked --features cpu. Rust 1.94 toolchain/프로젝트 local Windows tools를 유지한다. WSL interop socket 실패는 제품 실패와 구분하고 허용된 native 실행으로 다시 검증한다. 시험 도중 Rust/Cargo/vendor/root CLI collector를 변경하면 pinned 근거가 무효화된다.

패키지 독립 재검증: `python3 scripts/windows_candidate_package_review.py --first docs/validation/evidence/windows-candidate-package-current-licenses-retry-20261005-resume/candidate-a.zip --second docs/validation/evidence/windows-candidate-package-current-licenses-retry-20261005-resume/candidate-b.zip --final-proof docs/validation/evidence/windows-ime-cancel-current-default-20261005-resume/result.json`. 기본 packaging은 license 누락을 거절하며 개발 후보만 explicit allow-missing-licenses를 사용한다.

## 병렬 초안 보관

원래 /tmp 초안의 지속 사본을 `docs/agents/handoffs/20261005/`에 보관했다. first-paint-theme-plan.md는 첫 observed glyph와 first presentation 전체 증명을 구분한다. ffmpeg-p0-next.md는 실제 Windows FFmpeg/ffprobe + public DownloadRepository를 사용한 합성 cut/decode/취소/DB 재개 계획이다. 직접 FFmpeg 실행만으로 제품 다운로드/관리 설치 완료를 주장하지 않는다. 초안은 실행 결과가 아니다.

## 실행 경계 / suggested skills

선택 skills: 어려운 재현 문제에는 diagnose; UI 개선에는 impeccable; 추가 인수인계에는 handoff. skill 없더라도 기존 프로젝트 계약으로 수행 가능하다. 자동 GUI는 격리 가상 Windows desktop만 사용하며 사용자 데스크톱으로 fallback하지 않는다. user Documents/실영상/계정/clipboard 전역 변경 금지. 기존 Git·외부 설치·사용자 데이터 삭제 금지. 임시 합성 fixture와 소유 process tree만 관리한다. 외부 게시/유료 자원/시스템 보안 변경은 별도 승인이다.

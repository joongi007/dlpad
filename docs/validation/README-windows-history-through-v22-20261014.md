# dlpad

Rust 네이티브 yt-dlp GUI. 긴 영상 다운로드, 구간 저장, 파일 목록과 작업 상태를 관리합니다. 최초 릴리스는 Windows 우선이며 개발 후보를 구현·검수 중입니다. [범위 계약](../../docs/work-items/P0-windows-first-release.md).

## Windows 개발 검수 후보

현재 검증된 설치본은 [v22 설치 ZIP](../../docs/validation/evidence/dlpad-install-ui-feedback-v22-20261014.zip)입니다. 새 폴더에 모두 압축을 풀고 `dlpad-install.exe`를 실행하세요. 기본 설치 위치를 쓰거나 상위 폴더를 선택한 뒤 설치하고, 설치 확인 완료에서 앱 실행을 누릅니다. 기존 설치 폴더에는 설치 버튼이 비활성화되며 다시 검증으로 확인합니다.

설치 없이 실행하려면 [v22 portable ZIP](../../docs/validation/evidence/dlpad-user-preview-ui-feedback-v22-20261014.zip)을 풀고 `dlpad-cpu.exe`를 사용합니다. 기본 저장 위치와 폴더 선택, 다운로드 도구 설치·검증·선택, Windows 네트워크 관측, 영상 미리보기·구간 선택, 고급 옵션 설명과 S10 P0 로컬 진단을 포함합니다. 도구 목록은 캐시를 먼저 보여주고 갱신하지만 실제 선택·실행은 다시 검증합니다. yt-dlp 업데이트 조회와 새 버전 설치 후 명시 선택 흐름도 포함합니다.

설치 ZIP은 **17,150,135바이트**, SHA-256은 `985a0bf0aa7b7f0073aee7e7f71c9b14e3dd38174804a7c15f84139be3d7474c`입니다. 이는 서명되지 않은 개발 후보이며 공개 릴리스가 아닙니다. [원파일·압축 검토](../../docs/validation/evidence/windows-msvc-ui-feedback-v22-20261014-package-root-review.json).

## 검증 상태와 진행 중인 변경

최신 v22는 네 개 상위 탭, 설정의 기본 저장 폴더, 제목 파일명 기본 옵션, 원본을 보존하는 복사 내보내기와 화면 간 진행 상태, 초기 배경·배율 수정, Windows 드래그 등록 실패 대응을 포함합니다. Windows 전체 네 명령 **1,524 통과 / 0 실패 / 42 ignored**, source669·43 harness와 원로그·실행 파일 세 개를 독립 대조했습니다. [최신 전체 검사](../../docs/validation/evidence/windows-msvc-ui-feedback-v22-whole-20261014/root-review.json).

정상 설치·실행의 자연0/Job0와44개 private 화면을 직접 팀 검토했습니다. 한글577자 설치 경로/598자 실행 파일 경로에서 앱 창과 자연 종료, 원20객체·6부모·payload/journal 보존을 재검증했습니다. 원 수집기의 짧은 경로 가정 실패는 보존합니다. [정상 설치](../../docs/validation/evidence/windows-msvc-ui-feedback-v22-20261014-command-happy/root-review.json), [설치 화면](../../docs/validation/evidence/windows-msvc-ui-feedback-v22-20261014-command-happy/root-team-pixel-review.json), [긴 경로](../../docs/validation/evidence/windows-msvc-ui-feedback-v22-20261014-long-install-run-postguard-v1/root-review.json).

독립 Windows MSVC host의 소유 export에서 로컬 CI15단계·Cargo8명령·독립7capture와1,524/0/42를 대조했습니다. hosted CI/프로젝트 Git 복구는 포함하지 않습니다. 최신73개 화면과18구간 비용 검증은 진행 중이며 사용자 물리 검수·공개 릴리스 완료로 보고하지 않습니다. [로컬 CI](../../docs/validation/evidence/current-ui-feedback-v22-local-ci7-20261014/root-review.json).

아래 v21 기록은 보존한 이전 checkpoint입니다.

v21 Windows MSVC fmt·strict clippy·전체 test·release를 실제 실행하여 **1,452 통과 / 0 실패 / 38 ignored**, 43 harness·source664·실행 파일 세 개와 원로그를 독립 대조했습니다. [전체 검증](../../docs/validation/evidence/windows-msvc-ui-feedback-v21-whole-20261014/root-review.json).

분리 가상 화면의 정상 설치·검증·앱 실행 재시험은 원파일과 기록을 보존하고 앱·감독 프로세스 자연0/Job0를 확인했습니다. 최초 시험의 Verify 준비 전 요청 실패는 그대로 보존했고 실제 버튼 준비 상태를 관측한 새 시험으로 구분합니다. [설치·실행 검증](../../docs/validation/evidence/windows-msvc-ui-feedback-v21-20261014-command-happy-diagnosis-v2/root-review.json).

앱63화면을 직접 팀 검토했으며 200% 설정 선택란 일부 가림과 재시작 초기 검은 영역을 수정 대상으로 남겼습니다. 네 개 상위 탭, 설정 중심 폴더 관리, 제목 기반 파일명, 이름 지정 복사 내보내기와 화면 간 진행률은 2026-10-14 개발 소스에 통합했으며 최신 전체 빌드·설치본을 검증 중입니다. **v21 설치본에는 포함하지 않습니다**. [시각 검토](../../docs/validation/evidence/current-ui-feedback-v21-app63-20261014/root-team-visual-review.json), [작업 기록](../../docs/work-items/S00-settings-library-navigation-20261013.md).

긴 경로 설치·변조 거부 최신 재검사, CI·비용, 물리 IME·reader·OS DPI·clean OS·upgrade/rollback·서명 및 사용자 최종 UI/UX 검수는 전체 인수가 끝나지 않았습니다. Git 조회는 exit128이며 기존 `.git`을 보존했습니다. Git/hosted CI 연결과 앱 updater는 후속입니다. [남은 작업](../../docs/validation/P0-remaining-20261011.md), [보존한 이전 안내](../../docs/validation/README-windows-checkpoint-before-v21-20261014.md), [사용 안내](../../docs/development/windows-user-preview.md).

## 문서 읽는 순서

1. [제품 방향](../../PRODUCT.md)
2. [도메인 용어](../../CONTEXT.md)
3. [관련 요구사항과 인수 조건](../../docs/REQUIREMENTS.md), [관련 ADR](../../docs/adr/README.md)
4. UI 작업은 [디자인 가이드](../../DESIGN.md)와 [사용 흐름](../../docs/UX.md), 프로세스·DB 작업은 [아키텍처](../../docs/ARCHITECTURE.md), 도구·인증 작업은 [통합 계약](../../docs/INTEGRATIONS.md)
5. [기술 스택](../../docs/STACK.md), [실패 진단](../../docs/DIAGNOSTICS.md), [성능 검증](../../docs/PERFORMANCE.md), [구현 순서](../../docs/ROADMAP.md), [CI/CD와 배포](../../docs/CICD.md)

모든 코딩 에이전트와 기여자는 [AGENTS.md](../../AGENTS.md)와 [개발 안내](../../CONTRIBUTING.md)를 따른다. 결정 변경은 [ADR](../../docs/adr/README.md)에 기록한다. 외부 자료는 [출처](../../docs/SOURCES.md)에 모았다.

## 개발 검증

Python 문서 검사는 추가 패키지 없이 실행합니다. Windows에서는 `python`을 사용할 수 있습니다.

```sh
python3 scripts/check_docs.py
```

Rust 1.94.0과 rustfmt/clippy는 `rust-toolchain.toml`로 고정하고 `Cargo.lock`을 사용합니다.

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --release --locked
```

CI는 Windows를 기본으로 검사하며 Linux/macOS는 수동 선택 항목입니다. 이 명령 목록은 새 실행 결과를 뜻하지 않습니다. Windows native 검증·격리 GUI 실행은 [개발 안내](../../CONTRIBUTING.md)와 해당 작업 기록의 소유 프로세스·private desktop 계약을 따릅니다. 실제 OS/UI 검증을 단위 테스트로 대신하지 않습니다.

프로젝트 라이선스는 [MIT 원문](../../LICENSE)이며 제3자 notice의 완료 범위는 배포 후보 inventory에 따릅니다.


현재18의 새 개선: 실제 키보드 도움말2필터와 진단 번호 실패·수정·비활성5필터를 격리 시험 후 제품에 통합했다. 앱은 v17보다12,800B 증가했고 설치기·CLI·의존성·Cargo.lock은 그대로다. SQL snapshot 변조·누락 회귀3/0 및 CI 안전성 관련42/0도 로컬 통과했고 hosted 실행은 미검증이다. [한글 확정 actual FAIL](../../docs/validation/evidence/settings-ime-positive-commit-isolated-20261013/root-failure-review.json)은 preview A조Z가 commit A한국어Z로 바뀌지 않은 원자료이며 새 테스트를 준비한 뒤에 발견했다. 현재 검수 성공으로 덮지 않는다.

### 현재 개발 소스와 설치 검증 checkpoint (2026-10-13)

현재 개발 소스의 서로 다른 한글 조합 preview/commit anchor 수정은 재시작 후 Windows fmt/clippy/test/release 검증을 통과했다(1331 PASS/0 FAIL/33 ignored). 기존 중단된 whole 기록은 미완료로 보존했다. v19 개발 설치 ZIP의 원본 7파일·archive hash/CRC 검사와 분리 가상 화면의 정상 설치/실행·소유 실행 파일 1바이트 변조 거부를 확인했다. 이는 위 v18 배포 경로의 바이너리를 바꿨다는 뜻이 아니며, 최신 설치본의 canonical27/긴경로/CI 및 사용자 UI 검수는 별도 남아 있다.

도구 자동 선택·읽기 전용 검사·업데이트 흐름은 `docs/work-items/S04-managed-tool-auto-selection-20261013.md`, `S04-readonly-managed-tool-inspection-20261013.md`, `S04-managed-tool-version-update-flow-20261013.md`에서 담당 파일과 실제 시험을 추적한다. 새 도구 후보는 현재 격리 소스에서 검증 중이며 제품 통합 완료로 보고하지 않는다.

### 2026-10-13 최신 Windows 검수 후보 v20

현재 source의 Windows MSVC fmt/clippy/test/release build는 새 `windows-msvc-managed-tool-auto-whole-20261013` 증거에서 통과했다(1357 통과, 0 실패, 37 ignored). v20 설치/portable ZIP은 이 빌드에 묶인 사용자 검수 후보이며 정식 서명·공개 릴리스가 아니다. 도구 검사 및 자동 선택 처리를 포함하지만 최신 버전 조회/새 버전 설치 UI는 아직 후보 코드로 분리되어 이 설치본에 포함되지 않았다. 기본 릴리스 수용 결과와 최신 검수 후보 결과를 혼동하지 않는다.

2026-10-14 후속: 현재 개발 소스669에 검증된 후보53파일을 통합했다. 이전 v21 source664/ZIP은 보존했고 새 v22 Windows 전체 네 명령 및 native 설치·73화면·CI/비용은 진행 중이다. [현재 통합 작업](../../docs/work-items/S00-windows-v22-candidate-integration-20261014.md).

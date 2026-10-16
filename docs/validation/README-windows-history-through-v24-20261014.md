# dlpad

Rust 네이티브 yt-dlp GUI. 긴 영상 다운로드, 구간 저장, 파일 목록과 작업 상태를 관리합니다. 최초 릴리스는 Windows 우선이며 개발 후보를 구현·검수 중입니다. [범위 계약](../../docs/work-items/P0-windows-first-release.md).

## Windows 개발 검수 후보

현재 검증된 설치본은 [v24 설치 ZIP](../../docs/validation/evidence/dlpad-install-ui-feedback-v24-20261014.zip)입니다. 새 폴더에 모두 압축을 풀고 `dlpad-install.exe`를 실행하세요. 기본 설치 위치를 쓰거나 상위 폴더를 선택한 뒤 설치하고, 설치 확인 완료에서 앱 실행을 누릅니다. 기존 설치 폴더에는 설치 버튼이 비활성화되며 다시 검증으로 확인합니다.

설치 없이 실행하려면 [v24 portable ZIP](../../docs/validation/evidence/dlpad-user-preview-ui-feedback-v24-20261014.zip)을 풀고 `dlpad-cpu.exe`를 사용합니다. 기본 저장 위치와 폴더 선택, 다운로드 도구 설치·검증·선택, Windows 네트워크 관측, 영상 미리보기·구간 선택, 고급 옵션 설명과 S10 P0 로컬 진단을 포함합니다. 도구 목록은 캐시를 먼저 보여주고 갱신하지만 실제 선택·실행은 다시 검증합니다. yt-dlp 업데이트 조회와 새 버전 설치 후 명시 선택 흐름도 포함합니다.

설치 ZIP은 **17,158,327바이트**, SHA-256은 `e6582034a7fa3fab7c0c0e946bcf8d7cf55c1243939da2652acdb4982e3755b4`입니다. 이는 서명되지 않은 개발 후보이며 공개 릴리스가 아닙니다. [원파일·압축 검토](../../docs/validation/evidence/windows-msvc-ui-feedback-v24-20261014-package-root-review.json).

## 검증 상태와 진행 중인 변경

최신 v24는 네 개 상위 탭, 설정의 기본 저장 폴더, 제목 파일명 기본 옵션, 원본을 보존하는 복사 내보내기와 화면 간 진행 상태, 초기 배경·배율 수정, Windows 드래그 등록 실패 대응을 포함합니다. 선택 정보줄은 고정 높이로 표시하며 왼쪽 정렬과 파일명 우선 폭을 개선했습니다. Windows 전체 네 명령 **1,538 통과 / 0 실패 / 42 ignored**, source670·43 harness와 원로그·실행 파일 세 개를 독립 대조했습니다. [최신 전체 검사](../../docs/validation/evidence/windows-msvc-ui-feedback-v24-whole-20261014/root-review.json).

최신 선택 화면은 800/960px·100/150/200% 여섯 조합에서 UIA 선택 전후 행 위치·ID,32px 정보 영역, 원파일 보존·정상 종료를 대조했습니다. 캡처24장을 직접 팀 검수하고 Root가 해시와 크기를 재검산했습니다. [선택 화면 검토](../../docs/validation/evidence/current-library-selection-summary-native-v4-20261014/root-team-pixel-review.json).

v24 정상 설치·실행과 한글577자 설치 경로/598자 실행 파일 경로에서 앱 창·정상 종료, 원20객체·6부모·payload/journal 보존을 재검증했습니다. 변조된 실행 파일의 실행 거부와 원본·설치 기록 보존도 확인했습니다. 정상·긴경로 설치의88원화면도 직접 팀 검토하고 전체 해시·크기를 재검산했습니다. [정상 설치](../../docs/validation/evidence/windows-msvc-ui-feedback-v24-20261014-command-happy/root-review.json), [긴 경로](../../docs/validation/evidence/windows-msvc-ui-feedback-v24-20261014-long-install-run/root-review.json).

최신 v24 로컬 CI는 독립 소스 복사본에서15단계·Cargo8명령·7개 결과 수집을 실행했습니다. 2,027개 소스 기록과1,538/0/42를 Root가 원로그와 독립 대조했습니다. main Git은 복구하지 않았고 hosted CI 실행도 아닙니다. [로컬 CI](../../docs/validation/evidence/current-ui-feedback-v24-local-ci7-20261014/root-review.json).

최신 합성 목록 성능 시험은 세60초 평균 CPU0.676534%·0.546358%·0.624538%와 메모리18구간을 충족했습니다. 시작·종료 원카운터로 평균도 독립 재계산했습니다. 추가 strict10초 CPU진단은18구간 중3개 실패를 보존하며 원인 확정을 위한 후속입니다. 사용자 물리 검수·clean OS·최종 스택 인수·공개 릴리스는 미완료입니다. [성능 원검토](../../docs/validation/evidence/current-ui-feedback-v24-default18-cost-20261014/root-review.json).

이전 검증은 [고정된 README 기록](../../docs/validation/README-windows-history-through-v22-20261014.md)에 보존했습니다. 현재 남은 인수 조건은 [잔여 작업](../../docs/validation/P0-remaining-20261011.md)에 있습니다. Git 조회는 exit128이며 기존 `.git`을 보존했습니다. Git/hosted CI 연결과 앱 updater는 후속입니다.

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

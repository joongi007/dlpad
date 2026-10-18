# dlpad

A lightweight, Rust-native desktop GUI for yt-dlp.

URL 입력부터 영상 다운로드, 구간 저장, 파일 확인까지 지원하는 Windows 데스크톱 앱입니다. WebView·Electron 없이 Rust 네이티브 UI로 실행합니다.

## 주요 기능

- YouTube·치지직 VOD 다운로드와 여러 작업의 진행률·속도·이력 확인
- 영상 미리보기, 시각 구간 선택과 시간 입력, 빠른/정확한 구간 저장
- 저장 폴더 동기화·검색, 제목 기반 파일명과 원본을 보존하는 복사 내보내기
- yt-dlp·FFmpeg·Deno의 앱 관리 설치와 버전 관리
- Light/Dark/System 테마, 트레이·백그라운드 작업, 중단·재개와 시작 복구
- 실패 단계·도구 버전과 민감정보를 정제한 로컬 진단 보고서

사이트별 동작과 이어받기 지원은 설치한 도구 및 원본에 따라 달라집니다. 최초 릴리스는 Windows 우선이며 Linux/macOS는 후속입니다.

## 현재 상태와 실행

현재 **Windows 64비트 미리보기 v0.1.2**를 [GitHub Releases](https://github.com/joongi007/dlpad/releases/tag/v0.1.2)에서 받을 수 있습니다. 설치형·포터블 ZIP과 Ed25519로 서명한 앱 업데이트 메타데이터를 제공합니다. Windows Authenticode 코드 서명이 없는 개발 preview입니다. **v0.1.0 사용자는 v0.1.2를 한 번 수동 설치**해야 합니다. 개발 검수 번호와 Cargo 버전은 별개입니다. [소스 checkout 범위](docs/SOURCE-CHECKOUT.md), [현재 P0](docs/validation/P0-current-20261016.md).

Windows x64의 Rust MSVC toolchain과 C++ 빌드 도구가 준비된 환경에서 앱을 빌드합니다. 정확한 Rust 버전은 `rust-toolchain.toml`로 고정합니다.

```sh
cargo build -p dlpad --bin dlpad-cpu --features cpu --release --locked
```

실행 파일은 `target/release/dlpad-cpu.exe`입니다. 다운로드·미디어 처리용 외부 도구는 앱의 설정에서 관리하거나 기존 설치를 선택합니다. [개발 안내](CONTRIBUTING.md).

## 소개 페이지와 다운로드

[소개 페이지](https://joongi007.github.io/dlpad/)에서 기능·설치 방법과 최신 설치형·포터블 ZIP을 확인할 수 있습니다. 공개 릴리스가 없거나 조회에 실패하면 GitHub Releases 안내를 유지합니다. 웹 런타임 의존성·외부 폰트·분석 스크립트는 없습니다. [공개 및 실제 검증](docs/work-items/S11-github-publication-20261018.md).

```sh
python3 scripts/build_site.py --output /tmp/dlpad-site-preview
```

빌드 출력은 새 디렉터리여야 하며 기존 폴더를 지우거나 덮어쓰지 않습니다. 정확한 `vX.Y.Z` 태그와 Cargo 버전 일치 → Windows 전체 CI → 같은 실행의 원본으로 ZIP 패키징 → 미공개 preview draft 생성을 자동화했습니다. [v0.1.0 릴리스 실행](https://github.com/joongi007/dlpad/actions/runs/39101008422)이 실제 성공했으며 파일·체크섬을 대조한 뒤 공개했습니다. 공개 Release 이벤트를 받은 별도 workflow가 main의 사이트 배포를 요청해 다운로드 링크를 갱신하도록 구성했습니다. 후속 버전도 draft 검수 후 게시합니다.

**v0.1.2부터 Windows 앱 업데이트를 지원합니다.** 설정 → 앱 업데이트의 조회 채널에서 **미리보기 버전**을 선택하고 새 버전 확인 → 서명된 버전 다운로드 → 작업 종료 후 재시작을 선택합니다. 기존 버전을 보존하고 새 버전 시작 실패 시 이전 선택으로 돌아갑니다. 더 새로운 서명된 버전이 없으면 업데이트를 적용하지 않으며, 서명 메타데이터가 없는 릴리스는 수동 설치로 안내합니다. 고정 부모 실행 파일의 보안 패치·프로토콜/호환성 epoch·신뢰 키 교체는 현재 앱 업데이트 범위에 포함되지 않아 수동 설치가 필요합니다. 통합 시험과 제한은 [업데이트 작업](docs/work-items/S11-update-activation-20261018.md), 서명 공급은 [운영 안내](docs/UPDATE-SIGNING.md)를 확인하세요.

## 검증과 CI

PR·push에서 문서·Python 정책 검사와 Windows Rust 포맷·린트·테스트·release 빌드를 실행합니다. 개발 실행 파일과 로그를 artifact로 보관합니다. [v0.1.0 전체 검증·패키징](https://github.com/joongi007/dlpad/actions/runs/39101008422), [CI/CD 범위](docs/CICD.md). 공개 설치형·포터블 ZIP은 같은 CI의 원본 실행 파일을 재사용하며 재빌드하지 않습니다. Windows Authenticode 코드 서명과 Linux/macOS 업데이트는 후속입니다. Windows 업데이트 전환·복귀는 별도 합성 회귀 시험으로 검사하며, [v0.1.2 릴리스 실행](https://github.com/joongi007/dlpad/actions/runs/39138176097)의 전체 CI·동일 실행 패키징과 구분합니다.

```sh
python3 scripts/check_docs.py --source-only
python3 scripts/test_check_docs.py
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --features dlpad/cpu --locked -- -D warnings
cargo test --workspace --features dlpad/cpu --locked
cargo build --workspace --features dlpad/cpu --release --locked
```

원본 개발 폴더의 로컬 증거까지 확인할 때는 `python3 scripts/check_docs.py`를 사용합니다. 소스 checkout은 과거 실행 증거를 포함하거나 재현 성공을 보장하지 않습니다. v0.1.2의 실제 Windows 로컬 전체 검증은 원 stdout 집계 1,838 PASS/0 FAIL/44 ignored이며 별도 signed 전환·실패 복귀 7회귀와 설치·공개 업데이트 전환/다음 실행 검수도 통과했습니다. 미검증 OS·물리 입력·성능 인수 범위는 작업 문서에 구분했습니다.

## 문서 읽는 순서

1. [제품 방향](PRODUCT.md) → [도메인 용어](CONTEXT.md)
2. [요구사항](docs/REQUIREMENTS.md) → [관련 ADR](docs/adr/README.md)
3. UI는 [디자인](DESIGN.md)·[사용 흐름](docs/UX.md), 프로세스·DB는 [아키텍처](docs/ARCHITECTURE.md), 외부 도구는 [통합 계약](docs/INTEGRATIONS.md)
4. [기술 스택](docs/STACK.md)·[진단](docs/DIAGNOSTICS.md)·[성능](docs/PERFORMANCE.md)·[로드맵](docs/ROADMAP.md)

기여와 에이전트 작업은 [AGENTS.md](AGENTS.md) 및 [개발 안내](CONTRIBUTING.md)를 따릅니다. 작업 기록은 `docs/work-items/`에 보관합니다.

## 라이선스

[MIT](LICENSE). 외부 다운로드·미디어 도구의 공급·라이선스 조건은 각 도구에 따릅니다.

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

현재 **v37 개발 검수 단계**입니다. 아직 서명된 정식 릴리스나 앱 자동 업데이트를 제공하지 않습니다. 개발 검수 번호와 Cargo 버전0.1.0은 별개입니다. 설치 ZIP과 로컬 실행 증거는 소스 저장소에 포함하지 않습니다. [소스 checkout 범위](docs/SOURCE-CHECKOUT.md), [현재 P0](docs/validation/P0-current-20261016.md).

Windows x64의 Rust MSVC toolchain과 C++ 빌드 도구가 준비된 환경에서 앱을 빌드합니다. 정확한 Rust 버전은 `rust-toolchain.toml`로 고정합니다.

```sh
cargo build -p dlpad --bin dlpad-cpu --features cpu --release --locked
```

실행 파일은 `target/release/dlpad-cpu.exe`입니다. 다운로드·미디어 처리용 외부 도구는 앱의 설정에서 관리하거나 기존 설치를 선택합니다. [개발 안내](CONTRIBUTING.md).

## 소개 페이지와 배포 준비

[정적 소개 페이지](site/index.html)를 구현했습니다. 공개 릴리스가 생기면 설치형·포터블 ZIP을 연결하며, 릴리스가 없거나 조회에 실패하면 GitHub Releases 안내를 유지합니다. 웹 런타임 의존성·외부 폰트·분석 스크립트는 없습니다. [사이트 작업과 실제 검증](docs/work-items/S11-introduction-site-20261018.md).

```sh
python3 scripts/build_site.py --output /tmp/dlpad-site-preview
```

빌드 출력은 새 디렉터리여야 하며 기존 폴더를 지우거나 덮어쓰지 않습니다. Windows 자동 릴리스는 정확한 `vX.Y.Z` 태그와 Cargo 버전을 대조하고 같은 CI에서 검증한 실행 파일을 설치 ZIP·포터블 ZIP으로 묶어 미공개 preview draft를 만드는 구성입니다. 아직 첫 hosted 패키지·공개 배포를 검증하지 않았습니다.

## 검증과 CI

PR·push에서 문서·Python 정책 검사와 Windows Rust 포맷·린트·테스트·release 빌드를 실행하도록 [GitHub Actions](.github/workflows/ci.yml)를 설정했습니다. 개발 실행 파일과 로그를 artifact로 보관합니다. 2026-10-17 [Windows hosted CI 전체 통과](https://github.com/joongi007/dlpad/actions/runs/39040643416)를 확인했습니다(source `5d4cf73`). 소개 페이지와 Windows 설치형·포터블 초안 릴리스 자동화를 구현 중입니다. 서명·실제 공개·앱 자동 교체는 별도 게이트입니다. [CI/CD 범위](docs/CICD.md).

```sh
python3 scripts/check_docs.py --source-only
python3 scripts/test_check_docs.py
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --features dlpad/cpu --locked -- -D warnings
cargo test --workspace --features dlpad/cpu --locked
cargo build --workspace --features dlpad/cpu --release --locked
```

원본 개발 폴더의 로컬 증거까지 확인할 때는 `python3 scripts/check_docs.py`를 사용합니다. 소스 checkout은 과거 실행 증거를 포함하거나 재현 성공을 보장하지 않습니다. Windows 전체37의1,685개 테스트와 별도 설치기 취소/재검사 회귀는 로컬에서 통과했으며 미검증 OS·실환경 항목은 작업 문서에 구분했습니다.

## 문서 읽는 순서

1. [제품 방향](PRODUCT.md) → [도메인 용어](CONTEXT.md)
2. [요구사항](docs/REQUIREMENTS.md) → [관련 ADR](docs/adr/README.md)
3. UI는 [디자인](DESIGN.md)·[사용 흐름](docs/UX.md), 프로세스·DB는 [아키텍처](docs/ARCHITECTURE.md), 외부 도구는 [통합 계약](docs/INTEGRATIONS.md)
4. [기술 스택](docs/STACK.md)·[진단](docs/DIAGNOSTICS.md)·[성능](docs/PERFORMANCE.md)·[로드맵](docs/ROADMAP.md)

기여와 에이전트 작업은 [AGENTS.md](AGENTS.md) 및 [개발 안내](CONTRIBUTING.md)를 따릅니다. 작업 기록은 `docs/work-items/`에 보관합니다.

## 라이선스

[MIT](LICENSE). 외부 다운로드·미디어 도구의 공급·라이선스 조건은 각 도구에 따릅니다.

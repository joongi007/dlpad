# 개발 환경

현재는 Rust workspace와 Linux 개발 화면이 있다. 실행법과 임시 fixture 경로는 [README](README.md)에 있다. 릴리스와 다른 OS 검증은 아직 완료하지 않았다.

## 지금

Python 3.10 이상으로 `python3 scripts/check_docs.py` 실행. 추가 라이브러리·Node·컨테이너·에이전트 플러그인은 필요 없다.

## Rust 개발 환경

Rust는 rustup으로 설치하고 정확한 stable 버전과 rustfmt/clippy를 `rust-toolchain.toml`에 고정한다. workspace와 Cargo.lock은 추가되어 있다. 정상 Git이 없는 현재 환경에서는 commit을 수행하지 않았다. 앱용 Python 설치는 요구하지 않는 standalone yt-dlp 배포를 우선 검증한다. 개발 문서 검사에 사용하는 Python과 제품 런타임은 별개다.

| 플랫폼 | 개발·시험 준비 |
|---|---|
| Windows | MSVC Rust target, Visual Studio C++ build tools, Windows SDK. 프로세스 Job Object·한글 경로·DPI 시험 |
| macOS | Xcode Command Line Tools. Apple Silicon과 Intel을 별도 target으로 검증. 서명은 별도 release 환경 |
| Linux | C toolchain, pkg-config와 선택 GUI/backend의 X11/Wayland/font/DBus 개발 패키지. 실제 필요한 패키지를 feature 확정 후 CI에 고정 |

WSL은 Linux 빌드 환경이다. Windows 앱 동작 시험을 대체하지 않는다. Linux headless CI의 컴파일 성공은 GUI 실행 성공이 아니다.

## 완료 정의

연결된 R/S ID의 인수 조건 충족, 공통 검사 통과, 관련 실패 경로 검증, 변경된 문서/ADR 갱신, 새 의존성 비용 설명. PR 설명에는 실제 실행한 검증과 검증하지 못한 플랫폼을 적는다.

실제 사이트·계정 credentials는 fixture에 넣지 않는다. 테스트는 임시 폴더·가짜 실행 도구·로컬 HTTP 서버를 우선하며, 사용자의 Documents를 시험 대상으로 쓰지 않는다.

## 프로젝트 전용 도구와 실제 도구 시험

현재 복구된 Linux 개발 도구는 `.local-tools`에 있으며 Git/배포에서 제외한다. 이 디렉터리를 `/tmp` 정리 대상처럼 삭제하지 않는다. 전역 Rust 설치가 없는 이 checkout에서는 다음 환경으로 기존 cargo 명령을 실행한다.

```sh
export RUSTUP_HOME="$PWD/.local-tools/rustup"
export CARGO_HOME="$PWD/.local-tools/cargo"
export PATH="$CARGO_HOME/bin:$PATH"
```

가상 화면은 시스템 Xvfb 또는 `.local-tools/xvfb/usr/bin/Xvfb`를 사용한다. 프로젝트에 추출한 도구는 제품 의존성이 아니다. 추출형 Xvfb의 고정 xkbcomp 경로는 해당 가상 서버 프로세스 안에서만 PRoot로 연결하며 시스템 디렉터리를 수정하지 않는다. 실제 사용자 데스크톱 probe는 사용자 요청 없이 실행하지 않는다.

```sh
python3 scripts/test_native_probe.py
DLPAD_SMOKE_YTDLP=/path/to/yt-dlp DLPAD_SMOKE_FFMPEG=/path/to/ffmpeg cargo test -p dlpad-engine --locked --test local_media -- --ignored --nocapture
```

실제 도구 시험은 Python 3 로컬 서버와 FFmpeg 합성 파일만 사용한다. 외부 사이트·계정과 사용자 Documents를 사용하지 않는다. 일반 workspace 테스트는 이 시험을 ignored로 표시한다.

## Linux 크래시 복구 시험

```sh
cargo test -p dlpad-engine --test guardian --locked
cargo build --workspace --release --locked
python3 scripts/guardian_probe.py target/release/dlpad /tmp/dlpad-guardian.json
DLPAD_TEST_FONT=/path/to/local-font python3 scripts/native_probe.py target/release/dlpad /tmp/dlpad-recovery.json --page downloads --recover-jobs --seconds 5
```

Guardian 시험은 합성 worker와 자손만 SIGKILL하며 pidfd로 정리 identity를 고정한다. release probe는 화면 없이 같은 앱 바이너리의 내부 guardian과 합성 owner를 실행한다. 독립 engine 라이브러리 사용자는 guardian entry를 등록한 앱 또는 인접 개발 helper가 필요하며 누락 시 실패한다. workspace 테스트는 helper를 빌드하므로 개별 --lib 시험보다 공통 명령을 먼저 실행한다. Linux /proc/subreaper·pidfd 사용을 다른 OS 검증으로 표현하지 않는다.

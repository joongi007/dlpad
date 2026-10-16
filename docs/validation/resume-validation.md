# 중단 후 재개 검증

2026-09-30, Linux 6.18.33.2-microsoft-standard-WSL2 x86_64. Rust 1.94.0. 정상 Git 저장소 없음, 빈 .git 보존. 프로젝트 전용 Rust/가상 화면 도구는 .local-tools에 복구했으며 앱 배포물에 포함하지 않는다.

## 구현과 검사

테마 저장 직렬화/최신 선택 병합, 진단·scan busy 상태 분리, SQLite 저장/legacy JSON 읽기 fallback을 구현했다. 로그 편집/제외·MPD 첨부와 native watcher의 실제 구현에 맞춰 오래된 문서를 수정했다. 새 제품 dependency/schema 변경 없음.

- cargo fmt --all -- --check: 성공.
- cargo clippy --workspace --all-targets --locked -- -D warnings: 성공.
- cargo test --workspace --locked: 61 passed; opt-in 실제 도구 시험 1 ignored.
- DLPAD_SMOKE_YTDLP=<local-tool> DLPAD_SMOKE_FFMPEG=<local-tool> cargo test -p dlpad-engine --locked --test local_media -- --ignored --nocapture: 1 passed, 18.97s.
- cargo build --workspace --release --locked: 성공.
- python3 scripts/test_native_probe.py: 2 passed; desktop fallback 금지/실패 시 서버 회수.
- python3 scripts/check_docs.py: 성공. 링크와 필수 파일 검사이며 외부 URL/heading 검증 아님.

실제 도구는 yt-dlp 2026.09.27.232945와 FFmpeg 4.4.2. 합성 2초 MPEG4 파일을 127.0.0.1 서버에서 다운로드하여 byte 비교, 한글 경로 공개, DB 재열기 완료 유지, 빠른/정확한 0.2–1.2초 구간 저장과 FFmpeg decode를 검증했다. 이 시험은 프레임 정확도·사이트 지원·긴 VOD·미디어 검증 adapter 완성을 뜻하지 않는다.

## GUI

통합 앱 executable 7,787,168 bytes(비압축). 기존 spike와 기능·커널 환경이 달라 크기/CPU 차이를 단순 renderer 비교로 해석하지 않는다. 기존 로컬 한글 폰트를 읽었으며 재배포하지 않았다.

| 가상 화면 | RSS KiB | 5초 idle CPU | 증거 |
|---|---:|---:|---|
| 10,000행 목록 | 67,876 | 0.0% | [JSON](iced-isolated-library.json), [화면](iced-isolated-library.png) |
| 설정 | 67,900 | 0.0% | [JSON](iced-isolated-settings.json), [화면](iced-isolated-settings.png) |
| 진단 | 67,828 | 0.0% | [JSON](iced-isolated-diagnostics.json), [화면](iced-isolated-diagnostics.png) |

Xvfb 시험 창과 서버를 실행 후 종료/reap했다. 기본 probe는 사용자 DISPLAY를 변경하지 않고 별도 display로 GUI와 screenshot을 연결한다. TCP/pathname socket listener를 끄고 Linux abstract socket으로 접속한다. WSLg의 기존 socket 디렉터리는 수정하지 않았다. 추출형 서버가 고정 경로로 호출하는 xkbcomp는 PRoot로 해당 프로세스 안에서만 연결한다. [X server의 compiler 호출 코드](https://raw.githubusercontent.com/XQuartz/xorg-server/master/xkb/ddxLoad.c)를 확인했다.

사용자 요청 이전 WSLg desktop 창 측정도 성공했으나 요청 이후에는 Xvfb만 실행했다. 실제 OS 한글 IME·스크린리더·200% 배율 상호작용·3 OS CI·8시간 soak는 미검증이며 S00/전체 P0 완료와 최종 renderer 채택을 선언하지 않는다. S10 P0는 계속 첫 릴리스 필수 게이트다.

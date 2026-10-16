# 후속 P0 통합 검증

2026-10-01. 정상 Git 저장소가 없어 기존 .git을 보존했다. P0 전체, S00 최종 채택, P1/P2 착수를 선언하지 않는다.

## 구현과 선택 근거

기본/선택 저장 위치 생성 및 복원, watcher 재연결과 누락 보완, 실제 다운로드 속도/ETA, Linux 선택 인터페이스 수신/송신, Linux 진단 clipboard 시험을 통합했다. 공유 schema/Cargo dependency 변경 없이 기존 SQLite settings를 사용했다. runtime 비용은 최신 progress 하나, 관측 중 두 pipe의 각 64 KiB framing buffer 및 제한된 UI channel이다. release 바이너리는 7,850,240 bytes다.

WSL 마운트에서 native 변경 알림만으로 기본 폴더의 새 파일이 반영되지 않는 실패를 실제 GUI로 재현했다. root mtime 확인과 30초 reconciliation hint로 보완한 후 20개 합성 파일의 DB 반영을 확인했다. 실제 yt-dlp의 누락 progress 필드는 bare NA가 되어 JSON 파싱에 실패했으므로 고정 template에서 |null 대체를 명시했고 실제 도구로 재검증했다. Proposed UI 스택은 유지한다.

## 실행과 결과

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --release --locked
DLPAD_SMOKE_YTDLP=<existing-yt-dlp> DLPAD_SMOKE_FFMPEG=<existing-ffmpeg> cargo test -p dlpad-engine --test local_media --locked -- --ignored
python3 scripts/test_native_probe.py
python3 scripts/check_docs.py
```

workspace 74 tests 성공, 실제 도구 1 test 성공(21.20초), GUI 안전 회귀 3 tests 성공. 처음 시도한 unittest 모듈 실행은 scripts import 경로 오류로 실패했으며 직접 스크립트 실행으로 수정했다. 실제 도구 시험은 일반 실행에서 ignored이고 명시 실행으로 별도 검증한다. 모든 미디어는 FFmpeg 합성 fixture, 서버는 127.0.0.1이며 사용자 Documents 대신 임시 provider를 썼다. GUI는 별도 Xvfb에만 열고 소유 프로세스를 회수했다.

[기본 저장 위치 GUI 측정](iced-default-storage.json): 폴더 생성·선택 유지·20파일 반영, RSS 69,132 KiB, idle CPU 0.20%, 5초 표본. [진단 clipboard 측정](iced-storage-diagnostics-clipboard.json): 정제 보고서 2,957 UTF-8 bytes 실제 수신, RSS 68,560 KiB, 5초 idle 0%. clipboard 측정은 마지막 watcher 보완 직전 바이너리이며 해당 변경은 진단 코드에 영향을 주지 않는다.

실제 IME/스크린 리더, Windows/macOS, 장시간 soak, 실제 사이트, parent-death guardian/Windows spawn race, 관리 도구 공급, FFmpeg 선택 UI와 진단 cache 정책은 남았다. 계정 인증·외부 전송·공개 배포는 수행하지 않았다.

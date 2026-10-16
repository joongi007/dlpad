# Linux 크래시 guardian과 시작 복구 검증

2026-10-01. Linux/WSL2 6.18.33.2. 정상 Git가 없어 기존 빈 .git을 보존했고 commit/remote CI는 실행하지 않았다.

## 구현과 계약

작업별 headless guardian은 앱과 연결한 anonymous Unix socket의 EOF를 owner 소실로 처리한다. 도구를 새 PGID에 넣고 subreaper를 사용해 중첩·별도 세션 자손을 입양/회수한다. 아직 reap하지 않은 직접 leader와 입양 자식만 종료하며 /proc 전체 PID 검색을 하지 않는다. 도구가 그룹을 바꿔도 직접 leader PID는 회수 전까지 고정된다. 앱 바이너리의 guardian 진입점은 GUI/InstanceGuard/DB 초기화 전에 실행하므로 데스크톱 창을 만들지 않는다. readiness handshake가 실패하면 외부 도구의 정상 시작으로 처리하거나 약한 방식으로 fallback하지 않는다.

기존 argv·로그 256 KiB/행 64 KiB·metadata 8 MiB·취소/timeout 계약을 유지한다. 제어 값은 최신 AtomicU8, 소켓 ready는 1 byte, 입양 child enumeration은 64 KiB 상한이다. 일반 취소는 TERM 후 200 ms, 정리는 SIGKILL과 최대 3초 회수이며 커널의 종료 불가능한 작업은 오류다. guardian 자체 고장은 별도 출시 게이트다.

앱 시작은 기존 journal을 worker에서 대조하고 상태 변경 수를 안내한다. journal 없음은 파일 생성 없음, 손상/최종 symlink는 오류와 보존, 확인 중/실패는 시작 차단·다시 확인 제공이다. 자동 재시작하지 않으며 동일 ID 재개/작업 이력 UI는 아직 남았다. 기존 library/schema/Cargo dependency·lockfile는 변경하지 않았다.

## 실행 명령

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
DLPAD_SMOKE_YTDLP=<existing-yt-dlp> DLPAD_SMOKE_FFMPEG=<existing-ffmpeg> cargo test -p dlpad-engine --test local_media --locked -- --ignored
cargo build --workspace --release --locked
python3 scripts/guardian_probe.py target/release/dlpad docs/validation/linux-guardian.json
DLPAD_TEST_FONT=<local-font> python3 scripts/native_probe.py target/release/dlpad docs/validation/iced-startup-recovery.json --page downloads --recover-jobs --seconds 5
DLPAD_TEST_FONT=<local-font> python3 scripts/native_probe.py target/release/dlpad docs/validation/iced-startup-recovery-failed.json --page downloads --recover-jobs --corrupt-journal --seconds 5
python3 scripts/test_native_probe.py
python3 scripts/check_docs.py
```

최종 fmt/clippy/test/release 성공. Rust 90 tests(앱 8/core 17/engine 60/guardian integration 5), 실제 도구 opt-in 1 test 성공(23.79초), GUI 안전 4 tests 성공, 문서 검사 57개 성공. 합성 worker·임시 폴더·FFmpeg 합성 미디어/127.0.0.1 서버만 사용했고 사용자 Documents/실제 영상/credentials·외부 전송은 사용하지 않았다. GUI는 Xvfb에만 실행하며 소유 프로세스를 회수한다.

## 남은 게이트

Windows spawn/Job assignment·파일 identity, macOS crash, guardian 자체 SIGKILL/고장, OS sleep/logout, 8시간 soak·만료/디스크 부족, 동일 ID 재개/백그라운드, S00의 OS별 IME/접근성과 최종 스택 선택은 미완료다. P1/P2는 착수하지 않았다. 이 검증은 전체 S03/P0 또는 첫 릴리스 완료가 아니다.

## 측정과 한계

[release guardian](linux-guardian.json): GUI 없이 headless entry 실행, 합성 owner SIGKILL과 자손 회수 성공. guardian RSS 4,596 KiB, active polling CPU 3.02초 표본 0%. [시작 복구 화면](iced-startup-recovery.json): Interrupted와 이전 Attempt 보존 확인, RSS 68,944 KiB, 5.01초 idle 0%. [손상 DB 화면](iced-startup-recovery-failed.json): 원본 바이트 보존, 오류·다시 확인 버튼 확인, RSS 68,728 KiB, 5.03초 idle 0%. 짧은 표본의 0%는 비용 없음 또는 장시간 성능 보장을 뜻하지 않는다.

release 앱 7,901,184 bytes, 독립 engine 개발 helper 399,784 bytes. 앱은 같은 바이너리를 headless 재사용하므로 제품에 별도 helper를 묶는 것이 필수는 아니다. 직전 옵션 구현의 7,878,112 bytes 대비 앱은 23,072 bytes 증가했다. idle 앱은 guardian을 띄우지 않으며 외부 실행 중 한 guardian의 추가 비용을 앱 자체와 구분했다. yt-dlp/FFmpeg/미디어 비용은 별도다.

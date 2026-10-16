# 고급 옵션과 외부 FFmpeg 검증

2026-10-01, Linux/WSL2. 정상 Git 저장소가 없어 기존 .git 보존, commit/remote 검증 없음. 두 작업은 독립 P0 개발 범위이며 S00 최종 채택 또는 전체 S05/P0 완료가 아니다.

## 변경한 사용자 동작

고급 옵션을 펼쳐 속도 제한·재시도·조각 동시성을 조절하고 입력 옆 범위 오류, 기본값·예·비용·충돌·검증 여부를 본다. 값은 shell 없이 typed argv에 전달하며 빈 값은 기본값으로 복귀한다. 빠른/정확 구간 저장을 선택하고 실행 중 설정 변경은 차단한다. 작업 시작 전에 실행 계획의 URL은 생략한다.

외부 FFmpeg 경로는 snapshot으로 고정하고 -version을 소유 process runner에서 확인한다. adapter-local SQLite migration v3는 Attempt component의 canonical 경로/검증 버전을 저장한다. 실패 진단은 확인한 FFmpeg 버전을 포함하며 파일/버전 오류는 도구 확인 단계다. 선택 None의 기존 snapshot과 library schema는 유지한다. 외부 파일 변경·삭제, 추가 runtime dependency와 Cargo.lock 변경은 없다.

## 실패에서 얻은 수정

긴 숫자를 붙여넣으면 예전 입력을 유지하던 첫 구현이 회귀 테스트에서 실패했다. 최대 64자만 유지하되 범위 오류를 노출하고 작업 시작을 거절하도록 수정했다. 숫자 parser는 ASCII 정수/상한을 검사하며 +1, 공백, 소수, overflow와 예약 옵션을 거절한다.

## 실행 명령

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
DLPAD_SMOKE_YTDLP=<existing-yt-dlp> DLPAD_SMOKE_FFMPEG=<existing-ffmpeg> cargo test -p dlpad-engine --test local_media --locked -- --ignored
cargo build --workspace --release --locked
DLPAD_TEST_FONT=<local-font> DLPAD_FFMPEG=<existing-ffmpeg> python3 scripts/native_probe.py target/release/dlpad docs/validation/iced-ffmpeg-selection.json --page downloads --seconds 5
DLPAD_TEST_FONT=<local-font> python3 scripts/native_probe.py target/release/dlpad docs/validation/iced-advanced-options.json --page downloads --advanced --scroll-end --seconds 5
python3 scripts/test_native_probe.py
python3 scripts/check_docs.py
```

최종 fmt/clippy/test/release 성공. Rust 82 tests(앱 7/core 17/engine 58), 실제 도구 opt-in 1 test 성공(21.78초), GUI 안전 4 tests 성공. 문서 검사 54개 성공. 실제 도구 시험은 Python 127.0.0.1 서버·FFmpeg 합성 2초 미디어와 임시 폴더만 사용하며 실제 영상/계정/Documents에 접근하지 않는다. 실제 FFmpeg를 한글·공백 경로로 임시 복사해 명시 선택, 전체 및 빠른/정확 구간 저장·decode·DB reopen 이력을 확인한다. GUI 스크롤 입력도 별도 Xvfb에만 보내며 desktop 자동 fallback은 없다.

다른 OS, 실제 IME/접근성, 장시간 작업, 실제 사이트와 관리 도구 공급/설치·자막/추가 품질 UI는 미검증/미구현이다. 외부 파일의 버전 확인 뒤 변경을 차단하지 않으며 기록은 확인 시점의 증거다.

## 최종 가상 화면 측정

[FFmpeg 입력 화면](iced-ffmpeg-selection.json): RSS 68,768 KiB, 5.03초 idle CPU 0%. [고급 옵션 스크롤 화면](iced-advanced-options.json): RSS 68,528 KiB, 5.01초 idle CPU 0%. 한글 label/도움말과 입력·시작 버튼 접근을 스크린샷으로 확인했다. release 7,878,112 bytes로 직전 저장 위치 구현 대비 27,872 bytes 증가했다. 추가 라이브러리 없이 UI·validation·SQLite component 저장 코드가 늘었으며 외부 도구 크기는 이 숫자에 포함하지 않는다. 짧은 idle 표본은 8시간 soak나 OS IME/스크린 리더를 대체하지 않는다.

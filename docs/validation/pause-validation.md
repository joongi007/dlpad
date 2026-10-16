# 일시 중단·취소 통합 검증

2026-10-01 Linux/WSL. [작업 계약](../work-items/S03-pause-action.md).

## 사용자 동작

실행 중 `일시 중단`은 소유 프로세스 종료 결과가 Cancelled로 확인된 경우에만 Job을 Stopping→Paused로 저장한다. partial은 보존하고 이력에서 같은 ID로 새 Attempt를 시작한다. `작업 취소`는 Cancelled이며 이력에서 재개하지 않는다. 첫 종료 요청이 확정된 후 서로 바꾸지 않는다. 자연 완료나 도구 실패를 pause 성공으로 숨기지 않는다. idle/복구 gate/이미 중단 요청된 작업에는 명령을 보내지 않는다.

engine의 execute_with_control/resume_with_control은 pause intent를 받아 기존 cancellation runner를 사용한다. pause=true를 cancellation=true보다 먼저 Release 저장하고 worker가 Acquire 확인한다. 기존 execute/execute_with_progress와 resume API는 취소 의미를 유지한다. 새 schema/도메인 상태는 추가하지 않았다.

## 실행 결과

- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`: 통과.
- `cargo test --workspace --locked`: 137 tests 통과(app10/core20/engine96/진단보존3/guardian8). 관리 직접 다운로드와 자막 helper의 당시 통합 snapshot을 포함한다. 실미디어 opt-in 1개는 기본시험 ignored.
- `cargo test -p dlpad-engine --lib download::tests::running_pause_ack --locked`: 실제 실행 fixture가 partial을 쓴 뒤 pause ack와 보존·같은 ID 재개 확인, 1/1 통과(2.49초).
- `cargo build --workspace --release --locked`: 통과; 7,974,752 bytes. 직전 7,958,304보다 16,448 bytes 증가. 이 통합 비용은 pause/typed subtitles를 함께 포함하며 아직 UI에서 호출하지 않는 HTTP 설치 코드의 전체 배포 비용 측정은 아니다.
- `python3 scripts/test_native_probe.py`: desktop 주입 거절을 포함한 6/6 통과.
- `DLPAD_TEST_FONT=<local-font> python3 scripts/native_probe.py target/release/dlpad docs/validation/iced-job-pause.json --page history --recover-jobs --resume-job --pause-job --seconds 2`: 가상 화면에서 이력 재개→실제 Downloading partial 생성→다운로드 화면의 일시 중단 버튼→Paused/Attempt2/동일 partial bytes 보존. [측정](iced-job-pause.json), [화면](iced-job-pause.png).
- `python3 scripts/subtitle_smoke.py <existing-yt-dlp>`: 주 담당 독립 실행 통과(0.81초); manual ko와 명시적 auto ko/en 합성 VTT bytes 확인. 자막 production 다중 artifact 연결 완료는 아니다.

첫 GUI 입력은 버튼 좌표가 틀려 Downloading이 유지됐다. 가상 화면을 직접 확인하고 실제 버튼 좌표로 하네스만 수정한 뒤 통과했다. 임시 폴더·기존 로컬 폰트·loopback 합성 fixture를 사용했다. 사용자 desktop·Documents·실제 영상/자격 증명은 사용하지 않았다.

## 의존성과 미검증

주 담당이 ureq=3.4.2를 정확 pin하고 Cargo.lock을 갱신했다. MSRV1.85이며 프로젝트1.94에서 컴파일. default features를 끄고 rustls만 사용한다(ring/WebPKI roots); gzip/cookies/json/socks/platform verifier를 추가하지 않았다. lock에는 TLS/HTTP 포함 신규15packages. 연결/인증서/TLS의 런타임 비용과 최종 UI 호출 뒤 binary 크기는 관리 설치 연결 검증에서 따로 측정한다.

Windows/macOS pause 실행, OS sleep/logout, 실제 만료 URL·모든 partial format, guardian fatal orphan 보호, UI 백그라운드 복귀는 남는다. 8시간 headless guardian 시험은 시작됐지만 아직 완료가 아니며 이 2초 GUI 표본으로 대체하지 않는다. 전체 P0/S03/S00 완료가 아니다.

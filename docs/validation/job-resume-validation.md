# 동일 ID 작업 재개와 병렬 P0 통합 검증

2026-10-01 Linux/WSL, Rust 1.94.0. 정상 Git 저장소가 없으며 기존 .git을 보존했다. 사용자 Documents·실제 영상·계정 대신 tempfile 합성 fixture와 Xvfb를 사용했다.

## 사용자 결과와 계약

작업 이력에서 Paused/Failed/Interrupted 작업을 원래 ID와 옵션으로 명시적으로 재개한다. 새 Attempt를 만들고 같은 `.dlpad-partials/<job-id>` 경로를 재사용한다. 완료·취소·진행 작업은 재개하지 않는다. 새로운 UI 목록은 keyset 페이지당 128개, URL을 노출하지 않으며 옵션 없는 legacy 작업은 이유를 안내하고 버튼을 비활성화한다.

Download adapter v4의 options_json은 버전 1 DTO다. core는 128 KiB 상한과 URL·품질·구간·예약 옵션을 재검증하고 원래 argv snapshot과 비교한다. 누락·손상·변조·미지원 버전이면 새 Attempt 전에 실패한다. SQLite 원본 URL은 암호화된 비밀 저장소로 표현하지 않는다. 새 dependency는 없다. 이력은 작업 ID 순서이며 시간순 최신 목록이나 완료·취소 작업 복제 기능은 이번 범위가 아니다.

## 실제 검증

- `cargo clippy --workspace --all-targets --locked -- -D warnings`: 통과.
- `cargo test --workspace --locked`: 116 tests 통과(app 9/core 18/engine 78/진단 보존 3/guardian 8), 실제 미디어 opt-in 1개는 기본 시험에서 ignored.
- `cargo build --workspace --release --locked`: 통과. 앱 7,958,304 bytes. 이전 7,901,184 bytes보다 57,120 bytes 증가; 옵션 DTO/이력 UI·조건부 rename·고장 drain을 포함한 통합 비용이며 개별 기능별 비용으로 해석하지 않는다.
- `cargo test -p dlpad-engine --lib download::tests::resume_keeps_id --locked`: partial 보존 assertion 추가 후 1/1 통과(2.02초). 합성 실행 파일이 원래 partial bytes를 읽고 동일 Job ID의 두 번째 Attempt를 완료했다.
- `DLPAD_SMOKE_YTDLP=<existing-tool> DLPAD_SMOKE_FFMPEG=<existing-tool> cargo test -p dlpad-engine --test local_media --locked -- --ignored`: 실제 yt-dlp/FFmpeg 합성 전체·구간/decode 1/1 통과(24.75초).
- `python3 scripts/test_native_probe.py`: GUI 안전 회귀 5/5 통과. 재개 input injection은 desktop에서 실행 전에 거절한다.
- `DLPAD_TEST_FONT=<local-font> python3 scripts/native_probe.py target/release/dlpad docs/validation/iced-job-resume.json --page history --recover-jobs --resume-job --seconds 5`: 실제 release/Xvfb 버튼으로 startup Interrupted → Downloaded, 원래 ID와 Attempt 2개, 검증된 최종 파일 확인. [측정](iced-job-resume.json), [화면](iced-job-resume.png).

첫 GUI fixture는 저장 루트를 app data 안에 만들어 겹침 정책에 거절됐다. fixture를 별도 sibling 폴더로 수정한 뒤 통과했으며 제품 정책을 완화하지 않았다. 옵션이 없는 legacy 복구 작업의 비활성 버튼도 [별도 화면](iced-job-history.png)에서 확인했다. 최종 재개 GUI RSS 69,760 KiB/5.01초 CPU 약 0.60%; 짧은 동작 표본으로 idle/8시간 예산을 증명하지 않는다.

병렬 결과의 library v2 nullable identity migration은 주 담당이 조정하고 v1 실제 row 보존·재개통 테스트를 실행했다. 조건부 rename, hardlink ambiguity, scan rollback은 [전용 실증](library-identity.md)을 따른다. 진단 cache 0 byte와 사용자 파일 보존은 [filesystem 시험](diagnostic-cache-validation.md)을 통합 실행했다. guardian 종료 신호·bounded drain 시험도 포함하며 SIGKILL orphan 보호를 완료로 주장하지 않는다.

## 미완료

OS sleep/logout, 8시간 soak, Windows/macOS 프로세스·identity·실제 UI/IME/접근성은 미검증이다. guardian 자체 SIGKILL에 분리된 자손이 남는 출시 차단을 유지한다. UI 재개가 yt-dlp의 모든 사이트/만료 URL/partial format 호환을 보장하지 않는다. 일시 중단 명령과 백그라운드 복귀 UX, 관리 공급 설치, 실제 FFmpeg 처리 지표 연결은 후속 P0다. 전체 P0/S03/S00 출시 완료가 아니다.

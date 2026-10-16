# 숨김·복귀와 처리 지표 최종 통합

2026-10-02. 사용자가 요청한 통합 대기와 백그라운드 UI 범위만 수행했다. 전체 P0 완료 또는 공개 릴리스가 아니다. 정상 Git 저장소가 없어 기존 .git을 보존했으며 commit/원격CI를 수행하지 않았다.

## 사용자 동작과 파일

진행 중 닫기의 세 선택은 돌아가기, 중단 후 종료, 백그라운드 계속이다. Linux에서 private 복귀 endpoint가 준비되면 같은 창을 숨기고 기존 작업을 유지한다. 같은 앱/data로 명시 재실행하면 기존 창만 표시하고 launcher는exit0로 종료한다. 완료 이벤트는 창을 자동 표시하거나 focus를 요청하지 않는다. Windows/macOS·unsafe/stale endpoint는 숨김을 비활성화하고 창을 유지한다. 중단 후 종료를 선택한 뒤에는 다른 선택을 비활성화해 종료 의도를 고정한다.

수정 파일: app/main.rs·catalog.rs, engine/activation.rs·instance.rs·processing_endpoint.rs·metrics.rs·download.rs·process.rs·lib.rs, tests/local_media.rs, scripts/background_ui_probe.py. README·CONTEXT·UX·ADR0002·P0 상태·작업 계약을 갱신했다. Cargo/schema/lockfile 신규 변경은 없다.

Linux job-owned FIFO는 FFmpeg merge/audio/구간 처리 기록을 capacity1 최신 feed에 전달하며 UI는 처리 배속/가능한ETA를 다운로드B/s와 구분한다. stale/end/완료는 미측정으로 되돌리고 완료 증거로 사용하지 않는다. Windows/macOS 전용 progress transport는 미구현이다. 계측 준비 실패는 미측정으로 되돌리며 별도 진단 항목은 후속이다. 큰 u64 구간의 시간단위 변환은 checked 연산을 사용하며 overflow를 unknown ETA로 처리한다.

## 실행 명령과 실제 결과

프로젝트 전용 Rust1.94.0/Cargo home으로 다음을 실행했다.

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --release --locked
python3 scripts/check_docs.py
DLPAD_SMOKE_YTDLP=/home/joongi/.local/bin/yt-dlp DLPAD_SMOKE_FFMPEG=/usr/bin/ffmpeg cargo test -p dlpad-engine --test local_media --locked -- --ignored --nocapture
```

최종4개 Cargo 공통 명령 모두exit0. workspace186tests passed/4ignored(명시실행 도구·장시간 게이트), 실제합성미디어 통합 별도1passed/36.67초. 최초 실제도구 명령은 필수env 미설정으로 실패한 뒤 정확경로로 재실행했다. 초기 workspace의 flood시험은 완료 경로 누락으로 실패했으며 stdout/stderr 로그예산을 각각 분리해 재실행을 통과했다. [해당 수정](../work-items/S01-stream-evidence.md)을 따른다. 위 로컬 경로는 실제 실행 증거이며 코드나 의존성의 고정 홈 경로가 아니다.

최종 Linux release SHA256: e9fa3ce36b3e4a1285ef0ee5930179ee4fa126451730fd0cbdcb2bae020bcc7f,9882048B. [가상 화면 기록](background-ui-validation.md)의 최종 JSON과 이미지를 대조했다. primary/worker PID와 startticks·job/attempt를 유지한 숨김map0→복귀2, heartbeat 증가, secondaryexit0, owned프로세스/temp정리를 확인한다. 기존 Return/StopExit/unavailable/hidden완료/processing 화면 증거는 각각 기록된 이전binary해시 범위이며 최종binary 핵심재검증을 별도로 남긴다.

## 비용과 미검증 사항

새 라이브러리와 웹 UI/server를 추가하지 않았다. 복귀 listener1thread와 UI capacity1 tick1thread(250ms), 처리 endpoint당 reader1thread(4KiBbuffer·20ms poll·bounded64KiBparser)가 추가된다. 기본 thread stack virtualmemory와 hidden창/renderer는 유지된다. coarseXvfb RSS는 visible약68MiB/hidden약70MiB로 자원 해제나 비용 감소를 입증하지 않는다. 마지막 작업 완료 후에도 숨김 상태를 유지해 명시복귀를 기다린다.

실제 WM/트레이·3OSGUI/IME·logout/sleep·foreignUID handshake·sameuser TOCTOU 완전차단·실제사이트/8시간미디어/설치서명은 미검증이다. 두 기존8시간 합성 시험은 이번 binary 이전 snapshot이고 아직완료로 표시하지 않는다. 사용자 Documents/실제영상/계정/desktop은 시험에 사용하지 않았다.

최종 binary GUI 독립 대조: final-background.json은8.182초/hidden0→restore2/동일jobattempt/heartbeat/secondaryexit0, final-processing.json은37.192초이다. 두JSON SHA를 실제release와대조했고주담당이final-processing-processing.png/finished.png를직접보아2.50×분리표시와완료후제거를확인했다. 최종문서검사119개통과.

## 최신 사용자 범위의 통합 재확인 (2026-10-02)

통합 대기와 숨김·복귀 UI 요청에 맞춰 최신 공통 검증을 재실행했다. `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`, `cargo build --workspace --release --locked` 모두 exit0; 187 passed/5 ignored. 새 실제 HTTP failure 시험은 기본 실행에서는 ignored이며 해당 외부 도구 시험 완료로 확대하지 않는다. 현재 Cargo/lock에는 별도 S00 winit vendor patch가 포함돼 위의 이전 snapshot과 구분한다.

`DLPAD_TEST_FONT=/mnt/c/Windows/Fonts/malgun.ttf python3 scripts/background_ui_probe.py target/release/dlpad docs/validation/background-ui/integration-refresh-background.json --mode background --complete-hidden` exit0/36.462초. [새 JSON](background-ui/integration-refresh-background.json)의 SHA256 `7e5fe81444a31806189e3a84d51d1ccf809f48bd3c83df1396c9d2fbabacd469`를 실제 release와 독립 대조했다. 동일 worker/job/attempt 유지, hidden heartbeat 증가, 완료 후 자동 창 표시 없음, 명시 재실행 뒤에만 map0→2, launcher exit0, owned 프로세스·임시 fixture 정리를 확인했다. Xvfb만 사용해 사용자 화면을 방해하지 않았다.

Windows 복귀 backend는 이후 추가됐으며 이전 미지원 문장은 당시 Linux snapshot 범위다. [현재 Windows GUI 증거](windows-background-gui.md)는 초기 active 입력 실패를 보존하며, 이후 일반 실행 인자의 private desktop 합성 작업에서 숨김·동일 worker 유지·재실행 복귀·중단 후 종료를 3회 통과했다. 주 담당은 최종 JSON의 worker 소유권/시작 시간, heartbeat 증가와 두 앱 exit0를 직접 대조했다. 실제 영상·IME·로그아웃·절전과 macOS는 미검증이다. 전체 P0 완료를 뜻하지 않는다.

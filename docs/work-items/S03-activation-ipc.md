# S03: Linux private restore IPC와 격리 GUI 검증

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: diagnostic_cache agent; 신규 crates/engine/src/activation.rs·전용 tests, scripts/background_ui_probe.py, 이 계약과 전용 validation. lib.rs/app/Cargo는 주 담당 소유.
- 선행 작업: 기존 InstanceGuard 및 주 담당 hidden-window/restore UI

## 사용자 결과

같은 private data로 앱을 명시적으로 다시 실행하면 기존 앱의 창을 복귀시킨다. 별도 worker/job을 시작하지 않으며 확인할 수 없는 endpoint에서는 background를 활성화하지 않는다.

## 계약과 범위

주 담당이 Linux Unix socket/fixed restore IPC를 채택했다. API ActivationServer::bind(private_data)->Result, poll_restore()->bool, send_restore(private_data)->Result; 다른OS Unsupported. private root/기존 instance owner marker/peerUID 검증, 0600 socket, 고정≤64B message, bounded flag, nonblocking owned worker/deadline, Drop join과 정확 identity 소유 cleanup. 기존/stale endpoint는 filename만으로 삭제하지 않는다. arbitrary command/path 실행 권한이 없다. 세부 authority 계약을 주 담당에게 제안한 뒤 구현한다.

## 인수 조건

정상 request/ack/coalescing, peer/root/marker/foreign/stale/symlink/replacement 보존, oversized/invalid/idle request deadline와 owned thread/socket 종료, otherOS failclosed. GUI는 새 binary와 공유 interface 합의 후 Xvfb WM_DELETE_WINDOW/map_state/PID·heartbeat·journal/secondary restore로 실증한다.

## 검증과 handoff

주 담당이 private nonce 계약과 API를 채택하고 lib.rs를 연결했다. 신규activation.rs 구현: 기존owner marker/uid0700+socket0600+tokencreate_new0600/32hex nonce/peerUID/49B fixed request, boundedflag/thread/drop identity cleanup. shared app/Cargo 변경 없음. `cargo test -p dlpad-engine --lib activation:: --locked`10/10(0.63초), engineall-targetclippy(8.91초) 통과. 실제UID1000cleanup PermissionDenied 보존과 malformed/oversize/orphan/symlink/변조/drip/ACK실패경로 포함.

현재 backend 완료; 실제 hidden/restore UI와 새binary/OS 통합은 주 담당 후속이며 이 작업 상태는 그 gate 완료 후 통합할 수 있다. [전용 조사/검증](../validation/background-ui-validation.md)에 same-user rawpath race/foreign UID 실제시험없음/stale failclosed/nonLinux 미지원과 runtime 비용을 기록했다.

최종 release SHA256 e9fa3ce36b3e4a1285ef0ee5930179ee4fa126451730fd0cbdcb2bae020bcc7f의 background8.182초/processing37.192초 재시험과 화면검토 완료. [최종 증거](../validation/background-ui-validation.md). Linux Xvfb 범위로 done이며 다른 OS/실제 WM/외 UID/악의적 same-user path race gate는 문서에 유지한다.

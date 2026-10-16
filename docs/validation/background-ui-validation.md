# 닫기·background 복귀 조사와 검증 준비

2026-10-02. [GUI 작업 계약](../work-items/S03-background-ui-probe.md), [IPC 계약](../work-items/S03-activation-ipc.md).

감사 시작 시 app의 Close는 active worker/managed install이 있으면 StopExit/Return만 보여주고, 없으면 iced::exit였다. background 숨김·복귀는 아직 구현되지 않았으며 InstanceGuard 중복 실행은 AlreadyRunning으로 거절했다. UX 문서의 목표를 당시 구현 완료로 표현하지 않는다. 주 담당은 새 background UI와 lib.rs 모듈 연결을 소유한다. 아래 Linux 실증은 새 구현 binary에 한정한다.

Pinned iced_runtime0.14의 window::set_mode(id,Mode::Hidden), iced_winit0.14.1의 SetMode→raw.set_visible(false)를 실제 dependency 소스에서 확인했다. hidden은 같은 window/app/task를 유지하는 방법이며 renderer/GPU/window 자원 완전 해제를 의미하지 않는다. 트레이 dependency를 새로 추가하지 않고 private Linux Unix IPC의 명시 재실행으로 복귀하는 계약을 채택했다. Linux 외 IPC는 Unsupported이며 복귀 수단이 검증되지 않으면 background를 활성화하지 않는다.

## Linux restore-only backend

[activation.rs](../../crates/engine/src/activation.rs)는 API ActivationServer::bind/private-data, poll_restore()->bool, send_restore/private-data를 제공한다. caller가 InstanceGuard를 계속 소유해야 한다. root uid/mode0700·selected symlink/parent traversal 거절, 기존 .dlpad-data-owner exact marker/uid/non-group-world-write, kernel SO_PEERCRED의 effective UID를 대조한다. 새 activation.token은 create_new0600, Linux /dev/urandom16 bytes에서32hex instance nonce를 만든다. wire는 정확49B: ASCII `DLPAD_RESTORE_V1`16B+nonce32B+newline1B뿐이다. 명령/작업/path payload는 받지 않는다.

activation.sock은0600/create-only다. 기존 socket/file/token은 stale라는 이유로 삭제하지 않는다. listener는 nonblocking, 요청 buffer64B, per-cycle accept8, restore flag1개로 coalesce, 전체 request250ms/read 및 ACK250ms, sender nonblocking connect1초를 적용한다. foreground의 poll_restore는 atomic swap뿐이다. Drop은 owned stop flag/Condvar wake/thread join 후 root/marker/token내용·token dev+ino/socket dev+ino가 일치한 자기 파일만 개별 unlink한다. 실패/변경된 marker·token·path는 보존한다.

Cargo 추가/변경은 없고 기존 Unix libc만 사용한다. worker thread1개와 기본 thread stack virtual-memory 비용,25ms idle wait/최대40Hz wake가 추가된다. 실제 app RSS/CPU/hidden renderer 비용은 새 GUI binary 실증 때 측정해야 하며 여기서 수치를 만들어내지 않는다. OS/file I/O scheduling까지 hard realtime으로 제한했다고 주장하지 않는다.

검증: 처음은 주 담당 lib.rs를 수정하지 않고 `/tmp/dlpad-activation-check`의 오프라인 임시 crate에서8tests/clippy를 실행했다. 이후 주 담당 모듈 연결 후 `cargo test -p dlpad-engine --lib activation:: --locked` 최종10/10 통과(0.63초), `cargo clippy -p dlpad-engine --all-targets --locked -- -D warnings` 통과(8.91초). root/mode/marker/symlink/foreign/stale/replaced endpoint, changed/orphan/public/symlink nonce, fixed-message oversize/invalid, idle/drip whole deadline, sender ACK timeout/invalid ACK, coalescing/drop를 포함한다. 실제UID1000에서 rootchmod0500 cleanup PermissionDenied 시 socket/token/marker 보존과 다음 bind failclosed를 확인했다. Root UID이면 이 permission fixture는 skip하며 성공 증거로 확대하지 않는다.

외 UID process와의 실제 socket 통신 시험은 수행하지 않았다. SO_PEERCRED는 정상 local same-UID handshake에서 사용했으며 실제 foreign UID 보장 실증은 남는다. std path 검사/unlink 사이 악의적인 same-user race를 완전히 막지 못한다. unknown stale token/socket은 보존하므로 crash 후 endpoint가 남으면 background가 unavailable일 수 있다. handle-relative cleanup, stale identity recovery, Windows named pipe/macOS IPC는 남은 gate다.

## 격리 GUI harness

[background_ui_probe.py](../../scripts/background_ui_probe.py)는 Xvfb만 사용하고 WM 없는 환경에서 정확 owned 창에 WM_DELETE_WINDOW ClientMessage를 보내며 XGetWindowAttributes.map_state(0 Unmapped/2 Viewable)를 관측한다. primary PID/startticks와 synthetic worker PID/startticks·heartbeat 증가·job ID/attempt 수를 비교한다. 같은 binary/data로 secondary를 실행하면 exit0와 같은 창 복귀/worker 중복 없음을 요구한다. Return은 visible/live 유지, StopExit는 Cancelled journal·owned worker 종료 뒤 app exit를 요구한다. temp app-data는0700이며 media와 sibling이다.

AST/인자10건 통과. 실제 Xvfb의100x100 owned selfcheck window에서 Xlib struct ABI, WM_DELETE_WINDOW event 전달, map_state2→0을 확인했다. 이 selfcheck는 native product GUI/WM/트레이/hidden 작업 유지를 검증한 것이 아니다. 실제 binary 제공 후 inspect screenshot로 close 선택 좌표를 먼저 확인한다. 예상 좌표를 제품 GUI 성공으로 취급하지 않는다.

```sh
DLPAD_TEST_FONT=<fixture-font> python3 scripts/background_ui_probe.py target/release/dlpad /tmp/new-background-inspect.json --mode inspect
DLPAD_TEST_FONT=<fixture-font> python3 scripts/background_ui_probe.py target/release/dlpad /tmp/new-background-restore.json --mode background --background-point X,Y
```

새 product binary/hash와 native GUI 숨김/복귀, 실제 WM/tray, hidden 자원 측정, 마지막 작업 완료 정책, sleep/logout/otherOS는 아직 미검증이다. 사용자 desktop/실제 영상/계정은 사용하지 않으며 Xvfb 실패 시 visible fallback하지 않는다.

## 제품 Linux Xvfb 실증

2026-10-02 binary SHA256 `1ef5026d01476191b37bb3c8067db93dca29f64a6d465d8a81e5012da39fc134`, owned Xvfb WM 없음,960×640/한글 font fixture. `python3 scripts/background_ui_probe.py target/release/dlpad <evidence.json> --mode <mode>`를 실행했다. 닫기 버튼 실측 좌표 StopExit80,604 / Return180,604 / Background295,604.

- [Return](background-ui/return.json) 7.264초: visible worker/heartbeat 유지. [화면](background-ui/return.png)에서 닫기 선택 row 제거를 직접 확인했다.
- [StopExit](background-ui/stop-exit.json) 6.642초: app exit0, owned worker 종료, journal Cancelled/attempt1. 두 시험은 긴 temp 경로로 endpoint unavailable였으나 Return/StopExit는 정상이다.
- [Background/복귀](background-ui/background-short-root.json) 8.183초: socket 준비, 같은 app/worker PID/startticks와 job/attempt 유지, hidden map_state0/복귀2, hidden heartbeat 증가. 동일 data 재실행 launcher exit0. RSS visible68712/hidden70220/restored70228KiB는 단일 coarse 관측이며 GPU 해제나 비용 감소 증거가 아니다.
- [복귀 불가](background-ui/unavailable.json) 7.372초: 소유 temp foreign endpoint bytes 보존, visible worker 계속. [화면](background-ui/unavailable.png)의 Background disabled와 안전한 복귀 unavailable 안내를 직접 확인했다.

첫 [Background 실패](background-ui/background.json)는 긴 evidence/temp Unix socket 경로로 endpoint 준비 실패했다. JSON/오류 화면을 보존했고 harness data/media sibling을 짧은 /tmp TemporaryDirectory로 바꿔 정상 복귀를 통과했다. 모든 GUI/worker/Xvfb/temp는 정리됐다. 사용자 desktop/실제 영상/계정은 사용하지 않았다.

실제 WM/tray, hidden 마지막 작업 완료·sleep/logout, 외 UID 통신 및 다른 OS는 미검증이다. 새 binary에 이 hash 결과를 자동 확대하지 않는다.

추가 [hidden 완료](background-ui/complete-hidden-corrected.json) 37.939초: synthetic job Downloaded 이후 map0를 유지하고 명시 재실행 후에만 복귀2. 최초 complete-hidden log는 harness가 실제 Downloaded를 Completed로 잘못 기다린 뒤 hidden screenshot XGetImage BadMatch를 발생시킨 실패로 보존했다. terminal 상태명과 hidden error screenshot을 수정한 재시험이다. 실제 영상 처리 완료/OS background 정책 실증으로 확대하지 않는다.

## 합성 processing FIFO 표시

[processing JSON](background-ui/processing.json) binary SHA256 `1c95ebe9bbe5cbe7783a5342a7013e1ece4f6ba2c022fb21aa2e6bb1be1efbb5`,37.197초. owned synthetic yt 도구가 postprocessor argv의 FIFO에 out_time_us=1000000/speed=2.5x/progress=continue를 전송했다. [실행 중 화면](background-ui/processing-processing.png)에 다운로드0.0KiB/s와 별개 미디어 처리 배속2.50×/남은시간12초를 직접 확인했고, [완료 화면](background-ui/processing-finished.png)에 처리 측정 불가로 되돌아감을 확인했다. journal Downloaded/15byte synthetic artifact1/attempt1. 실제 FFmpeg/영상 배속 정확도 검증은 아니다.

## 최종 release 재검증

공통 fmt/clippy/workspace tests/release 완료 후 주 담당이 source freeze한 최종 binary SHA256 `e9fa3ce36b3e4a1285ef0ee5930179ee4fa126451730fd0cbdcb2bae020bcc7f`에 같은 필수 시험을 다시 실행했다. [final-background](background-ui/final-background.json)8.182초: endpoint 정상, hidden0→restore2, 같은 app/worker startticks·job/attempt, heartbeat 증가, secondary exit0. [final-processing](background-ui/final-processing.json)37.192초: synthetic FIFO opened, Downloaded/15B artifact1/attempt1. [진행 화면](background-ui/final-processing-processing.png)의 다운로드0.0KiB/s와 별개 처리2.50×/ETA12초 및 [완료 화면](background-ui/final-processing-finished.png)의 처리 측정 불가를 직접 검토했다. 모든 owned GUI/worker/temp 정리 확인. 이전 binary의 Return/StopExit/unavailable/완료숨김 기록은 해당 hash 범위로 유지하며 최종 binary에서 반복하지 않았다.

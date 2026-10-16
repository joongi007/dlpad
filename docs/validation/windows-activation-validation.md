# Windows background 복귀 backend 검증

2026-10-02, Windows 11 10.0.26200.9457 x64 / Rust 1.94.0 Windows GNU. GUI가 아닌 native backend의 완료 증거이며 Windows MSVC/macOS와 제품 GUI 복귀를 대체하지 않는다.

## 사용자 동작과 경계

동일한 사용자 SID와 canonical data root의 앱 재실행은 고정 복귀 요청만 전달한다. `ActivationServer::bind`, `poll_restore`, `send_restore`를 Linux와 동일한 공유 API로 제공한다. InstanceGuard가 server보다 오래 살아야 한다. 준비 실패에는 공유 UI가 background 숨김을 거절한다.

로컬 named pipe는 SID/root SHA-256 이름, FIRST_PIPE_INSTANCE, REJECT_REMOTE_CLIENTS, current-user-only protected DACL을 생성 시 설정한다. 클라이언트와 서버 모두 상대 PID를 query handle로 열어 TokenUser SID를 확인하며 상대 프로세스를 종료하지 않는다. nonce는 BCryptGenRandom의 128-bit 임의 값을 32-byte hex로 저장한다. 기존 nonce/pipe를 교체하거나 제거하지 않는다.

root와 marker의 owner SID/ACL, root native file ID, nonce의 protected single-user ACL/native ID/길이를 확인한다. Root는 SYSTEM/Administrators의 OS 권한을 허용하지만 다른 SID의 쓰기·삭제·ACL 변경 권한은 거절한다. 기본 root ACL을 자동 변경하지 않는다. 테스트 ACL 변경은 새 synthetic fixture에 한정했다. nonce 읽기는 다른 SID에 허용하지 않는다. root 디렉터리 handle은 삭제 공유 없이 유지한다. nonce 생성 도중 쓰기/sync/ACL 실패는 확인된 동일 native ID에 한해 정리하며 ID를 얻지 못하면 보존한다. Drop 역시 marker/root ID/nonce ID/bytes가 일치할 때만 nonce를 정리한다. 최종 경로 기반 삭제와 ancestor TOCTOU가 원자적으로 차단된다는 주장은 하지 않는다.

## Windows 프로토콜과 제한

요청은 `DLPAD_RESTORE_V1` + 32-byte nonce + newline, ACK는 `DLPAD_RESTORE_QUEUED_V1\n`, receipt는 `DLPAD_RESTORE_RECEIVED_V1\n`이다. 요청/응답 버퍼는 각 64 bytes 이하, 하나의 message만 처리하며 부분·초과·임의 명령은 거절한다. 요청 읽기부터 ACK/receipt까지 전체 250 ms deadline을 사용하고 접속 대기는 최대 1초다. 복귀 요청은 AtomicBool 하나로 병합한다. ACK는 창이 실제 표시됐다는 보장이 아니라 요청이 대기열에 들어갔다는 뜻이다.

첫 native 시험에서 정상 요청 3개가 OS 오류 233으로 실패했다. DisconnectNamedPipe는 읽지 않은 pipe 출력을 버린다. ACK 직후 연결을 끊지 않고 제한 시간이 있는 receipt 읽기를 기다리도록 고쳤다. FlushFileBuffers는 상대의 읽기 완료까지 막힐 수 있으므로 사용하지 않았다. 이 동작은 [Microsoft DisconnectNamedPipe 문서](https://learn.microsoft.com/en-us/windows/win32/api/namedpipeapi/nf-namedpipeapi-disconnectnamedpipe)에 근거한다. 기본 named-pipe ACL은 Everyone/anonymous 읽기를 허용할 수 있어 [Microsoft 보안 문서](https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipe-security-and-access-rights)에 따라 생성 시 private ACL을 지정했다.

Overlapped I/O/event/buffer는 완료 또는 CancelIoEx 후 실제 완료까지 살아 있다. Drop은 stop flag, owned pipe cancel, worker join을 수행한다. 사용자 desktop으로 전환하거나 창/콘솔을 띄우지 않는다.

## 실제 실행

`python3 scripts/windows_native_probe.py test`: activation 12개(독립 client 진입점 1개 포함), engine 71 pass / 1 ignored, app 14, core 21, diagnostic cache 3, Windows process 6 pass. Linux 전용 integration 파일의 Windows 0 tests를 해당 동작 검증으로 계산하지 않는다. 새 same-user child process는 CREATE_NO_WINDOW와 argv를 사용하고 3초 제한으로 소유 child만 종료한다.

시험은 정상 ACK/coalescing/Drop/rebind, 독립 process IPC, 중복 bind, 잘못된 nonce/명령/65-byte 메시지, idle/부분 요청/대기 읽기 중 Drop, 없는·잘못된 ACK, 없는·잘못된 receipt, foreign pipe/token, marker/nonce 변경, replacement native identity 보존, Everyone-writable root 거절, provisional identity 정리를 포함한다. 읽기 대기 중 Drop은 1초 미만, 연결 처리 및 ACK/receipt 누락은 지정 제한 시간으로 끝났다. 실제 생성 실패의 모든 OS 오류를 주입한 시험은 아니다.

최종 fmt/clippy/test/release의 정확한 argv, 시간, source input hash와 로그는 [최종 manifest](windows-activation-final.json)에 기록한다. 매 native host 실행이 `private-desktop=true`, `dlpad-input-desktop-unchanged=True`를 보고했다. 정상 Git 저장소가 없어 기존 .git을 보존했다.

## 미검증과 후속

Windows 제품 GUI의 숨김→재실행→창 복귀/worker 계속 실행은 별도 private desktop 실증이 필요하다. native IPC 성공만으로 R08 GUI 인수를 완료하지 않는다. 기본 AppData root 정책의 전체 사용자 환경, 실제 다른 사용자 SID/다른 로그온 세션/remote 접속, 관리자 우회, reparse ancestor 경합, Windows MSVC/macOS는 미검증이다. 제한적인 ACL 형식(allow ACE만 허용)이나 unsafe ACL/root/marker/stale endpoint에는 fail closed한다. 새 dependency나 제품 런타임을 추가하지 않았고 기존 windows-sys의 IO/Security_Authorization/Security_Cryptography bindings만 공유 Cargo 담당자가 활성화했다. 이 binding feature는 추가 daemon이나 GUI 서버를 실행하지 않는다.

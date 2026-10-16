# S03: Windows guardian 게이트 프로토콜 구현 계약

- 상태: completed (읽기 계약 감사; helper 구현·native 인수 미완료)
- 분류: architecture
- 우선순위: P0
- 요구사항: R08, R02
- 담당/수정 범위: soak_recovery_audit — 이 문서만
- 선행 작업: root 공유 schema 조정, Windows records/journal 및 PreparedChild

## 사용자 결과와 기존 근거

인증 metadata/download 실행 전에 durable 소유권 결합을 완료하고, app이 사라져도 original Job 정리 후에만 cookie reader receipt를 발급하는 helper 계약을 고정한다. 현재 코드를 이 기능이 구현 완료된 것으로 표현하지 않는다.

읽은 source SHA256: `process/windows.rs` e6117333895f53be9d632fa4d837bce379c2c34120325249b0c9682ffde2fedc, `cookies/windows/journal.rs` dd231d3dae0e0ba2fd1d5eaec3d28f27a79122177418008d1c8a08a8dd9fb4db, `cookies/windows_records.rs` 1bb0549b41a9bda7351de73f605266eaefde2e4d6f9b33454d9cb2cb367968f9.

`PreparedChild::prepare_with_stdio` (windows.rs:535) 생성은 original kill-on-close Job에 suspended tool을 결합하며 NUL stdin과 원 stdout/stderr writer의 새 duplicate만 tool HANDLE_LIST로 전달한다. `creation_times`는 original helper/tool process handle에서 얻지만 그 숫자만으로 정리 권한을 부여하지 않는다. `cancel`은 original Job signal→child wait→confirm_terminal이다. Journal reopen은 SID/native anchors/receipt pins 검증 및 `try_lock`을 요구한다. `commit`은 write/set_len/sync_all 후 재검증하며 실패 시 poisoned이다. 두 slot의 Intent/Bound/Resolved와 4096-byte record, 256-byte completion receipt는 기존 format 계약이다.

## 필요한 순서와 역할

1. app이 lease slot의 Intent와 nonce를 durable commit한다. metadata slot 0, download slot 1을 재사용하지 않는다.
2. helper가 private config/control 및 receipt capability를 검증하고 original Job+tool suspended 상태를 만든다. 이 시점까지 tool instruction은 실행되지 않는다.
3. helper→app native creation frame은 original process handles에서 얻은 helper/tool creation times를 담는다. app은 nonce/slot과 expected helper identity를 확인하고 durable Bound를 commit한다.
4. app이 journal lock을 실제 release한 뒤 helper가 자신의 새 open으로 journal try_lock, original anchor/SID/session nonce/record digest/Bound identity를 재검증한다. inherited journal handle을 공유해 lock 획득으로 가장하지 않는다.
5. helper→app ACK 이후 app→helper Begin이 와야만 ResumeThread를 호출한다. ACK 전에 Begin, duplicate Begin, 다른 nonce/slot은 거절한다.
6. 실행 완료/EOF/cancel/timeout은 original Job의 모든 active process 종료를 확인한 뒤 original receipt writer에만 completion을 쓰고 sync한다. app 재획득·resolve도 original receipt native identity와 payload matching을 검사한다.

## bounded 전송 제안과 shared 결정

root가 최종 frame/schema를 조정한다. config는 length-prefixed versioned framing, 최대 64 KiB와 총 argv 수/UTF-16 단위 상한을 둔다. Windows argv는 shell 문자열로 재해석하지 않고 기존 quote/build 계약으로 만든다. command/path 값은 오류 로그에 출력하지 않는다. control은 소형 fixed frame에 magic/version/kind/slot/nonce를 포함하고 creation에는 두 u64, ACK/Begin에는 Bound digest 결합을 요구한다. 예시 128-byte 이하 frame 상한은 제안이며 기존 schema 변경으로 채택된 것은 아니다.

startup 전체 deadline과 Bound/ACK/Begin 각 단계 deadline을 명시하고 blocking read를 무기한 두지 않는다. Linux의 기존 wait_begin_only 10초는 참고이며 Windows에서 같은 값/취소 interruptible read를 root가 결정한다. EOF/partial frame/oversize/unknown version/type/index/nonce mismatch는 Resume 금지 및 original Job 정리로 종료한다. ACK 송신 직후 EOF도 Begin 성공으로 취급하지 않는다. app의 cancel/timeout은 준비 중에도 control 종료를 통해 전달하며 tool이 suspended면 실제 tool side effect 0을 검증한다.

stdout/stderr는 tool→app 원 pipe로 직접 흐른다. helper가 중계해 로그 순서나 EOF를 바꾸지 않는다. helper의 writer duplicates는 tool 생성 직후 닫아 helper 생존 때문에 EOF가 지연되지 않게 한다. control/journal/receipt/Job HANDLE은 tool HANDLE_LIST에서 제외하며 helper inheritance 목록도 필요한 capability만 allowlist한다. Job HANDLE의 자손 노출은 kill-on-close/종료증명의 전제 자체를 깨므로 실패 인수에 포함한다.

## 실패 인수와 파일 소유권

실제 Windows synthetic fixture에서 정상 exit0/비영 종료/metadata 실패/download 실패/cancel/timeout/stdout-stderr backpressure와 EOF를 확인한다. 단계별 app kill(helper prepared, Bound 전/후, ACK 전/후, Begin 후), helper kill, 동시 kill, malformed/partial/no Begin, journal Busy/changed SID·anchors/receipt 교체를 구분한다. worker marker, original Job empty, original receipt bytes/hash, source/sentinel 보존, no secret in returned logs/DB, control HANDLE 미상속을 원시 증거로 남긴다. helper kill 또는 receipt 불확정은 clean receipt를 app이 조작하여 보완하지 않고 pending private copy를 보존한다.

root는 cookies 공개 wiring/shared records schema/process integration 및 helper dispatch를 조정한다. 독립 helper 담당에게는 새 Windows guardian module 및 전용 fixture만 배정할 수 있으며 현재 windows.rs/journal.rs와 동시 수정하지 않는다. Journal helper reopen/verify/receipt API는 현재 pub(super) 경계여서 최소 crate-private facade가 필요하고 root가 소유해야 한다. Journal unlock/relock은 기존 Journal 객체 lifetime과 lease pending-reader ownership에 맞춘 명시 API가 필요하다. numeric creation frame은 native provenance 대신 사용할 수 없으며 original anchors 검증을 생략하지 않는다.

## 실제 검증과 잔여

이 작업은 source/API 읽기와 문서 작성만 수행했다. Rust/Cargo/Windows native/manager 실행은 하지 않았다. 위 startup frame·deadline·facade의 최종 타입, suspended Job termination receipt 발급 구현, 실제 Windows failure matrix는 후속 구현·검증이다. Linux 기존 gate를 읽었다는 사실을 Windows 인수로 확대하지 않는다.

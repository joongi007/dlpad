# S04: 원 Deno cache의 Windows 독립 guardian 수명 계약 검토

- 상태: done (읽기 전용 계약 제안)
- 분류: review
- 우선순위: P0
- 요구사항: R04, R05, R08, R11, R12
- 담당/수정 파일: metrics_recovery — 이 문서만
- 선행 조건: OwnedDenoCache 기반, typed Windows runtime environment, 독립 managed/paired guardian

## 범위와 인수 조건

현재 소스 동결을 유지하고 Rust/Cargo/실제 프로세스를 변경하거나 실행하지 않는다. 최소 원 capability handoff, Windows 잠금의 실제 의미, app 종료와 helper 원 Tool Job 수명, 실패 보존과 필요 native 인수를 제안한다. cache path나 기존 owner record를 지금 다시 읽은 결과만으로 원 권한을 만들어내지 않는다. 아래 설계는 주 담당 채택·구현 전 제안이며 릴리스 기능 완료가 아니다.

## 현재 코드의 정확한 경계

`process/windows_helper_launch.rs:106`의 launch_encoded는 control reader/events writer/receipt/stdout/stderr 5개만 원 HANDLE_LIST로 넘긴다. paired는 원 cookie receipt를 index 3에 추가하여 6개다. inherited()는 DuplicateHandle로 실제 원 HANDLE을 복제하고 helper 생성 뒤 parent 임시 복제본을 즉시 닫는다. 추가 cache capability는 이 정확한 allowlist에만 포함해야 한다. tool의 stdio/handle allowlist에는 포함하지 않는다.

`process/windows_runtime_environment_config.rs`는 version/cache_path UTF-16 wire만 검증한다. `to_native()`는 path에서 policy를 구성하며 owner identity 검증이 아니다. 현재 `windows_paired_capture.rs:135`는 Some policy를 `runtime environment handoff unavailable`로 거절한다. 즉 기존 Some wire의 pure deserialize 성공은 guardian cache 권한 전달 완료가 아니다.

`process/windows_managed_guardian.rs:151`은 원 tool을 suspended로 prepare한 뒤 actual creation을 Prepared frame에 전달한다. app durable Bound와 Ack 이후 원 journal을 획득하고 helper Ack, Begin, Resume 순서를 수행한다. cookie/paired도 대응 gate다. `windows_owned_deno_cache.rs`는 현재 원 root/runtime/cache/owner/cache.lock을 보유하고 cache.lock에 app 자신의 try_lock을 유지하지만 helper 전달 기능은 없다.

읽은 소스 SHA256: helper launcher `7b502e6bb8390def38004d21fd1a36f3b2f6d1d4d6c8d95f27684c601f83e040`, runtime config `bef3783ae4e60a0b73f0502e156fb9ed9da59a93a0779a36f8a70311824e1215`, owned cache `6b55fda873f650ecc1753d1d9c1174b2410eb9a07fc3f585e831a96594b30e07`다. 코드 위치는 이 읽기 snapshot 기준이다.

## 잠금 의미와 선택

[Microsoft LockFileEx](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-lockfileex)는 호출 프로세스의 byte-range 잠금이며 프로세스 종료/잠긴 파일 close 시 OS가 해제한다고 설명한다. 해제 시점이 지연될 수 있어 유한 재시도가 필요하다. [원 native 잠금 실험](S03-windows-lock-lifetime-probe.md)의 실제 3회 결과도 parent original close 뒤 child가 같은 원 file identity의 복제 HANDLE을 유지하는 동안 별도 open이 잠금을 획득했다. 따라서 DuplicateHandle은 원 file capability를 전달하지만 app의 잠금을 helper에 이전하지 않는다.

최소 후보는 **원 cache.lock HANDLE 1개 추가 + typed 원 identity attachment + 기존 Ack gate에 잠금 인계 포함**이다. 원 lock 복제만 넘기고 app 객체를 drop하는 방식은 거절한다. helper는 app unlock 이후 자신의 LockFileEx/try_lock으로 잠금을 새로 획득해야 한다. 고정 current SID/no-reparse/regular/link1 및 granted access 검증도 필요하다. 잠금 실패 시 shell/path 우회 또는 무한 대기는 없다.

## 최소 권한과 원 identity attachment

app 원 wrapper가 verify한 root/runtime/cache/owner/lock의 실제 Windows native IDs와 원 owner bytes digest를 typed attachment에 넣는다. 별도 domain/version을 사용하고 bounded canonical encoding/hash를 검증한다. policy 경로와 attachment cache 원 HANDLE final path가 같아야 한다. 기존 cache path wire만으로 expected ID를 추론하거나 helper가 관측한 현재 ID를 expected로 대체하지 않는다.

helper는 전달된 원 lock HANDLE을 소유하고 inheritance를 즉시 해제한다. 원 lock ID/권한/ACL을 attachment와 대조한다. root/runtime/cache/owner를 다시 열어도 **app이 보증한 원 IDs와 모두 일치하는지** 확인하고 root/runtime/cache directory pins를 tool Job 종료까지 보유한다. cache/owner/lock private SID ACL, no-reparse/type/단일 link, 원 owner bytes의 native IDs/digest를 대조한다. root 기존 ACL은 변경하지 않는다. 다시 열기의 목적은 retained directory pins와 original-ID 비교이고 새 권한 채택이 아니다. directory pin은 delete 공유 없이 유지하여 app 종료 뒤 cache namespace가 교체되는 것을 차단해야 한다.

이 방식은 최소 inherited HANDLE 1개를 늘리지만 원 directory reopen 경로가 지원되지 않으면 fail closed한다. 더 강한 대안은 root/runtime/cache/owner/lock 원 HANDLE 5개를 함께 전달하는 것이나 entry parser/권한 검증/allowlist 변경이 더 크다. 어느 쪽도 child tool에 cache lock/owner 원 HANDLE을 상속시키지 않는다. 원 cache는 package 경로와 분리되며 yt-dlp의 Deno child에도 동일 환경을 전파한다.

## 순서와 app crash

1. app은 관리 Deno 원 package와 cache wrapper를 verify하고 app cache lock을 유지한 채 원 execution Intent와 helper config/capability를 준비한다.
2. helper는 cache 원 identity attachment를 검증하고 원 directory pins를 잡는다. tool은 suspended로만 prepare된다. cache 잠금을 아직 획득하지 못해도 이 단계에서 Resume는 0이다.
3. app은 실제 tool creation에 원 journal Bound를 durable commit한다. cache 원 lock과 namespace를 마지막 검증한 뒤 명시 unlock하고, 기존 AppAck를 전송한다. 단순 Drop에 의존하지 않는다.
4. helper는 AppAck 이후 원 cache.lock에 자신의 nonblocking lock을 획득하고 모든 원 IDs/owner bytes/ACL을 재검증한다. 원 managed/cookie journal acquisition과 cache acquisition이 모두 성공한 뒤에만 HelperAck를 전송한다.
5. app은 원 HelperAck 뒤 Begin을 보낸다. helper는 Begin을 검증하고 즉시 원 cache 권한을 마지막 확인한 뒤 tool을 Resume한다. 이미 받은 helper Ack를 cache 없는 guard의 Ack로 대체하면 안 된다.
6. helper가 실제 original Tool Job ActiveProcesses 0을 확인한 뒤 cache postflight inventory와 원 native/ACL 확인을 수행한다. 원 journal receipt commit과 terminal 전송 수명 동안 cache pins/자신의 잠금을 유지하고, 종료 직전에 자신의 잠금을 해제/close한다. cache 검증 오류와 tool cleanup 확인 여부를 구분하는 fixed outcome은 root 공유 계약에서 결정한다.

app이 3의 unlock 뒤 죽더라도 기존 durable Bound/no receipt는 pending으로 보존된다. 경쟁 worker가 lock을 먼저 잡으면 helper는 원 deadline 안에서 실패하고 suspended tool을 원 Job으로 취소한다. helper가 독립 잠금을 잡은 이후 app 원 HANDLE close가 helper 자신의 잠금을 유지하는지는 별도 실제 인수 대상이다. 현재 duplicate 실험 결과만으로 그 성공을 확정하지 않는다. helper가 죽으면 Job kill-on-close를 적용하며 receipt 없는 pending은 유지한다. cache에는 자동 삭제가 없으므로 잠금 부재만으로 cache를 청소하거나 새 unknown cache로 재생성하지 않는다.

metadata/download 및 version/runtime discovery는 순차 원 reader마다 인계·회수한다. app의 `reacquire()`는 nonblocking 재시도와 전체 원 pins 검증 뒤에만 다음 reader를 준비한다. 현재 OwnedDenoCache에는 unlock/reacquire 상태 API가 없으므로 `AppLocked -> Released -> HelperLocked -> AppLocked`를 명시하는 후속 수정이 필요하다. helper 종료의 typed terminal confirmation 전에 app이 cache lock을 다시 얻었다는 사실만으로 전체 reader 종료를 확인하지 않는다.

## 수정 담당과 실제 native 인수

주 담당이 cache wrapper 인계 상태/원 attachment, runtime policy와 config binding을 조정한다. launcher는 원 추가 HANDLE 허용 개수·위치/중복·동일 inode 금지 및 inheritance restore를 맡는다. managed/cookie/paired helper는 동일 acquisition gate 및 원 Tool Job 종료까지의 cache 수명을 적용한다. download/버전 조회 경로는 managed Deno가 실제 선택된 경우만 적용하고 외부 Deno cache를 변경하지 않는다.

필수 실제 Windows private CLI 인수는 다음과 같다.

- app-held 단계 별도 open Busy, explicit unlock 뒤 helper 자신의 lock Ack, app 원 HANDLE close/원 app TerminateProcess 이후에도 helper-live 별도 open Busy, original Job empty 뒤 lock 획득 성공. 단발 lock duplication 결과를 대체하지 않는다.
- unlock 전 app crash, unlock/Ack 사이 crash, helper Ack/Begin 사이 crash, 실행 중 crash: tool0 또는 actual Job termination/원 receipts 보존을 각 단계로 관측한다.
- 경쟁 scanner/두 worker가 gap을 점유하면 한 worker만 Resume, 다른 쪽은 bounded tool0/pending 보존. lock 강제 해제/권한 완화/전역 직렬화로 실패를 숨기지 않는다.
- 원 cache/owner/lock 교체, 원 IDs와 다른 현재 cache, 잘못된 SID/ACL/reparse/hardlink/partial owner record는 원 외부 sentinel bytes 보존과 함께 실패한다. env path가 original native capability와 다른 경우 reject한다.
- tool의 실제 inherited HANDLE 목록에 원 cache/lock/owner capabilities가 없고, 관리 yt-dlp의 실제 Deno child에는 두 고정 환경키가 정확히 전파되는 것을 합성 fixture로 관측한다.
- 원 cache postflight logical bytes/entry budget 보고와 fixed failure, 원 cache/source/외부 사용자 cache 변경·삭제 0. 32 MiB는 runtime hard quota가 아니다.

이 작업은 공식 문서 조회와 원 source 읽기만 수행했다. 새 native 실험/구현/Cargo/GUI/시스템 설정 변경은 0이다. 제안은 root 결정 대기이며 기존 cache native fullworkspace 결과를 guardian lifetime 성공으로 확대하지 않는다.

## 후속 staging 구현 인계

주 담당의 `windows_deno_cache_binding.rs` 공유 계약을 사용하여 `/tmp/dlpad-owned-deno-cache-handoff.rs`에 API를 실제 작성했다. 프로젝트 Rust는 수정하지 않았다. `binding()`은 locked 상태/원 검증 후 원 root/runtime/cache/owner/lock IDs와 bounded owner bytes SHA를 반환한다. `lock_file()`도 locked 검증 후에만 원 File을 빌려준다. `release_for_guardian()`은 검증 후 명시 unlock하고 Released 상태로 바꾼다. `reacquire()`는 Released에서 nonblocking 잠금을 직접 획득하고 원 검증 실패 시 잠금을 다시 해제하며 locked 상태를 승인하지 않는다.

`from_inherited(&CacheBinding, File)`은 no-create reopen으로 app attachment의 원 identities/원 owner bytes SHA를 대조한다. inherited 원 lock의 grantedAccess는 bounded NtQueryObject로 read/write+metadata 범위만 허용하고 DELETE/ACL 변경 권한을 거절한다. handle inheritance를 즉시 해제한다. current SID/private ACL/containment와 원 lock identity를 검증하지만 잠금은 아직 획득하지 않는다. 재검증은 원 owner SHA도 반복 비교한다. helper의 실제 own lock은 이후 explicit reacquire가 수행한다.

staging 새 3회귀는 explicit release/helper acquisition/parent 재획득, attachment·digest·경로·원 capability 불일치 보존, owner bytes의 의미상 동일한 newline 변경도 원 raw SHA mismatch로 거절하고 실패 획득 잠금을 해제하는 경계를 포함한다. 같은-process unit은 app 실제 종료 뒤 helper 잠금 생존의 native 증거가 아니며 문서의 별도 프로세스 인수 조건은 유지한다. 기존 기반 회귀 4개를 보존했다. 다른 담당이 준비하는 실패회귀는 주 담당이 후속 병합한다.

실행 명령은 `.local-tools/cargo/bin/rustfmt --edition 2024 /tmp/dlpad-owned-deno-cache-handoff.rs`로 exit 0이다. Cargo/native 실행은 0이며 source integration·실제 Windows 검증은 주 담당 책임이다. staging source SHA256: `eb3b1b11cd1b8708c171ae324d945e9ebaeece7b4261b62cc188bab506c8f3e4`.

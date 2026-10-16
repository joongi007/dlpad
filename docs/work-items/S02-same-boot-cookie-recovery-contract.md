# S02: same-boot 쿠키 recovery 소유 증거 계약

- 상태: done
- 분류: review
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: metrics_recovery — 이 문서만; cookies/process/guardian/기존 review 읽기 전용
- 선행 작업: 이전 boot recovery 기록 기반 실제 검증, terminal GC/GU·LC/LU 계약

## 사용자 결과

GU/LU로 복사본을 보존한 뒤 동일 boot에서 실제 reader/service 정리가 나중에 완료되면, 다음 시작에 안전한 증거가 있는 복사본만 제거할 수 있어야 한다. 현재 recover_sessions는 same-boot record를 항상 deferred로 남긴다. 이 검토는 그 제한을 숨기지 않으며 done은 제안 검토 완료다.

## 현재 기반과 보존할 제약

[기존 recovery 검토](S02-cookie-startup-recovery-review.md)와 [이전 boot 구현](S02-cookie-recovery-records.md)의 anchored cache/owner/cookie/record/marker native identity, 랜덤 nonce, bootUUID, canonical record digest를 유지한다. Linux record4KiB/marker512B, scan128+1/import128 capacity를 확장하지 않는다. unknown/partial/tampered/extra 파일은 보존하고, foreign entry도 capacity를 차지한다. same-boot 자동삭제는 아직 미구현이다. 평문 cookie/source path/URL/argv/account를 일반DB·log·diagnostic에 넣지 않는다.

앱 PID가 끝났거나 launcher를 reap했다는 사실만으로 service reader가 끝났다고 추론하지 않는다. unit 이름/숫자PID 단독, systemd query 실패, 다른 InvocationID, missing GC 또는 GU/LU는 삭제 허가가 아니다. capacity128에 도달하면 새로운 cookie import를 고정 CacheFull로 거절하고 자동 foreign/unknown 삭제를 하지 않는다.

## 반드시 먼저 해결할 reader 시작 순서

현재 service guardian은 실제 tool을 spawn한 뒤 R을 보낸다. StartTransientUnit 성공 후 GetUnit/InvocationID/cgroup을 읽고 record에 추가하는 것만으로는 reader 시작 이전 durability를 보장하지 않는다. cookie argv가 이미 service command에 들어 있으므로 tool이 먼저 파일을 열 수 있다. callback/packet을 넣어도 guardian pre-tool gate가 없으면 동일한 crash 공백이 남는다.

인수 순서는 (1) 기존 private record의 pending-reader intent와 boot/nonce를 sync, (2) guardian 자체만 Start, (3) exact service unit/InvocationID/cgroup evidence 확보, (4) private record durable bind, (5) 소유 control의 단일 Begin 허가, (6) guardian tool spawn/R이다. crash 또는 I/O failure가5이전이면 Begin을 보내지 않고 socket EOF로 tool 미시작을 보장한다.3실패는 정확 reader identity 미확정이므로 같은boot 보수 보존한다. record를 실패 전에 자동 completed로 바꾸지 않는다.

## 옵션 A: bounded startup identity packet

cookie 실행에만 optional gated launcher/guardian marker를 사용한다. launcher가 Start 후 guardianMainPID가 pre-tool gate에서 대기하는 동안 exact identity를 확인하고 app-side에 versioned fixed-prefix identity packet을 보낸다. 예: I/version +16byte owned unit nonce +16byte InvocationID +bounded canonical cgroup path length/string +cgroup native dev/inode 및 확인한 cgroup mount identity. unit 문자열은 nonce에서 `dlpad-job-<32hex>.service`로 재구성하거나 최대64byte로 검증한다. cgroup 문자열은 예시1024byte 상한·normal absolute components·정확 owned unit suffix를 요구한다. packet 전체 예시2KiB 상한이며 record4KiB에 맞지 않으면 Begin을 보내지 않는다. cookie path/content는 packet에 없다.

app worker는 lease의 anchored recovery record에 이 정보와 reader phase를 바인딩하고 file/directory sync 성공 후 B(Begin)를 보낸다. launcher는 B를 read하지 않고 guardian만 pre-tool B reader가 된다. 같은 guardian-side endpoint를 launcher와 guardian이 동시에 read하면 Begin이 잘못 소비되는 경쟁이 생기므로 금지한다. launcher I write가 끝나기 전 guardian은 R/G를 쓰지 않는다. 현재 terminal R/GC/GU/LC/LU parser는 I→durable callback→B→R 상태로 cookie-only 확장한다. I와 terminal 모두 bounded buffer/poll/deadline을 사용하며 frame정보는 Debug/error/UI에 출력하지 않는다.

장점: 실행한 service가 실제 확인한 identity를 app에게 전달하고 Begin gate와 연결된다. 단점: protocol revision/partial packet/record-I/O/timeout 상태와 app/launcher/guardian 역할을 함께 바꿔야 한다. 새 필수 ProcessSpec 필드를 추가할 필요는 없다. cookie 전용 `run_with_lifetime`/`run_metadata_with_lifetime` wrapper가 optional execution context를 내부 process runner에 전달하고 marker를 선택하면 기존 모든 ProcessSpec fixture literal을 유지할 수 있다.

## 옵션 B: process lifetime observer

외부 API는 cookie 전용 optional observer를 받고 기존 process::run* API는 observerNone으로 위임한다. observer의 최소 동작은 pending intent, bind reader identity, confirm terminal, retain unconfirmed이며 반환값이 Begin gate의 허가를 결정한다. 메서드는 sensitive identity를 owned private type으로 받으며 Serialize/Debug를 일반 공개하지 않는다. UI thread가 아닌 해당 실행 worker에서 bounded private I/O를 수행한다.

observer callback만으로 identity 전달이 저절로 생기지는 않는다. 실제 Start는 재실행된 launcher에 있으므로 (A)의 private I packet, 별도 inherited control FD, 또는 native launcher→parent의 동등한 bounded IPC가 여전히 필요하다. 부모가 새 unit을 추정하여 목록/로그를 검색하는 방식은 금지한다. parent가 native service Start를 직접 담당하는 대형 개편도 가능하지만 Child/stdout/stderr/statusproxy 계약 변경이 커서 이번 최소안보다 비용이 높다.

권고는 A를 내부 IPC로, B를 download/session에 보이는 optional wrapper interface로 조합하는 것이다. root만 공유 process interface/protocol을 결정하고, cookies 담당이 private record의 version/digest 재바인딩을 조정한다. callback에 cookie capability를 임의 logging하거나 mandatory ProcessSpec field로 기존 호출자를 깨지 않는다.

## 같은 lease의 여러 reader와 crash 기록

metadata와 download는 별도 invocation이며 같은 lease를 사용한다. reader가 끝날 때마다 confirmed receipt를 durable하게 기록하고 다음 reader intent를 먼저 sync한다. 활성/미확인 reader를 다음 invocation으로 덮어쓰지 않는다. 세션당 예시2개 또는 작은 고정 reader array만 허용하고 넘으면 fail-closed한다. resume은 명시 새 세션을 쓰므로 과거 lease에 새 reader를 자동 추가하지 않는다. 버전 조회에는 cookie/reader pin을 넣지 않는다.

record는 pending-no-reader, bound-reader, confirmed-done, unconfirmed 단계와 정확 phase sequence를 가진다. marker는 갱신 후 record digest/native identity에 다시 바인딩한다. 두 파일 갱신 crash의 미완결 조합은 기존 원칙대로 보존한다. confirmed receipt는 root가 실제 GC+LC/Child reap/pipe EOF 인수를 확인한 뒤 private record에 기록하고 sync한다. receipt는 cookie/source/path/임의 tool 로그를 포함하지 않는다.

## recovery 증거 판단

| 관측 | same-boot 자동정리 판단 |
|---|---|
| boot가 이전 kernel이고 기존 전체 anchored 기록이 유효 | 현재 이전boot 기반 절차 유지 |
| 해당 reader의 durable confirmed receipt와 exact record/marker/file가 유효, 다른 pending reader 없음 | 안전한 삭제 근거. missing 현재 GC/unit GC는 과거 확인 receipt를 무효화하지 않음 |
| 같은 owned unit+같은 InvocationID, terminal, 같은 기록된 cgroup identity가 실제 populated0이고 cgroup.procs도empty, 모든 reader intent가 resolved | 실행종료 증거로 갱신 후 삭제 가능. managerquery 전후 identity를 재확인하고 mutation이면 보존 |
| manager 연결 실패/permission/timeout/잘못된 response | 보존·deferred, 새 blanket PID kill/StopUnit 없음 |
| unit LoadState not-found/GC, cgroup 증거도 없음, confirmed receipt 없음 | 단독으로 unsafe. 이름만 없다고 삭제하지 않음 |
| 현재 unit의 다른 InvocationID 또는 같은cgroup경로의 다른 inode | 이름 재사용/foreign state이므로 현재unit을신호·삭제하지 않음. 원 reader 종료의 독립 증거가 없으면 보존 |
| GU/LU 또는 missing GC/LC | acknowledgment 실패 자체는 삭제 허가가 아님. 이후 exact manager+cgroup 종료 증거를 새로 얻으면 보수적으로 resolved로 전환 가능 |
| reader identity packet을 받기 전 Start/조회 모호 실패 | pending intent 보존. cookie tool Begin이 실제 미전송임을 durable하게 입증하는 optional receipt가 없으면 보존 |

cgroup path absent/inode changed만으로 original cgroup inode가 사라졌다고 추정하는 안은 추가 위험이 있다. populated cgroup을 kernel이 제거할 수 없다는 사실과, 다른 이름으로 rename된 살아있는 cgroup을 원래 path에서 못 보는 경우는 다르다. stored pathname만으로 inode 부재를 확인했다는 표현은 금지한다. managed cgroup rename/move가 배제되는 계약과 그 한계를 채택하지 않는 한, sameboot unitGC+cgroupabsent+missingreceipt는 보존해야 한다. 조회가 가능한 원래 sameidentity cgroup의 empty 증거가 가장 좁은 안전 인수다.

현재 manager AddRef는 launcher bus 연결에 묶여 있다. launcher가 종료하면 unit가GC되어 나중에 sameidentity 조회 기회를 잃을 수 있으므로, A/B만으로 모든 GU/LU 잔여를 같은boot에 반드시 회복한다는 보장은 없다. eventual cleanup을 더 넓게 보장하려면 manager가 terminal ownership receipt를 앱전용 private 영역에 durable 작성하도록 helper/watchdog 수명을 확보하거나, 별도 trusted lifetime receipt/guard를 채택해야 한다. 임의 guardian guardFD 하나는 guardianSIGKILL 후 descendants 수명을 덮지 않아 단독 근거가 아니다. 모든 reader를 덮는 durable receipt를 못 얻는 경로는 capacity128 아래 보존되고 이전boot foundation에서 회복한다. 이 제한을 nextstartup requirement 전체 완료로 표현하지 않는다.

## 실제 인수 fixture

- metadata/download Begin 이전 bind-record file/dir sync 확인; 실패/앱 crash 시 tool-start marker0, source/copy/sentinel 유지.
- 같은boot GU/LU 뒤 service가 늦게 종료: 같은Invocation/cgroup empty 조회 또는 trusted durable receipt로만 resolved; 정확 cookie copy만 unlink.
- manager unavailable→다음 시작 available: 미확정 시 deferred, available이면 재검증. existingunit/다른Invocation을 Stop하지 않음.
- unitGC/missingGC/Invocation reuse/cgroup rename·identity 교체: 독립receipt 없는 삭제0. 실제 다른 unit rename/manager 재시작 대신 owned adapter/fixture로 실패를 주입.
- metadata confirmed→download pending crash: reader array에 metadata receipt가 있어도 download unresolved를 덮지 않음.
- record/marker/cache/owner/cookie tamper/partial write/foreignextra/packet truncation/oversize: 삭제0/Begin0, bounded FD/I/O, secretDebug/log/DB 미노출.
- 128capacity/fullscan+1: unknown/미확인 보존, 새 import고정오류, 무제한scan·큐 없음.
- 이전boot foundation 회귀와 Linux 실제 private GUI CloseWait/cancel/terminal22 인수는 별도 snapshot별로 재검증.

## 검증과 handoff

코드/기존문서를 rg/sed로 읽고 이 전용 문서만 작성했다. source/Cargo/process/manager/실cookie를 변경·실행하지 않았다. root가 protocol gate, optional observer API, private record revision과 GC 이후 proof 정책을 선택한 뒤 별도 수직 구현/실제 시험을 배정해야 한다. 다른 OS는 이 Linux contract로 완료 처리하지 않는다.

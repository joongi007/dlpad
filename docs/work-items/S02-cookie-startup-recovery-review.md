# S02: 보류 쿠키 복사본의 시작 시 복구 검토

- 상태: proposed
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: stress_recovery — 본 문서만. 후속 구현 담당·공유 계약은 root 결정
- 선행 작업: 세션 importer/validate_for_tool/defer_cleanup; root의 실행 종료 확인 및 recovery ownership 계약

## 사용자 결과

앱이 비정상 종료해 남은 세션 쿠키 복사본을 다음 시작 때 안전하게 정리한다. 아직 읽는 도구가 있거나 소유권을 증명할 수 없으면 복사본을 보존하고 비밀 없는 고정 상태만 알린다. 이 문서는 설계 제안이며 기능 구현·실증 완료가 아니다.

## 읽기 계약과 현재 사실

AGENTS, docs/INTEGRATIONS.md, docs/ARCHITECTURE.md, docs/adr/0002-runtime-and-storage.md, cookies.rs, instance.rs 및 process/user_service.rs의 기존 계약을 검토했다. 사용자 데이터 single-instance lock은 살아 있는 앱만 직렬화한다. 앱 소실 후 systemd 사용자 manager의 guardian/cgroup이 살아 있을 수 있고 lingering이면 로그인 세션 종료도 tool 종료 증거가 아니다. 앱 lock 획득·PID 없음·mtime 오래됨·파일명 패턴은 삭제 허가가 아니다.

현재 lease는 anchored FD/dev+ino/uid/mode를 검증해 정리한다. defer_cleanup는 unlink 없이 FD를 닫으며 복사본을 보존한다. 이 자체로 재시작 ownership record나 실행 중인 reader 수명 보호가 생기지 않는다. root는 importer 15개/download 5개/engine 173개 시험 exit0을 보고했으나 startup recovery는 그 시험 범위에 없다.

## 제안: 소유권 기록과 실행 수명 보호를 함께 도입

일반 jobs/attempts/settings DB 대신 앱전용0700 recovery 영역의 create-new0600 bounded 기록을 사용한다. record에는 버전, 랜덤 소유 토큰, cache/owner/copy native identity, copy의 상대 단일 basename, 생성 boot identity, 단계, 필요시 backend/unit InvocationID/cgroup identity만 넣는다. 원본 path, cookie bytes, URL, 계정, tool 전체 argv는 넣지 않는다. path·토큰·unit 정보도 Debug/error/log/일반 진단에서 숨긴다. cache root는 caller가 이미 알고 있는 한 경로이며 임의 절대경로를 record에서 받아 삭제하지 않는다.

소유 토큰은 충돌 저항성을 가진 OS randomness로 만들고 같은 토큰의 private marker를 owner 안에 create-new로 둔다. dev/ino만 기록하면 재사용된 inode를 옛 소유 대상으로 오인할 수 있어 marker 토큰과 owner/file/cache identity를 함께 검사한다. marker/record도 no-symlink, 정확uid/mode/type/link count, 작은 size 상한으로 읽는다. 동일UID 악성 프로세스가 기록까지 위조하는 위협에 대한 보장은 하지 않는다. 이것은 앱의 정상 소유·동시 실행/사고 방지 계약이다.

기록은 준비→복사본 identity 확정→reader 준비→reader 사용→종료 확인→정리 완료 단계로 원자적으로 갱신한다. flush/rename 시 같은 private 영역을 유지하며 변경 후 directory durability도 고려한다. reader에게 cookie path를 전달하기 전에 확정 identity와 reader 보호가 durable해야 한다. 생성 직후 crash로 file identity가 기록되지 않은 경우에는 이름만으로 삭제하지 않고 보존한다. 구형 기록 없는 owner도 자동 삭제하지 않는다. 임의 추가 파일은 재귀 삭제하지 않는다.

## boot/session guard와 guardian 소유권 비교

| 선택 | 제공하는 증거 | 제약/제안 |
| --- | --- | --- |
| 앱 single-instance/session lock | 이전 앱이 같은 data root를 사용 중인지 | reader가 앱보다 오래 살 수 있어 단독 사용 금지 |
| boot identity 변경 | 이전 kernel의 프로세스가 지금 살아 있지 않음 | 실제 같은 host/kernel boot ID 출처를 검증해야 하며 현재 boot의 reader 증거는 아님. suspend/로그아웃/WSL 배포판 재시작을 reboot로 추정하지 않음 |
| per-lease kernel advisory guard | 해당 inode의 lock을 보유한 reader가 있음 | 앱만 보유하면 불충분. guardian→reader 전체 수명에 보호를 유지해야 함. guard inode 교체 방지·동일 inode 검증·핸들 전달 allowlist 필요 |
| user-service active ownership | 동일 InvocationID/unit/cgroup의 reader 존재/종료 | systemd query 실패·다른 InvocationID·GC로 unit 없음만으로 cleanup 확정 금지. cgroup이 정확하게 비었거나 기존 confirmed-cleanup receipt 등 독립 종료 증거 필요 |

제안 우선순위는 Linux 기본 user-service의 정확한 invocation/cgroup 종료 증거와 per-lease guard를 함께 설계하는 것이다. guardian만 guard를 들고 강제 종료될 때 detached tool이 남는 공백을 해결해야 한다. 가능한 방식은 실제 tool 및 모든 관련 자손이 guard FD를 수명 동안 보유하도록 검증하거나, manager가 같은 cgroup의 reader가 완전히 종료될 때까지 guard를 보유하는 프로토콜이다. tool의 임의 FD close/daemonize를 고려하면 전자의 락 단독 모델은 아직 채택할 수 없다. 기존 systemd backend에 일반 FD 상속을 추가하지 않으며 root가 프로토콜을 먼저 결정한다.

Direct 개발 fixture도 앱의 락만으로 종료를 추정하지 않는다. 수명 guard가 실제 reader 전체를 덮는 실증이 없거나 direct guardian가 소실된 같은 boot에서는 자동 정리를 보류한다. Windows는 native file identity/ACL 및 Job Object 수명 계약을 실제 검증하기 전 unavailable이며 Linux 설계를 그대로 완료 처리하지 않는다. macOS도 별도 guard/소유 프로세스 실증이 필요하다.

## 삭제 허가와 실패 정책

1. 전용 root의 single-instance guard를 먼저 획득하고 bounded record만 검사한다. 제안 상한은 record 4KiB, 한 번 시작 시 128건, 총 읽기512KiB, 동시 복구1건이다. 넘는 항목은 다음 명시적 복구로 보류하며 무제한 scan하지 않는다.
2. record/marker와 cache/owner/file의 anchored identity·type·uid·정확0700/0600·single-link를 확인한다. 토큰/identity 불일치, symlink, extra file, 손상/미완결 record는 fail-closed 보존한다.
3. reader 종료 증거를 확보한다. 정확한 이전 boot가 확정되거나 현재 boot의 확정 종료 receipt/cgroup-empty 계약을 만족해야 한다. manager unavailable, lock busy, cgroup unreadable, InvocationID 재사용·변경은 보류한다. PID 또는 unit 이름만으로 process를 종료하지 않는다.
4. 동일 owner/guard identity에 대한 배타적 recovery 권한을 획득하고 제거 직전에 다시 확인한다. 정확한 copy만 unlink하고 marker/guard/record의 자신의 항목과 비어 있는 정확 owner만 정리한다. source/cache root/foreign file은 삭제하지 않는다. 실패 후 재시도는 idempotent이며 중간에 바뀐 대상은 보존한다.

Advisory lock과 identity 재확인은 같은UID 악성 concurrent rename/unlink를 OS원자적으로 차단하지 않는다. 현재 importer와 같은 위협 경계를 유지하며 더 강한 보장 요구 시 별도 OS primitive/아키텍처 검토를 한다. 오래 남은 비밀을 임의 시간 만료로 삭제하는 것은 종료 증거를 대체하지 않는다.

## 구체적 합성 fixture 인수 조건

- 정상 종료 및 종료-confirmed defer record: 다음 시작에서 정확 copy만 제거하고 source와 cache root 보존. 두 번째 복구는 변화 없음.
- 앱 SIGKILL 후 reader 생존, guardian SIGKILL 후 manager 정리 진행 중, lingering 로그인 종료: 같은 boot 복구가 cookie를 지우지 않음. 실제 cgroup-empty 확인 후에만 제거.
- 정확 이전 boot의 합성 record와 현재 boot record를 구분하고 boot 출처 unavailable/변경 판정 불명확은 보존. 실제 reboot 인수는 합성 시험과 별도 보고.
- PID/start 재사용, unit 이름 재사용·InvocationID 변경, manager 부재·timeout·unit GC, guard busy/guard inode 교체: 고정 보류 상태와 copy/source 보존.
- record/marker/cache/owner/copy 교체·symlink·hardlink 증가·permission 변경·extra sentinel, 토큰 불일치, inode 재사용을 모사한 record: 외부 항목에 write/delete하지 않음.
- 각 durable 기록 단계/cleanup 단계에서 crash 주입: unrecorded copy는 보존, identity 확정+reader 미시작은 정책에 따라 안전하게 회수, partial cleanup은 idempotent. 레코드와 실제 reader release 순서가 뒤집히지 않음.
- source/cookie/path/record 토큰이 Debug/error/log/진단/일반 DB에 나오지 않음. record4KiB/128건·기형 길이·FD/queue/time 상한 실패 경로 시험.
- 실제 Linux private Xvfb/DBus 합성 tool로 reader 수명과 manager cleanup 증거를 수집한다. Windows/macOS 미시험은 미검증으로 구분하며 사용자 계정·실제 쿠키는 사용하지 않음.

## 검증과 handoff

문서 검토만 수행했다. cookies.rs/tests/session_cookies.rs 및 공유 source/Cargo/schema는 변경하지 않았다. 실제 startup cleanup 명령·fixture·실제 reboot 시험은 아직 없다. 다음 root 결정은 durable recovery record의 소유 모듈/API, backend 종료 receipt, guardian 수명 guard 전달 모델, crash durability 및 미기록 구형 owner 보존 정책이다. 승인된 계약 뒤 별도 수직 구현 작업을 시작한다.

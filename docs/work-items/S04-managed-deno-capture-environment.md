# S04: Windows capture 관리 Deno 환경 원 cache gate

- 상태: implementation-ready (임시 사본; native 통합 대기)
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: soak_recovery_audit; 본 문서와 `/tmp/dlpad-managed-deno-capture/`의 3 capture 사본만
- 선행 작업: verified Deno authority/factory, immutable binding wire 및 optional cache helper capability(root)

## 사용자 결과

검증된 관리 Deno 환경의 원 cache authority를 캡처 전체 동안 보유하며 helper gate 전후 원 락과 binding을 검증한다.

## 계약과 범위

Some policy는 authority/binding 필수. 기존 ExecutionControl deadline/cancel로 try-lock 획득을 제한하고 journal mutation 전 verify한다. journal release 후 AppAck 전 cache release, 실제 helper terminal/exit/EOF 확인 후 원 journal reacquire 및 cache reacquire를 먼저 수행하고 resolve/confirm은 그 다음에만 허용한다. None은 기존 경로. helper launch/config/authority/shared product/Cargo 변경과 실제 native 실행은 제외한다.

## 인수 조건

poison/mismatch/authority 없음은 fail-closed, cache MutexGuard는 캡처 전체 보유, nested mutex 없이 immutable wire binding 복사. speculatively reacquire/drop을 terminal로 간주하지 않고 기존 typed unconfirmed/pending 보존. 3 원/수정 SHA 및 diff 제공, native 전 완료 주장 금지.

## 검증과 handoff

임시 수정 사본 rustfmt 후 root 통합/실행 인수 대기.

`/tmp/dlpad-managed-deno-capture/`에 3 수정 사본/원·수정 SHA manifest/3 diff를 동결했다. `rustfmt --edition 2024 /tmp/dlpad-managed-deno-capture/*.rs` exit0. 프로젝트 원본 3 SHA 불변, Cargo/native/제품 게시 없음.

초기 policy 검증·authority Arc clone·bounded Mutex try-lock·원 cache verify 및 immutable binding 비교를 journal mutation 전에 수행한다. launch는 original cache lock File을 optional extra capability로 전달한다. cache MutexGuard/authority는 capture 전체 보유하고 from_native는 root의 immutable binding 복사 API를 전제로 nested Mutex를 사용하지 않는다.

통합 협의에서 초기 요청의 resolve 이후 cache reacquire 순서는 실패 시 pending을 이미 지우는 문제를 발견해 정정했다. paired/managed는 actual Terminal/EOF/helper exitcode → journal reacquire → cache 1회 try-reacquire → journal resolve (paired cookie confirm 추가), cookie는 actual helper proof → cache reacquire → reader confirm이다. 재획득 실패는 typed unconfirmed이며 Bound/원 receipt와 cache authority를 보존하고 pending을 정리 성공으로 바꾸지 않는다. 실제 Windows 양방향 cap/gate/성공·실패/취소 검증은 root의 잔여 인수다.

# S02: Windows cookie reader gate facade 초안

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: diagnostic_cache — crates/engine/src/cookies/windows.rs, 신규 crates/engine/src/cookies/windows/reader_gate.rs 및 이 문서
- 선행 작업: actual product import/journal/marker 및 root helper public recovery native 시험

## 사용자 결과

Windows reader를 durable Intent→Bound→독립 guardian receipt→Resolved로 연결하는 구체 facade 초안을 제공한다. root 승인으로 제품 windows.rs 및 새 reader_gate.rs에 구현한다. Cargo/공유 facade는 root 담당이다.

## 공유 계약 제안

OwnedCopy.journal을 Option<Mutex<journal::Journal>>로 전환한다. process::run이 SessionCookieLease를 borrow하는 현재 구조에서는 Arc가 불필요하다. `WindowsGatedCookieReader<'a>`는 안정된 Mutex/reader_pending AtomicBool borrow, 원 Intent/slot/nonce, Option launch 및 BoundIdentity/단계 flag를 보유한다. 별도 worker로 lease가 이동해야 하는 경우에만 Arc의 소유 수명을 다시 설계한다.

WindowsCookieLaunch는 원 Intent, owner_path, slot/nonce, 실제 receipt File 및 original receipt anchor를 반환한다. Debug는 정제된다. `take_launch`는 한 번만 허용한다. `bind_identity(guardian_created,tool_created)`는 verified Spawned frame을 받은 뒤 원 Intent를 정확히 비교해 Bound를 저장한다. `release_for_guardian`은 Bound commit 이후 native lock을 놓는다. `confirm_terminal`은 actual helper 종료/Job ack 이후 재잠금·원 Bound 확인·actual receipt resolve·모든 receipt 일치 확인 후에만 pending=false로 바꾼다. Drop은 어떤 상태도 완료로 바꾸지 않는다.

nonce는 CSPRNG이며 validate/slot/Mutex/CSPRNG 성공 후 CAS, durable begin 직전 pending=true로 만든다. 이후 실패는 Intent 생성 전이라도 보수적으로 pending 보존한다. journal_begin 시 실제 writer anchor가 원 receipt에 일치하는지 확인한다.

## Mutex 전환 영향과 통합 주의

windows.rs initialize는 Mutex::new(Journal), validate/cleanup은 try_lock guard로 기존 native 검증을 수행해야 한다. cleanup이 guard를 보유한 상태에서 validate_for_tool이 다시 동일 Mutex를 try_lock하면 실패하므로 `validate_for_tool_with_locked_journal` 같은 private helper 또는 preflight 검증을 guard 하나로 수행하는 구조가 필요하다. 삭제 자격 확인부터 mutation까지 native lock을 유지하고, 삭제 완료 후 guard를 놓은 뒤 self.journal.take/drop해야 borrow/lock 수명이 맞는다. scanner 재구성 역시 Mutex::new로 전환한다. 기존 native tests의 direct journal 접근은 get_mut/try_lock으로 갱신한다. root가 공유 소스를 조정한다.

parent receipt writer는 launch success/failure 양쪽에서 explicit inherited whitelist 복제 직후 즉시 drop해야 한다. facade의 take_launch만으로 외부 launcher가 File clone을 보유하지 않음을 증명할 수 없다. 이 부분은 root launcher가 소유하며 actual helper→public recover native 시험에서 중요하게 확인된 계약이다. protocol frame만으로 terminal을 믿지 않고 owned helper process 종료/Job ack와 실제 receipt를 각각 확인한다.

## 인수 조건

실제 Windows product 두 reader 순서, wrong slot/nonce/creationtime, take 두 번/bind 전 release/확인 전 pending clear 금지, journal busy/reacquire failure, receipt 누락·partial·교체, panic/Drop 뒤 pending 보존, parent receipt writer 즉시 닫힘, cancellation/EOF keeper cleanup 후 원 receipt 증거를 시험한다. 실제 public recovery 정상 결과를 parser·draft 성공으로 대신하지 않는다.

## 검증과 handoff

현재 cookies.rs/Windows journal/guardian/guardian config를 읽고 임시 Rust draft를 작성했다. rustfmt --edition 2024 임시 파일 exit0. compile/native 실행은 수행하지 않았다. 실제 제품 연결/Mutex 전환/종료 acknowledgement 검증은 root 후속이며 초안 완료를 P0 완료로 확대하지 않는다.

실제 구현: OwnedCopy Mutex 전환, validate_with_journal private helper 및 cleanup 단일 guard로 재잠금 방지, 새 reader_gate child 연결. source freeze와 root native 검증 대기.

실제 source READY/frozen: OwnedCopy Mutex/cleanup 단일 guard, reader_gate pub(crate) reexport, begin_windows_reader의 cookies 부모 가시성을 구현했다. native gate invalid-order/Drop pending 회귀 1개를 추가했다. root wrapper/capture/recovery 연결과 전체 fmt/native compile/test를 기다린다. 담당 Cargo/format은 실행하지 않았다.

# S02: cookie lease의 durable reader gate

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: metrics_recovery — crates/engine/src/cookies.rs 및 이 문서
- 선행 작업: reader journal, C/I/B 및 launcher receipt 기반

## 사용자 결과

Linux 인증 reader 실행 전에 durable Intent를 생성하고 native identity bind 이후에만 실행을 허용한다. 실제 receipt resolve를 sync한 뒤에만 pending을 해제한다. 이 lease 단계만으로 제품 gate 연결이나 같은 부팅 복구 완료를 주장하지 않는다.

## 계약과 범위

기존 공개 begin_reader 및 Windows 경로를 보존한다. Linux journal을 Option<Mutex<ReaderJournal>>로 바꾸며 모든 journal 접근은 짧은 try_lock만 사용한다. GatedCookieReader는 lease를 borrow하고 launch handoff는 한 번만 이관한다. metadata0/download1 슬롯은 재사용하지 않는다. 각 session worker는 새로운 lease를 사용한다. 기록 실패·미확인 종료·panic은 pending을 보존한다. root가 process 연결, 다른 담당자가 download 연결을 수행한다.

## 인수 조건

Intent-before-handoff, capability exact bind, receipt resolve-before-clear, 반복 take/begin/confirm 거절, mutex busy 및 poison 보존, Drop pending 보존, 정상 metadata/download 두 슬롯, 기존 recovery anchor/allowlist/cleanup 계약을 유지한다.

## 검증과 handoff

구현과 소스 동결 이후 root가 Cargo 검증한다. 자체 Cargo/native 실행은 하지 않는다.

## 구현 handoff

Linux 내부 API `begin_gated_reader(index: u8)` 및 `GatedCookieReader::{take_launch, bind_identity, confirm_terminal}`를 추가했다. guard는 journal Mutex와 pending atomic을 borrow한다. Intent 생성에는 현재 boot를 recovery record와 대조하고 원래 marker digest 32바이트 및 내부 난수 nonce 16바이트를 사용한다. capability 전 필드가 다른 bind를 거절한다. handoff는 한 번이며 실제 receipt resolve의 디스크 commit 성공 뒤에만 atomic pending을 false로 바꾼다.

OwnedCopy의 Linux journal을 Option<Mutex<ReaderJournal>>로 변경했다. recovery create/reopen 및 anchor/allowlist 검증과 cleanup 경로를 동일 mutex에 맞췄다. 모든 공유 journal 접근은 try_lock이며 실행 전체 동안 보유하지 않는다. cleanup은 journal proof 확인 후 소유 Mutex를 소비해 파일을 정리한다. poisoned/busy 검증은 삭제를 허용하지 않는다. 기존 legacy 공개 guard 및 Windows 구현은 변경하지 않았다.

단위 시험 3개를 추가했다. 합성 cookie/임시 cache에서 metadata0와 download1 각각 실제 256바이트 receipt 저장 후 resolve, missing receipt 및 반복 handoff/confirm 거절, 잘못된 capability 및 미확인 guard Drop의 복사본 보존, mutex busy/poison의 fail-closed를 다룬다. unit의 receipt는 순수 synthetic identity를 사용하므로 native manager empty proof 검증을 대체하지 않는다.

`.local-tools/cargo/bin/rustfmt --edition 2024 crates/engine/src/cookies.rs` 실제 exit 0. 소스 SHA256 `e21f034c6ba73a1849ea2120a760e2e0fcf6184a7f332e5bae800102c5f3c8dd`로 root에게 동결 handoff했다. Cargo는 실행하지 않았다. root의 실제 컴파일/전체 시험, process 및 download의 C/I/B 연결과 native crash 복구 인수가 남았다. 슬롯 재사용은 불가하며 worker마다 새 lease를 생성하는 상위 계약이 필요하다.

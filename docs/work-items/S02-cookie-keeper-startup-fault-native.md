# S02: keeper startup fault의 native 인수 fixture

- 상태: done
- 분류: test
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: metrics_recovery — 신규 tests/cookie_keeper_startup_fault.rs 및 이 문서
- 선행 작업: root/stress debug nonce 및 after-start/get-unit/invocation hook

## 계약과 인수

OS 난수 16바이트의 nonzero nonce로 정확한 reader/keeper/parent 세 unit 이름을 정하고 GetUnit으로 기존 unit 부재를 확인한다. 합성 private cookie/cache/journal 및 sentinel만 사용한다. C 전송 뒤 debug startup fault 세 단계에서 tool 실행 0, LU 또는 GU와 EOF, 원 Intent bytes 보존 및 소유 launcher 종료와 세 nonce unit GC를 bounded 관찰한다. unknown identity를 unit 이름만으로 Stop하지 않는다.

동일 nonce override의 정상 metadata0도 actual I/native identity pin→Bound sync→B→R/GC/LC/EOF→실제 receipt resolve→세 unit GC를 확인한다. live/pending sibling은 후속 범위다. 기존 helper fixture 기능을 신규 파일에 read-only 복제해 사용하며 원본 시험 파일은 변경하지 않는다. root가 Cargo/native 실행하고 실제 release env 무시 검증은 별도 인수다.

## 검증과 handoff

자체 Cargo/native 실행 금지. 소스 정렬·동결 후 root 실제 ignored test 실행에 인계한다.

## 구현 동결

ignored native 시험 두 개를 작성했다. fault 시험은 after-start/get-unit/invocation 세 단계를 각각 새 OS 난수 nonce로 실행한다. hook의 1초 pause 안에서 reader의 실제 PID/startticks/pidfd와 keeper/parent의 nonzero Invocation을 관측해야 성공할 수 있다. 세 unit이 만들어지지도 않았는데 GetUnit absent만으로 GC를 통과시키지 않는다. tool marker 부재·R 부재·LU/GU 후 EOF와 원 record bytes/receipt zero를 요구한다. 원 helper Child만 유한 종료 관찰하며 unknown unit Stop은 하지 않는다.

양성 metadata0는 동일 nonce override의 exact I unit, actual service/native pinned cgroup identity, bind 이후 B, RGCLC+EOF, 실제 receipt resolve와 세 unit GC를 확인한다. unit GC 관찰은 각각 15초 상한이며 조회 subprocess에는 기존 3초 상한을 사용한다. 관리 namespace/외부 sibling 시험은 이 fixture 범위가 아니다.

기존 cookie_service_gate.rs의 read-only helper prefix를 신규 파일에 복제해 기본 argv/FD/manager query 계약을 유지했다. 원본 파일은 변경하지 않았다. 신규 OwnedHelper Drop의 kill 이후 reap 관찰도 2초 try_wait로 제한했다. test-local source modules의 내부 unit과 실제 ignored native 두 시험을 결과 보고에서 구분해야 한다.

`.local-tools/cargo/bin/rustfmt --edition 2024 crates/engine/tests/cookie_keeper_startup_fault.rs` 실제 exit 0. SHA256 `19eae84592c7adea67bf18ac24d4d2c4812db43b7898239966c931a710a3a250`로 root 동결 인계했다. 자체 Cargo/native 실행은 하지 않았으므로 실제 fault cleanup 성공을 아직 주장하지 않는다. root의 helper/debug hook 통합 및 actual 실행이 남았다.


## root 실제 native 인수 결과

[세 번째 실행 result](../validation/evidence/cookie-keeper-startup-third/result.json)과 [원시 native 로그](../validation/evidence/cookie-keeper-startup-third/native.log)를 읽어 대조했다. root 실행의 `cargo check --workspace --all-targets --locked`, `cargo build -p dlpad-engine --bin dlpad-process-guard --locked` 및 `cargo test -p dlpad-engine --test cookie_keeper_startup_fault --locked actual_keeper_ -- --ignored --nocapture --test-threads=1` 모두 exit 0이다. 실제 필터는 2시험 PASS/0FAIL/29filtered, 15.35초이며 내부 모듈 단위시험을 native 성공 수로 합산하지 않았다. source-before/after 동일도 result에서 확인했다. fixture source hash는 앞선 handoff와 같은 `19eae84592c7adea67bf18ac24d4d2c4812db43b7898239966c931a710a3a250`다.

정상 metadata0는 실제 I의 reader PID 599004/startticks 4317459, nonce unit/native pin을 확인하고 Bound/B/RGCLC/EOF 및 actual receipt resolve와 세 unit GC의 strict assert를 통과했다. fault 시험은 after-start(PID599052/start4317855), after-get-unit(PID599082/start4318161), after-invocation(PID599114/start4318469)의 각각 실제 reader와 keeper/parent nonzero Invocation 및 active 관측을 기록했다. 세 단계 모두 LU/GU+EOF·tool marker 부재·R 부재·Intent bytes 및 zero receipt 보존·소유 helper 실패 종료·원 PID 종료와 세 nonce unit GC assert를 통과했다. unknown unit 이름을 추정해 Stop하지 않았다.

의도한 fault 로그에 fixed synthetic startup failure 및 첫 두 단계의 Broken pipe가 함께 남았다. 이 출력 자체를 성공 근거로 쓰지 않고 실제 프레임/종료/GC/원본 보존 assertions와 child test 결과로 판정했다. 이전 실패 evidence는 변경하지 않았다.

이 작업의 Linux debug-hook native fixture 인수는 완료했다. release의 환경 override 무시, live/pending foreign sibling, 실제 재부팅, 다른 OS 및 전체 workspace 완료는 이 결과 범위가 아니며 root의 별도 검증을 따른다. 이 담당은 native 명령을 직접 실행하지 않고 root 원시 결과를 독립 검토했다. Rust 소스 동결을 유지한다.

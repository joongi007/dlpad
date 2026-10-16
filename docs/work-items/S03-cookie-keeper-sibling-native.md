# S03: 제품 keeper 종료 후 sibling 보존 native fixture

- 상태: done
- 분류: native-regression
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: metrics_recovery — 신규 tests/cookie_keeper_sibling.rs 및 이 문서
- 선행 작업: S03-cookie-keeper-sibling-native-design, stress의 launcher 전용 debug receipt barrier

## 계약과 인수

실제 C/I/durable bind/B/R과 receipt 생성 경로에서 K checkpoint 뒤 fixture 소유 EOF worker를 같은 parent의 live 또는 activating+nonzero Job sibling으로 만든다. original Invocation/PID/start/pidfd/Job를 고정하고 B ack 뒤 reader/keeper GC와 parent 보존을 분리 검증한다. sibling EOF 자연 종료 후 exact owned release와 parent GC를 확인한다. prototype 결과로 대체하거나 populated parent를 새 empty proof로 간주하지 않는다.

기존 helper fixture를 신규 파일에 read-only 복제하며 원본 service_gate는 변경하지 않는다. sibling controller는 argv systemd-run --user --pipe --wait를 사용하고 EOF writer는 fixture만 보유한다. barrier FD는 launcher에만 explicit 상속한다. 원 source/cookie/sentinel 및 실제 receipt identity를 검사한다. unknown manager/unit identity에 Stop하지 않는다. 제품/Cargo/schema 변경과 자체 native 실행은 금지한다.

## 검증과 handoff

root가 debug hook 연결 뒤 실제 두 ignored test를 실행한다. 구현 전 scope 기록이며 실제 결과는 아직 없다.

## 구현 handoff

ignored native 시험 actual_cookie_keeper_live_sibling_preserved_after_receipt와 actual_cookie_keeper_pending_job_preserved_after_receipt 두 개를 작성했다. K 전에 sibling 난수 nonce, 기존 unit 부재 조회, private EOF pipe 및 argv를 준비한다. K 뒤 원 256바이트 receipt를 검증하고 sibling을 시작하여 2초 안에 실제 active 또는 activating/nonzero Job 및 native PID 증거를 확인해야 B를 보낸다. timeout을 완화하지 않는다.

keeper의 원 Invocation/PID/start/pidfd와 parent 밖 ControlGroup을 B 전에 고정한다. keeper/reader GC 뒤에는 parent의 원 Invocation/native inode가 active로 유지되고 sibling의 원 PID/start/pidfd와 pending Job이 보존되는지 분리 확인한다. parent의 새 populated 상태를 새 empty proof로 취급하지 않는다. receipt는 앞선 엄격한 proof의 기록이다. 최종적으로 fixture 소유 sibling writer만 닫아 worker의 자연 exit0을 받고 exact Invocation 확인 뒤 Unref와 parent 자동 GC를 관측한다. parent Stop 요청은 없다.

TOOL은 실제로 모든 non-stdio socket FD 부재와 barrier 환경 키 부재를 검사한다. 원 cookie/source/sentinel 보존을 유지한다. sibling controller는 spawn 직후 RAII로 소유하고 실패 경로는 자체 writer 종료와 bounded reap/소유 controller kill만 수행한다. unknown unit 이름을 추정해 신호를 보내지 않는다.

`.local-tools/cargo/bin/rustfmt --edition 2024 crates/engine/tests/cookie_keeper_sibling.rs` 실제 exit 0. SHA256 `3af61d235be8033c2a82ffb3a7cdf4f36f055b229dbbca5153a51a4123c12c08`로 root에 동결 전달했다. 자체 Cargo/native는 실행하지 않았다. 제품 두 case의 실제 결과는 root 인수이며 prototype PASS로 대체하지 않는다.

Root 첫 CPU all-target check가 ready 배열과 참조의 비교형 오류로 exit101이었다. native는 실행하지 않았다. 제품 조건을 변경하지 않고 fixture 비교를 `*b"K"`로 수정했다. 원 실패 로그는 sibling-first에 보존하고 second에서 재검증한다.

Second check/helper exit0/source 일치, 실제 native2FAIL. 이 host systemd-run은 --expand-environment=no를 지원하지 않아 sibling을 만들지 못했다. literal Python argv에는 shell/환경 변수 확장 문자열이 없으므로 해당 fixture CLI 옵션만 제거하고 third로 재검증한다. 제품 종료 증거나2초 deadline은 변경하지 않는다.

## 실제 실행의 fixture 실패와 교정

root의 [첫 compile](../validation/evidence/cookie-keeper-sibling-first/check.log)은 K 비교의 array/reference 타입 오류로 실패했다. root가 `*b"K"`로 교정했다. [두 번째 native](../validation/evidence/cookie-keeper-sibling-second/native.log)는 설치된 systemd-run의 expand-environment 옵션 미지원, [세 번째 native](../validation/evidence/cookie-keeper-sibling-third/native.log)는 CLI property parser의 AddRef=yes 미지원으로 실패했다. 이 단계에서는 sibling 실제 보존이 아직 입증되지 않았으며 실패 결과를 남겼다. literal worker argv에는 변수·shell 실행이 없어 root가 해당 미지원 option만 제거했다.

공식 [systemd run 구현](https://github.com/systemd/systemd/blob/main/src/run/run.c)은 bus-client의 StartTransientUnit에 AddRef를 자동 포함하지만 이 참조는 해당 connection 수명까지만 유지한다. fixture에는 service terminal 뒤 원 속성 확인까지 살아 있는 별도 참조가 필요하므로 임시 busctl Ref를 쓰지 않는다.

추가 승인 범위에서 신규 test-local libsystemd FFI만 작성했다. CLI AddRef property를 제거하고 K 전에 고정 owned UID bus socket과 지속 connection을 준비한다. method timeout은 2초이며 K 뒤 sibling 상태/native proof, exact GetUnit path와 Invocation 전후 대조, Unit.Ref 획득을 완료해야 B를 보낸다. 원 barrier absolute2초 제한을 유지한다. sibling EOF와 실제 원 종료속성까지 connection을 보유하고 마지막 exact path/Invocation 재검증 후 같은 connection으로 Unit.Unref한다. error/message/bus/library native 자원은 fixture 안에서 해제한다. 제품 user_service.rs를 복제하거나 공개하지 않았고 Cargo/dependency 변경은 없다.

최종 source SHA256 `b0e9c18a22c3639c31d57bd33dff54f4fbdce30393d5a259dced524a8e9acf70`, 개별 rustfmt 실제 exit 0. 수정 뒤 Cargo/native는 이 담당이 실행하지 않았으며 root에 동결 인계했다. 실제 sibling 두 case 성공은 여전히 다음 native 인수로 남는다.


## root의 네 번째 실제 제품 native 결과

[네 번째 result](../validation/evidence/cookie-keeper-sibling-fourth/result.json)와 [원시 native 로그](../validation/evidence/cookie-keeper-sibling-fourth/native.log)를 독립적으로 읽어 대조했다. root의 `cargo test -p dlpad-engine --test cookie_keeper_sibling --locked actual_cookie_keeper_ -- --ignored --nocapture --test-threads=1`은 실제 exit 0, 2PASS/0FAIL/29filtered, 14.37초다. source-before/after 동일도 result에서 확인했다. 앞선 compile/CLI 옵션/CLI AddRef 실패 증거는 그대로 보존했다.

live case는 실제 guardian PID624442/start4487687의 I/native proof 뒤 sibling Invocation `d6cfe9389042451f90d68ba13c845e06`, PID624473/start4487901을 고정했다. pending case는 guardian PID624528/start4488414 뒤 sibling Invocation `7969864fcf004e8e91ae765e062c3a64`, PID624559/start4488615, nonzero Job9185를 고정했다. 두 case 모두 receipt K checkpoint 이후 삽입, exact 지속 bus Unit.Ref 및 2초 이내 B, 실제 RGCLC/EOF와 receipt resolve, reader/keeper GC 및 parent 원 Invocation/native identity 보존, sibling 원 PID/pidfd 생존을 요구하는 strict assertions를 통과했다. pending Job9185의 동일성 assertion도 통과했다.

그 뒤 fixture의 원 EOF writer만 닫아 sibling 자연 exit0/원 종료속성/pidfd 종료를 확인하고 exact path/Invocation을 다시 검사해 같은 지속 connection의 Unit.Unref로 release했다. sibling 및 parent GC와 원 source/cookie/sentinel 보존 assertions까지 통과했다. parent를 직접 Stop하거나 unknown unit 이름으로 신호를 보내지 않았다. 이 실제 제품 결과는 앞선 prototype의 단순 의존성 결과와 구분한다.

이 작업의 Linux debug-barrier live/pending 제품 native 인수는 완료다. production release env 무시, 다른 OS, GUI/전체 workspace 및 비용은 별도 검증 범위다. root 원시 결과를 문서 검토했으며 이 담당은 Cargo/native를 직접 실행하지 않았다. Rust 동결을 유지한다.

# S02: 실제 same-boot worker 종료 뒤 쿠키 복구

- 상태: ready (소스 작성·freeze, 실제 native 실행 대기)
- 분류: native-regression
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: stress_recovery; 신규 crates/engine/tests/cookie_recovery_gate.rs 및 이 문서
- 선행 작업: root recover_sessions same-boot receipt 연결, metrics journal, 최신 native guardian

## 사용자 결과

실제 인증 작업의 worker가 destructors 없이 종료되거나 실행 중 강제 종료된 뒤, 확인된 native terminal receipt만으로 개인 복사본을 회수할 수 있는지 확인한다.

## 계약과 범위

Linux ignored 시험 두 개와 내부 worker. 부모가0700 임시 fixture와 합성 source/sentinel을 소유한다. worker는 DownloadRepository.execute_with_session 공개 API와 user-systemd backend를 사용한다. source literal은 합성값만 사용한다. 실제 계정/네트워크/기존8h/외부PID에 접근하지 않는다. 이번 Child handle과 proc start identity가 일치할 때만 kill/reap한다. 모든 관측과 복구 polling은 상한이 있다. 제품/Cargo/기존 fixture 수정 없음.

## 인수 조건

success worker는 실제 artifact를 만든 뒤 process::exit(0)으로 lease destructors를 건너뛰고, 부모 recover_sessions가 recovered1을 확인한다. crash worker는 실제 tool ready 중 부모가 이번 worker만 종료한다. 최초 복구의 Busy/deferred 또는 이미 확정된 즉시 회수 결과를 정확히 구분하고 최대10초 내 receipt 기반 recovered1을 확인한다. source/sentinel/artifact 보존과 private copy/owner 삭제를 확인한다. Intent-only/no-proof 보존은 별도 기존 시험 범위다.

## 검증과 handoff

신규 시험 파일 최초 source SHA256 `9f02db031525e1ed94a932c2fc7d250992c8c25a1ec3bea39bc0ce96312422ee`. 소유 파일만 rustfmt 실행 PASS. Cargo 및 native 실행은 하지 않았다. root의 최신 helper와 recover_sessions/journal 연결 뒤 명시 native 시험을 실행해야 한다.

실행 후보: `cargo test -p dlpad-engine --test cookie_recovery_gate --locked recovers -- --ignored --nocapture --test-threads=1`. 내부 worker는 root 환경 변수가 없으면 아무 작업도 하지 않는다. 각 부모는 owned Child RAII와 proc start identity로 이번 worker만 종료하고 reap한다. worker는 부모 fixture를 TempDir로 소유하지 않으므로 worker 종료로 fixture가 사라지지 않는다.

Kill/reap과 최초 recover 호출 사이에 guardian이 이미 정상 EOF/receipt를 완료할 수 있으므로 crash 첫 결과는 deferred1 또는 이미 recovered1을 구분해 고정 counts만 출력한다. deferred를 반드시 관측했다고 주장하지 않는다. 이후 최대10초 내 recovered1이 필요하다. tool sleep은15초지만 worker readiness deadline12초 후 즉시 parent kill, 이후 recovery deadline10초/총30초 상한이다. 실제 환경에서 guardian terminal cleanup/owned units와 FD 비용은 root 원시 검증 대상이며 이 파일 작성만으로 native PASS 또는 P0 완료를 주장하지 않는다.

root 첫 Cargo 컴파일은 cancel 인자의 `&AtomicBool` 대 `&Arc<AtomicBool>` 불일치 E0308로 실패했다. 원시 `/tmp/dlpad-cookie-sameboot-native-first.log` 보존. 담당 파일을 Arc로 수정한 뒤 소유 rustfmt PASS. 새 frozen SHA256 `e10bf397780e01bbba11d3761637417ff680b564bc742d51ddc3626051823c4e`; 재컴파일/native 결과는 아직 미검증이다.

root 최초 실제 두 부모 시험은2FAIL: success recovered1/deferred0/rejected1, killed recovered0/deferred1/rejected1. `/tmp/dlpad-cookie-sameboot-native-fourth.log` 보존. sentinel은 의도적 외부 cache 항목이므로 scanner가 rejected1로 세는 기존 계약과 fixture rejected0 기대가 어긋났다. 이를 숨기는 filter나 scanner 변경은 하지 않는다. worker 시작 전 동일 cache recover 기준을 정확히 recovered0/deferred0/rejected1 및 truncated=false로 확인하고 source/sentinel bytes를 확인한다. 이후 모든 scanner rejected값은 기준1과 같아야 하며 recovered1 및 crash deferred/receipt polling 인수는 그대로 유지한다. 잘못된 이전 filter의0tests 로그도 보존하며0tests를 PASS로 계산하지 않는다. 두 부모만 선택하는 실제 실행 명령은 위 recovers 필터를 사용한다.

수정 후 소유 rustfmt PASS, frozen SHA256 `c6776455d7153b18038c346f9301c2fa1b399e4b92b9339b2a2845a423498e69`. 담당 에이전트는 Cargo/native 재실행하지 않았다. root 실제 재검증 대기.

최신 root debugON 실제 재검증 `evidence/cookie-dependency-current-native-eighth/recovery.log`를 독립 읽기 대조했다. 두 부모2PASS4.78초이며 killed first(recovered0,deferred1,rejected1), success first(1,0,1)을 확인했다. result.json source_unchanged=true/trace_enabled=true 및 recoveryexit0도 확인했다. source/API 작성과 최초 오류를 별도 보존하며 이 결과는 실제 same-boot 성공/강제worker종료의 합성 두 인수다. 실제 reboot/unknown early identity 전체 cleanup 보장은 별도다.

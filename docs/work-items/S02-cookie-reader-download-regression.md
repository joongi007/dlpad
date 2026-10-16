# S02: 인증 download reader 확인과 worker unwind 회귀

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R08, R10
- 담당/수정 범위: soak_recovery_audit — crates/engine/tests/session_cookie_download.rs, 이 문서
- 선행 작업: cookies reader guard 및 download metadata/download reservation 통합 sourcefreeze

## 사용자 결과

인증 도구의 종료 확인이 있는 성공·실패·취소는 정상 정리하고, reader 확인이 없는 worker unwind는 복사본·회복 기록을 보존하는지 실제 합성 프로세스로 시험한다.

## 계약과 범위

기존7개는 성공/metadata·download 실패/실제 취소/파싱 실패/재개에서 cleanup 성공으로 pending 해제를 이미 관측한다. 이를 begin_reader 재예약 가능으로도 직접 확인한다. Guardian SIGKILL은 SessionCleanupDeferred와 pending 지속을 확인한다. cookies.rs/download.rs/process.rs/Cargo/공유 문서는 수정하지 않고 Cargo 실행은 root에 넘긴다.

실제 download 내부 Rust panic injection hook은 공개 API에 없다. 제품에 테스트 hook을 추가하지 않는다. 추가 scoped-worker fixture는 실제 managed process::run 완료 직후 reader confirm 전에 Rust panic을 일으켜 guard unwinding/cleanup 거절/FD 닫힘/Drop 보존을 확인한다. 이는 execute_with_session 내부 panic 주입의 증명이 아니라 공개 reader guard와 실제 도구를 조합한 경계 시험이다. 실제 계정/네트워크/manager/사용자 자료를 사용하지 않는다.

## 인수 조건

- 기존 실제 정상·실패·취소 경로 후 reader 재예약/confirm/cleanup 성공.
- 실제 guardian SIGKILL 후 새 reader 예약 거절, caller defer 후 원본/복사본/records 보존.
- 실제 도구가 종료해도 confirm 전 scoped Rust worker unwind이면 cleanup ReaderUnconfirmed, lease 무효/모든 소유 FD 닫힘/Drop 후 copy+records 보존.
- DB/source/sentinel 비밀 비노출과 보존, 기존7개 회귀 유지.

## 검증과 handoff

기존7개 정상 성공/metadata·download 실패/취소/재개/typed JSON 실패 이후 `begin_reader().confirm()` 재예약을 보강했다. guardian SIGKILL의 실제 deferred 이후에는 `ReaderUnconfirmed`를 assert한다. Eighth scoped worker는 실제 version 무쿠키 확인과 metadata cookie tool exit7/guardian 정리 후 confirm 전 고정 문구로 panic하고, caller cleanup 거절·lease 무효·FD 닫힘·Drop 후 copy/records/source/sentinel/DB 불변을 확인한다. 새 프로세스/제품 hook은 추가하지 않았다.

소유 rustfmt 완료 및 sourcefreeze 상태다. 하위 담당은 Cargo를 실행하지 않았다. Root는 이전 snapshot에서 engine library185PASS/2ignored, 기존 download7PASS/importer20PASS 및 engine all-target clippy exit0를 보고했다. [이전 실제 결과](../validation/evidence/cookie-reader-guard-engine/result.json)는 새 eighth 실행을 포함하지 않는다.

Root가 동결 SHA `de17c503ab7e65c112e94d31c84e8bb29b61ded5cc6fd557b53e34d18f602b70`에 실제 `cargo test -p dlpad-engine --test session_cookie_download --locked`를 실행했다(session50675). 결과 exit0, 8PASS/0FAIL/0ignored, 시험1.63초/compile11.07초다. Root는 eighth의 실제 managed tool terminal 뒤 scoped Rust panic/guard cleanup 보존 경계를 직접 검토했다. 이 새8case의 clippy는 아직 미실행이고 이전 전체 engine/clippy 결과는 기존7case source 근거다. 제품 내부 panic 주입·다른 OS·실제 인증은 검증하지 않았다.

# S02: 전용 reader container의 gated launcher 연결

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: root — cookies.rs(기존 canonical digest의 crate 내부 재노출), process.rs, process/guardian.rs, process/user_service.rs, tests/cookie_reader_models.rs, 이 문서; stress_recovery — 별도 cookie_service_gate.rs/native workitem
- 선행 작업: actual dedicated slice strict empty proof, capability 교환, receipt FD 검증

## 사용자 결과

쿠키 reader를 시작하기 전 launcher가 native identity를 전달하고 app의 durable bind 후 Begin을 기다린다. 종료 후 원래 전용 컨테이너의 실제 빈 그룹을 확인해 receipt를 sync한다.

## 계약과 범위

새 내부 cookie-user-launcher marker는 control/writer/record-lock FD와 record native dev/ino만 받는다. C는 launcher만 읽고 guardian만 B를 읽는다. writer/lock는 guardian/tool로 전달하지 않는다. 새 의존성/schema 변경 없음. 전용 top-level nonce slice와 service 두 unit만 관리하며 같은 bus AddRef/Invocation 확인으로 Stop/Unref한다. 기존 ungated 실행은 유지한다.

## 인수 조건

실제 manager C→I→durable bind→B→R→GC/LC/EOF, native parent populated0+빈procs 및 pidfd 종료 뒤 256B sync를 확인한다. beforeB EOF/잘못된B는 tool0/GU를보존. 정상 슬롯 resolve/reopen, 변조 FD·partialreceipt·manager실패는 보존. 실제 app importer 연결 전 전체 same-boot 복구 완료로 표시하지 않는다.

## 검증과 handoff

root 독립 dedicated slice 실제 exit0/held+newopenat strict empty PASS. service만 AddRef/RemainAfterExit는 원 cgroup을 보존하지 못했으므로 기존 proof 기준을 완화하지 않고 owned container 수명으로 해결한다. 구현 및 native 시험 진행 중.

첫컴파일은 private reader_records 접근1error였고 canonical hash를 cookie module의crate 내부함수로재노출해 second check exit0. actual receiptFD4회귀 PASS. 첫 ignored native3case는 Iheader 이전 EOF로 모두 실패했고 helper stderr가null이라 fixed원인추적이필요하다. 원시를보존했다. 독립검토의foreignsibling Stop경계는 exactownedservice만failureStop, parent는현재strictEmpty뒤에만Stop으로보강했으나 queuedStart/noactivejobs 및 초기sliceidentity실패의자원보존 인수는여전히남는다. appadapter연결전완료로표시하지않는다.

## Cargo 실행 문맥 실패의 최소 재현과 수정

직접 동결 driver는 native3PASS였지만 cargo test는 StartTransientUnit의 environment 거절3FAIL. 같은 driver에 고정 합성 `CARGO_BIN_EXE_dlpad-process-guard=/synthetic`만 추가하면3FAIL을 재현했다. Cargo의 hyphen 포함 빌드 helper 힌트는 systemd 변수명 계약에 맞지 않는다. launcher가 이 정확한 prefix의 부적합 이름만 제외하고 나머지 잘못된 이름은 값 노출 없이 사전 거절한다. 정상 runtime 환경·literal value는 유지한다. 근거: [systemd 공식 실행 환경 문서](https://github.com/systemd/systemd/blob/main/man/systemd.exec.xml). 새 의존성/런타임 프로세스 비용 없음. 수정 후 실제 Cargo/native 재검증 중이며 첫 실패 원시는 보존한다.

수정 후 `cargo test -p dlpad-engine --test cookie_service_gate --locked -- --ignored --nocapture --test-threads=1` 실제 manager 3PASS(두 정상슬롯, beforeB EOF, wrongB). helper build exit0. [실패/직접 성공/수정후 Cargo 원시](../validation/evidence/cookie-service-environment-regression-20261004/cookie-env-fix-native.log)를 보존했다. 초기 identity 실패 자원 관리/queued sibling 경계와 app adapter 통합은 아직 남아 상태 in-progress를 유지한다.

환경 파싱/주소실패경로 unit1PASS. 첫 engine clippy는 기존 추가 parent Drop의 중첩조건2건으로실패; path검증도let-chain에포함해정리했고 재검증중. 실패로그는동일evidence에보존했다.

두번째 `cargo clippy -p dlpad-engine --all-targets --locked -- -D warnings` exit0. 전체workspace/앱연결/추가native경합 인수는 후속으로 유지한다.

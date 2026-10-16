# S03: 실제 제품 guardian user service 실험

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R08
- 담당/수정 범위: soak_recovery_audit; scripts/systemd_product_guardian_probe.py, 전용 work-item/validation/evidence
- 선행 작업: S03-systemd-execstartex-prototype

## 사용자 결과

현재 immutable 제품 guardian을 동일 MainPID의 transient service로 실행하고 소유 종료와 literal argv/표준 출력 계약을 합성 fixture로 검증한다.

## 계약과 범위

README → PRODUCT → CONTEXT → R08/ADR/ARCHITECTURE 및 실제 guardian 소스 계약을 읽었다. FD0 socket을 FD>=3으로 복제하고 exec하며 고유 unit/InvocationID/MainPID/start/pidfd를 확인한다. 제품/Cargo/schema/기존 8h/사용자 desktop은 수정하지 않는다. 정확 snapshot SHA를 시작과 끝에 확인한다. 실패 unit은 해당 InvocationID를 확인한 경우에만 reset-failed한다.

## 인수 조건

EOF/K/T/T→K cancel/guardian SIGTERM/SIGKILL/소유 endpoint owner+guardian SIGKILL/natural exit42를 시험한다. worker/setsid child/leaf의 동일 cgroup과 정확 identity 종료, 외부 owned sentinel 보존, literal argv/worker stdin null/stdout stderr/서비스 종료 status를 기록한다. runtime 45초와 bounded synthetic 로그, owned tempfile cleanup을 유지한다.

## 검증과 handoff

실제 immutable SHA 시작/끝 일치, 8개 합성 case exit0/PASS. 제품 guardian 동일 MainPID/FD>=3/R/argv/stdin/stdoutstderr/exit42 및 EOF/K/T/cancel/TERM/KILL/owner+guardian KILL cleanup을 검증했다. 네 실패 입구는 exit2/output 미생성. 최초 raw와 최종 source/evidence를 보존했다. 정확 명령/종료시간/제약은 전용 validation을 참조한다. 전체 R08 및 제품 통합 채택을 선언하지 않는다.

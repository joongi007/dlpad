# S03: user service 생산 backend 통합 시험

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R08
- 담당/수정 범위: soak_recovery_audit; crates/engine/tests/user_service_guardian.rs, scripts/user_service_engine_probe.py, 전용 validation/evidence
- 선행 작업: root/metrics의 user-systemd backend READY

## 사용자 결과

실제 process::run/run_metadata 경로로 service backend의 종료·취소·출력·crash 소유권 계약을 검증한다.

## 계약과 범위

기존 guardian.rs 시험과 process API를 읽었다. backend env는 subprocess별 DLPAD_GUARDIAN_BACKEND=user-systemd로 지정한다. Rust ignored driver는 자기 test binary 안에서 호출되고 기존 dlpad-process-guard helper를 사용한다. root/metrics만 제품 source/backend/Cargo를 담당한다. 소유 synthetic fixture만, 기존 unit/8h PID/global 설정/desktop은 건드리지 않는다. READY 이전에는 driver를 실행하지 않는다.

## 인수 조건

pipe EOF/exit0/42, metadata Unicode와 $argv/환경, cancel/timeout, owner/guardian/두 프로세스 SIGKILL, missing manager/unknown backend failclosed, FD leak, backpressure 및 bounded evidence를 명시적으로 선택 실행한다. 정확 PID/start/pidfd/InvocationID/cgroup과 owned unit GC, sentinel 보존을 확인한다. 기본 cargo test에 user manager 의존성을 추가하지 않는다.

## 검증과 handoff

root READY compile 후 명시 선택 primary3case 및 전체13case 실제 exit0/PASS. 정확 binary/helper SHA 시작끝일치. cancel/timeout/crash/unitGC/exit0/42/metadata/실패입구/FD12회/4.04MB bounded-output 검증을 전용 validation/evidence로 보존했다. Python 문법/소유 Rust rustfmt 완료. product backend/default/ADR 채택과 workspace 최종검증은 root 담당이다.

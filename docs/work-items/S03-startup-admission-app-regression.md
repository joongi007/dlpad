# S03: startup admission/reaper 최신 앱 정상 회귀

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: soak_recovery_audit; 이 work-item, 전용 validation/evidence/guardian-startup-wait-app-20261004
- 선행 작업: root admission/reaper full7 PASS 및 immutable97cfbcec release READY

## 사용자 결과

최신 admission/reaper 구현에서도 기본 service 다운로드·숨김복귀·중단종료가 정상이며 startup fault 테스트 환경이 없는 일반 동작임을 확인한다.

## 계약과 범위

기존 user_service_app_probe를 수정 없이 재사용한다. 단독 chmod0500 /tmp dlpad-cpu immutable snapshot을 읽기 전용 사용하고 private Xvfb/private 접근성 D-Bus와 소유 synthetic tool/library만 실행한다. backend/debug bus/startup fault env는 unset. 이전 a67 release fault-env 원자료와 기존 모든 evidence/shared snapshot/제품/Cargo/8h/desktop을 변경하지 않는다.

## 인수 조건

active-hidden/complete-hidden/stop-exit3flow, same app/worker/job/attempt 복귀, Cancelled/Downloaded journal/artifact, exact app guardian exe/SHA/Inv/MainPID/start/pidfd/cgroupGC, unrelated library/process sentinel 및 owned cleanup을 반복 확인한다. 다른 OS/8h 검증으로 확대하지 않는다.

## 검증과 handoff

root READY snapshot97cfbcec로 실제 기본 GUI3flow exit0/PASS. actual primary의 backend/debugbus/startupfault key 부재, samebinary service 보호/GC, journal/artifact/복귀/sentinel 보존과 총11PID·3cgroup 종료 부재를 재검사했다. 원시/source/상세범위는 startup-admission-app-regression validation/evidence에 보존. 제품/Cargo/probe/8h/sharedsnapshot 변경없음.

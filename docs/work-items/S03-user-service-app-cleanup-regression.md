# S03: 최신 startup cleanup release 앱 회귀

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: soak_recovery_audit; scripts/user_service_app_probe.py의 release fault env 옵션, 이 work-item, 전용 validation/evidence
- 선행 작업: root 최신 alias/EP source 전체 검증 및 a67d7224 CPU immutable release READY

## 사용자 결과

최신 release 앱의 기본 service 다운로드·숨김복귀·중단종료를 다시 확인하고 debug startup fault 환경이 실제 전달되어도 release 동작을 방해하지 않음을 검증한다.

## 계약과 범위

단독 읽기 전용 /tmp dlpad-cpu snapshot을 사용한다. --release-fault-env after-start 옵션은 테스트 Python/child env에만 설정하며 actual app와 guardian environ에서 key 존재만 확인한다. DLPAD_GUARDIAN_BACKEND/debug bus override는 계속 unset한다. 기존 raw/source snapshot, 공유 제품/Cargo/8h/desktop/다른 unit은 변경하지 않는다.

## 인수 조건

3flow 실제 private Xvfb GUI PASS, same app guardian exe/SHA/InvocationID/MainPID/start/pidfd/cgroupGC 및 journal/artifact/sentinel 보존을 반복 검증한다. 실제 after-start env가 있는 release에서 정상 job 결과를 확인한다. 환경 값 전체·자격 증명을 출력하지 않는다.

## 검증과 handoff

root READY 후 최신 단독 a67d7224 release에서 startup fault env after-start가 실제 primary/guardian에 존재하는 상태로 GUI3flow exit0/PASS. default backend env 부재/samebinary service/GC/journal/artifact/sentinel 유지, 총11PID 및3cgroup 부재 재검사. source/raw/상세한계는 전용 validation/evidence 보존. 공유 source/Cargo/기존 evidence/immutable snapshot은 수정하지 않았다.

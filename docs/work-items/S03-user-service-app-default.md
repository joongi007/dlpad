# S03: 등록 CPU 앱 기본 user service 검증

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: soak_recovery_audit; scripts/user_service_app_probe.py, 이 work-item, 전용 validation/evidence
- 선행 작업: root 등록 앱 기본 user-systemd 채택 및 CPU release READY

## 사용자 결과

단독 dlpad-cpu executable에서 명시 backend env 없이 네이티브 다운로드·close-stop·숨김복귀 흐름이 기본 user-service 보호를 사용하는지 검증한다.

## 계약과 범위

기존 CPU background/accessibility fixture는 읽기 전용 import로 재사용한다. newly owned Xvfb/private accessibility D-Bus만 사용하고 실제 host user manager의 고유 dlpad-job service를 관찰한다. 정확 PPID guardian exe dev/inode/hash=동일 단독 immutable tmp app, unit InvocationID/MainPID/start/cgroup/pidfd, 실제 앱 backend env 부재를 기록한다. 기존 테스트/source/Cargo/8h/unit/global 설정/desktop은 변경하지 않는다. READY 이전 GUI를 실행하지 않는다.

## 인수 조건

helper 없는 immutable Linux tmp 단독 snapshot으로 active-hidden/complete-hidden/stop-exit UI를 선택 실행한다. same app/worker/job·attempt의 hide/restore, 다운로드 완료 artifact, close-stop Cancelled journal, 소유unit GC/workerguardian identity 소멸, 라이브러리 sentinel 및 외부 owned process 보존을 확인한다. 다른 OS/전체 P0 완료로 확대하지 않는다.

## 검증과 handoff

root release READY 후 단독 chmod0500 ea7595 snapshot으로 실제 GUI3flow exit0/PASS. primary 실제 backend/debug env 부재, helper없음, same app guardian exe inode/SHA, 3unit exact InvocationID/MainPID/start/cgroup/pidfd와 GC, active/complete 숨김복귀와 Cancelled/Downloaded journal/artifact, library/process sentinel 보존을 검증했다. 총11기록PID 및3cgroup 부재를 추가조회했다. 실행/source/evidence/범위한계는 전용 validation에 기록했다. shared product/Cargo/probe/8h/snapshot은 수정하지 않았다.

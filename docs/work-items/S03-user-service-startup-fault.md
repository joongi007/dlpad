# S03: accepted user-service startup 실패 정리

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R08
- 담당/수정 범위: soak_recovery_audit; scripts/user_service_engine_probe.py, 기존 ignored driver(필요 시), 이 work-item, 전용 validation/evidence
- 선행 작업: root startup EP/shutdown alias 및 metrics debug startup hook READY

## 사용자 결과

서비스가 실제 도구를 시작한 뒤 launcher 초기화가 실패해도 원본 socket shutdown이 도구/setsid 자손/pipe/cgroup을 정리하는지 검증한다.

## 계약과 범위

DLPAD_TEST_SYSTEMD_STARTUP_FAULT 정확 값 after-start/after-get-unit/after-invocation, debug-only 선택 시 1초 관찰 창을 사용한다. release에는 hook env 조회가 없다. 별도 trace/전체 unit 조회/phase 파일은 만들지 않는다. 실제 worker/guardian/unit InvocationID/MainPID/start/cgroup을 live 확인한 경우만 인수한다. EP 먼저면 Other fixed error, R 먼저면 일반 launcher exit126 가능성을 구분한다. 원시 기존13case 및 앱GUI3flow 증거/source를 보존한다. Cargo/공유 제품은 root/metrics 담당이며 READY 이전 실행하지 않는다.

## 인수 조건

명시 선택한 3fault case에서 실제 live worker/guardian와 setsid 자손을 pin/검증 후 초기화 실패 outcome, exact identity 소멸/pidfd ready/pipe 유한 반환/cgroup 제거/unit GC/무관 sentinel/FD4→4/log 상한을 확인한다. 관측하지 못한 unit/tool은 FAIL 또는 unverified로 기록하며 추정 PASS 금지. missing-manager는 Other / Linux user-service protection setup failed fixed 오류와 tool 미시작을 assert한다.

## 검증과 handoff

root READY 뒤 actualstartup3fault+missingmanager 실제 exit0/PASS, 이어 같은alias/helper strict13회귀도exit0/PASS. startup3case 모두liveunit/worker/setsid자손 관측→R먼저/Exited126→원socketshutdown cleanup/cgroupGC/FD4→4 확인. missingmanager Other fixedEP오류+tool미기동 확인. oldraw/GUI3자료보존, 새source/evidence/post-run재검사와 상세한계는 전용 validation에 기록했다. Rust driver/Cargo/shared product는 수정하지 않았다.

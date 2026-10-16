# S00: 모니터 종료 운용의 재중단 확인 준비

- 상태: done (단일 기준점·복구 절차 범위)
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — 이 문서, wsl-interruption-recovery.md, 신규 baseline evidence; stress_recovery — 별도 재접속 감사 workitem
- 선행 작업: snapshot collector와 wsl-resume-checklist.md

## 사용자 결과

화면만 끄고 자리를 비운 뒤 재접속하면 새 boot UUID와 이번 기준점을 비교해 WSL 재부팅 여부를 구분한다. 명령별 terminal 증거를 대조해 완료된 시험과 중단된 시험을 구분한다.

## 계약과 범위

읽기 전용 수집과 새 프로젝트 파일만 생성한다. 사용자 화면·전원·서비스·WSL 설정을 변경하지 않는다. 정상 Git 저장소가 없으며 기존 .git을 보존한다. 호스트 watchdog과 자동 세션 재개는 구현하지 않았다.

## 인수 조건

완전한 snapshot JSON/exit0와 원시 boot/uptime을 독립 대조한다. 수집기 실패 경로 회귀를 실행한다. 재접속 절차와 한계를 기록한다.

## 검증과 handoff

`python3 -m unittest discover -s scripts -p test_wsl_interruption_snapshot.py`: 6 PASS.
`python3 scripts/wsl_interruption_snapshot.py --output <동일 context canonical project>/docs/validation/evidence/wsl-monitor-off-session-baseline-20261004`: sandbox는 읽기 전용 파일시스템 오류로 실패, 승인된 프로젝트 쓰기 context에서 exit0로 새 수집 성공. 기존 자료를 덮어쓰지 않았다.
Root가 원시 JSON을 대조했다: UTC2026-10-04T01:03:18.321736+00:00, boot0bc0c937-eee9-4d90-95dc-31a2122b6352, uptime51532.95초, Interop enabled. 이전 kernel journal exit1/available=false. 이번 snapshot은 runner/agent 생존을 추적하지 않는다.
미검증: 실제 모니터 off 이후 연속성, 실제 WSL 중단 원인, 호스트 독립 감시·자동 재개. WSL이 종료된 동안 내부 에이전트는 실행하지 못한다. 다음 접속에서 새 snapshot과 각 실행의 terminal 자료를 대조한다.

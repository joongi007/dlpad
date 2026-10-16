# S00: 모니터 종료 전 현재 WSL 기준점

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — 이 문서, WSL 복구 문서, 신규 baseline evidence
- 선행 작업: 기존 snapshot 수집기와 재접속 체크리스트

## 사용자 결과

모니터를 끄기 전 부팅 기준점을 보존해 다음 접속 시 VM 재시작 여부를 비교한다.

## 계약과 범위

create-only 읽기 수집. 설정·서비스·전원·WSL 재시작·사용자 화면 변경 없음. Git 상태는 not a git repository이며 기존 .git을 보존했다.

## 인수 조건

완전한 JSON/exit0 확인, 원시 자료 대조, 관측 제한 명시, 수집기 회귀 실행.

## 검증과 handoff

`python3 -m unittest discover -s scripts -p test_wsl_interruption_snapshot.py`: 6 PASS.
기존 main을 동일 context canonical PROJECT 절대경로로 실행: 새 `wsl-monitor-off-user-baseline-20261004/snapshot.json`, exit0. Root JSON 대조: UTC2026-10-03T23:57:28.192508Z, boot0bc0c937-eee9-4d90-95dc-31a2122b6352, uptime47189.72초, Interop enabled. 이전 kernel journal exit1/available=false는 관측 제한이다.
미검증: runner/agent 생존, 화면 종료 이후 연속성, 실제 중단/절전, Windows 이벤트 재조회, 호스트 독립 감시와 자동 재개. WSL 종료 시 내부 agent도 멈춘다. 다음 접속에 새 snapshot/명령별 terminal 증거를 대조한다.

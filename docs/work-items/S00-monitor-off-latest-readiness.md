# S00: 모니터 종료 대비 최신 기준점 검증

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — 이 문서, 신규 WSL snapshot evidence
- 선행 작업: 기존 WSL snapshot 수집기 및 재접속 체크리스트

## 사용자 결과

모니터를 끄기 전 최신 WSL 기준점을 보존한다. 재접속 시 boot UUID와 시험 terminal 증거를 비교해 중단 여부와 재개 지점을 구분할 수 있다.

## 계약과 범위

설정·전원·서비스·사용자 desktop·WSL 재시작 변경 없이 읽기 전용 자료를 새 evidence에 기록한다. 기존 정상 Git 저장소가 없어 .git은 보존한다. 독립 감사는 soak_recovery_audit가 별도 신규 문서 하나를 담당한다.

## 인수 조건

create-only snapshot exit0, 완전한 JSON 검토, 수집기 실패 경로 회귀 통과, 관측 제한 기록.

## 검증과 handoff

`python3 -m unittest discover -s scripts -p test_wsl_interruption_snapshot.py`: 6 PASS.
`python3 scripts/wsl_interruption_snapshot.py --output <canonical-project>/docs/validation/evidence/wsl-monitor-off-latest-readiness-20261004`: host context exit0. 기본 sandbox에서는 프로젝트 mkdir가 EROFS로 실패했고, 승인된 프로젝트 쓰기 context에서 동일 create-only 수집을 완료했다.
원 JSON: UTC2026-10-04T03:20:45.550301+00:00, boot0bc0c937-eee9-4d90-95dc-31a2122b6352, uptime60607.81초, WSLInterop enabled. 이전 기준점과 같은 boot이다. Previous kernel journal 접근 불가 및 종료 요청자 unconfirmed를 유지한다.
최신 Linux workspace runner는 terminal exit0, 기본 fmt/clippy/test/release와 CPU clippy/test/release 모두 PASS. windows-cookie-late-seed-linux-workspace-first/result.json의 source_final SHA를 현재 파일과 대조해 불일치0을 확인했다.
미검증: 모니터 종료 이후 연속 실행, 향후 WSL 중단 방지, host 독립 watchdog·자동 재시작. Boot-only snapshot은 agent나 runner 생존 증명이 아니다. WSL VM이 종료되면 내부 agent도 멈춘다. 재접속 때 기존 체크리스트를 따라 새 snapshot과 명령별 terminal을 먼저 대조하며 미완료 시험을 성공 처리하지 않는다.

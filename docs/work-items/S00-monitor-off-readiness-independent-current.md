# S00: 모니터 종료 전 WSL 관찰 준비 독립 감사

- 상태: completed (읽기 감사; 최신 snapshot은 주 담당 별도)
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: soak_recovery_audit — 이 문서만
- 선행 작업: wsl_interruption_snapshot.py, 재접속 체크리스트, 기존 baseline/8h terminal

## 범위와 인수 조건

읽기 전용으로 boot 기준점·PID start/namespace 관측 범위·완료 증거·재접속 절차를 대조한다. 모니터/전원/WSL/보안/서비스 설정, desktop, 프로세스 종료·재시작을 수행하지 않는다. 기존 snapshot을 현재 live 실행 증명으로 표현하지 않고 종료 원인 미확정과 자동 재개 미보장을 유지한다.

## 실제 확인

[세션 baseline](../validation/evidence/wsl-monitor-off-session-baseline-20261004/snapshot.json)은 UTC `2026-10-04T01:03:18.321736+00:00`, boot `0bc0c937-eee9-4d90-95dc-31a2122b6352`, uptime51532.95초, WSLInterop enabled를 기록한다. 이전 kernel journal 접근은 exit1/available=false이다. runner 추적은 요청하지 않았고 host_visibility_verified=false이다. 이는 그때의 boot 기준점이며 새로 생성하는 주 담당 snapshot이나 현재 프로세스 상태를 대신하지 않는다.

[8h 독립 결과](../validation/evidence/soak-terminal-independent-20261004/result.json) 및 [root 검토](../validation/evidence/soak-terminal-independent-20261004/root-review.json)는 Linux synthetic headless download 28801.956초/guardian 28804.093초 완료, 17개 원시 파일 hash, host의 11 owned PID 부재, SQLite reopen/artifact 검증을 기록한다. 따라서 옛 launch의 running 선언이나 역사 live snapshot을 현재 실행 중으로 해석하면 안 된다. 옛 archive interrupted는 읽기/복사 시점의 terminal race였고 원본을 보존했다. 이것은 WSL shutdown의 증거가 아니다.

수집기는 다른 boot에서 old PID를 조회하지 않고 same boot에서 PID/start를 비교한다. stat 오류를 observation-unavailable로 구분하고 namespace metadata가 있어도 host visibility가 확정됐다고 주장하지 않는다. launch 없는 boot-only 명령은 개발 agent·Cargo·GUI 생존을 판정하지 않는다. 출력을 create-only로 만들고 실패한 부분 directory를 덮어쓰지 않는 절차가 문서에 있다.

## 남은 제한과 구체적 재접속 절차

[재접속 체크리스트](../validation/wsl-resume-checklist.md)의 기본 명령은 boot-only이므로 현재 실행 ownership manifest가 없다면 PID 추적을 추가하지 않는다. 체크리스트가 21:59 자료를 최신이라고 표현하는 부분은 이후 recovery 문서의 01:03 baseline보다 오래됐다. 실제 재접속에는 주 담당의 새 baseline 경로/UTC를 먼저 확인하고 같은 execution context에서 canonical project root를 얻은 뒤 fresh output snapshot을 생성한다. 기존 directory나 JSON 파일 존재만으로 성공을 판단하지 않고 exit0와 완전한 bounded JSON을 확인한다.

boot가 달라지면 이전 실행을 성공으로 추정하지 않고 각 실행의 durable terminal을 별도로 확인한다. same boot도 agent/터미널이 종료될 수 있으므로 boot 유지가 작업 연속성을 증명하지 않는다. sandbox absent는 host read-only 대조 없이는 중단 확정이 아니며 PID 번호 재사용에 signal을 보내지 않는다.

Windows executable 존재는 eventlog 재검증이 아니고, 이전 kernel journal 접근 실패는 사건 부재 증명이 아니다. shutdown requester는 계속 unconfirmed이다. 프로젝트 host watchdog은 설치하지 않았고 WSL VM이 꺼진 동안 내부 collector/agent는 기록·경고·재개할 수 없다. 모니터만 끄면 절전·host update·OOM·터미널 종료가 방지된다고 보장하지 않는다. 실제 monitor-off 연속성·WSL 재부팅 시험·자동 재개 인수는 수행하지 않았다.

## 실행 결과와 handoff

이번 작업은 source/문서/원시 JSON 읽기와 이 문서 작성만 수행했다. Cargo/native/GUI/새 snapshot/설정 변경/프로세스 조작은 하지 않았다. 주 담당이 새 create-only snapshot과 통합 검증을 수행하며, 사용자에게 그 최신 관측 UTC와 자동 재개 한계를 알려야 한다.

# S00: WSL 종료 후 개발 시험 복구

- 상태: done
- 분류: verification
- 우선순위: P0
- 요구사항: R08, R10, R11
- 담당/수정 범위: root; 본 문서·종료 증거·지속 로그, 제품 코드 변경 없음
- 선행 작업: 기존 지표·진단 통합

## 사용자 결과

실제 WSL 종료로 사라진 실행 세션을 완료로 처리하지 않고 보존한 source에서 검증을 이어간다.

## 계약과 범위

이전 boot794988eb에서12:14:28 systemd System Power Off, 새boot0bc0c937에서20:48:56 부팅을 확인했다. 종료 요청 주체는 미확인. 읽은 이전 kernel journal에는OOM/panic 증거가 없었다. Windows System 제한 조회는exit1/빈출력으로 원인을 증명하지 못했다. 에이전트는 WSL 종료/재시작 명령을 실행하지 않았다. .git/사용자data/시스템설정 변경 없음.

## 인수 조건

중단 실행과 완료 evidence를 구분하고 workspace 검증/격리 GUI를 새 run으로 지속 경로에 기록한다. 이전 soak boot identity는 현재와달라 live 주장을 철회하고 terminal없는시험은미검증으로유지한다. 기존 증거를 삭제하지 않는다.

## 검증과 handoff

journalctl --list-boots/-b -1 조회 및 현재boot/uptime 확인. evidence/wsl-interruption-20261003에 원시 기록. Linux98926 session missing와/tmp logs유실로 전체 검증 결과는재실행대상이다. 이전65unit/CPUclippy/release 및 저장된Windows native 결과는 각범위만유지한다.

## 인수 결과

새boot에서 [공통 재실행](../validation/evidence/cpu-metrics-workspace-recovery/result.json) fmt/defaultclippy/default200PASS/CPU325PASS/release 모두exit0, 각5ignored를 확인했다. 지표·큰보고서·공식설치취소·한글입력 실제 격리 GUI 새결과를 보존했다. [새8시간실행](S03-soak-recovery-run.md)은immutable Linux/tmp copies와지속archive로진행하며이전미완료시간을합산하지않는다. 종료요청주체는끝내확인되지않았고원인규명완료나8hPASS를뜻하지않는다. 복구작업범위만완료다.

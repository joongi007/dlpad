# S00: WSL 재중단 대비 terminal 상태 문서 갱신

- 상태: completed
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 파일: soak_recovery_audit — 이 문서, docs/validation/wsl-interruption-recovery.md
- 의존성: 기존 snapshot/checklist, soak-terminal-independent-20261004 원시 결과와 root 검토

## 범위

모니터만 끄고 개발을 맡기는 사용자 운용에 맞춰 과거 live/8h 미완료 관찰과 이후 terminal 완료를 구분한다. 기존 snapshot의 관측 범위 및 재접속 절차를 읽기 감사한다. 시스템 설정·서비스·watchdog 설치·runner 종료/재시작과 제품 수정은 없다.

## 인수 조건

- 과거 snapshot은 해당 UTC의 역사 관찰이며 현재 live라고 주장하지 않는다.
- 이후 8h 완료와 archive collector race를 원시 terminal 증거로 구분한다.
- 새로운 snapshot의 필요성과 기록 공백/자동 복구 한계를 명시한다.

## 검증

기존 snapshot·체크리스트·terminal result/root-review를 읽고 recovery 문서에 당시20:01 live와 이후8h 완료를 별도 절로 기록했다. Archive interrupted는 terminal read/copy race와 구분하며 원본을 보존했다. 이미 완료된 soak 상태 보완에는 새 snapshot이 필요하지 않고, 현재 개발 boot 기준점/재접속에서는 새 create-only snapshot이 필요하다고 명시했다. 이번 작업은 새 snapshot/Windows collector를 실행하지 않았다.

`python3 scripts/check_docs.py`: exit0, 419 documents PASS. 실제 제품·시스템·서비스·watchdog 설치·프로세스 signal/재시작은 없으며 종료 요청자는 여전히 미확정이다. 기존 Linux headless 합성/direct guardian8h 범위를 다른 OS/GUI/인증/backend 장기 PASS로 확대하지 않았다.

Root 후속 create-only snapshot `wsl-monitor-off-readiness-current-20261004/snapshot.json` 실제 exit0를 독립 JSON 대조했다. UTC2026-10-03T20:54:18.282247Z/같은 boot/uptime35100.62/Interop enabled이다. Recovery 문서에 현재 boot 비교 기준점을 추가했다. Runner 추적이 없어 개발 프로세스 live 증명으로 표현하지 않았으며 Windows eventlog 재검증·host watchdog·자동 복구도 없다.

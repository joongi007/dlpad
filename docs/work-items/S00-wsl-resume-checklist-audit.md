# S00: WSL 재접속 복구 체크리스트 읽기 감사

- 상태: completed
- 분류: documentation-audit
- 우선순위: P0
- 요구사항: R08, R10, R11
- 담당/수정 범위: stress_recovery; 이 문서 및 docs/validation/wsl-resume-checklist.md만
- 선행 작업: WSL interruption snapshot, soak ownership/archive 계약

## 사용자 결과

다음 세션이 WSL 중단과 단순 agent 재접속을 구분하고, 완료 증거를 보존하며 중단된 범위만 재개할 수 있다.

## 계약과 범위

기존 WSL collector/soak archive/scripts/workitems를 읽기만 했다. 설정·서비스·watchdog 설치, WSL restart, 프로세스 signal, 기존 evidence 수정 및 새로운 soak 실행은 없다. 체크리스트 명령은 실행할 수 있도록 실제 CLI와 검증 조건을 대조했으며 이번 감사에서 native collector를 재실행하지 않았다.

## 인수 조건

boot 변경 시 옛 PID 미조회/미신호; 같은 boot라도 PID/start 재사용을 구분한다. 8시간은 terminal/exit/DB/cleanup 자료 없이 PASS가 아니다. immutable copy/source SHA 및 부분 완료 범위를 보존하며 이전 boot 시간과 새 시간을 합산하지 않는다. /tmp 유실과 archive 지연을 증거 결손으로 구분한다.

## 검증과 handoff

실제 파일 읽기로 wsl_interruption_snapshot.py의 --output/--launch 및 create-only/64KiB/boot-before-PID 동작, soak_evidence_archive.py의 scope terminal 판정, 최신 launch artifact/ownership 경로를 대조했다. 체크리스트는 준비 완료이며 현재 boot/live/새로운 8h terminal 성공을 관측했다고 주장하지 않는다. 실행 담당자는 체크리스트의 새 이름으로 snapshot을 수집하고 원시 결과를 독립 확인해야 한다.

문서 검증: `python3 scripts/check_docs.py` 실제 exit0, 390 documents PASS. 외부 URL 및 heading anchor는 해당 checker 검증 범위 밖이다.

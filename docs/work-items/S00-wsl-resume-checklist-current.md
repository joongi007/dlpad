# S00: WSL 재개 체크리스트 현재 관측 계약 반영

- 상태: completed
- 분류: docs
- 우선순위: P0
- 요구사항: R08, R11
- 담당 파일: soak_recovery_audit — docs/validation/wsl-resume-checklist.md, docs/validation/wsl-interruption-recovery.md, 이 문서
- 의존성: S00-wsl-resume-readonly-gap-audit, 최신21:59 boot-only snapshot 및8h terminal 감사

## 범위와 인수 조건

Boot-only 기본명령, 역사 live와 최신 boot-only 구분, namespace/observation-unavailable, 완료terminal/archive race와 freshoutput실패를 실제문서에 반영한다. Rust/scripts/Cargo/설정/서비스/프로세스를 수정하거나 실행하지 않는다. 기존6tests를 반복하지 않고 docs checker로 링크를 검증한다.

## 검증과 잔여

담당 recovery/checklist에 기본 boot-only명령 및 현재ownership인자선택을 반영했다. 최신21:59boot-only와20:01historicallive/이후완료8h를 구분하며 unavailable/namespace/clock추정/부분output/collector race를 기록했다. `python3 scripts/check_docs.py`: exit0,448 documents PASS. 기존Python6tests는요청대로재실행하지않았다. Rust/scripts/Cargo/native/settings변경·실행없음. Host독립watchdog/자동알림·재개·실제reboot/모니터off연속성은보장하거나인수하지않는다.

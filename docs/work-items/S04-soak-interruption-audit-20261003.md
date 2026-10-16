# S04: WSL 종료 뒤 기존 soak 증거 감사

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R08, R10
- 담당/수정 범위: soak_recovery_audit; 이 workitem, docs/validation/soak-interruption-audit-20261003.md, 전용 evidence/soak-interruption-audit-20261003/
- 선행 작업: S04-soak-restart, S04-soak-resume-audit-20261003

## 사용자 결과

중단된 장시간 시험의 과거 running 기록을 현재 실행 또는 8시간 성공으로 해석하지 않는다. 보존된 증거와 재실행 계약을 구분해 인계한다.

## 계약과 범위

AGENTS → README → PRODUCT → CONTEXT → R04/R08/R10 → ADR-0002 순서를 읽었다. ARCHITECTURE와 기존 harness/launch 기록을 읽기 전용 조회했다. 공유 제품 소스와 기존 원시 증거를 변경하지 않는다. 새 시험 실행·프로세스 signal·WSL 시스템 변경을 수행하지 않는다.

## 인수 조건

- 현재 boot와 알려진 PID/start를 직접 조회해 이전 실행의 생존 여부를 판단한다.
- /tmp, terminal result, SQLite와 보관 파일 SHA를 확인한다.
- 미완료를 성공으로 표시하지 않으며 원시 증거를 보존한다.
- 실제 harness argv와 새 임시 폴더/바이너리 소유권 계약을 기록한다.

## 검증과 handoff

2026-10-03T11:53:36Z host collector exit0: boot 변경, 기존 9 PID와 archive PID 부재, 기존 3 /tmp 폴더 부재. 보존 8개 파일 크기/SHA 모두 일치. 4 terminal result 모두 없음. SQLite는 원본 임시 폴더와 함께 소실하여 reopen 검증 불가능. 따라서 감사 작업만 완료했고 8시간 인수 시험은 interrupted/unverified다. 실제 명령·수치·미검증 사항은 전용 validation/evidence에 기록했다. 새로운 실행이나 프로세스 종료는 없다.

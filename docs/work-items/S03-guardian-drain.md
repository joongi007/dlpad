# S03: guardian 고장 후 pipe drain 종료 상한

- 상태: done
- 분류: reliability
- 우선순위: P0
- 요구사항: R08, R10
- 담당/수정 범위: guardian_fault; crates/engine/src/process/guardian.rs, crates/engine/tests/guardian.rs, scripts/guardian_probe.py, docs/validation/guardian-drain.md
- 선행 작업: S03-guardian-fault

## 사용자 결과

Linux guardian이 죽어 escaped 자손이 출력 pipe를 유지하더라도 worker가 끝없이 기다리지 않고 실패를 보고한다. 살아 있는 자손 전체 정리 보장은 별도 미완료 조건으로 유지한다.

## 계약과 범위

AGENTS/README 문서 순서와 ADR-0002를 이미 읽었다. process.rs는 main 담당이므로 연결 변경을 제안하고 합의한다. API/schema/Cargo 변경 없이 소유 출력 pipe의 Read adapter를 검토한다. /proc 전체 검색·무관 PID 종료·cgroup 쓰기·관리자 변경·실제 desktop/data를 사용하지 않는다. Git 정상 저장소가 없으므로 기존 .git을 보존하고 파일 담당을 나눈다.

## 인수 조건

- synthetic guardian SIGKILL + inherited stdout/stderr pipe holder의 기존 join 대기를 재현한다.
- guardian 종료 뒤 drain에 상한을 두고 UI worker가 실패를 보고할 수 있다.
- metadata/log/progress drain의 기존 바이트 상한을 유지한다.
- 정상 종료·cancel·timeout·flood·metadata의 회귀 검증이 성공한다.
- 남은 fixture는 pidfd로만 종료하고 회수한다. bounded worker 종료를 orphan 보호 성공으로 기록하지 않는다.

## 검증과 handoff

[전용 검증 기록](../validation/guardian-drain.md): 변경 전 새 regression exit 101(3초 초과)로 재현했다. main 담당자의 process.rs 연결 뒤 guardian integration 8개(2초 상한), process 회귀 12개, clippy를 통과했다. wrapper 3개 단독 시험도 통과했다. UI/전체 workspace/release 통합은 main 담당자에게 전달했다. guardian SIGKILL 자손 전체 보호는 이 작업의 완료와 구별하며 P0 차단 조건으로 유지한다. process.rs는 guardian exit 이후 BrokenPipe를 무시하고 blocking drain.join을 수행하므로 Spec timeout도 더 이상 검사하지 않는다. main에 interruptible pipe와 bounded join grace를 제안했다.

# S03: Linux guardian 자체 종료·고장 검증

- 상태: partial
- 분류: reliability
- 우선순위: P0
- 요구사항: R08, R10
- 담당/수정 범위: guardian_fault; crates/engine/src/process/guardian.rs, crates/engine/tests/guardian.rs, scripts/guardian_probe.py, docs/validation/guardian-fault.md
- 선행 작업: S03-linux-crash-guardian

## 사용자 결과

Linux 작업 guardian의 일반 종료 신호에도 소유 도구·자손을 정리한다. 감시자 자체 SIGKILL은 별도로 실험하고 보호 한계를 표시한다.

## 계약과 범위

AGENTS→README→PRODUCT→CONTEXT→R08→ADR-0002 및 ARCHITECTURE를 읽었다. 기존 socket/argv/process adapter 계약을 유지하고 Cargo/schema/app/process.rs를 변경하지 않는다. 정상 Git 저장소가 없어 worktree를 만들 수 없으며 기존 .git을 보존한다. 파일 담당을 분리한다. 임시 합성 Python 프로세스와 pidfd만 사용하고 사용자 데스크톱·데이터를 사용하지 않는다.

## 인수 조건

- TERM/INT/HUP/QUIT가 guardian에 직접 전달되면 기존 leader·중첩·setsid 자손 종료와 회수를 수행한다.
- 무관한 프로세스는 종료하지 않는다.
- guardian SIGKILL의 직접 leader/escaped descendant 보호 차이를 재현하며 실패를 성공으로 기록하지 않는다.
- 새 의존성은 없으며 signal handler는 할당/락/I/O 없이 AtomicU8 종료 요청만 남긴다.

## 검증과 handoff

종료 신호 안전 개선과 guardian 통합 7 tests/clippy/release helper 검증을 완료했다. [전용 검증 기록](../validation/guardian-fault.md)에 명령·비용·실패 증거를 기록했다. release SIGTERM probe exit 0, SIGKILL probe exit 2(직접 leader 종료·자손 2개 생존)이다. 전체 P0 보호는 미완료이며 통합 담당자에게 app fallback/OS kernel scope 실험을 제안했다. SIGKILL에는 signal handler가 실행되지 않으며 Linux PDEATHSIG는 fork 자손으로 상속되지 않는다. 전체 트리를 커널 단위로 소유하는 제약이 없으면 모든 guardian 고장을 보장할 수 없다. 실제 재현 증거와 이후 결정을 검증 문서에 남긴다.

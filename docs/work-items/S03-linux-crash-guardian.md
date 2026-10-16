# S03: Linux 앱 크래시의 프로세스 트리 정리

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R13
- 담당/수정 범위: 주 담당; crates/engine/src/process.rs, process/guardian.rs, src/bin/dlpad-process-guard.rs, tests/guardian.rs, crates/app/src/main.rs, scripts/guardian_probe.py, 검증/개발/ADR 문서
- 선행 작업: argv-only process runner, bounded drains, cancellation/timeout와 Interrupted 복구

## 사용자 결과

Linux 앱을 SIGKILL해도 작업별 guardian이 제어 파이프 EOF를 감지하고 소유 도구와 자손을 정리한다. 정상 종료·취소·timeout도 같은 소유권 경로를 사용한다.

## 계약과 범위

앱 바이너리의 내부 headless 진입점은 GUI/DB 시작 전 실행된다. engine 개발 시험은 작은 전용 helper를 사용한다. guardian은 tool PGID와 subreaper의 직접 입양 자식을 정리한다. 도구 PID는 정리 전까지 reap하지 않아 재사용된 PID를 kill하지 않는다. 감시 파이프·최신 제어 값·bounded child enumeration. Cargo dependency/schema 변경 없음. Windows/macOS·sleep·8시간 soak와 전체 S03 완료는 제외.

## 인수 조건

앱 SIGKILL 뒤 중첩 자손과 setsid 자손 종료, 무관한 fixture 생존. 정상 leader 종료/비정상 코드·취소·timeout·출력 상한 회귀. helper 실패는 weak fallback 없이 오류. 실제 release 앱의 headless 경로 시험과 공통 검사.

## 검증과 handoff

Linux 합성 worker SIGKILL 뒤 중첩/setsid 자손과 guardian 회수, 무관한 프로세스 생존을 확인했다. 정상 leader 종료·그룹 변경·helper 누락과 취소/timeout/출력 상한 회귀 통과. 실제 release 앱 headless entry에서 owner SIGKILL과 자손 회수, RSS/CPU를 확인했다. fmt/clippy/워크스페이스 90 tests/release 성공, 실제 도구 합성 통합 1 test(23.79초), GUI 안전 4 tests 성공. [명령·측정과 남은 게이트](../validation/guardian-validation.md). 다른 OS/guardian 자체 고장·8시간 soak·전체 S03/P0 완료는 아니다.

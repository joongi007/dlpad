# S03: 일시 중단과 취소 구분

- 상태: implemented
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08
- 담당/수정 범위: 주 담당; engine/download.rs, app/main.rs, app/catalog.rs 및 전용 문서
- 선행 작업: S03-job-resume, 기존 core Paused 정책

## 사용자 결과

다운로드 일시 중단은 소유 프로세스 종료가 확인된 후 Paused를 저장하고 이력에서 같은 ID로 재개한다. 취소와 중단 후 종료는 기존 Cancelled 의미를 유지한다.

## 계약과 범위

ADR-0002와 UX. 새 pause intent를 cancellation보다 먼저 전달하고 실제 cancellation 결과에만 Paused를 기록한다. I/O·도구 실패는 성공적인 일시 중단으로 숨기지 않는다. partial을 삭제하거나 프로세스를 동결하지 않는다. core 상태/schema/Cargo는 변경하지 않는다. 자막 에이전트와 options.rs 담당은 겹치지 않는다.

## 인수 조건

정상 중단 ack→Paused, 취소→Cancelled, 같은 ID 재개→새 Attempt, 실패 시 Failed, partial 보존. 복구 gate/idle/중복 명령 차단. GUI는 Xvfb와 합성 fixture만 사용.

## 검증과 handoff

[검증 기록](../validation/pause-validation.md)에 실제 pause/partial/재개 회귀와 가상 GUI, fmt/clippy/137 tests/release 결과를 기록했다. 사이트별 이어받기와 종료/sleep, 모든 OS 실증을 단위 테스트로 대체하지 않는다.

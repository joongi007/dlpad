# S03: 시작 시 작업 journal 복구

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08
- 담당/수정 범위: 주 담당; crates/engine/src/download.rs, crates/app/src/main.rs, catalog.rs, scripts/native_probe.py, 검증/아키텍처 문서
- 선행 작업: Interrupted/publication recovery adapter, single-instance, Linux crash guardian

## 사용자 결과

첫 화면에서 이전 작업을 파일과 대조하는 worker를 실행하고 변경한 상태 수를 안내한다. 확인 중/실패 시 시작 차단, 실패 후 다시 확인 가능. 자동 다운로드 재시작 없음.

## 계약과 범위

기존 DB 없으면 생성하지 않는다. 최종 DB symlink/폴더 거절, 실패한 DB를 덮지 않는다. 기존 recover의 상태/완료 증거 규칙 재사용. 상태 안내와 다른 UI/작업 상태를 분리. schema/dependency 변경 없음. 작업 목록/동일 ID 재개 UI는 후속.

## 인수 조건

Downloading→Interrupted와 idempotency, 없음→쓰기 없음, DB 손상·symlink 보존, UI ready/failed 시작 gate와 오류 재시도. Xvfb에서 합성 이전 작업 DB 복구 확인. 실제 사용자 데이터 사용 금지.

## 검증과 handoff

기존 Downloading→Interrupted/idempotency·Attempt 보존, 없음/손상/symlink 파일 보존, UI 시작 gate/재시도 테스트 통과. Xvfb에서 합성 journal 복구/자동 재시작 부재와 손상 journal 보존·오류 안내/재시도 버튼을 확인했다. fmt/clippy/워크스페이스 90 tests/release 성공, 실제 도구 합성 통합 1 test(23.79초), GUI 안전 4 tests 성공. [명령·측정과 남은 게이트](../validation/guardian-validation.md). 다른 OS/guardian 자체 고장·8시간 soak·전체 S03/P0 완료는 아니다.

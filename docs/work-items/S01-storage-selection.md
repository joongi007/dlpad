# S01: 기본 저장 위치와 선택 유지

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R02, R03, R08
- 담당/수정 범위: 주 담당; crates/engine/src/location.rs, lib.rs, crates/app/src/main.rs, catalog.rs, 저장/검증 문서
- 선행 작업: OS Documents adapter, Store settings, single-instance guard, library reconciliation

## 사용자 결과

첫 시작은 저장한 위치 또는 OS Documents/dlpad를 준비한다. 사용자는 직접 입력한 위치를 적용하거나 기본 폴더로 돌아간다. 조회/권한 실패는 입력으로 복구하며 실패한 선택이 기존 설정을 덮지 않는다.

## 계약과 범위

core 상태/schema/dependency 변경 없음. app data는 OS API 기반 전용 디렉터리; fixture는 명시적 test data 또는 저장 없음. 긴 파일 작업/설정 저장은 worker. 미디어와 data 루트 겹침 및 최종 symlink 거절. 테스트는 주입한 임시 Documents provider만 사용.

## 인수 조건

기본 위치 생성/재사용·선택 유지·조회 실패/기존 file/겹침 실패·기존 선택 보존. 실제 사용자 Documents를 시험에 사용하지 않음. 공통 검사 및 가상 화면 native flow.

## 검증과 handoff

기본 위치 생성/재사용·설정 복원·조회/권한/겹침 실패 테스트 통과. Xvfb에서 임시 Documents/dlpad 생성과 생성 이후 합성 파일 20개 DB 반영 확인. Windows/macOS redirected Documents는 미검증. 공통 fmt/clippy/test/release 통과. 전체 74 tests 및 실행 명령/측정은 [통합 검증](../validation/followup-validation.md)에 기록한다. 작업 범위 완료이며 S00 최종 스택 또는 전체 P0 완료를 뜻하지 않는다.

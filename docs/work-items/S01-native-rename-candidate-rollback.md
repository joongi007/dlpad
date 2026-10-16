# S01: native 이동 후보 변경의 전체 DB 복구 회귀

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: root crates/engine/src/library.rs 전용 feature 회귀·native 검증·문서; metrics_recovery /tmp 초안만
- 선행 작업: native 후보 게시 직전 검증 통합 및 GUI60초 종료

## 사용자 결과

목록 갱신 중 이동한 파일의 크기나 시간이 바뀌거나 여러 후보 중 하나가 잘못되면 이전 DB 목록 전체를 보존한다.

## 계약과 범위

기존 after_first feature hook과 자체 tempfile을 사용한다. 원 file HANDLE의 identity가 유지되는 in-place 변경 및 서로 다른 두 rename 후보를 시험한다. SQL 후보 검사 순서를 가정하지 않으며 기존 watch/게시 정책과 schema를 변경하지 않는다.

## 인수 조건

동일ID 크기 변경·동일크기 mtime 변경 두 경우와 복수후보 중 한 파일 변경 모두 실패하고 이전 LibraryEntry 전체가 DB 재열기 뒤 동일해야 한다. 변경된 원 파일과 유효한 다른 파일은 삭제·수정하지 않는다. 실제 Windows feature 시험을 실행한다.

## 검증과 handoff

/tmp 초안 검토 완료, 실제 게시·컴파일·실행은 후속이다. 기존 GUI 성능 증거는 추가시험 소스 이전 snapshot으로 구분한다.

실제 Windows feature 목록24회귀 PASS(추가2회귀 포함), feature Clippy/all-targets PASS, native Job자연0·입력desktop불변·현재sourceSHA를 root가 검토했다. [원결과](../validation/evidence/windows-native-library-profile-regressions-20261005-continue/result.json). 전체 최신workspace/GUI는 별도 후속이며 이두회귀 범위만 완료다.

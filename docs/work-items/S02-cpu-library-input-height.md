# S02: CPU 라이브러리 경로 입력 높이

- 상태: in-progress
- 분류: regression
- 우선순위: P0
- 요구사항: R06, R08
- 담당/수정 범위: library_identity — crates/app/src/accessible_library.rs, 이 workitem
- 선행 작업: DESIGN 기본 입력 높이 32px, 기존 CPU 라이브러리 통합, GUI 종료 및 담당 파일 독점

## 사용자 결과

라이브러리 저장 경로 입력의 최소 높이를 32px로 설정한다.

## 계약과 범위

기존 단일 행 TextEdit에 min_size(vec2(0.0,32.0))만 추가한다. label 연결, 경로 검증 및 worker 계약은 그대로 사용한다. 공유 source/Cargo/Windows 빌드는 담당자가 실행하지 않는다.

## 인수 조건

담당 파일 포맷 통과 및 source freeze 전달. 통합 담당자의 공통 검증 뒤 별도 가상 화면에서 밝음/어두움과 배율 75–200%의 실제 입력 높이, label/focus, 루트 적용 동작을 확인한다. 이번 수정만으로 GUI 인수를 완료하지 않는다.

## 검증과 handoff

단일 TextEdit 최소 높이 추가 및 rustfmt --edition 2024 crates/app/src/accessible_library.rs exit0. source freeze를 root에게 전달했다. Cargo/Windows/GUI는 실행하지 않았고 통합 및 실제 GUI 인수까지 상태는 in-progress다.

# S06: 선택 인터페이스의 수신·송신 관측

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R01, R08
- 담당/수정 범위: 주 담당; crates/engine/src/metrics.rs, crates/app/src/main.rs, catalog.rs, 관련 문서
- 선행 작업: S06 bounded network parser, S02 live progress

## 사용자 결과

설정에서 관측할 인터페이스를 명시적으로 선택한다. 다운로드 B/s와 해당 인터페이스의 수신·송신을 구분하며 다른 앱 트래픽 포함을 설명한다. 미지원/초기 sample/카운터 reset은 측정 불가.

## 계약과 범위

네트워크 요청 없이 OS 카운터만 읽는다. Linux provider, 다른 OS는 None. 1초 worker sample, UI channel 1, 선택 revision으로 이전 인터페이스의 늦은 메시지 거절. 이름 검사 및 inline 오류, 빈 입력 적용으로 끄기. 설정 유지와 OS provider 확장은 후속.

## 인수 조건

유효하지 않은 이름·전환 뒤 늦은 sample·reset/unknown 검증, 실제 Linux loopback 카운터 관측. 설정은 스크롤 가능하여 확대로도 입력 접근. 통합 검사와 isolated GUI.

## 검증과 handoff

실제 Linux lo 카운터 읽기·입력 거절·선택 전환 뒤 오래된 sample·reset/unknown 테스트 통과. Xvfb 설정 화면과 개별 수신/송신 표시 확인. 설정 유지와 Windows/macOS provider는 미구현. 공통 fmt/clippy/test/release 통과. 전체 74 tests 및 실행 명령/측정은 [통합 검증](../validation/followup-validation.md)에 기록한다. 작업 범위 완료이며 S00 최종 스택 또는 전체 P0 완료를 뜻하지 않는다.

# S00: 현대적 Windows UI 및 GTMUX 대조

- 상태: completed
- 분류: audit
- 우선순위: P0
- 요구사항: R07, R08
- 담당/수정 범위: root — docs/validation/modern-desktop-design-review-20261012.md, 이 기록
- 선행 작업: 사용자의 전체 UI/UX 재평가 요청

## 사용자 결과

현재 UI가 정돈되지 않아 보이는 이유와 다음 구현 기준을 실제 참고 자료 및 제품 화면에 근거해 설명한다.

## 계약과 범위

Windows Fluent 공식 가이드와 현재 GTMUX tokens/Button 소스를 읽기 대조한다. Rust 네이티브를 유지한다. 연구 완료를 UI 변경 완료로 표시하지 않는다.

## 인수 조건

공식 출처, 현재 화면의 구체적 문제, 사용자 조작에 연결되는 수정 기준을 기록한다.

## 검증과 handoff

Fluent layout/typography/forms와 progressive disclosure 자료를 검색했다. GTMUX 원 tokens/Button 및 dlpad context loader/product reference를 직접 읽었다. 연구 문서를 작성했다. 실제 전체 화면 개편 및 사용자 인수는 미완료다.

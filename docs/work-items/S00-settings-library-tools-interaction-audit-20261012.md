# S00: 설정·목록·도구의 상호작용 잔여 감사

- 상태: completed
- 분류: UX
- 우선순위: P0
- 요구사항: R01, R07, R08, R13
- 담당/수정 범위: root — 이 기록 및 결과 통합; diagnostic_cache — /tmp 감사 보고서만
- 선행 작업: DESIGN/GTMUX 기준과 기존 scoped UI 인수 기록

## 사용자 결과

고급 옵션 외 화면에서도 hover·선택·focus·입력 커서·스크롤·disabled/오류 이유를 구체적인 사용 행동으로 검토한다.

## 계약과 범위

제품 source 읽기만 수행하며 기존 성공한 좁은 검증을 전체 완료로 확대하지 않는다. 근거 없는 디자인 결함을 주장하지 않고 source 차이, 실제 캡처, 미검증 상태를 구분한다. 수정 파일의 소유권과 실제 검증은 root가 조정한다.

## 인수 조건

미해결 P0 항목마다 사용자 행동·관련 source·GTMUX/DESIGN 기준·현재 증거와 필요한 actual 검증을 기록한다. 계획을 구현 완료로 쓰지 않고 기존 해결 항목을 중복하지 않는다.

## 검증과 handoff

감사 준비 단계다. Cargo/native/제품 코드 수정은 이 감사의 범위가 아니다.

- root가 독립 보고서의3제품source SHA를 현재 파일과 대조하고 감사 결과를 docs/validation/UI-UX-settings-library-tools-audit-20261012.md에 통합했다. 이번 완료는 읽기 감사의 범위이며 longpath keyboard 공개·selected/focus, tools busy/error combined-state 실제 인수는 남는다. Linux 실제 GUI는 Windows 최초 릴리스 범위 밖으로 보류하고 Windows NIC는 fallback 표시로 완료 처리하지 않는다.

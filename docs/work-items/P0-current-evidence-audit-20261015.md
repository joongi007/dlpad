# P0: Windows 현재 요구사항별 잔여 감사

- 상태: done (read-only 감사 범위, P0 완료 아님)
- 분류: review
- 우선순위: P0
- 요구사항: R01–R13, S00–S10 Windows-first
- 담당/수정 범위: p0_audit — .local-tools/session-recovery-20261013/p0-current-audit-20261015-v1/REPORT.md, evidence-pins.json, 이 초안만
- 선행 작업: v27 whole/source674와 기존 private 검증, Root가 검색/help/focus 후보 통합 조정

## 사용자 결과

이미 완료한 frozen 8h·v25 원인분리 진단을 재준비하지 않고 실제 부족한 구현/인수와 별도 릴리스 승인 gate를 구분한다.

## 계약과 범위

AGENTS/README→PRODUCT→CONTEXT→REQUIREMENTS/ROADMAP→관련 ADR와 work-item을 읽었다. Windows만 P0 최초 인수이며 S10 로컬 P0 포함. 다른 OS/P1 config·knownissue/P2 upload/AI는 현재 P0 필수로 확장하지 않는다. 제품, 공유schema/Cargo/lock, 기존 docs/evidence는 변경하지 않는다. Root가 초안을 docs/work-items로 채택할 수 있다.

## 인수 조건

R01~13 각각 scoped 증거와 누락을 표로 기록하고 actual raw 결과/전체 test.log를 읽는다. strict CPU·실제 사이트·CI/서명 세 다음 수직 작업을 제안한다. 실제 인증·사용자 Documents/media·시스템설정·visible desktop 접근이 없다.

## 검증과 handoff

실제 rg/read/json/hash 및 원 test.log 정규식 합산43/1593/0/42. Git status exit128 보존. Cargo/native/서버/사이트 실행0. 원data를 재실행하거나 모든 SHA를 재대조한 완료 감사가 아니며 표의 부분/미검증을 그대로 유지한다. 근거 pin은 evidence-pins.json. Root가 현재 source·최신 P0-index 갱신 및 후속 구현을 담당한다.

# S02: Windows native UI startup receipt 회수

- 상태: in-progress
- 분류: integration
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — crates/app/src/accessible.rs, 이 문서
- 선행 작업: 실제 helper receipt 제품 public recover 성공

## 사용자 결과

Windows 접근성/CPU UI는 startup recovery worker에서 public 쿠키 회수를 호출한다. deferred/rejected/truncated가 있으면 인증 pending 표시를 유지한다. 완료 receipt만 있는 원사본을 회수하고 새 다운로드를 자동재개하지 않는다.

## 계약과 범위

I/O는 기존 background worker에서 수행한다. cache missing은 pending=false, 읽기/권한 오류는 고정오류. UI catalog/schema/Cargo 변경없음. native GUI 시각 동작은 별도검증한다.

## 인수 조건

Windows native app lint/build 및 기존 startup회귀, actual GUI startup recovery는 실제실행한범위만표시한다.

## 검증과 handoff

구현전상태. helper receipt publicrecover1PASS는 UI 시각시험의대체가아니다.

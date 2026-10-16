# S00: 접기 헤더의 실제 펼침 상태 접근성 연결

- 상태: done
- 분류: fix
- 우선순위: P0
- 요구사항: R07, R08, R13
- 담당/수정 범위: root; accessible.rs, accessible_diagnostics.rs, 본 문서와 전용 검증 증거
- 선행 작업: S00-plan-header-toggle-diagnosis, S00-atspi-expanded-state

## 사용자 결과

실행 계획·진단·첨부 접기 헤더의 펼침 상태를 접근성 소비자가 읽을 수 있게 한다.

## 계약과 범위

ADR-0001/0004와 DESIGN을 따른다. 기존 egui 토글/키보드/애니메이션은 유지하고 공개 Context node builder로 현재 CollapsingState open 값을 전달한다. 애니메이션 비율이나 본문 존재로 목표 상태를 추정하지 않는다. Linux state mapping은 별도 담당 vendor patch와 통합한다. 제품·schema·Cargo 의존성 변경 없음.

## 인수 조건

실제 egui AccessKit tree의 closed→open→closed에서 expanded false/true/false. Linux 격리 AT-SPI GetState의 Expandable/Expanded와 실제 키보드/클릭 토글을 대조. Windows compile/tests 및 공통 회귀 검사. 다른 OS reader는 실증 전 완료로 주장하지 않는다.

## 검증과 handoff

기존1669 binary의 한글 시험 종료 후 새 release를 빌드한다. 사용자 desktop은 사용하지 않는다. 구현·검증 중.

## 통합 인수 결과

[실제 통합](../validation/cpu-expanded-state-validation.md): egui actualtree 및adapterstate/event 회귀PASS. Linux 실행계획 pointer/AT-SPI action/Enter/Space와진단·MPD header closed/open/closed/reopen 실제GetState를 확인했다. default200/CPU339/Windows255PASS, ignored5/5/1별도, fmt·clippy·release PASS. root보존DB·원Failedjob·소유PID부재·PNG독립확인. 다른OS reader/전배율/전체P0·스택완료와구분한다.

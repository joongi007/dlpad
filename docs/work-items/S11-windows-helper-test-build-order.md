# S11: Windows helper 인수 시험의 선행 빌드

- 상태: done (CI 설정 범위)
- 분류: ci
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — .github/workflows/ci.yml, 이 문서
- 선행 작업: 실제 독립 helper 정상/실패 native 시험

## 사용자 결과

Windows CI는 독립 helper 실행 시험 전에 현재 소스의 debug dlpad-process-guard를 명시적으로 빌드한다. 이전 빌드의 우연한 잔여 exe에 의존하지 않는다.

## 계약과 범위

Windows native job에서 lint 뒤 Tests 전에 `cargo build -p dlpad-engine --bin dlpad-process-guard --locked`를 실행한다. Cargo/schema/dependency 변경은 없다. Linux/macOS 작업 순서는 유지한다. 정상 Git/remote가 없어 Actions 실제 실행은 미검증이다.

## 인수 조건

Windows 전용 선행 step이 Tests보다 먼저 있고 현재 소스 locked build 명령이 정확해야 한다. native 로컬 동일 명령 build 후 실제 helper4tests PASS 근거와 remote 미검증을 구분한다.

## 검증과 handoff

통합 전 상태. YAML 계약 확인과 문서 검사 후 결과를 기록한다. native 로컬 선행 build 및 helper4tests는 windows-independent-helper-failures-native-first에서 exit0이며 Actions 성공을 의미하지 않는다.

Root가 실제 workflow 문자열의 Windows 조건·locked 명령·Tests 이전 순서를 assertion으로 검사했다: PASS. remote Actions와 MSVC 실행은 미검증이며 Windows GNU 로컬 통과로 대체하지 않는다. 단독 native `cargo test --workspace --locked` 실행자도 debug guard 선빌드가 필요하다. 추가 helper 빌드 시간/디스크 산출물만 발생하고 런타임 비용/의존성은 늘지 않는다.

# S00: CPU lock 변경 후 Windows 기본 기능 실제 검증

- 상태: done
- 분류: regression
- 우선순위: P0
- 요구사항: R11, S00
- 담당/수정 범위: library_identity — 신규 workitem 및 전용 validation 증거
- 선행 작업: root CPU Cargo.lock 고정 및 cpu-lock-review.json

## 사용자 결과

CPU 후보 의존성 추가로 기존 Windows 기본 기능이 깨지지 않았는지 실제 native GNU 환경에서 확인한다.

## 계약과 범위

기존 scripts/windows_native_probe.py를 읽기 전용으로 실행한다. Cargo/lock/제품/runner는 수정하지 않는다. Windows 전용 Cargo home/target/소유 temp를 사용하고 private CLI desktop 및 bounded owned Job을 유지한다. CPU feature Windows GUI, MSVC, macOS는 이 범위가 아니다.

## 인수 조건

fmt/clippy/test/release가 실제 실행되고 각 source/vendor before-after 입력이 같아야 한다. 네 작업 사이 동일 입력도 확인한다. binary SHA/크기를 기록한다. source 변경 시 성공을 새 입력으로 소급하지 않는다.

## 검증과 handoff

실제 네 검사 exit0, source235개 모든 before/after/작업간 동일, 116passed/1ignored. docs/validation/windows-cpu-lock-default.md 및 전용 원시 증거에 기록. Windows 의존 edge0.60.2→0.61.2의 기본 기능 검증이며 CPU Windows 접근성/GUI 완료를 주장하지 않는다.

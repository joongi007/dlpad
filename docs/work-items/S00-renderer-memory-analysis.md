# S00: 접근성을 유지하는 renderer 메모리 원인 조사

- 상태: done
- 분류: investigation
- 우선순위: P0
- 요구사항: R01, R11
- 담당/수정 범위: library_identity — 전용 workitem/validation만
- 선행 작업: dlpad-accessible Glow/AccessKit private Xvfb RSS196156KiB 관측

## 사용자 결과

80MiB idle 목표를 완화하거나 접근성 기능을 제외하지 않고, 초과 비용의 원인과 검증 가능한 renderer 선택지를 정리한다.

## 계약과 범위

기존 ADR/성능 요구/후보 소스/로컬 dependency source/측정 자료를 읽는다. 외부 사실은 primary docs로 확인한다. 제품/Cargo/lock/기존 harness/renderer 구현은 변경하지 않는다. 읽기 전용 근거에 없는 원인을 확정으로 표현하지 않는다. GUI 시험은 기존 private Xvfb 증거만 사용하고 새 실험은 주 담당에게 scope/인수와 함께 제안한다.

## 인수 조건

- 실제 관측의 binary/환경/dataset와 측정 의미를 구분한다.
- Glow/glutin/Mesa/폰트/fixture/AccessKit 비용 후보를 코드 근거와 분해 실험으로 연결한다.
- Rust native+IME+실제 접근성 유지 조건을 포함해 80MiB 도달 선택지를 비교한다.
- renderer/library 전환의 크기/runtime/구현비용과 OS 검증 게이트를 기록한다.

## 검증과 handoff

진행 중. .git 보존. 읽기 전용 조사로 미측정 선택지를 완료 주장하지 않는다.

조사 결과: docs/validation/renderer-memory-analysis.md 및 software-renderer-pin.json. CPU 구현/실측은 S00-cpu-accessible-host로 분리하며 실제 예산 통과는 아직 주장하지 않는다.

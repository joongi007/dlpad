# S00: egui0.33 호환 CPU renderer 고정 후보 조사

- 상태: done
- 분류: investigation
- 우선순위: P0
- 요구사항: R01, R11
- 담당/수정 범위: library_identity — 전용 validation/pin/source evidence/workitem
- 선행 작업: renderer 메모리 조사, root same-binary Mesa 3종 budget FAIL

## 사용자 결과

접근성·IME와 기존 egui0.33.3을 유지하는 CPU renderer 후보의 정확 release/checksum/source/license/의존성·통합 위험을 제시한다.

## 계약과 범위

공식 crates.io metadata에서 egui_software_backend0.0.2의 ^0.33 호환이 확인됐다. 소량의 공개 고정 source archive를 프로젝트 전용 새 폴더에 bounded 다운로드하고 필요한 소스만 읽는다. 제품/Cargo/lock/기존source를 수정하거나 wholeegui0.34 upgrade하지 않는다. 실제CPU GUI spike는 root 공유 Cargo 합의 후 후속이다.

## 인수 조건

- 고정 release/APIchecksum/다운로드 실제bytes/license/features/새deps 확인.
- renderer API와 접근성/IME 연결에 필요한 정확 통합 범위를 제안한다.
- unmeasured CPU/RSS/3OS는 완료로 표현하지 않는다.

## 검증과 handoff

진행 중. 예산완화나 기능제외를 해결책으로 사용하지 않는다.

조사 결과: docs/validation/renderer-memory-analysis.md 및 software-renderer-pin.json. CPU 구현/실측은 S00-cpu-accessible-host로 분리하며 실제 예산 통과는 아직 주장하지 않는다.

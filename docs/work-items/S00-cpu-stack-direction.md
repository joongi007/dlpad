# S00: CPU 네이티브 후보 통합 방향

- 상태: done
- 분류: decision
- 우선순위: P0
- 요구사항: R06, R08, R10, S00
- 담당/수정 범위: 주 담당; 본 문서, docs/STACK.md, docs/adr/0001-native-ui.md
- 선행 작업: CPU host 구현 및 hidden-init Linux 통합 검증

## 사용자 결과

접근성과 메모리 예산을 함께 충족하는 CPU 후보에 기존 제품 기능을 순차 연결한다. 기본 제품을 교체한 릴리스나 전체 S00 완료로 표현하지 않는다.

## 계약과 범위

egui0.33.3/AccessKit/직접 egui-winit/softbuffer/egui_software_backend0.0.2 후보를 기능 통합 방향으로 사용한다. Rust native 제약과 기존 engine/core 계약을 유지한다. Cargo/schema/기본 entry 변경 없음.

## 인수 조건

실측과 실제 입력·reader 증거에 근거를 연결하고 최종 채택과 미검증 게이트를 분리한다. 추가 crate 용량·런타임 비용을 기록한다.

## 검증과 handoff

현재 hidden-init CPU workspace206 passed/0 failed/5 ignored, entry10 tests, fmt/clippy/release 성공. Linux 새binary RSS50624KiB/idle0%10.099초는 root 직접 측정이며 모든 기능 비용이 아니다. 이전 CPU28b4 snapshot의 actual 한글·무음Orca·4개 다운로드 GUI 증거는 각각 해당hash 범위다. Windows 새binary CLI131tests는 root235입력 독립대조; 실제GUI 결과는 별도 수집 중이다. macOS/MSVC·전체reader 사용성·전체UI parity·목록10000/8시간·설치 게이트는 남는다.

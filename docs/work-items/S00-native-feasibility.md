# S00: Rust 네이티브 UI의 비용과 OS 동작 검증

- 상태: ready
- 분류: feature
- 우선순위: P0
- 요구사항: R06, R08, R10, R11
- 담당/수정 범위: 미배정; workspace, native UI spike, toolchain, native CI, ADR-0001
- 선행 작업: 현재 설계 문서

## 사용자 결과

gtmux 스타일의 목록·설정 화면을 Rust 네이티브로 보여주면서 가벼움과 한글 입력을 실제 확인한다. 이 spike는 다운로드 앱 완성본이 아니다.

## 인수 조건

1. Iced tiny-skia와 wgpu를 같은 10,000행 목록으로 비교하고 배포 크기·idle/scroll/background CPU·메모리를 기록한다.
2. Light/Dark/System, 한글 IME, 복사/붙여넣기, 200% 배율, keyboard focus, screen reader를 OS별 확인한다.
3. 창 해제/재생성, tray 없는 Linux 복귀, system font fallback을 확인한다.
4. 정확한 Rust toolchain·Cargo.lock·필요 OS 패키지를 추가하고 3 OS native CI를 실제 실행한다.
5. 실패 시 원인을 기록하고 대안 GUI 1개와 비교한 뒤 ADR-0001을 수정한다.

## 검증과 handoff

Linux 네이티브 창·renderer·메모리/CPU 측정을 수행했다. [측정 기록](../validation/S00.md)과 [현재 상태](../validation/P0-status.md)를 따른다. Rust 1.94.0을 고정했으며 중단 이후 프로젝트 전용 개발 도구를 복구했다. 한글 IME·접근성·3 OS 실행 증거가 없어 최종 GUI 선택은 Proposed다.

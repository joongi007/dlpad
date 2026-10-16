# S03: 작업 유지 숨김과 명시적 창 복귀

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: 주 담당; app/main.rs·catalog.rs, engine/instance.rs, CONTEXT/ADR/상태 문서. diagnostic_cache는 GUI probe와 전용 검증 문서만 담당한다.
- 선행 작업: Linux activation IPC와 instance guard

## 사용자 결과

진행 중 닫기에서 백그라운드 계속을 선택하면 동일 작업을 유지하며 창을 숨긴다. 명시적으로 같은 앱을 다시 실행하면 기존 창으로 복귀한다.

## 계약과 범위

복귀 endpoint가 준비된 환경에서만 숨김을 허용한다. 숨김 이후 완료가 창을 자동 표시하거나 포커스를 요구하지 않는다. 같은 instance lock을 실행 내내 유지하며 secondary는 작업을 만들지 않는다. Windows/macOS 복귀와 트레이는 미지원으로 표시하고 숨김을 비활성화한다. 새 앱 전용 data directory만 private permission으로 생성하고 기존 폴더 권한은 변경하지 않는다.

## 인수 조건

가상 화면에서 숨김/복귀·동일 worker 유지·중복 작업 없음·돌아가기·중단 후 종료를 실증한다. 복귀 불가/지원되지 않은 환경에서는 visible window를 유지한다. UI에서 파일 I/O 없이 atomic request flag만 poll한다.

## 검증과 handoff

Linux 구현 및 해당 snapshot의 통합 검증 완료. 아래 최종 기록의 binary hash 범위에 한정한다.

최종 주 담당 통합검증: fmt/clippy/workspace tests/release 성공, 실제 합성미디어1/1(36.67초), Xvfb 숨김·복귀/완료후숨김/처리표시 성공. [최종 통합 기록](../validation/background-processing-integration.md)을 따른다. Windows/macOS 복귀·전용처리transport와 전체P0는 이 완료 범위에 포함하지 않는다.

## 최신 요청 범위 재확인

2026-10-02 사용자 요청은 통합 대기와 백그라운드 숨김·복귀 UI다. 위 Windows 미지원 표현은 최초 Linux 작업 당시 범위다. 이후 Windows named-pipe 복귀 backend를 연결하고 native GNU CLI 검증을 통과했으나, 초기 격리 GUI의 active 입력 harness는 실패했으나, 이벤트 순서 관측 후 harness의 입력 간격을 수정했다. 이후 일반 실행 인자로 격리 GUI의 합성 작업 중 숨김·동일 worker 유지·명시적 재실행 복귀·중단 후 종료를 3회 확인했다. 실제 영상·IME·로그아웃·절전 시험은 포함하지 않는다. [Windows 격리 GUI 결과](../validation/windows-background-gui.md)를 따른다. macOS 복귀는 여전히 미지원이다.

# S04: CPU 관리 도구 패널

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R06, R08, R11
- 담당/수정 범위: library_identity — 신규 accessible_tools.rs와 전용 문서
- 선행 작업: 기존 ToolManager, Accepted ADR-0003 소유권 분리, root managed candidate/tool_error 공유

## 사용자 결과

CPU UI에서 관리 yt-dlp의 설치·목록 갱신·활성 선택·보관 버전 제거·검증 후 다운로드 적용·설치 취소를 제공한다.

## 계약과 범위

단일 worker 및 명령/결과 channel capacity1, 설치 목록 엔진 상한128. 모든 filesystem/network/digest는 worker, UI는 catalog와 snapshot만 사용한다. 외부 파일 제거는 엔진 소유권 검사로 거절한다. request_stop은 새 명령을 막고 설치 취소 요청 후 진행 호출 종료를 기다려 stop ack한다. 단일 filesystem/SQLite 호출 즉시 중단은 보장하지 않는다. root는 entry/Cargo/shared wiring을 담당한다.

## 인수 조건

실패시 마지막 inventory 유지·active 없는 Use 거절·외부 manifest 제거 보존·worker panic stop ack·종료 후 명령 차단을 temporary fixture로 검증한다. 실제 설치 다운로드/취소 및 GUI/3OS는 별도 통합 인수다.

## 검증과 handoff

담당 모듈 작성 후 own rustfmt/source freeze를 전달한다. 컴파일 전 테스트 작성과 실제 실행 결과를 구분한다.

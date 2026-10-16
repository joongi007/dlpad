# S00: 사용자 데스크톱과 분리한 GUI 시험

- 상태: done
- 분류: maintenance
- 우선순위: P0
- 요구사항: R08, R10, R11
- 담당/수정 범위: 주 담당; scripts/native_probe.py, scripts/test_native_probe.py, AGENTS.md, .github/workflows/ci.yml, README.md, docs/validation/S00.md
- 선행 작업: native probe

## 사용자 결과

기본 GUI 시험은 가상 X 화면에서 실행하여 사용자 창의 포커스를 가져가지 않는다.

## 계약과 범위

Xvfb를 프로젝트 전용 도구로 설치하며 관리자/시스템 변경 없음. 가상 화면 자동 할당, TCP 비활성화, 소유 프로세스 종료/reap. Xvfb가 없으면 기본 시험은 실패하며 사용자 화면으로 fallback하지 않는다. 실제 desktop 시험은 명시적 --desktop 옵션만 허용.

## 인수 조건

가상 화면에서 screenshot/RSS/CPU 측정과 프로세스 회수. 사용자 DISPLAY 불변. 가상 화면은 실제 WSLg renderer·IME·접근성 검증을 대체하지 않는다.

## 검증과 handoff

Xvfb 목록·설정·진단 창 3회 실행/스크린샷 성공; 프로세스 모두 종료/reap. python3 scripts/test_native_probe.py: 2 passed. 최초 Xvfb 시작은 WSLg socket/누락 xkbcomp로 실패했으며 도구 전용 추출과 프로세스 내부 경로 연결로 수정했다. TCP/pathname X socket listen 비활성화, Linux abstract socket 사용. 시스템 파일 변경 없음. 사용자 요청 후 실제 desktop probe 재실행 없음.

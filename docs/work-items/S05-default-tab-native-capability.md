# S05: 기본 Iced Tab 두 입력 native capability

- 상태: implemented; native 검증 대기
- 분류: validation
- 우선순위: P0
- 요구사항: R01, R07
- 담당/수정 범위: soak_recovery_audit — scripts/default_tab_navigation_probe.py, 이 문서
- 선행 작업: S05-default-tab-instrumentation-audit, immutable default SHA 94e62f13b940c96cea379af8507458859917fd5a9156d6f070c5ca26c0559856

## 계약과 인수

private Xvfb·private bus·0700 합성 fixture에서 첫 URL 클릭 후 x/Tab/y/BackSpace/ShiftTab만 보내고 단계별 PNG와 event 통계를 수집한다. 기존 960×640 screenshot의 URL 내부 좌표 (200,160)를 사용하며 실제 초기 PNG는 root가 검토한다. CPU AT-SPI name·GrabFocus·clipboard·OCR로 입력 값을 추정하지 않는다. 측정 수집 성공과 keyboard navigation 인수는 구분한다. no job/tool·소유 회수·전후 hash가 필수이다.

## 검증과 handoff

GUI 실행은 root 담당이다. 실제 포커스·caret·입력 결과의 screenshot 검토 전 `keyboard_navigation_verified=false`, `input_state_review=REVIEW_REQUIRED`를 유지한다. 전체 앱 탐색이나 다른 OS 인수는 범위 밖이다.

Python `compile(..., 'exec')` 실제 exit 0. 수집기 동결 SHA256은 `23e99f33cc0931a1a5cd088b2c5f1c1eb6c9fd335712e285937c210feb4938c0`이다. 실행 진입점은 `python3 scripts/default_tab_navigation_probe.py --binary <immutable-default> --output <fresh-evidence>`이다. 측정 수집 성공은 키 전달과 원시 screenshot·소유 회수·미실행 검증 완료를 뜻하며 키보드 이동 성공을 뜻하지 않는다.

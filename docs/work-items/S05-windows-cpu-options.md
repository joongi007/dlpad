# S05: Windows CPU typed 옵션 통합 CLI 검증

- 상태: done
- 분류: regression
- 우선순위: P0
- 요구사항: R01, R06, R07, R08, R11
- 담당/수정 범위: library_identity — 이 workitem, docs/validation/windows-cpu-options/ 증거와 전용 요약
- 선행 작업: root typed DownloadForm/Quality validation/입력32px/docked actions/scroll 통합 및 source freeze

## 사용자 결과

통합 옵션 패널 소스가 실제 Windows GNU CPU feature에서 lint·시험·release되는지 확인한다.

## 계약과 범위

기존 runner 읽기전용 사용, Windows 전용 target/private CLI/owned Job에서만 실행. 제품/Cargo/runner/GUI/clipboard 변경 없음. 입력 manifest와 runner/lock before-after·검사간 동일성 및 새 exe SHA 보존.

## 인수 조건

clippy/test/release actual exit0 및 실제 시험 개수 기록. 소스 변동시 성공 소급 금지. 옵션 GUI·IME·실제32px은 별도 GUI 인수이며 CLI 결과로 대체하지 않는다.

## 검증과 handoff

실제 clippy/test/release exit0, 200 passed/1 ignored. 239입력 before/after·검사간·현재동일, runner/lock 불변을 확인하고 새 CPU 바이너리 READY를 root에 전달했다. GUI/clipboard/MSVC/macOS는 미실행이다.

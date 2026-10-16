# S02: Windows CPU 공통 스타일 높이 수정 CLI 검증

- 상태: done
- 우선순위: P0
- 요구사항: R06, R08, R11
- 담당: library_identity
- 수정 파일: 이 workitem 및 docs/validation/windows-cpu-appearance-sizing/ 전용 증거와 요약
- 의존성: root accessible.rs configure_context all_styles_mut spacing32 수정 및 소스 동결

## 범위와 인수 조건

기존 Windows GNU runner로 CPU clippy/test/release를 private CLI에서 실행한다. 제품/Cargo/runner 변경과 GUI/clipboard 접근 없이 입력·runner·lock before/after, 검사간 동일성과 새 바이너리 SHA를 보존한다. 실제 32px 높이·테마·IME 인수는 별도 GUI 담당자의 범위이며 CLI 통과로 주장하지 않는다. 이전 popupfix 증거는 유지한다.

## 실제 결과

CPU clippy/test/release exit0, 188 passed/1 ignored. 238입력 및 runner/lock 동일성, 현재소스 대조와 새 binary SHA를 전용 증거에 기록하고 root에 READY를 전달했다. GUI 높이·테마·IME는 미검증 범위다.

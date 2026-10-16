# S02: Windows CPU 비포인터 메뉴 닫힘 수정 CLI 검증

- 상태: done
- 우선순위: P0
- 요구사항: R06, R11, R13
- 담당: library_identity
- 수정 파일: 이 workitem과 docs/validation/windows-cpu-appearance-popupfix/ 전용 증거
- 의존성: accessible_settings.rs 두 메뉴 ui.close 수정 및 root 소스 동결

## 범위와 인수 조건

기존 Windows GNU runner를 수정하지 않고 CPU clippy/test/release를 private CLI에서 수행한다. 입력 manifest와 runner/lock SHA의 before/after 및 작업간 동일성을 확인하고 새 CPU binary SHA를 보존한다. GUI와 clipboard는 실행하지 않는다. 이전 188시험 성공은 이전 source 증거로 보존하며 새 GUI 인수를 CLI 성공으로 대체하지 않는다. 실패 시 원시 로그를 유지하고 제품/Cargo 수정은 root가 조정한다.

## 실제 결과

Windows GNU CPU clippy/test/release가 모두 exit0이며 188 passed/1 ignored다. 238개 입력의 작업 전후·작업간·현재 대조가 동일하고 runner/lock SHA도 유지됐다. 새 바이너리 READY를 root에게 전달했다. GUI/clipboard 및 macOS/MSVC는 이번 검증 범위가 아니다.

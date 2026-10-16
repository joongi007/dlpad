# S00: Windows Clippy의 test module 이후 항목 정리

- 상태: completed (Windows 함수 배치 및 CLI 검증 범위)
- 분류: fix
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root; process.rs, README.md, docs/validation/P0-status.md 및 이 문서
- 선행 작업: Windows 현재source fmt0/Clippy101 원시 결과

## 사용자 결과

같은 cleanup tracing 코드를 Windows에서 경고 없이 빌드한다.

## 계약과 범위

cfg에따라Windows에서마지막testmodule뒤cleanup_stage_error가남아Clippy가거절했다. 함수몸체/정책을변경하지않고testmodule앞으로이동한다. 경고억제/schema변경없음.

## 인수 조건

현재WindowsClippy및기본/CPU회귀를다시실행한다. 원시실패보존.

## 검증과 handoff

수정중. Linux이전7명령통과는이전layoutsource범위다.

Second Windows 실제fmt/clippy/test/release 모두exit0/sourcebeforeafter+root현재SHA일치/inputdesktopunchanged true. root-review에각harness개수와artifact증거를보존했다. 함수내용은바꾸지않았다. Linux동일layout7검증은다음수행하며WindowsGUI/IME/clipboard/macOS/실제설치는미검증이다.

후속 Linux 실제 검증: cookie-keeper-options-layout-linux/result.json에서 기본/CPU 전체7명령 exit0. Root가 source-before/source-after 동일 및 기록된 모든 입력의 현재SHA 일치를 독립 확인했다. 동일layout Linux 검증 대기는 해소됐다.

# S03: Windows reader 종료 확인과 유한 소유 프로세스 대기

- 상태: in-progress
- 분류: fix
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: metrics_recovery — crates/engine/src/process/windows.rs 및 이 문서; root — process.rs 공통 실행 흐름과 전체 native 검증
- 선행 작업: Windows cookie backend; 원자적 Job Object spawn

## 사용자 결과

Windows 인증 실행에서 root 종료만으로 모든 reader가 종료했다고 판단하지 않는다. Job의 ActiveProcesses=0을 유한 시간에 확인하고, 실패하면 쿠키를 보존한다. 취소·Drop의 무한 Windows wait도 제거한다.

## 계약과 범위

Ownership::confirm_terminal()은 보유 Job handle의 accounting ActiveProcesses=0만 확인한다. Query 실패/기한 초과는 typed OwnershipCleanupError::TerminalUnconfirmed이다. Child.wait과Drop의INFINITE대기를유한timeout으로바꾸며ownedhandle만사용한다. timeout후강제종료의실제확인없이정상확인으로변환하지않는다. root가capturer 반환분류/pipe유한drain과연결하며그전Windows쿠키UI는Unavailable유지한다. 기존 argv/nullstdin/원자적 spawn/kill-on-closeJob계약을보존한다. 새 Cargo/schema/thread 없음.

## 인수 조건

실제Windows native compile/clippy/test,정상root와자손종료/취소/timeout/Query실패경계,살아있는reader에는confirm없음. Drop에INFINITE없음. 원본·합성무관process보존,쿠키unknown보존. 제품GUI·다른OS의검증과구분한다.

## 검증과 handoff

root 읽기에서 Child.wait 및 Child.Drop의INFINITE와 Jobterminal미확인을확인했다. 신규backend연결만으로 인증UI를활성화하지않는다. 모듈 sourcefreeze 후root가통합한다.

## Windows module 구현 handoff

confirm_terminal은 보유 JobObjectBasicAccountingInformation의 ActiveProcesses=0을 확인한다. Query 실패 또는2초 절대상한 초과는 typed TerminalUnconfirmed이며20ms 이하 sleep으로 polling한다. root process 반환 흐름에서 이 확인을 호출하기 전에는 Windows 인증 UI를 활성화하지 않는다.

Child.wait는 owned process handle의 WaitForSingleObject를2초 상한으로 바꿨다. wait/status 오류는 typed 미확인으로 반환한다. Drop은0ms 상태 확인 후 아직 살아 있는 owned handle만 TerminateProcess하며 성공 시에도 최대2초만 기다린다. Drop은 확인 결과를 호출자에게 전달할 수 없으므로 cookie 삭제의 근거가 아니다. Job kill-on-close를 보존하며 PID 조회·추측 신호·새 스레드나 Cargo 의존성을 추가하지 않는다.

Windows 전용 내부 시험은 empty Job 종료 확인, 실제 suspended child의 Job 미종료30ms timeout typed 확인 후 TerminateJobObject와 실제 terminal 확인, 유효한 non-Job event handle Query 실패 typed 확인을 추가했다. 합성 current test executable만 사용하고 사용자 desktop/data는 접근하지 않는다. rustfmt만 실행했으며 native compile/clippy/test는 root 통합 대기다. 이 문서는 실제 Windows 실행 PASS를 아직 주장하지 않는다.

## root 공통 실행 연결

Windows spawn실패도typedStartupUnconfirmed로분류하고, 정상root·pipe회수후Job accounting확인을호출했다. ready이후모든반환오류는terminal확인여부로분류해미확인일때쿠키readerconfirm을막는다. Windows pipe drain의별도무한경계는다음보강대상이며그전authUI를활성화하지않는다. nativecompile/runtime검증전이다.

## root actual Windows Job 시험

같은 [실제 Windows 로그](../validation/evidence/windows-cookie-runtime-first/test.log)에서emptyJob확인,nonJob Query오류typed,live suspendedJob timeouttyped3개PASS를직접대조했다. 기존Windowsprocess integration6개도PASS이며engine library94PASS/1ignored였다. 전체명령은뒤cookiefixturehardlink준비오류로exit101이므로workspacePASS로표현하지않는다. privateDesktop/inputdesktopunchanged를확인했고WindowsauthGUI는미활성이다. pipe유한drain과추가실패인수는남는다.

## root 동결 Windows 재검증

[전체 actual Windows 결과](../validation/evidence/windows-cookie-runtime-stable/root-review.json) session3271 exit0/19.84초, before/after/current271입력SHA일치. cookiefixture4PASS와Jobterminal3PASS를확인했다. 중간실행은fixture최종수정으로wrapperexit3이며 [원시관측](../validation/evidence/windows-cookie-runtime-retry-observed/test-inputs.json)을보존했다. [명시nocapture결과](../validation/evidence/windows-cookie-runtime-nocapture/root-review.json) exit0/fixture4PASS에서hardlink생성은OS32공유거절이며link검출증명이아니다. 뒤broadenedACLvalidate/cleanup거절분기는실제실행했다. 실제productauthGUI,writer실패주입,Windowspipe유한drain,재시작회수는남은인수다.

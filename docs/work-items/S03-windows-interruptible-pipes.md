# S03: Windows stdout/stderr 유한 drain

- 상태: in-progress
- 분류: fix
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: metrics_recovery — crates/engine/src/process/windows.rs 및 이 문서; root — process.rs stop/drain 연결, 통합 검증
- 선행 작업: Windows Job terminal 확인

## 사용자 결과

Job 종료 확인을 잃거나 writer가 pipe를 계속 보유해도 worker가 stdout/stderr join에서 무한히 기다리지 않는다. 읽기 중단은 자손 종료 확인으로 취급하지 않고 쿠키를 보존한다.

## 계약과 범위

Windows InterruptiblePipe<T:Read+AsRawHandle>는 std ownedpipe를독점읽는다. PeekNamedPipe로가용byte만읽고,20ms마다boundedstopflag를확인한다. ERROR_BROKEN_PIPE는EOF,다른오류는반환한다. stop은aborted=true와고정TimedOut오류를전달한다. 공유reader나부족byte를기다리는Read를추가하지않는다. new(pipe,stop,aborted)->ioResult<Self>, Read구현을root가Linux와동일stop계약으로연결한다. 새dependency/thread/schema없고기존drainthread를사용한다.

## 인수 조건

actualWindows anonymouspipe의literalbytes+EOF,writeralive/bytes0시stop으로유한반환,잘못된handle오류를실제시험한다. root는제품capturer에연결후cancel/timeout/원status/outputlimit/쿠키unknown보존을native검증한다. pipe중단을정리완료로표시하지않고사용자desktop은사용하지않는다.

## 검증과 handoff

Job종료/Child대기는유한화했지만일반File.read와thread.join은현재별도무한경계다. helper sourcefreeze 후root연결·검증하며그전WindowsauthUI는활성화하지않는다.

## 구현 동결 handoff

windows.rs에 InterruptiblePipe<T:Read+AsRawHandle>를 추가했다. new는 owned exclusive reader handle을 PeekNamedPipe로 검증한다. Read는 available byte count 이하만 기존 reader로 읽는다. empty live writer는20ms sleep으로 stop을 재확인하며 stop은 aborted flag와 고정 TimedOut 오류를 반환한다. broken pipe는EOF, 기타 Win32 오류는 원래 오류를 반환한다. 빈 read buffer는 즉시0이다. 공유 reader를 지원하지 않는 계약이므로 Peek와Read 사이 다른 reader가 byte를 소비하지 않는다.

native 내부 회귀3개는 actual anonymous pipe literal NUL/UTF-8 byte와 writer close EOF, writer 살아 있고 byte0인 상태에서30ms 후stop/1초 이내반환/aborted, valid nonpipe event handle의Peek 오류와 실제Read미호출을 확인한다. 시험용 stopper thread만 추가했고 제품은 기존 drain thread를 재사용한다. Job terminal/Child bounded wait 코드는 보존했다.

source SHA256 `5493b12507d893876ce1c8fc4f2938d50d0386d5d7e2558d9285a4365a445f89`, rustfmt exit0. Cargo/native 시험은 아직 미실행이며 root가 공유 process.rs 연결 후 수행한다. prior Windows271 input freeze PASS는 이번 helper 검증 근거가 아니다. 중단/EOF는 reader descendants 종료 증명이 아니며 root의 Job ActiveProcesses=0과 typed unknown 보존이 별도 필요하다.

## root 실행 흐름 연결

Windows도StopDrains의unwind stop과최대1초drain관측뒤중단을적용했다. Windowsstdout/stderr를각각독점InterruptiblePipe로감싸고aborted시typedOutputDrainInterrupted를반환하여cookieguardconfirm을막는다. 정상drain뒤Jobterminal확인은유지한다. 원status/cancel/outputlimit을그대로보존해야하며actualnative검증전이다.

## root 실제 Windows 통합

[actualWindows원시대조](../validation/evidence/windows-interruptible-pipes-first/root-review.json) session35562 exit0/27.72초. pipebytes+EOF/livewriterstop/nonpipe오류3개의실제PASS를로그에서대조했고Job/cookie/process회귀도통과했다. 입력before/after/current일치,privateDesktop/inputdesktopunchanged. Linux전체/WindowsClippy·release/실제authGUI는후속검증이다.

## Linux 교차 회귀

공통capturer의WindowsJob/pipe연결후`cargo test -p dlpad-engine --locked` root session43688 exit0. [명령·검증소스](../validation/evidence/cross-platform-cookie-linux-regression/result.json). 기존Linux회귀를확인한범위이며전체workspace/release/새Windowssyntheticexample/GUI완료가아니다.

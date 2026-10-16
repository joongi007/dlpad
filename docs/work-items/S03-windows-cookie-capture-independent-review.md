# S03: Windows authenticated capture 독립 검토

- 상태: completed (읽기 검토; 후속 회귀 필요)
- 분류: validation
- 우선순위: P0
- 요구사항: R02, R08
- 담당/수정 범위: soak_recovery_audit — 이 문서만
- 선행 작업: root 실제 Windows product 네 mode PASS, capture/reader gate/public recovery 통합

## 실제 코드 확인

Prepared는 kind/nonce/slot 및 원 helper process handle creation time을 검증한다. reader bind는 original Intent와 현재 journal 전체 encode를 비교한 뒤 durable Bound를 만들며 release 이후만 app Ack를 보낸다. helper Ack의 exact digest 후 Begin을 전송한다. running 종료는 Terminal exact slot/nonce/digest, 중복 Terminal 거절, 원 events EOF, original helper process status/code 일치, original journal reacquire/Bound/receipt resolve를 요구한다. Drop은 reader pending을 해제하거나 receipt를 조작하지 않는다.

stdout/stderr는 별도 evidence budget과 최대 line/metadata drain을 재사용한다. 두 drain은 stdout metadata가 stderr flood에 밀리지 않도록 분리되어 있으며 종료 뒤 1초 관측 후 stop flag, 두 join을 수행한다. 오류 경로도 join을 생략하지 않는다. PeekNamedPipe 기반 독점 reader는 stop을 확인하므로 일반 빈 pipe read를 무기한 기다리지 않는다. thread::spawn 실패의 panic 같은 시스템 자원 실패는 io::Result로 처리하지 않으며 별도 fault injection 인수는 없다.

capture deadline은 app 시작부터 absolute이고 config/gate/Begin send도 같은 ExecutionControl을 사용한다. running 취소·outputlimit·timeout은 fresh false cancel flag와 새 2초 budget으로 Cancel/TimedOut을 보낸다. protocol 실패·startup 취소는 TerminalUnconfirmed와 pending 보존으로 귀결되며 정상 Cancelled outcome과 구분된다. 오류 경로는 추가 2초 helper 관측과 original Child Drop 정리, 1초 drain 관측이 있어 전체 wall-clock가 원 spec timeout 또는 2초 하나로 제한된다고 주장하면 안 된다.

download adapter는 Windows cookie에서 실제 gated process API를 사용하고 authenticated returned logs를 omission 상수로 바꾼다. free metadata는 typed duration만 공개하며 JSON/IO errors도 고정 omission 오류로 변환한다. 내부 capture의 raw metadata/log buffer는 bounded이나 호출자가 download adapter를 거치지 않고 내부 process result를 노출하면 이 omission 정책을 우회한다. 이 API는 crate-private이므로 현재 제품 public download 결과 경계가 실제 privacy 인수 대상이다.

Windows public recovery는 marker/native ACL·IDs·strict namespace/journal lock/actual receipt를 다시 읽는다. helper completion을 얻지 못한 Intent/Bound는 추측 정리하지 않으며 private copy를 보존한다. 숫자 helper PID나 path absence를 receipt 대신 사용하지 않는다.

## 구체적인 코드 순서 문제와 필요한 회귀

running loop는 Terminal frame을 읽어 `terminal=Some(code)`로 저장한 뒤 다음 iteration에도 oversized/cancel/deadline를 다시 평가하고 Cancel/TimedOut을 전송한다. helper가 이미 Terminal을 기록하고 종료하면 late command write가 BrokenPipe로 실패하여 유효 terminal/원 receipt가 있어도 전체 result가 TerminalUnconfirmed로 바뀐다. 이는 코드에서 가능한 순서가 확인된 문제이며 실제 native 재현은 아직 하지 않았다. 보존 방향이라 무단 삭제가 생기는 문제는 아니지만 정상 완료가 deferred로 바뀔 수 있다. root에 전달했다.

최소 회귀는 Terminal 수신 뒤 EOF 대기 구간에서 cancel 또는 마지막 drain의 oversized flag가 켜지는 실제 helper fixture이다. Terminal 관측 이후 새 command를 보내지 않고 original EOF/status/receipt 엄격 검증을 계속하는 수정이 후보이며, proof/deadline를 완화하지 않는다. 정상 결과와 late cancellation 정책은 root가 결정해야 한다.

추가 실제 fixture가 필요한 경계는 helper Ack 전 startup cancellation(안전 보존·tool side effect 0), partial/malformed/duplicate event 뒤 두 drain 회수, helper kill 시 pending+source/sentinel 보존, Terminal/status 불일치, metadata 8 MiB/64 KiB line 초과 backpressure, normal Terminal 뒤 descendant stdout writer 지연이다. 네 native authenticated mode 성공을 이 모든 실패 인수로 확대하지 않는다.

## 실행과 제한

제품 code/Cargo/fmt/build/GUI/native를 실행하거나 변경하지 않았다. freeze 중 source와 기존 결과를 읽고 해당 문서만 작성했다. root가 source 수정·fixture·통합 검증을 담당한다. 현재 코드 순서 위험을 실제 실패 원인 확정으로 표현하지 않는다.

## Root 실제 재현과 수정

Source freeze 해제 전 Linux windows-cookie-product-linux-workspace-first nineoperations terminal PASS/currentSHA357개 대조했다. 이후 cfgtest-only Terminalparse직후cancel hook을 추가해 Windows actualhelper/realcookielease fixture를실행했다. windows-cookie-late-terminal-native-red exact1 실패exit101/TerminalUnconfirmed 실제재현. root가 terminal.is_none && cleanup_deadline.is_none 조건으로새명령을Terminal전으로제한했다. green exact1PASS 및engine all-target Clippyexit0, 두operation source_unchanged/inputdesktopunchanged true. 실제원256receipt/validate/lease.cleanup/source·foreign보존까지통과. native원helper exit/Terminalcode/EOF/actualreceipt확인은유지한다. latest전체workspace와GUI는후속검증이다.

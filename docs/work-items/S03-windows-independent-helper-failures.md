# S03: Windows 독립 helper 실패 회귀

- 상태: done (독립 helper 실패 3case 범위)
- 분류: test
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: metrics_recovery — /tmp/dlpad-helper-failure-tests-draft.rs 및 이 신규 문서만; root GO 후 통합 담당자가 launcher tests에 반영
- 선행 작업: root HANDLELIST5 독립 helper launcher/entry, 정상 helper case, helperbinary 선빌드

## 사용자 결과

실제 별도 helper 프로세스의 config 전송 취소/만료와 ownlock ACK 이후 앱 소유 control EOF를 검증한다. 부모가 살아있는 EOF fixture를 실제 app 강제 종료라고 표현하지 않는다. 큰 config 전송·실제 app kill·제품 public startup recovery는 별도 범위다.

## 계약과 범위

기존 windows_helper_launch.rs tests의 정상 helper setup/read_frame을 재사용하는 code fragment를 /tmp에서 준비한다. project Rust/schema/Cargo는 source freeze 종료와 root GO 전 수정하지 않는다. synthetic TestFixture, 실제 original helper process handle/creation 및 bounded PeekNamedPipe 출력만 사용한다.

1. 실제 Prepared creation대조→app durable Bound/release→appAck/helperAck→Launch를 destructure하여 original Child/events/stdout/stderr를 유지하고 control writer만 drop한다. 실제 Terminal nonzero/helper nonzero/toolstdout0/receipt256을 확인하고 helper original wait 뒤 app reacquire/resolve/fullproof/sourcecopybytes를 확인한다.
2. config전송 전 Atomic cancel=true는 transmit Interrupted/config_sent=false/sendBegin거절을 확인한다. controlEOF로 helper code126과 output0/receipt0/원Intentbytes보존을 확인한다.
3. 실제 expired absolute deadline도 TimedOut/config_sent=false/sendBegin거절 및 동일 미시작보존을 확인한다.

Child::wait는 현재원handle 유한2초 API를 사용한다. pipe/event 읽기는 nativePeek/절대기한으로 무한대기를 막는다. 실패시 retainedChild Drop 및 rootowned nativehost Job가 소유scope를 정리하며 foreignPID 신호를 보내지 않는다. config/parser/domain에 실제cookie/account는 없다.

## 인수 조건

각 성공은 실제 native helperbinary/input hash 및 rawTerminal/원Childexit/actualreceipt로 확인한다. Intent는 전송전취소/만료에서 바뀌지 않고 source/copybytes는 모든case 보존한다. 값만생성한 parser통과를 실행증거로 확대하지 않는다. unknown/partial/timeout은 PASS가 아니다.

## 검증과 handoff

현재 초안 단계로 자체 Cargo/native실행 없음. root helperbuild와 실제Windows 테스트가 필요하다. 기존 runtime Request scopedthread5PASS는 독립helper 실패case의 근거로 대체하지 않는다. 이번파일만 작성하며 모니터/전원/사용자desktop/시스템설정은 변경하지 않는다.

## Root 통합

Root가 두 최신 Linux/Windows 전체 검증의 terminal PASS와 현재 source SHA 351/353개 일치를 확인한 후 source freeze를 해제하고 windows_helper_launch.rs tests에 초안을 통합했다. Cargo/schema/제품 실행 경로 변경은 없다. native 실패 3case 실행 결과는 아직 미검증이다.

## 실제 native 검증

Root가 windows-independent-helper-failures-native-first의 raw log와 세 operation JSON을 직접 확인했다. Windows GNU native `cargo clippy -p dlpad-engine --all-targets --locked -- -D warnings`, `cargo build -p dlpad-engine --bin dlpad-process-guard --locked`, `cargo test -p dlpad-engine --lib --locked process::windows_helper_launch::tests:: -- --nocapture` 모두 exit0. 정상1+실패3=4 PASS/135 filtered/0 failed. 세 실행 모두 source_unchanged=true/input_desktop_unchanged=true. `cargo fmt --all` exit0, 문서 검사497 PASS. 실제 app kill, 큰 config, 제품 recover 연결 및 변경 후 전체 workspace 재검증은 남는다. 부모가 살아있는 control EOF만 시험했으며 앱 crash 전체 인수를 주장하지 않는다.

## 변경 후 전체 Windows 검증

windows-helper-failures-native-workspace-first/result.json의 capability/fmt/workspace CPU clippy/test/release 모두 exit0/PASS이며 root가 현재 source SHA353개와 원시 결과를 대조했다. native GUI/실제 app kill/제품 회수 권한 인수는 이 검증에 포함되지 않는다. Linux 변경 후 전체 runner는 별도 진행 중이며 terminal 전 성공으로 표시하지 않는다.

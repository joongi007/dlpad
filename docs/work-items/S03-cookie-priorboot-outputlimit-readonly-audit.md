# S03: Priorboot 통합 출력상한 terminal 실패 읽기 감사

- 상태: completed (읽기 감사; 원인 미확정)
- 분류: diagnosis
- 우선순위: P0
- 요구사항: R08, R10
- 담당/수정 파일: soak_recovery_audit — 이 문서만
- 의존성: cookie-priorboot-dependency-first/lib.log, 기존 output-limit/terminal reset/moved leader 감사

## 계약과 인수 조건

원시 TerminalUnconfirmed 실패 위치와 이전 간헐 실패를 대조하고 최소 재현/관측 계약을 제안한다. process/guardian/user_service/tests/Cargo 수정이나 native 실행은 없다. 타임아웃·proof를 완화하지 않는다. diagnose skill의 원시 재현 후 관측 원칙을 적용하되 이번 담당은 읽기 감사라 실행·수정 단계는 root 후속으로 남긴다. 원인 미확정은 그대로 기록한다.

## 원시 실패의 정확한 범위

[lib.log](../validation/evidence/cookie-priorboot-dependency-first/lib.log)의249PASS/2FAIL/2ignored 중 process 실패는 process.rs1357의 **첫 oversized run_with_progress unwrap**이다. 다음 cancellation 단계 unwrap이 아니다. 합성 tool은100000-byte progress prefix를 flush하고 자연 종료한다. 그러므로 새 cookie gate나 priorboot pending 기능이 이 호출에서 직접 사용된다고 추정할 수 없다. 별도 priorboot fixture 선행조건 실패는 root 담당이며 이 오류와 원인이 같다고 주장하지 않는다.

당시 error는 Other/TerminalUnconfirmed이며 공개 typed masking으로 원래 단계/errno/frame byte/EOF 여부를 알 수 없다. 소스의 `cleanup_stage_error`가 debug-only `DLPAD_TEST_CLEANUP_STAGE_TRACE=1`일 때 force-command/command-half-close/owned-child-wait/terminal-frame-eof 단계와 kind/raw errno를 기록하도록 되어 있다. 원 lib.log에는 해당 trace가 없으므로 이 기록만으로 terminal parser 실패인지 강제 명령 실패인지 확정할 수 없다.

현재 엄격 경로는 oversized 관찰→먼저 try_wait→unreaped이면 T/200ms grace→필요 시 K→write half-close→owned child reap→stdout/stderr bounded drain→exact terminal+EOF 확인이다. 이미 reap된 helper의 poststatus K는 제거되어 있다. 기존 수정이 사라졌다고 단정하지 않는다.

## 이전 간헐 기록과 차이

[출력상한 race 작업](S03-output-limit-terminal-race.md)은 이전 전체시험197PASS/1FAIL 뒤 단독 exact PASS를 보존했고, 이후 poststatus K 제거/reader bounded drain/native22 인수를 기록한다. [reset 읽기 감사](S03-terminal-control-reset-review.md)의 unread K 뒤 GC/ECONNRESET 모델은 후보 재현이며 이번 실제실패 원인 증거가 아니다. [moved leader 감사](S03-guardian-moved-leader-terminal-regression.md)의 단독20/20 및 suite 병렬20/20도 기존 전체부하 실패를 확정 해결하지 못했다. 최근5th workspace PASS와 이번 library 병렬 FAIL은 부하/타이밍 의존 가능성을 보여줄 뿐이다.

## 최소 재현/관측 제안 (미실행)

Root가 source/helper/testbinary SHA와 실제 backend 선택을 고정하고 exact 첫 oversized test를 단독 반복한 뒤 동일 binary library 병렬 조건에서 반복한다. 정상 반복 한 번으로 해결이라 판단하지 않는다. 새 evidence에 회차/UTC/elapsed/test exit 및 bounded log를 보존한다. 실행 command 예시는 root의 고정 lib binary에 `--exact process::tests::observed_output_limit_and_cancellation_remain_bounded --nocapture`와 debug trace 환경을 전달하는 것이다. 제품 release에는 trace를 추가하지 않는다.

우선 현재 debug stage trace만 사용해 force-command vs terminal-frame-eof 등 실패 위치를 구분한다. 그것으로 부족한 경우 root 소유 source에 test/debug-only 다음 **상한 있는 비밀 없는 구조적 trace**를 검토한다: outcome 전환/try_wait reaped/최종 T-K 여부/half-close/readerfinish/GC-GU 발신/childreap monotonic 상대시각, terminal used byte count/허용된 frame enum/EOF·errno·deadline 원인. Tool args/log/cookie/native privatepath/환경 전체는 기록하지 않는다. Terminal bytes는 임의 raw payload 대신 known frame enum 또는 unknown/count로 제한한다.

검증 전 순위 없는 분리 질문은 다음과 같다. 실제 stage를 관측하기 전 특정 가설을 원인으로 채택하지 않는다.

- Force-command 실패이면 socket write kind/errno와 try_wait→K 시각 경계가 보여야 한다. Terminal parser만 실패한다는 trace는 이 후보를 반증한다.
- Guardian readerfinish/owned cleanup 실패이면 GU 및 해당 단계의 false/errno가 있어야 한다. GC+실제EOF는 이 후보를 반증한다.
- Unread command reset이면 terminal prefix 뒤 ECONNRESET와 상대 reader drain ordering이 관측되어야 한다. No-byte deadline이나 GU는 다른 경로다.
- 단순 terminal deadline/pipe 보유이면 childreap 후 EOF 없이1초 경과 또는 drain abort가 보여야 한다. 즉시 explicit wrongframe/errno는 이 후보를 반증한다.

Receipt/GC/EOF 기준을 완화하거나 reset을 EOF로 바꾸거나 timeout을 늘려 green으로 만드는 안은 없다. 실제 재현 후 해당 관측 seam의 회귀를 먼저 만들고 root가 수정/전체통합을 수행한다. Reader cleanup 미확인은 계속 typed 오류와 privatecopy 보존으로 처리한다.

## 실행과 handoff

### Root 고정 binary 후속 반복

[first result](../validation/evidence/outputlimit-fixed-binary-first/result.json)의 relocated test binary는 인접 guardian 누락으로 NotFound/exit101이었다. 원래 TerminalUnconfirmed 재현이 아니며 원시 로그를 유지한다. [second result](../validation/evidence/outputlimit-fixed-binary-second/result.json)는 test SHA37e63b65…/인접 helper SHA9800c026…를 함께 고정한 exact12회 모두 exit0이다. [parallel result](../validation/evidence/outputlimit-fixed-binary-parallel/result.json)는 동일 두 binary로 parallelism4/총12회 모두 exit0이며 각 raw log의1PASS도 대조했다. Second trace_enabled=true이고 root는 병렬도 같은 trace 환경이라고 보고했다. 병렬 result에는 해당 필드가 없어 필드 존재로 주장하지 않는다.

원래 오류는 미재현/원인 미확정이며 exact 반복은 전체 library 동시 부하의 완전 재현이 아니다. Proof/deadline 변경은 없고24PASS를 원래 failure 해결 완료로 바꾸지 않는다. Agent는 root evidence만 읽었고 실제 반복을 실행하지 않았다.

원시 log/담당 source/기존 workitem 및 diagnose skill을 읽었다. 이번 task는 source/tests 변경·Cargo/native 실행·실제 재현을 하지 않았다. 문서만 작성했고 원인은 계속 미확정이다. Root 현재 debug stage trace를 이용하는 최소 반복 인수와 필요 시 bounded 내부 terminal trace를 후속으로 제안한다.

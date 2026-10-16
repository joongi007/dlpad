# S03: Windows guardian runtime

- 상태: in-progress (root 통합; 실제 helper dispatch 미연결)
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R02
- 담당/수정 범위: soak_recovery_audit — tmp 초안; root — process/windows_guardian.rs, process.rs, windows.rs의 cancel_status와 cookies.rs 재export, 이 문서 통합 검증
- 선행 작업: Windows fixed frames, GuardianJournal facade, PreparedChild

## 계약과 인수

Request로 argument를 묶고 suspended prepare→Prepared→app Ack→own journal verify→helper Ack→Begin→Resume 순서를 구현한다. 단일 absolute deadline, bounded PeekNamedPipe 128-byte partial framing, EOF/cancel/timeout/malformed 시 original Job 정리, Job empty 뒤 original receipt.complete 및 Terminal만 허용한다. tool direct pipes를 유지하며 PID lookup/thread/shell/새 의존성을 도입하지 않는다. parent wiring/Cargo는 root 담당이다.

## 검증과 handoff

현재 source freeze 때문에 초안은 tmp에만 작성한다. 제품 구현 완료 또는 native PASS를 주장하지 않는다. API 호환성/컴파일/실제 Windows 실패 인수는 root 통합 검증이다.

tmp 초안 SHA256은 `20925a096c4c6fd1d71ff42edc6343fdbeff7e925eb4dde53291cff086c285fe`이며 rustfmt 실제 exit 0이다. Request와 bounded partial frame polling, 양방향 Ack 구분, journal 독립 획득, gate 실패와 running 정리를 구현했다. Cargo/native 실행은 하지 않았다.

위 초기 초안의 임시 code 1은 후속 root 계약으로 제거했다. 최신 tmp SHA256은 `f72abd185f7438902bb332f6c3d3bb964deb557edada7f33add85c82060c6ba2`이다. root가 추가하는 `PreparedChild::cancel_status`의 실제 ExitStatus를 preBegin acquired 실패 Terminal에 사용한다. `cookies::GuardianJournal` 재export 계약으로 import를 변경했다. 마지막 Resume 직전 deadline race의 Interrupted는 원 status를 반환하지 않으므로 receipt/Terminal 없이 fail closed한다. Resume 오류도 확정 proof 없이 receipt/Terminal을 발급하지 않는다. rustfmt 실제 exit 0이며 프로젝트 Rust는 계속 게시하지 않았다.

## Root 통합과 첫 native 실패

Root가 초안을 프로젝트에 게시하고 cancel_status를 연결했다. run은 원 도구 Win32 종료코드 u32를 반환하며 helper entry/parent parser는 후속 연결이다. 첫 native engine all-target Clippy는 ExitStatus::into_raw 미지원으로exit101였으며 raw로그를 windows-guardian-runtime-control-native-first에 보존했다. Windows ExitStatus.code()의 i32를 u32로 변환해 native bit pattern을 유지하도록 수정한다. codeNone은 고정 실패이며0/1로대체하지 않는다. helper 실행 성공으로 tool failure를 덮어쓰지 않는다.

Control 두 native 회귀는원 pipe의partial/EOF/deadline만검증하며 full Request/appcrash/Jobreceipt/entry/launcher 인수가아니다. 컴파일 실패 때문에 첫 Control 회귀는 실행되지 않았다. 수정 후 새 evidence에서 검증한다.

수정 후 Windows native-second: engine all-target Clippy exit0 및 Control2회귀 정확2PASS, source unchanged/input desktop unchanged를 root가 확인했다. 두 시험은 실제 anonymouspipe partial64+64 bytes, 완성frame뒤EOF, partial17뒤EOF거절, elapseddeadline거절이다. 전체 Request 실행·독립 appcrash·helper entry/launcher는 아직 인수되지 않았다. 전체루프5합성 fixture를 별도 workitem으로 준비하고 안정화된 snapshot에서 공통 전체검증을 수행한다.

Root가 전체 Request5native case를 통합하고 실제Windows exit0/5PASS로확인했다. 독립childhelperprocess entry/appcrash/publicstartup은미완료다. Windows전체4검증 및 Linux기본/CPU7검증은새runtime-workspace-first에서진행중이다.

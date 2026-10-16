# S02: OS 공통 bounded Netscape parser

- 상태: in-progress
- 분류: refactor
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: stress_recovery — crates/engine/src/cookies.rs, 신규 crates/engine/src/cookies/format.rs, 이 문서
- 선행 작업: Windows ACL 검토; Unix cookie20시험/실행guard

## 사용자 결과

Windows backend에서도 Unix와 같은 형식·용량 제한·비밀 버퍼 정리를 사용할 수 있도록 인증 parser를 OS 독립 helper로 옮긴다. 기존 Unix 동작을 보존한다.

## 계약과 범위

기존 validate/SensitiveBytes/SensitiveBlock/scrub만 분리하고 privacy는 parent module 한정으로 유지한다. I/O·ACL·recovery·lease public API·Cargo는 변경하지 않는다. 오류 순서와 byte/line/entry 경계, exact literal copy, volatile wipe를 유지한다. parser 복제로 OS별 의미를 갈라놓지 않는다. 기존 parser 시험을 OS공통 module로 이동하여 nonUnix에서도 동일 검증이 가능하게 한다. 새 dependency/thread 없음, buffer 상한 유지.

## 인수 조건

유효LF/CRLF/HttpOnly와유효0row 형식의기존정책, invalidUTF8/NUL/header/7field/flag/path/expiry, exact1MiB/16KiB/4096entry경계가동일하다. Debug에secret없음. 기존Unix20/다운로드8시험과Windowsnativebuild를root가실행한다. 이작업은Windows파일지원완료가아니다.

## 검증과 handoff

agent는rustfmt/sourcefreeze를제공하고Cargo는root가통합검증한다. 공유schema/lockfile변경없음. root는해당cookies.rs를동시에수정하지않는다.

## 구현 동결 인계

기존 parser와 SensitiveBytes/SensitiveBlock/scrub를 신규 cookies/format.rs로 원문 로직 그대로 이동했다. helper와 buffer field 가시성은 cookies parent 한정 pub(super), scrub는 private다. Unix source reader가 동일 helper를 import하며 I/O/recovery/lease API는 변경하지 않았다. nonUnix backend 연결 전 private helper 미사용 경고만 module 수준 cfg_attr(not(unix),allow(dead_code))로 제한했다. 실제 Windows 소비자가 연결되면 이 임시 제한을 제거할 수 있다.

OS 공통 unit6개를 추가했다: LF/CRLF/HttpOnly/literal/zero/빈name-value, malformed/UTF8/NUL, 1MiB exact/overflow와 오류 우선순위, CR포함16KiB line, HttpOnly4096entry, Debugredaction/직접scrubzero. Unix SensitivePath Debug 시험은 유지한다. 비밀 소거 volatile write와 SeqCst compiler fence는 변경하지 않았다. 추가 deps/thread/buffer 비용 없음. Cargo/Windows 실제 build는 root 검증 대기이며 작성한 시험을 PASS로 보고하지 않는다.

## 실제 공통 parser scoped 검증

root가 default/CPU workspace 시험·두 all-target clippy·fmt를 실제 exit0으로 확인했고 공통 format unit6개의 각 ok를 로그에서 대조했다. [명령 결과](../validation/evidence/owned-child-reader-workspace-retry/workspace-result.json)와 default-test.log/cpu-test.log에 보존한다. 보고 당시 release2개는 진행 중이므로 그 성공을 이 기록에서 주장하지 않는다. Windows backend 작성은 별도이며 이 parser 시험 성공을 Windows cookie ACL/runtime 인수로 쓰지 않는다.

# S02: Windows 완료 receipt 회수 구현 감사

- 상태: proposed
- 분류: review
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: metrics_recovery — 이 신규 문서만
- 선행 작업: Windows deferred-cookie-recovery-design; root 공유 계약 및 실제 native 검증

## 사용자 결과와 결정

Windows startup 회수 요구를 실제 구현으로 연결한다. 정상 session copy/Job 정리는 있으나 recover_sessions는 Unavailable이다. root의 native boot 후보3process×3query 안정성은 unsupported layout/재부팅별 유일성 미확인으로 authority=false다. 이를 이전 부팅 삭제 권한으로 채택하지 않는다.

최소 제품 경로는 원래 guardian이 작성한 완전한 terminal receipt로 사본을 회수하는 것이다. 이 증거는 실제 reader 전체가 종료되었음을 확정하므로 boot identity 없이도 검증할 수 있다. 같은 부팅 회수부터 구현하되 실제 boot 동일 여부를 추측하는 필드를 만들지 않는다. receipt 없는 Intent/Bound는 보존하며 이전 부팅 자동 회수는 별도 잔여 게이트다.

## 정확한 현재 진입점

- cookies.rs: recover_sessions의 non-Linux Unavailable 분기, SessionCookieLease의 owned Windows copy 및 reader_pending, begin_reader/confirm/cleanup/Drop. Linux begin_gated_reader와 GatedCookieReader의 durable 모델은 의미 참고이며 그대로 Windows FD로 이식하지 않는다.
- cookies/windows.rs: OwnedCopy import, owner_valid, validate_for_tool, cleanup, defer_cleanup. 현재 defer는 핸들을 닫고 파일을 남기며 startup journal을 만들지 않는다. 기존 pins/native identity/ACL 검증을 새 journal에도 재사용 가능한 최소 private helper로 root가 분리한다.
- process.rs: run_metadata_with_cookie/run_with_cookie는 Linux 전용; run_capture_inner의 Windows windows::spawn 및 confirm_terminal. download.rs의 non-Linux begin_reader와 postprocess confirm 경로를 실제 gate API로 연결해야 한다.
- process/windows.rs: Attributes::new의 HANDLE_LIST/JOB_LIST, suspended, resume_controlled, Ownership::new/confirm_terminal 및 Child. CreateProcess에서 PROC_THREAD_ATTRIBUTE_JOB_LIST로 원자적 할당 후 IsProcessInJob을 검증한다. 이전 설계의 생성 후 할당 표현보다 강한 현재 계약이다.
- process/guardian.rs는 Linux 전용 모듈이며 Unix socket/systemd 코드를 Windows에서 실행할 수 없다. Windows 전용 guardian entrypoint와 helper dispatch를 새로 연결해야 한다.

## 최소 구현 분할

1. 신규 cookies/windows_journal.rs: original owner/copy/record/receipt의 volume+128bit FileId, 현재SID/protected DACL/no-reparse/link1, nonce/digest 및 전체 allowlist를 commit·재열기·삭제 직전 검증한다. Intent/Bound/Resolved 두slot과 bounded record를 제공한다. LockFileEx 비차단 잠금, create-new receipt, FlushFileBuffers 실패는 보존한다. 원래 marker anchor를 현재관측으로 대체하지 않는다. 최대128sessions/2readers/4096record/256receipt의 공유 포맷은 root 승인 후 결정한다.
2. 신규 process/windows_guardian.rs: 원래 unnamed Job 및 process/thread handle을 독점 보유한다. reader를 suspended/JOB_LIST로 생성한 후 app Bind commit→Begin 이전 Resume하지 않는다. 제어EOF/취소/기한은 원래 Job에만 적용하고 bounded wait/ActiveProcesses=0 후 receipt capability에 write+flush한다. receipt 생성 시점에는 추가 process 할당이 불가능하도록 내부 launch state를 봉인한다.
3. root cookies/process/download 통합: begin_gated_reader→single-use handoff→actual identity Bind→Begin→Ready, terminal receipt resolve 후에만 reader_pending을 해제한다. startup scanner는 original native anchor/전체 receipt/잠금 검증으로 삭제한다. nonempty pending/no receipt는 summary에 보존으로 남긴다. 공개 ProcessSpec 필수필드 증가는 피하고 선택적 observer를 사용한다.

Guardian에는 제한된 write-only receipt와 동일 record-lock ownership 및 제어channel만 허용한다. writer/lock/Job/control handle은 실제 도구 HANDLE_LIST에 포함하지 않는다. receipt writer의 original FileId·mode에 해당하는 Windows ACL/access rights·nonce를 guardian이 확인한다. 모든 보조 채널은 fixed bounded frame이며 비밀경로/쿠키/record를 stderr·DB·argv diagnostic에 넣지 않는다. helper dispatch를 사용자가 arbitrary receipt writer로 악용할 수 없도록 original capability 및 handle type/access validation이 필요하다.

## Job와 private host 경계

제품 Job는 KILL_ON_JOB_CLOSE만 설정하며 breakaway 권한을 주지 않는다. tool이 breakaway를 시도해도 원래 reader Job에서 벗어나지 않는 실제 fixture를 작성한다. guardian은 reader Job 밖에 있어야 하며 앱 종료 때 그 Job handle을 닫는 코드와 guardian이 살아있는 상태를 혼동하지 않는다. guardian도 종료되면 마지막 Job close가 kill을 유도하지만 receipt 없는 삭제 근거는 아니다. 부모 Job를 우회하거나 시스템 정책을 변경하지 않는다.

현재 windows_cli_host.cs는 자신이 만든 소유 Job에 CLI child를 할당하고 host 종료 때 남은 grandchildren을 종료한다. 이 host 안의 app만 죽이는 시험에서는 guardian이 살아남아 receipt를 쓸 수 있지만 host 전체 종료 시험은 guardian도 죽일 수 있다. 이는 제품 앱 외부 watchdog/자동 재개 보장이 아니다. nested Job 미지원/상위 Job 제한으로 atomic assignment 실패 시 Resume0/failclosed해야 하며 breakaway fallback을 넣지 않는다. 실제 제품 실행 context와 private-host context를 구분하여 로그에 남긴다.

## 실제 native 인수

- public import→metadata0 실제 guardian gate→download1→app만 kill→guardian EOF cleanup→actual receipt flush→public recover 성공. 원래 source와 외부 sentinel은 보존한다.
- Intent 전/후, Bind 전/후, Begin/Resume 경계 crash: actual tool marker0 또는 실제시작을 확인하고 receipt 없는 사본은 보존한다.
- exit0/42/126, 취소/timeout, 실제grandchild, breakaway시도: 원래 process handle 및 Job ActiveProcesses0/actual receipt를 검증한다. exit126을 protection failure와 혼동하지 않는다.
- guardian kill/app+guardian kill/host 전체 종료: receipt 없으면 보존. Job kill-on-close 결과는 실제 original process handle로 확인하며 PID 숫자로 신호하지 않는다.
- partial/mismatched receipt, ACL/owner/FileId/reparse/hardlink 변경, locked/extra/oversize record: public recover가 보존한다. 최대capacity에서 failclosed하고 원본을 삭제하지 않는다.
- guardian 관리 handle이 tool에 상속되지 않음을 native handle fixture로 확인한다. duplicate record lock이 남는 경우 recovery Busy를 실제 시험한다.

전체 시험은 실제 Windows private temp/synthetic cookie와 hidden CLI에서 실행하고 exact source/helper/tool hash·native handle identity·stderr omission·bounded terminal을 기록한다. reboot나 실제 계정은 필요 없다. 실제 다음 부팅 삭제는 boot authority가 검증되기 전 완료로 보고하지 않는다.

## 검증과 handoff

이번 작업은 기존 코드와 설계 읽기, 이 신규 문서 작성만 수행한다. 제품/schema/Cargo 수정, GUI/native 실행은 없다. root가 위 세 단계의 공유 인터페이스를 확정한 뒤 독립 신규 모듈을 배정하고 수직 public recovery 시험으로 통합한다. 구현되지 않은 안전한 Unavailable을 완료로 인정하지 않는다.

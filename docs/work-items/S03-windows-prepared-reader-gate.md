# S03: Windows reader의 durable Bind 이전 실행 gate

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: root — process/windows.rs 및 이 문서; 새 protocol은 별도 담당
- 선행 작업: Windows journal/record 실제 native 및 전체검증 PASS

## 사용자 결과

독립 guardian이 도구를 suspended/원래Job에 원자적으로 할당하고, 실제 생성identity를 제공한 다음 durableBind·Begin 전에는 실행하지 못하도록한다. 제품startup회수는 전체연결 후에만 활성화한다.

## 공유 API와 범위

PreparedChild::prepare(spec), prepare_with_stdio(spec,stdout:&File,stderr:&File), creation_times()->(guardianCreated,toolCreated), resume(self,ExecutionControl)->SpawnOutcome, cancel(self)->Result<()>를 Windows backend 내부에 제공한다. 원래process/thread/Job 핸들을 RAII로 소유하며 PID 재조회/Job 재구성/새dependency는 없다. 기존 spawn은 같은API를 사용하고 기존 취소/기한 정책을 유지한다. 직접stdio는 별도 stream전달thread 대신 helper가 받은 pipe writer만 도구 HANDLE_LIST에 넣는다. control/receipt/journal 핸들은 도구에 상속하지 않는다.

Protocol은 최대256KiB JSON config(UTF16argv,원WindowsReaderRecord,slot,Duration), 고정identity frame, Begin/Cancel/Timeout opcode를 사용한다. identity는 원process GetProcessTimes의 FILETIME이며 durable삭제단독권한이 아니다. helper는 원Jobempty 뒤에만 원receipt를 쓴다.

## 인수 조건

실제 Windows에서 prepared도구의 marker/stdout0, nonzero원FILETIME, cancel시 원Jobempty, Begin이후 정상실행, directstdio/관리handle 비상속을 검증한다. Linux전체컴파일과nativeWindowsClippy/test를 통합확인한다. 실패경로에 실행0을 확인하고 receipt나Resolved를 위조하지 않는다.

## 검증과 handoff

파일잠금의 guardian복제 수명은 별도native실험이 필요하며 미확인인 동안 appcrash회수를 완료로 보고하지 않는다. 공유schema root조정, 다른OS동작/실제미디어/사용자desktop 시험없음.

## 초기 구현과 실제 gate 검증

PreparedChild가 원 process/thread/unnamed Job을 보유하고, 기존 spawn도 prepare→resume를 사용하도록 연결했다. creation_times는 원 핸들의 GetProcessTimes를 사용하며 cancel은 원 Job 종료·Child wait·ActiveProcesses0을 모두 확인한다. dependency/Cargo/schema 변경 없음. directstdio 및 독립 guardian/제품 journal 연결은 아직 구현하지 않았다.

고정 프로젝트 toolchain `cargo fmt --all` exit0. 첫 cargo fmt는 RUSTUP_HOME 미지정으로 fmt command 없음(exit101)이었으며 경로 지정 후 성공했다. 실제 Windows 격리 CLI에서 `cargo test -p dlpad-engine --locked process::windows::tests::prepared_identity_is_native_and_cancel_never_runs_tool -- --nocapture` exit0, 정확히1test PASS/112filtered. 새 windows-prepared-gate-native-first의 raw log와 source SHA unchanged, input desktop unchanged를 root가 확인했다. 생성 identity 두 값 nonzero/반복동일, suspended child live/출력0, cancel확인 뒤 EOF/출력0을 검증했다.

Linux 전체 runner의 첫 실행은 PATH tool unavailable로 Cargo 실행 전에 실패했다. windows-prepared-gate-linux-first를 보존하고 고정 PATH로 linux-second 재실행 중이다. Windows native-workspace-first 전체4명령도 진행 중이며 terminal 전에는 완료로 인정하지 않는다. 잠금 수명 실험은 metrics_recovery가 별도 파일/증거를 소유한다.

Windows native-workspace-first의 capability/fmt/clippy/test/release 모두 exit0, passed=true/cargo_checks_complete=true/source_unchanged=true로 terminal 완료를 root가 확인했다. Linux 전체는 아직 실행 중으로 성공으로 기록하지 않는다.

독립 실험 raw native-parent.log를 root가 읽었다. 원 잠금 유지 중 재열기는 WouldBlock, 같은 native identity의 child가 복제 handle을 보유해도 부모 original close 후에는 acquired였다. duplicate_preserves_lock=false이므로 복제만으로 exclusive lock을 인계할 수 없다. guardian은 직접 원 journal 재잠금을 얻고 원 Bound를 검증한 ACK 뒤에만 Begin을 허용해야 한다. 인계 사이 receipt 없는 Intent/Bound는 scanner가 보존한다. 이 인계는 후속 구현이며 원 receipt와 원 Job empty 증명을 생략하지 않는다.

## 직접 stdio 연결

prepare_with_stdio가 기존 출력 writer를 current process에서 각각 새 inheritable handle로 복제하고, NUL stdin과 이 두 handle만 명시 HANDLE_LIST에 넣는다. 원 writer의 inherit flag를 변경하지 않는다. child stdout/stderr capture는 None이며 별도 전달 thread가 없다. protocol/receipt/journal/Job handle은 명시 목록 밖에 있다. 기존 prepare는 원래 bounded pipe capture를 유지한다.

실제 Windows `cargo test -p dlpad-engine --locked process::windows::tests::prepared_ -- --nocapture`: exit0/source unchanged/input desktop unchanged, 정확히2test PASS. 실행 전 cancel/출력0 및 Resume 이후 원 pipe 출력/정상exit/Jobempty/EOF를 검증했다. windows-prepared-direct-stdio-native-first의 원시 로그를 root가 대조했다.

앞선 gate snapshot의 Linux 기본/CPU 전체7명령과 version2개가 모두 exit0/PASS/sourceunchanged로 completed를 확인했다. stdio 추가 snapshot의 Linux 전체 및 Windows 전체는 새 전용 evidence에서 진행 중이며 성공을 선반영하지 않는다. guardian entry/Bound/lock ACK/receipt/제품startup scanner 연결은 계속 미완료다.

직접 stdio 추가 snapshot의 Windows GNU 전체4명령과 capability가 exit0/PASS로 terminal 완료했다. root가 result.json의 passed/cargo_checks_complete/source_unchanged와 현재 전체 입력 SHA를 다시 대조해 모두 일치함을 확인했다. Linux 전체 runner는 아직 live이며 terminal을 확인할 때까지 완료로 표시하지 않는다. 원래 관리 handle 제외는 HANDLE_LIST 구현을 확인한 상태이며, 실제 새 guardian의 receipt/control/journal 비상속 시험은 guardian 연결 뒤 수행한다.

직접 stdio snapshot의 Linux 기본/CPU 필수7명령과 version2개가 모두exit0/PASS/sourceunchanged로 terminal 완료했다. root는 전체 source_final SHA를 현재 파일과 대조해 일치를 확인했다. 이 뒤 새 journal handoff 작업을 시작하므로 이 검증은 handoff 변경 이전 snapshot이다.

이전 목표 turn은 직접 stdio 코드·native2회귀·Windows전체검증 완료로 progress다. 이번 turn은 Linux 전체terminal/currentSHA를 확인하고 root가 windows_records.rs의 pending_bound_identity 및 Empty/Intent/Resolved 실행 인계 거절 회귀를 추가했다. metrics_recovery는journal파일, soak_recovery_audit는새fixedframe파일을소유하며 root는공유record/parentmodule을소유한다. 정상Git없음으로worktree생성하지않고기존.git보존/명시파일분리로진행한다.

새 pending Bound 순수 회귀 `cargo test -p dlpad-engine --locked execution_handoff_rejects_empty_intent_and_resolved_slot` 실제exit0/정확1PASS. Empty/Intent/Resolved는 handoff 실행권한이 아니며 원 receipt 검증 권한은 별도로 유지한다.

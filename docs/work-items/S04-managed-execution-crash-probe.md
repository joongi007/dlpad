# S04: 관리 pending marker 실제 Linux 프로세스 종료

- 상태: done
- 분류: test
- 우선순위: P0
- 요구사항: R05, R08, R13
- 담당/수정 범위: metrics_recovery — 신규 examples/managed_execution_crash_fixture.rs, scripts/managed_execution_crash_probe.py, 이 문서만; collector 종료 GO 전 Rust는/tmp 초안만
- 선행 작업: durable ManagedExecution marker와 begin_execution/finish 실제 API

## 사용자 결과

실제 합성 child가 begin_execution 뒤 ready를 보내고 원 owned child를 SIGKILL한 뒤 fresh ToolManager의 select_existing이 ExecutionPending으로 거절되는지 확인한다. 정상finish 대조는 mutation이 다시 허용돼야 한다. 외부 source/sentinel을 보존한다.

## 계약과 인수

새 private temporary root에서 고정합성 실행파일을import하지만 실행하지 않는다. childstdout fixedready를 boundedpoll로 확인하고 Linux/proc pid startticks를 확보한다. 같은 subprocess original Child.kill/wait만 사용하며 PID lookup으로foreign signal하지 않는다. 정상case는stdinF→finish→zeroexit, crashcase는ready뒤SIGKILL→negativeSIGKILL exit를확인한다. freshprocessprobe의 typedExecutionPending 또는clear fixedJSON과원source/sentinel SHA를확인한다. walltimeout/pipe출력 상한/ownchildkill-reap를유지한다. 현시험은실제Linux process scope며 guardian readerhandoff/자동marker회수/Windows실증이아니다.

## 검증과 handoff

root Windows collector live동안 Rustproject추가를보류하고/tmp초안/Python만준비한다. root GO후 example게시·정확 toolchain build·nativeprobe실행을진행한다. 실제실행전PASS를주장하지않는다. 기존Rust/Cargo/schema/사용자데스크톱·데이터·계정은변경하지않는다.

## 실제 검증 결과

root의 Windows collector 종료 GO 뒤 신규 example을 게시했다. 개별 rustfmt와 Python py_compile은 exit 0이다. 프로젝트 전용 toolchain으로 `cargo build -p dlpad-engine --example managed_execution_crash_fixture --locked`가 exit 0이었다. 기존 Rust/Cargo/schema는 수정하지 않았다.

`python3 scripts/managed_execution_crash_probe.py --fixture /mnt/d/project/dlpad/target/debug/examples/managed_execution_crash_fixture --output /mnt/d/project/dlpad/docs/validation/evidence/managed-execution-crash-linux-first`는 실제 두 사례 PASS였다. 원 설치 실행파일 SHA 전후 비교를 추가한 동일 명령의 출력 디렉터리 `managed-execution-crash-linux-second`도 exit 0, 두 사례 PASS였다. [최종 원시 결과](../validation/evidence/managed-execution-crash-linux-second/result.json)에 원 PID/start ticks, 정상 exit 0과 실제 SIGKILL exit -9, fresh manager의 clear/pending, source/sentinel/원 설치 파일 SHA 보존을 기록했다. SIGKILL 사례에는 60바이트 원 `.execution-pending`이 남았다.

fixture 바이너리는 47,641,480바이트 debug 개발 산출물이며 실행 전후 SHA256 `762b7c80075dc9c853b11e14da523ea14a913b79e96f73ddb7145e0aa1d8785f`가 같다. 이는 제품 release 크기 측정이 아니다. 소유 임시 폴더와 child/pipe는 각 사례 종료 때 정리했다. ready 읽기는 수신 중 상한을 적용하지만 합성 helper의 `capture_output`/`communicate` 출력 검사는 수집 후 상한이다. 이 도우미를 임의 외부 도구용 bounded capture로 주장하지 않는다.

실제 tool reader/guardian FD 인계, reader 종료 뒤 자동 회수, Windows 강제 종료는 미검증이다. 이 시험은 관리 실행의 durable pending 생성 후 앱과 유사한 소유 프로세스 강제 종료 시 변경 거절을 입증한다.

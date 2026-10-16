# S04: Windows 관리 실행 pending의 실제 프로세스 강제 종료

- 상태: done
- 분류: test
- 우선순위: P0
- 요구사항: R05, R08, R13
- 담당: metrics_recovery
- 수정 파일: 신규 scripts/windows_managed_execution_crash_observer.cs, scripts/windows_managed_execution_crash_probe.py, 이 문서
- 의존성: 기존 공통 managed_execution_crash_fixture example, Windows GNU 도구, 기존 windows-cli-host 비공개 데스크톱/Job

## 범위와 인수 조건

공통 example을 수정 없이 native Windows로 빌드한다. C# observer는 shell 없이 argv quoting을 적용한 실제 자식 Process를 실행한다. 원 핸들의 GetProcessTimes 생성 FILETIME과 ready를 확인한 뒤 원 TerminateProcess 핸들로 명시 exit 73을 전달하고 유한 wait로 확인한다. 정상 F/finish/exit 0 대조와 fresh manager의 clear 또는 ExecutionPending 거절을 비교한다. source, foreign sentinel, 설치 payload SHA는 전후 일치해야 한다. 모든 실행은 기존 비공개 데스크톱 CLI host의 소유 Job 아래에 두며 visible fallback을 하지 않는다.

합성 evidence 폴더 안의 원 데이터만 사용한다. 실패 raw와 합성 폴더를 보존한다. 출력은 observer에서 스트림별 4096 문자로 수신 중 제한하고 외부 host도 기존 로그 상한과 deadline을 유지한다. guardian FD 인계, 실제 도구 reader 생존 및 pending 자동 회수 완료로 확대하지 않는다. Cargo/기존 Rust/schema는 변경하지 않는다.

## 검증

구현 전 계약이며 실제 native 결과는 실행 후 기록한다.

## 실제 첫 두 차례 검증

`python3 scripts/windows_managed_execution_crash_probe.py /mnt/d/project/dlpad/docs/validation/evidence/windows-managed-execution-crash-first`와 `windows-managed-execution-crash-second`가 각각 terminal exit 0이었다. native GNU example 빌드, native C# 컴파일, 실제 비공개 데스크톱 observer의 정상/강제종료 2개 사례가 모두 통과했다. 두 차례의 원시 로그는 그대로 보존했다. 첫 observer의 ready ReadLine은 합성 고정 출력의 수집이었으며 수신 상한을 주장하지 않는다. 두 번째는 ReadReady에 수신 중 4096 문자 상한을 적용했다.

두 번째 정상 사례 PID 31028의 원 생성 FILETIME은 134355734916708365이며 exit 0 뒤 fresh manager 변경이 허용됐다. 강제종료 사례 PID 19272의 원 생성 FILETIME은 134355734917780630이며 원 핸들의 TerminateProcess 명시 code 73으로 종료하고 fresh manager는 pending을 거절했다. 실제 `.execution-pending`은 60바이트로 보존됐다. 설치 payload/source/sentinel SHA는 각 사례 전후 같았다. 원 바이너리 SHA256은 `ed18ffb3b14afa28dd82fb6a0225b668eb14045c9679f55bfc0ad237d3aa0876`이다.

각 실행 host는 비공개 데스크톱과 원 input desktop 변경 없음, 기존 Rust 입력 SHA 전후 일치를 보고했다. 소유 child는 유한 wait 후 종료됐고 합성 owned-normal/owned-crash 폴더는 evidence로 보존했다. guard/tool reader handoff 및 자동 회수는 시험하지 않았다. collector 자체 소스의 전후 SHA까지 추가한 세 번째 최종 실행은 별도 결과를 기록한다.

## 최종 검증 및 한계

세 번째 최종 명령의 출력 디렉터리는 `docs/validation/evidence/windows-managed-execution-crash-third`이며 실제 terminal exit 0, 두 사례 PASS다. [최종 결과](../validation/evidence/windows-managed-execution-crash-third/result.json)에서 collector C#/Python 자체 소스의 전후 SHA 일치와 각 빌드/컴파일/native 실행의 기존 Rust 입력 일치, 비공개 desktop/input desktop 불변을 확인했다. [실제 원 프로세스 결과](../validation/evidence/windows-managed-execution-crash-third/native-result.json)의 정상 code 0/clear와 강제 code 73/pending, source·설치 payload·sentinel 보존이 모두 통과했다. Python py_compile exit 0이다.

합성 root는 이번 작업 evidence 안에만 남겼다. 원 child 종료와 host Job 종료가 완료됐으며 GUI/사용자 desktop/실제 계정/프로젝트 밖 기존 데이터는 사용하지 않았다. 최초 릴리스 Windows 범위에 따른 프로세스 종료 시 pending 보존의 실제 native 증거이며, 독립 guardian의 도구 생존/receipt·자동 회수 수직 기능 완성 증거로 확대하지 않는다.

# Linux guardian 고장과 출력 drain 상한

2026-10-01 Linux/WSL2, 합성 Python 프로세스·임시 폴더만 사용했다. GUI·실제 영상·사용자 Documents·network·cgroup/system 변경은 없다.

## 실패 재현과 변경

이전 process adapter는 guardian의 exit를 얻은 뒤 control socket의 BrokenPipe를 무시하고 stdout/stderr thread를 join했다. guardian SIGKILL로 leader는 종료돼도 escaped 자손이 inherited 출력 pipe를 열어두면 EOF를 얻을 수 없어 process timeout 300ms를 넘어 대기했다.

추가된 실제 process::run 회귀를 **변경 전** 빌드에서 실행한 결과, 3초 상한을 넘겼으며 exit 101로 실패했다: `worker blocked beyond process timeout on escaped pipe holder`. thread join에서 Spec timeout을 다시 확인하지 않던 결함을 재현한 것이다. fixture owner와 자손은 소유 pidfd로 종료했다.

Linux InterruptiblePipe는 reader가 독점 소유한 descriptor에 O_NONBLOCK을 설정하고 poll 20ms/stop flag로 읽는다. 정상 EOF는 aborted=false이며 강제 중단은 aborted=true와 IO TimedOut이다. main 담당자가 process.rs에 연결한 StopDrains RAII는 조기 오류에도 reader를 중단한다. guardian exit 이후 두 reader에 최대 1초 drain grace를 주고 미완료 시 stop 요청, 양쪽 join, 강제 중단 확인 후 TimedOut 오류를 반환한다. 비 Linux adapter는 기존 읽기를 유지한다.

강제 stop 시 아직 읽지 않은 출력은 생략될 수 있다. 이는 완료 증거로 사용하지 않으며 작업 실패로 반환한다. guardian 고장으로 살아 남은 자손의 종료를 보장하는 기능은 아니다.

## 실제 검증

프로젝트 전용 Rust 1.94.0/Cargo 환경에서 실행했다.

```sh
cargo test -p dlpad-engine --lib process::guardian::pipe_tests --locked
cargo test -p dlpad-engine --test guardian --locked -- --test-threads=1
cargo test -p dlpad-engine --lib process:: --locked
cargo clippy -p dlpad-engine --all-targets --locked -- -D warnings
```

- InterruptiblePipe 단독 3 tests 통과: writer가 열린 채 idle read 중단, 정상 데이터+EOF, pending data보다 stop 우선.
- guardian integration 8 tests 통과, 4.58초. 새 regression은 guardian만 pidfd SIGKILL하고 process::run worker가 **2초 이내 TimedOut**을 보고하는지 확인한다. escaped pipe holder는 여전히 살아 있음을 확인하여 drain 종료와 자손 보호를 구별한다. 무관한 Python 프로세스 생존도 확인했다. fixture만 pidfd 종료 후 named waitpid로 회수한다.
- process 단위 회귀 12 tests 통과, 0.42초: metadata, 양쪽 출력/progress, flood/log 상한, line limit, 취소, timeout, argv/실행 실패 포함.
- clippy 경고 없음.

새 dependency/schema/thread/buffer는 없다. 기존 두 drain thread를 사용하며 유휴 pipe당 poll 최대 약 50회/초로 stop을 확인한다. GUI release binary 크기·CPU/RSS 및 workspace 전체 검증은 통합 담당자의 결과로 기록한다. 짧은 회귀는 8시간 soak를 대체하지 않는다.

## 남은 범위

[guardian 자체 고장 보호 한계](guardian-fault.md)는 그대로다. guardian SIGKILL/fatal 고장 시 모든 자손 정리는 출시 차단 조건이며, bounded worker 실패 반환만으로 이를 완료 처리하지 않는다. app fallback에는 자손 ownership 증명과 guardian exit 선행 감지가 필요하다. app/guardian 동시 고장에는 OS kernel-owned scope가 별도 검증되어야 한다. Windows/macOS와 실제 logout/sleep은 미검증이다.

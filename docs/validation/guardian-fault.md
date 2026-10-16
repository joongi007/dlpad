# Linux guardian 자체 종료와 고장

2026-10-01, WSL2 Linux 6.18.33.2-microsoft-standard-WSL2. 합성 Python 프로세스와 임시 폴더만 사용했다. GUI·사용자 Documents·실제 영상·네트워크를 사용하지 않았다.

## 바뀐 동작

guardian에 직접 TERM/INT/HUP/QUIT가 전달되면 signal handler가 AtomicU8 강제 정리 요청만 남긴다. main loop가 기존 PGID, 아직 reap하지 않은 직접 leader, subreaper 입양 자손을 종료·회수한다. 외부 executable로 exec하면 handler가 유지되지 않으므로 도구의 신호 처리 정책을 변경하지 않는다. API/schema/Cargo 의존성 변경은 없다.

## 실제 명령과 결과

프로젝트 전용 Rustup/Cargo, toolchain 1.94.0을 사용했다.

```sh
cargo test -p dlpad-engine --test guardian --locked -- --test-threads=1
cargo clippy -p dlpad-engine --all-targets --locked -- -D warnings
cargo build -p dlpad-engine --bin dlpad-process-guard --release --locked
python3 scripts/guardian_probe.py target/release/dlpad-process-guard /tmp/dlpad-guardian-fault/term.json --fault guardian-sigterm
python3 scripts/guardian_probe.py target/release/dlpad-process-guard /tmp/dlpad-guardian-fault/kill.json --fault guardian-sigkill
```

- Rust guardian integration: 7 tests 통과, 3.25초. 추가된 2 tests는 네 가지 종료 신호와 SIGKILL 한계를 각각 검증한다. 기존 owner SIGKILL, 무관한 프로세스 생존, helper 누락, leader group 이동, detached pipe holder 정리도 통과했다.
- clippy 경고 없음, release helper 빌드 통과.
- release **standalone helper** 크기 400,136 bytes, RSS 2,032 KiB. 활성 3초 샘플 CPU 0.0%; 장시간 성능이나 GUI 앱 크기 측정이 아니다.
- SIGTERM probe: exit 0; leader/일반 자손/setsid 자손이 모두 종료·회수되었다. owner를 종료하기 전에 guardian pidfd로 종료를 확인하여 socket EOF가 대신 정리한 결과와 구별한다.
- SIGKILL probe: **exit 2 (보호 미충족)**. 직접 leader는 PDEATHSIG로 종료했지만 일반/setsid 자손 2개가 살아 있었다. `limitation_reproduced=true`, `protection_passed=false`. 시험 harness가 소유 pidfd로 남은 fixture만 종료하고 named waitpid로 회수했다. 테스트 통과는 한계를 정확히 재현했다는 뜻이며 제품 보호 성공이 아니다.

probe JSON은 임시 산출물이며 재실행할 수 있다. app의 새 release 바이너리는 통합 담당자가 전체 빌드 후 추가 검증한다.

## 남은 출시 차단 조건과 선택

SIGKILL은 handler를 실행하지 않는다. guardian 자체 fatal 고장/강제 종료 시 자손 전체 보호는 완료되지 않았다. 현재 parent-death 보호는 직접 child에 한정되며 fork 자손으로 상속되지 않는다. 따라서 guardian 단독 고장과 app/guardian 동시 고장을 분리해 검증해야 한다.

app가 살아 있으면 guardian exit를 stream drain join보다 먼저 감지하는 fallback을 검토할 수 있다. detached 자손이 pipe를 유지하면 EOF 기반 drain만으로 종료를 기다릴 수 없다. 안전한 fallback에는 앱 subreaper의 입양 자손 소유 분리 또는 guardian에서 받은 pidfd ownership 계약이 필요하다. PID 이름 검색이나 전역 process kill은 허용하지 않는다. 앱과 guardian이 모두 SIGKILL일 때에는 둘째 사용자 공간 감시자만 추가해도 보장되지 않으며, kernel-owned delegated cgroup/job scope 등 OS 소유 정책의 실험이 필요하다. 관리자 권한·시스템 보안 변경 없이 제공되는 범위를 먼저 확인해야 한다.

Windows/macOS, OS logout/sleep, guardian SEGV/abort, 8시간 실측은 이 작업에서 검증하지 않았다. TERM/INT/HUP/QUIT 개선만으로 S03/P0 완료를 보고하지 않는다.

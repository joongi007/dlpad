# 실제 제품 guardian user service 합성 검증

2026-10-03 현재 WSL boot `0bc0c937-eee9-4d90-95dc-31a2122b6352`에서 기존 immutable dlpad CPU 바이너리를 읽기 전용으로 사용했다. 제품 소스/Cargo/schema, 기존 8h 프로세스/unit, 사용자 desktop을 변경하지 않았다. 추가 설치/의존성은 없고 Python 표준 ctypes로 이미 설치된 libsystemd를 호출한다. Proposed 옵션의 실험이며 제품 통합/R08 전체 완료를 뜻하지 않는다.

## 실행과 증거

```text
python3 scripts/systemd_product_guardian_probe.py /tmp/dlpad-soak-binaries-0bc0c937-ep58_dk9/dlpad-cpu /mnt/d/project/dlpad/docs/validation/evidence/systemd-product-guardian-20261003-final
```

실제 exit 0, 8개 case PASS. 시작/끝 binary SHA256 `166946017de0b3d0589bf3d02bd864d004d90f3457ff2e314fe5e10a5f865417`가 동일하다. [최종 결과](evidence/systemd-product-guardian-20261003-final/result.json), 각 case JSON 및 별도 stdout/stderr, 실행된 source/helper snapshot을 보존했다. 초기 실행도 [별도 결과](evidence/systemd-product-guardian-20261003/result.json)에 보존했고 최종 실행은 endpoint owner PID/start identity도 직렬화했다. 초기 uppercase output 경로는 canonical 프로젝트 경로 계약에 맞지 않아 시작 전 exit2로 거절되었으며 이후 canonical 경로로 실행했다.

## 실제 계약과 결과

제품 `crates/engine/src/process/guardian.rs`는 FD>=3 control socket, readiness `R`, `T` graceful 후 200ms kill, 다른 byte `K` immediate kill/EOF shutdown, worker stdin null/stdout·stderr 상속, subreaper 및 child exit/signal 전파 계약을 가진다. 전용 Python wrapper는 FD0 socket을 F_DUPFD>=3 inheritable로 복제하고 원래 FD0를 닫은 뒤 실제 snapshot으로 exec한다. 모든 case에서 wrapper PID=service MainPID=실제 guardian PID이며 `/proc/PID/exe` dev/inode가 snapshot과 일치했다. 별도 launcher PID가 없다.

D-Bus StartTransientUnit(mode=fail) + ExecStartEx `a(sasas)`/`no-env-expand`, Type=exec, KillMode=control-group, Restart=no, RuntimeMaxSec=45s, TimeoutStopSec=2s를 사용했다. CollectMode=inactive로 실패 서비스 exit 증거를 잠시 유지하고 해당 InvocationID가 일치하며 ActiveState=failed인 경우에만 fixture reset-failed한다. 모든 최종 unit은 not-found였다.

| case | 전체 identity 소멸 ms | ExecMainCode / Status | 결과 |
| --- | ---: | --- | --- |
| eof | 41.14 | 2 / 9 | PASS |
| K | 40.86 | 2 / 9 | PASS |
| T | 252.67 | 2 / 9 | PASS |
| cancel | 60.90 | 2 / 9 | PASS |
| sigterm | 40.93 | 2 / 9 | PASS |
| sigkill | 2113.59 | 2 / 9 | PASS |
| owner_guardian_sigkill | 2130.35 | 2 / 9 | PASS |
| exit42 | 41.03 | 1 / 42 | PASS |

시간에는 20ms 폴링, 서비스 StopTimeout, 프로세스 관찰 오버헤드가 포함된다. guardian SIGKILL과 owner+guardian SIGKILL은 worker가 SIGTERM을 무시하도록 설정되어 StopTimeout=2s 뒤 manager가 cgroup 전체를 SIGKILL한 결과다. 동시 종료 case는 두 exact pidfd에 연속 SIGKILL을 보내며 원자적인 동시 syscall이라고 주장하지 않는다. EOF/K/T/cancel/TERM은 실제 제품 guardian의 cleanup, SIGKILL은 manager의 cgroup cleanup을 검증했다. cancel은 control T 뒤 20ms에 K를 보냈고 UI/엔진 cancel 통합은 별도다.

각 case의 guardian/worker/setsid child/leaf는 같은 고유 cgroup과 정확 PID/start identity를 확인한 뒤 pidfd를 열었다. owner case의 endpoint owner는 별도 소유 Popen이며 endpoint를 인계한 뒤 테스트 부모 endpoint를 닫았다. 모든 원래 identity가 zombie까지 남지 않고 없어졌고 모든 pidfd가 ready, cgroup이 제거됐다. 외부 소유 sentinel은 전체 시험 중 원래 identity로 살아 있었고 끝에 자체 종료/wait했다. 소유 temporary directory도 제거됐다.

7개 literal argv($VAR, ${VAR}, prefix substitution 문자열, 공백, 한글, %n, quote)는 실제 worker에서 보존됐다. worker stdin은 /dev/null, stdout/stderr는 각각 정확히 21B 합성 marker이며 분리된 fd를 통과했다. exit42 case는 service ExecMainCode=1/ExecMainStatus=42, 종료/kill case는 2/9로 실제 worker 종료 signal이 guardian을 통해 전파됐다. 로그는 이 합성 worker의 고정 출력이며 64KiB 상한을 사후 검증했다. 임의 악성 도구의 무한 출력 제한/제품 bounded drain을 이 테스트로 검증했다고 주장하지 않는다.

## 실패 입구와 미검증

[입구 실패 결과](evidence/systemd-product-guardian-20261003-final/admission-failures.json): 프로젝트 밖 경로, 기존 evidence, `..` traversal, 틀린 snapshot SHA는 각각 실행 전 exit2이며 새 output을 만들지 않았다. 실패 시에는 확인한 소유 pidfd만 정리하며 RuntimeMaxSec는 owner 자체 예외에 대한 서비스 유한 수명 상한이다.

실제 네이티브 UI/다운로드/zero-exit status/live output pipe/backpressure/missing user manager/다른 OS는 이 실험 범위 밖이다. 기존 raw systemd-run argv 실패를 덮어쓰거나 shell escaping으로 우회하지 않았다. 다음 결정은 root의 제품 service lifecycle, exit-status retention와 output/owner semantics 통합 및 회귀 시험이다.

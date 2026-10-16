# Linux user-service 생산 engine 통합 검증

2026-10-03 현재 boot `0bc0c937-eee9-4d90-95dc-31a2122b6352`에서 `DLPAD_GUARDIAN_BACKEND=user-systemd`를 실제 production process::run/run_metadata 경로로 검증했다. 기본 cargo test는 user manager가 필요하지 않으며 Rust `user_service_driver`는 ignored이고 명시적으로 선택 실행한다. 환경 변수는 각 test binary subprocess에만 지정하므로 test process 전체의 환경 mutation race가 없다. 기존 8h/다른 unit/global 설정/사용자 desktop을 대상으로 한 동작은 없다.

## 실제 명령과 결과

root가 READY compile 후 정확 test binary/helper를 고정했다. 첫 primary3case(exit0,exit42,metadata)가 모두 통과한 뒤 새 evidence directory에서 전체13case를 실행했다.

```text
python3 scripts/user_service_engine_probe.py /mnt/d/project/dlpad/target/debug/deps/user_service_guardian-85c63d951435e313 /mnt/d/project/dlpad/docs/validation/evidence/user-service-engine-primary-20261003 --case exit0 --case exit42 --case metadata
python3 scripts/user_service_engine_probe.py /mnt/d/project/dlpad/target/debug/deps/user_service_guardian-85c63d951435e313 /mnt/d/project/dlpad/docs/validation/evidence/user-service-engine-full-20261003
```

둘 다 실제 exit0/PASS. [primary 원시](evidence/user-service-engine-primary-20261003/result.json), [전체 원시](evidence/user-service-engine-full-20261003/result.json), 각 case JSON/driver log, 실행된 probe/helper/driver source를 보존했다. test SHA256 `f63251eb99b4f00e5841ef9aa08d542f3c47caaf40e95e94b59d96558304188c`, helper SHA256 `722cb555801827a33c762494bb1ce27be998240f239ead22015376bd63a71ccb`는 실행 시작/끝 동일했다. product module/Cargo 컴파일과 공유 source는 root/metrics 담당이며 이 작업은 test/harness/docs만 수정했다.

| case | 실제 engine 결과 | driver 경과 ms | 결과 |
| --- | --- | ---: | --- |
| exit0 | Exited / 0 | 548 | PASS |
| exit42 | Exited / 42 | 546 | PASS |
| metadata | Exited / 0 | 552 | PASS |
| cancel | Cancelled / None | 859 | PASS |
| timeout | TimedOut / None | 947 | PASS |
| owner_sigkill | owner SIGKILL | — | PASS |
| guardian_sigkill | Exited / None | 2199 | PASS |
| owner_guardian_sigkill | owner SIGKILL | — | PASS |
| missing_manager | UnexpectedEof | 10 | PASS |
| unknown_backend | Other | 1 | PASS |
| fdleak | Exited / 0 | 1765 | PASS |
| backpressure | Exited / 0 | 655 | PASS |
| outputlimit | OutputLimit / None | 747 | PASS |

## 검증한 계약

exit0/42는 별도 세션의 합성 child/leaf가 stdout/stderr를 보유해도 service+실제 guardian cleanup 후 pipe EOF에 도달했고 종료 코드 0/42를 보존했다. metadata는 한글/$VAR/${VAR}/prefix/공백/%n/quote argv와 fake caller environment `expanded sentinel`를 실제 worker에서 그대로 읽었고 stdin=/dev/null, stderr marker를 확인했다.

cancel은 readiness 이후 500ms에 실제 Arc<AtomicBool>를 설정하여 process::run이 Cancelled를 반환했다. timeout은 실제 ProcessSpec 600ms에서 TimedOut이었다. 경과 시간은 fixture startup, graceful stop, manager wait/drain을 포함하므로 timeout 값과 정확히 같다는 주장은 하지 않는다.

owner SIGKILL은 test driver pidfd만, guardian SIGKILL은 확인된 고유 unit InvocationID/MainPID/start/cgroup의 guardian pidfd만 신호를 보냈다. 두 프로세스 case는 owner/guardian pidfd에 연속 SIGKILL을 보내며 원자적인 동시 종료를 주장하지 않는다. guardian SIGKILL은 SIGTERM 무시 worker 때문에 TimeoutStopSec=2s 포함 약2.2s 후 정리되었다. guardian crash의 engine 결과는 Exited/exit_code=None이며 성공 코드 0으로 바뀌지 않았다. owner crash에는 결과 JSON이 없고 driver -9 및 외부 관찰 cleanup 증거로 판단했다.

고유 service를 관찰한 10case 모두 worker/setsid child/leaf/guardian의 동일 cgroup 및 exact PID/start를 확인하고 pidfd를 열었다. owner는 별도 소유 Popen이다. 전체13case의 기록된 53개 identity는 원래 PID가 zombie까지 남지 않고 사라졌으며 pidfd가 ready였다. 관찰한 10개 cgroup이 제거되고 해당 unit은 not-found로 GC됐다. [종료 후 재검사](evidence/user-service-engine-full-20261003/post-run-verification.json)에서도 모든 기록 PID 경로/cgroup 부재와 실행 source SHA 일치를 확인했다. 외부 소유 sentinel은 원래 identity로 살아 있었고 끝에 자체 종료/wait했다. 소유 temporary 폴더도 제거됐다.

backpressure는 stdout와 stderr에 각각 20,000×101B(각2,020,000B)를 실제 pipe로 보내고 deadlock 없이 exit0, retained log 4096B/truncated=true였다. outputlimit은 100,000B 단일 line으로 실제 OutputLimit을 반환했다. 12회 빠른 서비스 실행 FD 시험은 각 exit0이며 driver FD count 시작4/끝4였다. 다른 정상 driver case들도 4→4였다. FD count는 process descriptor 누적 검증이고 메모리/모든 manager 리소스 장기 leak benchmark를 대신하지 않는다.

## 실패 경로와 한계

missing manager는 debug-only `DLPAD_TEST_SYSTEMD_BUS_PATH`에 fixture 소유의 존재하지 않는 절대 socket path를 지정했다. 실제 host manager나 사용자 설정은 바꾸지 않았다. engine은 UnexpectedEof(10ms)를 반환했고 worker 최초 tool-started marker가 없었다. unknown backend는 unsupported Linux guardian backend 오류(1ms), marker 부재였다. user manager 없는 환경의 전체 제품 UI/오류 문구 전달은 후속 범위이며 readiness 오류가 진단 원인을 얼마나 전달하는지 별도 평가가 필요하다. release에는 debug missing-manager hook이 없다는 source 계약을 준수했다.

이 결과는 해당 Linux host의 user-service backend 통합 capability다. 전체 R08/P0 채택, Windows/macOS, 실제 미디어/8h/GUI, unit restart race/manager failure 중단, 장기 RSS/FD/manager leak 및 release mode test는 root 통합 검증과 별개다. 기본 backend 변경이나 ADR 채택은 이 작업에서 하지 않았다.

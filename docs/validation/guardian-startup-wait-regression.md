# startup child wait 실제 공개 경로 및 manager 회귀

2026-10-03 root가 새 StartupPermit/reaper 및 bounded child wait 구현의 모델/공개회귀/clippy/driver/helper READY를 제공한 뒤 실제 시험을 실행했다. 이 작업은 새 public helper harness/work-item/docs만 추가했고 기존 engine/public Rust driver 또는 제품/Cargo는 수정하지 않았다. 원시 기존 strict13/startup4/GUI3 자료도 보존했다.

## 실제 명령

```text
python3 scripts/user_service_engine_probe.py /mnt/d/project/dlpad/target/debug/deps/user_service_guardian-85c63d951435e313 /mnt/d/project/dlpad/docs/validation/evidence/guardian-startup-wait-native-20261003/strict13
python3 scripts/user_service_engine_probe.py /mnt/d/project/dlpad/target/debug/deps/user_service_guardian-85c63d951435e313 /mnt/d/project/dlpad/docs/validation/evidence/guardian-startup-wait-native-20261003/startup4 --case startup_after_start --case startup_after_get_unit --case startup_after_invocation --case missing_manager
python3 scripts/public_startup_helper_probe.py /mnt/d/project/dlpad/target/debug/deps/user_service_guardian-85c63d951435e313 /mnt/d/project/dlpad/docs/validation/evidence/guardian-startup-wait-public-20261003
```

세 명령 실제 exit0/PASS: manager strict13+startup4=17case, public EP/noR=2case. [strict13](evidence/guardian-startup-wait-native-20261003/strict13/result.json), [startup4](evidence/guardian-startup-wait-native-20261003/startup4/result.json), [public2](evidence/guardian-startup-wait-public-20261003/result.json)에서 결과/source/log 및 종료 재검사를 보존했다. native test driver SHA `c0e9d0fa393cb221d73c5b85efa16510d49af90918d2fb1e53987660fbc7ec35`, 실제 helper SHA `a33ee5adf153f45ab6dcb4a7faf8ef5244ece37a5f2c20064449686a6980cd45`는 각 실행 전후 동일했다. public copied driver도 같은 native SHA였고 원본은 읽기 전용이었다. root가 별도로 실행한 guardian_startup_wait test binary의 결과와 이 범용 user_service_driver를 통한 결과를 구분한다.

## 실제 manager17 회귀

strict13의 exit0/42, metadata literal/Unicode argv·환경/stdin null, actual cancel/timeout, owner/guardian/두 프로세스 crash, missing-manager fixed EP/unknown backend, FD12회4→4, 4.04MB concurrent stdout/stderr drain/4096B retained, oversized line OutputLimit 모두 통과했다. startup3fault는 live worker/guardian/setsid child/leaf와 고유 unit Inv/MainPID/start/cgroup을 확인하고 R-before-launcher-failure Exited126을 반환했다. missing-manager는 fixed Other/보호 setup 오류이며 최초 tool marker가 없었다.

strict13의53기록 identity/10cgroup, startup4의16identity/3cgroup이 정확히 없어졌고 해당 고유 service가 not-found GC, pidfd ready, 외부 owned sentinel 보존을 확인했다. 각 source SHA snapshot과 재검사 원시도 보존했다. test env는 subprocess별로 지정하고 user manager/global settings/기존 unit/8h/user desktop을 변경하지 않았다.

## 공개 process::run EP/noR 실제 child cleanup

production executable override는 필요하지 않았다. 소유 temporary folder에 실제 integration test executable을 복사하고 그 옆 dlpad-process-guard만 전용 Python fake helper로 두어 기존 adjacent helper 선택 계약을 이용했다. mode=direct는 unregistered standalone development driver에서만 선택하며 앱 기본 backend를 약화한 것이 아니다. fake helper/driver mode0500과 exact PID/start/parent/pidfd를 검증했다. 제품 파일/원본 helper/driver를 덮어쓰지 않았다.

fake helper는 작은 stdout/stderr marker를 쓰고 EP를 보내거나 R 없이 기다렸다. 공개 process::run이 원본 endpoint를 닫는 것을 socket EOF 기록으로 확인한 뒤에도 fake helper가30초 잠들어 pipe writer를 유지하게 했다. 자연 종료가 아니므로 실제 자연 grace5초→owned kill/reap이 필요했다.

| case | 부모 관찰 sec | engine ms | 실제 오류 | 결과 |
| --- | ---: | ---: | --- | --- |
| EP-sleep | 5.054 | 5036 | Other / Linux user-service protection setup failed | PASS |
| no-R | 15.127 | 15089 | WouldBlock / errno11 | PASS |

no-R는 실제 attach readiness10초 뒤 EOF, 추가 natural grace5초 뒤 owned kill/reap이므로 약15.1초였다. 이는 현재 startup failure 정책의 실제 관측이며 정상 cancel/timeout latency로 표현하지 않는다. 상위 harness deadline23초 안에 반환했다. EP-sleep은 readiness fixed frame 즉시 처리 뒤 grace5초에 약5.05초 반환했다. helper가 EOF를 기록한 시점 이후 자연 grace가 있었고, helper/owner exact identity가 zombie 없이 사라졌으며 pidfd ready, driver FD4→4, 실제 지정 tool은 시작하지 않았고 driver logs144/145B, unrelated process sentinel 보존/자체 wait, 소유 temp folder 제거를 확인했다.

## 경계

public2는 malformed startup helper에 대한 manager-independent public cleanup 시험이고 service cgroup 검증으로 확대하지 않는다. service cgroup/pipe 반환/GC는 manager17에서 별도로 검증했다. [전체 요약](evidence/guardian-startup-wait-native-20261003/summary.json)은 native17+public2 총73기록 identity/13관찰 cgroup과 동일 driver SHA를 기록한다.

kernel에서 SIGKILL에 즉시 반응하지 않는 child의 실제 deferred reaper 이관/queue full, manager 재시작/race, registered release GUI/latest startup-wait, 다른 OS/실제 media/장기8h는 이 두 harness가 검증한 범위를 넘어선다. root의 별도 모델/public unit 및 전체 workspace/release 검증과 함께 인수해야 한다.

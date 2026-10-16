# accepted user-service startup 실패 및 alias 회귀

2026-10-03 root가 원본 control socket alias의 error 시 EP/shutdown(Both)와 metrics의 debug-only startup fault hook을 구현한 뒤 실제 engine 경로를 검증했다. hook은 정확 after-start/after-get-unit/after-invocation 선택 시에만 1초 관찰 창을 준다. service가 실제 도구와 setsid 자손을 시작한 상태를 관찰하지 못하면 FAIL/unverified로 판단하도록 했다. 제품 source/Cargo는 root/metrics 담당이고 이 작업은 기존 범용 ignored Rust driver를 수정하지 않고 전용 Python harness/work-item/docs/evidence만 확장했다.

## 명령과 원시 증거

root unit/clippy/helper/driver READY 후 binary/helper를 freeze하여 실행했다.

```text
python3 scripts/user_service_engine_probe.py /mnt/d/project/dlpad/target/debug/deps/user_service_guardian-85c63d951435e313 /mnt/d/project/dlpad/docs/validation/evidence/user-service-startup-fault-20261003 --case startup_after_start --case startup_after_get_unit --case startup_after_invocation --case missing_manager
python3 scripts/user_service_engine_probe.py /mnt/d/project/dlpad/target/debug/deps/user_service_guardian-85c63d951435e313 /mnt/d/project/dlpad/docs/validation/evidence/user-service-engine-alias-regression-20261003
```

둘 다 실제 exit0/PASS. [startup4 원시](evidence/user-service-startup-fault-20261003/result.json)와 [strict13 회귀 원시](evidence/user-service-engine-alias-regression-20261003/result.json), case별 JSON/driver logs, 당시 실행한 probe/helper/driver source를 보존했다. test SHA256 `78358950b695bbfaf694af078b61a3110fe832bb34c806ddbb955f58ccf1b9d0`, dlpad-process-guard helper SHA256 `4135df3b75a28e5e60c2fa6516d3759a945c1751bd0c2658b3447a00ac3e2a07`는 각 실행 전후 동일했다. 이전 engine13/raw UnexpectedEof, 실제 CPU 앱 default GUI3flow와 그 source/evidence는 수정하지 않았다.

## startup 실패 실제 결과

| case | 실제 failure path | driver 경과 ms | FD | 결과 |
| --- | --- | ---: | --- | --- |
| startup_after_start | R-before-launcher-failure | 1059 | 4→4 | PASS |
| startup_after_get_unit | R-before-launcher-failure | 1062 | 4→4 | PASS |
| startup_after_invocation | R-before-launcher-failure | 1061 | 4→4 | PASS |
| missing_manager | EP setup error | 11 | 4→4 | PASS |

세 startup case 모두 실제 worker/guardian/setsid child/leaf가 살아 있는 1초 창에서 고유 unit InvocationID/MainPID/start/cgroup을 검증하고 정확 pidfd를 열었다. 원래 literal argv/환경/stdin null도 유지됐다. readiness R가 먼저 도착하여 process::run은 Exited/exit_code=126을 반환했고 bounded log에 user-service synthetic startup failure를 기록했다. 이를 EP setup error와 혼동하거나 성공 코드 0으로 판단하지 않았다. 경과 약1.06초에는 인위적 debug 관찰 지연1초가 포함되며 production startup latency라고 해석하지 않는다.

launch 초기화 오류가 원본 socket을 shutdown한 뒤 모든 도구/자손/guardian이 없어졌고 output pipe reader가 유한 시간 내 반환했다. 3개 고유 cgroup 제거/unit not-found GC, exact pidfd ready, 3case 각 owner까지5개 identity 종료를 확인했다. [종료 후 재검사](evidence/user-service-startup-fault-20261003/post-run-verification.json)는 missing-manager owner까지 총16개 기록 PID 경로 부재 및3cgroup 부재를 다시 확인했다. FD 각4→4, retained log 각97B, driver log 각144B였고 무관 소유 sentinel은 동일 identity로 살아 있다가 시험 끝에 자체 종료/wait했다. 소유 temporary 폴더도 제거됐다.

missing manager는 기존 debug bus override를 소유 nonexistent socket 경로로만 지정했다. 실제 host/user manager 또는 환경 설정을 변경하지 않았다. 11ms에 Other / `Linux user-service protection setup failed` fixed EP 오류를 정확히 assert했고 최초 tool-started marker가 없었다. 기존 최초 결과의 UnexpectedEof가 개선됐음을 별도 새 증거로 기록했다.

## strict13 회귀와 범위

같은 새 alias/helper에서 exit0/42/metadata/cancel/timeout/owner SIGKILL/guardian SIGKILL/owner+guardian SIGKILL/missing-manager/unknown backend/FD12회/backpressure/outputlimit의 strict13도 모두 PASS했다. 이번 missing-manager predicate는 새 Other fixed 오류를 반드시 요구한다. 4.04MB stdout/stderr drain은4096B retained/truncated와 정상 exit0, oversized line은OutputLimit, FD12회4→4였다. [추가 재검사](evidence/user-service-engine-alias-regression-20261003/post-run-verification.json)는 총53기록 identity PID 경로 및10관찰 cgroup 부재, 실행 source SHA 일치를 확인했다.

이는 Linux debug production engine 및 accepted-unit failure cleanup의 실제 관측이다. startup3case는 모두 R-before-error 경로였고 EP-before-R는 missing-manager 실패에서 검증했다. release의 fault env 무시, 실제 GUI의 absent-manager 안내, manager 재시작/race/외부 unit 변경, 이 새로운 backend 장기8h 및 다른 OS를 이 결과만으로 완료라고 하지 않는다. root의 전체 workspace/release/ADR 통합 검증과 구분한다.

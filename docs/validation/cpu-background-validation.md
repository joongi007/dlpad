# CPU 백그라운드 Linux 실제 GUI 검증

2026-10-03, guardian_fault. 소유 Xvfb/private D-Bus와 합성 tool만 사용했다. 제품/공유 harness는 수정하지 않았다.

## 입력과 실행

고정 CPU executable SHA256 `c4dbd4171f8700d7f4f6dc3c2df536f7725830a17fce5075ae5a92bb68dc03af`. accessible/software_host/CPU entry/Cargo.lock 및 binary의 실행 전후 5개 hash가 일치했다.

```sh
python3 scripts/cpu_background_probe.py --binary target/release/dlpad-cpu --expected-sha c4dbd4171f8700d7f4f6dc3c2df536f7725830a17fce5075ae5a92bb68dc03af --output docs/validation/evidence/cpu-background/all-1 --case all
python3 scripts/cpu_background_probe.py --binary target/release/dlpad-cpu --expected-sha c4dbd4171f8700d7f4f6dc3c2df536f7725830a17fce5075ae5a92bb68dc03af --output docs/validation/evidence/cpu-background/unavailable-2 --case unavailable
```

## 실제 결과

첫 실행 세션 97457 exit1: active-hidden, complete-hidden, stop-exit PASS. 숨긴 동안 heartbeat 증가/동일 worker identity, 재실행 restore exit0/원 app identity/같은 job과 1 attempt, 숨긴 채 Downloaded 뒤 복귀, 실제 중단 후 종료와 worker cleanup을 확인했다.

unavailable 첫 실행은 AT-SPI Enabled/Sensitive 비트가 켜져 있어 assertion 실패했다. 실제 PNG에는 비활성 버튼과 복귀 불가 안내가 보인다. 제한 대조 세션 38128 exit0에서는 해당 버튼을 실제 클릭해도 창이 visible이고 같은 worker가 유지됨을 확인한 뒤 돌아가기/중단 후 종료를 완료했다. 기존 synthetic activation.sock의 inode/hash가 teardown 이후에도 그대로였다. **접근성 disabled state 불일치는 미해결**이며 실제 기능 PASS로 덮지 않는다.

각 원시 trial의 result.json, PNG, jobs.sqlite, bounded app/bus log를 `evidence/cpu-background/all-1/` 및 `unavailable-2/`에 보존했다. 최초 실패를 삭제하지 않았다. `terminal-review.json`은 5 trial의 모든 app/bus/secondary/worker host PID가 없고 SQLite integrity_check가 ok임을 확인한다. source-before.sha256/source-after.sha256은 동일하다. 로그 64KiB/합성 worker 120초 제한, job partial 64KiB 제한을 사용했다.

## 범위와 한계

Linux CPU native GUI 합성 다운로드 기능 검증이다. 실제 영상/사이트/계정, Windows/macOS CPU 백그라운드, 트레이 및 전체 P0 완료를 주장하지 않는다. 사용자 display/clipboard/입력 desktop을 사용하지 않았다. 기존 8시간 run/archive를 재시작하거나 종료하지 않았다.

## callback-only 후속 회귀

2026-10-03 notify callback-only binary SHA256 `b8bbf93a2cc4b05dfe202edfe1c92fca6aff7566b694eb63f4427df7d7a87409`로 active-hidden/complete-hidden을 기존 harness의 --case 지정으로 실행했다. 세션42342/66268 각각 exit0. 증거 prefix는 cpu-background/callback-active와 callback-complete이다. 같은 worker/heartbeat 증가, secondaryrestore0/같은 app과 job·1attempt, hidden Downloaded 후 복귀, StopExit 또는 idleclose cleanup이 PASS했다. 두 case의 모든 소유 app/bus/secondary/worker host PID는 없고 SQLite integrity ok, 5개 source/binaryhash는 실행 초기와 종료 후 동일하다. callback-terminal-review.json과 callback-source-before/after.sha256을 보존했다. unavailable/다른 OS는 이번 후속 2case 결과에 포함하지 않는다.

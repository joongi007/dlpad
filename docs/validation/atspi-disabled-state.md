# AT-SPI disabled 상태 검증

locked accesskit_atspi_common0.14.2 src/node.rs348–351는 read-only 지원 여부가 false인 Button에 disabled 여부와 무관하게 Enabled/Sensitive를 넣는다. consumer0.31.0 is_read_only_supported()는 Button을 포함하지 않는다. 기존 실제 unavailable PNG는 비활성이나 GetState=1124075776은 Enabled/Sensitive를 포함했다.

최소 수정 제안은 기존 ReadOnly 조건을 유지하고 else를 `else if !state.is_disabled()`로 제한한다. TextInput readonly 정책과 Editable 부수 의미는 이 작은 수정 범위 밖이다. root가 vendor/Cargo 조정을 담당한다.

회귀 설계: 실제 accesskit_consumer::Tree에 Window root와 Button 또는 TextInput child를 구성하고 read_only×disabled 4조합을 순회한다. NodeWrapper::state(false)의 Enabled/Sensitive는 `!disabled && !(read_only_supported && read_only_or_disabled)`와 일치해야 한다. ReadOnly는 기존 조건을 유지한다. 단순 bool 구현복제 대신 consumer의 역할별 해석을 사용하는 8개 경계 fixture이다. scratch 제안 코드는 /tmp/dlpad-atspi-disabled-regression.rs에 작성했다. 아직 cargo 실행/제품 보정 성공/새 GUI 성공을 주장하지 않는다.

## 최신 실제 결과

root가 최소 vendor 보정 및 실제 consumerTree 8fixtures/1test PASS, CPU59tests/clippy/release PASS를 확인한 뒤 READY했다. 정확 CPU SHA256 `522e2304fc38618df696cfca1f126e08bc77634f48ac1db631d3c797fe8f8cbd`.

```sh
python3 scripts/cpu_background_probe.py --binary target/release/dlpad-cpu --expected-sha 522e2304fc38618df696cfca1f126e08bc77634f48ac1db631d3c797fe8f8cbd --output docs/validation/evidence/atspi-disabled-state/unavailable --case unavailable
python3 scripts/cpu_background_probe.py --binary target/release/dlpad-cpu --expected-sha 522e2304fc38618df696cfca1f126e08bc77634f48ac1db631d3c797fe8f8cbd --output docs/validation/evidence/atspi-disabled-state/active-hidden --case active-hidden
```

세션54215/4946 각각 exit0 PASS. unavailable은 disabled Background의 Enabled/Sensitive 없음, 실제 클릭해도 visible 유지, 안내/foreign endpoint identity/hash 보존 및 StopExit cleanup을 확인했다. 두 case 모두 active URL TextInput의 Enabled/Sensitive 없음 assertion 통과했다. active-hidden은 heartbeat 증가/동일 worker 유지/secondary restore0/같은 app과 job·1attempt/StopExit PASS. 기존 최초 실패 증거는 cpu-background/all-1에 그대로 보존했다.

새 evidence의 각 case result.json/PNG/jobs.sqlite와 terminal-review.json, source-before/after.sha256에 실제 원시 증거를 남겼다. 6개 binary/source hash 일치, 모든 소유 app/bus/secondary/worker host PID 없음 및 SQLite integrity ok를 확인했다. Linux 격리 GUI 범위이며 Windows/macOS, 실제 스크린리더 낭독과 다른 역할 전체를 검증한 것으로 확대하지 않는다.

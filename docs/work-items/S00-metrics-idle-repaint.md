# S00: 지표 freshness를 유지하는 표시 변경 재그리기

- 상태: in-progress (root 통합·관련 회귀 통과; 전체 회귀·실측 진행)
- 분류: performance
- 우선순위: P0
- 요구사항: R01, R08, R11
- 담당/수정 범위: soak_recovery_audit 초안; root 통합·검증 — crates/app/src/accessible_metrics.rs, 본 문서, 전용 evidence
- 선행 작업: root loaded cost 기준 표본, accessible_metrics 원 소스; 통합·native 재측정 root 담당

## 사용자 결과

표시 값이 같으면 불필요한 지표 repaint를 요청하지 않으면서 실제 volume/network 표본과 freshness는 1초마다 갱신한다.

## 계약과 범위

loaded 비용 CPU round2/3 1.08/1.25%, empty0.31%는 기존 관측이며 1Hz metrics repaint가 원인이라는 결론은 아직 없다. private 인터페이스·중앙 문자열·GiB/KiB 표시를 유지한다. channel1의 오래된 표본 거절 대신 최대1개 Arc Mutex mailbox를 사용하여 미소비 중에도 최신 timestamp/revision을 대체한다. UI는 긴 I/O를 하지 않으며 worker의 OS 관측 주기1초/stop 확인50ms/stale3초를 유지한다.

## 인수 조건

표시 format이 동일한 fresh sample은 timestamp를 대체하고 wake가 없어야 한다. 표시 값·revision·관측 실패 상태 변화는 wake해야 한다. stale timer poll은 mailbox의 최신 timestamp를 먼저 읽어 오래된 timestamp로 unknown을 표시하지 않아야 한다. worker panic/failure와 정상 stop은 원 ACK 및 repaint를 유지한다. queue/thread 수 증가 없이 최신 sample 최대1개다. 실제 비용 개선은 root의 격리 native 재측정 전 미검증이다.

## 검증과 handoff

소유 /tmp 사본 rustfmt만 허용하고 Cargo/native/프로젝트 Rust는 수정·실행하지 않는다. meaningful mailbox seam 회귀와 기존 선택/오래된 revision/stale/panic/stop 회귀를 준비해 root에 넘긴다. poison을 정상 표본으로 처리하지 않는다.

원 `accessible_metrics.rs` SHA `0aa7ae51ca9f9173b0d67ddedc2a847f3c9db6981d45855f77df13fea17165d5` 불변을 확인했다. /tmp 사본 SHA `eb7a966934f507e438ad578a0cc0d44130bcbe2c5fd39ac5802b70f370e82272`, rustfmt exit0. 결과 mailbox는 Sample 최대1개이며 UI의 최근 복사본도 고정1개다. UI try_lock 경합에는 기존 latest를 유지하고 stale timer를 유지한다. poison에는 failed/stop을 표시한다. 관측 실패 bool이 바뀌면 unknown 표시가 같아도 repaint하여 실패 전이를 숨기지 않는다.

기존5개 회귀를 mailbox로 연결하고 새4개 회귀(표시 동일 fresh 대체 및 stale poll, 값/revision/unknown/error 전이, network 표시 rounding, 정상·panic 실제 egui repaint callback/ACK)를 준비했다. stop ACK는 stopped.store 뒤 repaint하며 panic에도 수행한다. UI stale 타이머가 약3초마다 poll할 수 있으므로 모든 idle wake를 없앴다는 주장은 하지 않는다. 실제 컴파일·회귀 실행·loaded CPU 개선은 미검증이다.

## root red/green 실행

root는 원worker에 fresh_metrics_survive_a_paused_ui_consumer를 먼저 추가했다. Windows cargo test -p dlpad --features cpu --locked fresh_metrics_survive_a_paused_ui_consumer는 compile성공 후실제assert FAILED/exit101이다. 4.25초UI미소비 후sample이오래돼없어지는actuallatest-only실패를재현했다. 초안SHA검증·통합후 같은회귀를유지하고cargo test -p dlpad --features cpu --locked accessible_metrics::tests를실행해두후보의관련전체시험을통과했다. fresh표본을덮어쓰면서동일표시wake억제, value/revision/error변경, terminalACK/panic을유지한다. 원IO진단3case는read/write operations/bytes증가0이므로측정중fullscan만으로CPU초과를설명하는가설은약해졌다. native CPU 개선/메모리예산은아직미검증이고전체workspace회귀를진행한다.

후속전체WindowsGNU CPU fmt/clippy/test/release 네명령 PASS를root가currentSHA로확인했다. 동일diagnosticcollector/warmup2조건 재측정CPU0.46482/0.93230/0.78064%로세표본모두1%내다. WS84467712/79474688/79187968bytes이며첫표본80MiB초과로전체FAIL을유지한다. 원CPU변경효과를단일원인·장시간성능으로확대하지않고quietHostfalse유지. windows-library-cost-metrics-warmup2-20261005-resume/root-review.json에기록했다. 최신표본기능은검증됐으나전체S00성능·메모리·스택선정은미완료다.

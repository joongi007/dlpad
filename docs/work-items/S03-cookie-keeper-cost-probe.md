# S03: cookie keeper 단독 프로세스 비용 probe

- 상태: completed (release 단독 helper 측정 범위)
- 분류: spike
- 우선순위: P0
- 요구사항: R04, R11, R13
- 담당/수정 범위: metrics_recovery — 신규 scripts/cookie_keeper_cost_probe.py 및 이 문서
- 선행 작업: 현재 debug dlpad-process-guard의 keeper argv 계약

## 계약과 인수

실제 keeper child를 argv로 실행하고 자체 readonly pipe를 stdin에 연결한다. parent만 CLOEXEC writer를 유지하고 입력 bytes 없이 EOF로 종료한다. private /tmp fixture 및 fresh project evidence만 사용한다. 다섯 bounded round에서 helper/source SHA·크기, PID/startticks, RSS/PSS, FD 수, 10초 idle CPU tick delta/monotonic duration, spawn→sample 및 EOF→reap 지연을 기록한다.

/proc 관측 실패를 0으로 바꾸지 않는다. startup의 blocked pipe read를 bounded 확인하고 idle 구간에는 sampler/watchdog polling을 넣지 않는다. cleanup은 자기 Child/pipe만 종료·reap한다. 이 단독 helper 비용은 systemd manager/unit·앱/launcher/guardian 전체 비용을 대표하지 않는다. root가 실행하며 이 담당은 Python 문법 검증만 한다. Cargo/Rust/기존 probe 수정 금지.

## 검증과 handoff

구현 전 계약 기록. root 실제 실행 결과를 받을 때까지 비용 수치를 주장하지 않는다.

## 구현 handoff

생성한 child의 PID/startticks, executable dev/ino 및 exact argv를 확인하고 2초 startup 상한 내 pipe_read blocked 상태를 요구한다. 10초 idle 동안은 sample polling을 하지 않고 앞뒤 CPU ticks/monotonic 시간만 비교한다. 각 RSS/PSS 관측은 smaps_rollup의 실제 두 필드가 모두 있어야 하며 실패는 round failed다. helper의 원 SHA/size/dev/ino를 시작·끝에 대조한다.

EOF 뒤 5초 종료 관찰, 실패 정리는 소유 child kill 및 2초 reap 관찰로 제한한다. parent는 입력 byte를 쓰지 않는다. 자체 pipe/FD/임시 directory만 정리한다. round evidence 64KiB 상한, proc read 64KiB 상한, FD 열거 4096 상한을 유지한다. probe source/helper bytes 및 SHA와 parent FD 전후를 기록한다.

`python3 -m py_compile scripts/cookie_keeper_cost_probe.py` 실제 exit 0. SHA256 `b845e083fce125f55082b91e3e2bcdd17fa59aea5627b3429bde2558d715e41d`로 root에 동결 전달했다. 제안 실행은 `python3 scripts/cookie_keeper_cost_probe.py /mnt/d/project/dlpad/docs/validation/evidence/cookie-keeper-direct-cost-first`다. 실제 keeper 실행이나 비용 측정은 하지 않았으므로 RSS/CPU 수치를 아직 주장하지 않는다. 해당 실행은 root 담당이다.

## root 실제 측정

`cookie-keeper-release-direct-cost-first`에서 immutable /tmp release helper 사본525256bytes를 사용했다. probe exit0/5roundPASS/helper SHA 유지/FDbaseline4→4/각 keeper FD3/EOF exit0을 원시 JSON에서 대조했다. RSS/PSS 및 지연 요약: `{"rss_kib_range": [2124, 2144], "pss_kib_range": [610, 632], "cpu_ticks_deltas": [0, 0, 0, 0, 0], "spawn_sample_ms": [11.795, 10.898, 12.183, 13.875, 11.081], "eof_reap_ms": [5.324, 5.678, 5.419, 5.314, 5.46]}`. CPU는 각10초 idle 구간에서100Hz tick delta0이었으며 더 미세한 사용량0을 보장하지 않는다. systemd unit·앱 전체 비용과 실제8시간 성능은 별도 인수다.

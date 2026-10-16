# WSL 재접속 복구 체크리스트

이 문서는 실행 절차의 읽기 감사 결과다. 2026-10-04 작성 시 새 collector, WSL 재시작, 전원 설정 변경, 서비스/watchdog 설치 및 프로세스 종료는 실행하지 않았다. WSL VM이 종료된 동안 내부 agent나 시험이 계속 실행됐다고 판단하지 않는다. 이전 종료 요청자는 여전히 미확인이다.

## 1. 작업과 원시 증거를 먼저 보존한다

AGENTS.md → README.md → PRODUCT.md → CONTEXT.md → 관련 요구사항/ADR을 다시 읽고 현재 workitem 담당 범위를 확인한다. 정상 Git 상태가 없으면 기록하고 기존 .git을 삭제하거나 재구성하지 않는다. 이전 evidence와 실패 로그를 덮어쓰지 않는다. 통합 담당자가 고정한 source/build 작업을 확인한 뒤 코드 수정이나 Cargo 재실행을 시작한다.

기준 자료는 [WSL 관찰](wsl-interruption-recovery.md), [고정 8시간 실행 계약](../work-items/S03-soak-recovery-run.md), [launch](evidence/soak-recovery-20261003T123539Z-b2e1b9/launch.json), 같은 directory의 ownership.json 및 archive/archive-status.json이다. launch의 state=running은 당시 선언이며 현재 생존이나 성공을 뜻하지 않는다.

## 2. 현재 boot부터 비교한다

아래 명령은 기존 CLI와 실제 대조했다. `<fresh-name>`을 기존에 없는 고유 이름으로 바꾼 뒤 실행한다. 상위 evidence directory는 있어야 하며 output 자체가 이미 있으면 실패하는 것이 정상이다.

```sh
python3 -c "from pathlib import Path; print(Path('scripts/wsl_interruption_snapshot.py').resolve().parent.parent)"
python3 scripts/wsl_interruption_snapshot.py --output <canonical-project>/docs/validation/evidence/<fresh-name>
```

기본 명령은 boot-only 관찰이며 개발 프로세스 live를 증명하지 않는다. 추적할 현재 실행의 명시 ownership launch가 있을 때만 `--launch <canonical-project>/docs/validation/evidence/<owned-run>/launch.json`을 추가한다. 이미 완료된8h launch를 현재 개발 추적용으로 사용하지 않는다.

Collector는 입력/출력64KiB, journal 명령별5초, create-only snapshot.json을 사용한다. Launch가 있으면 boot부터 비교하고 다른 boot에서는 oldPID를 읽지 않는다. Sameboot일 때만 PID/start를 대조한다. 실제 UTC와 결과를 확인하며 기존 snapshot을 현재 관측으로 인용하지 않는다. `estimated_boot_utc`는 wallclock−uptime 추정이므로 journal UTC와 차이가 나도 원인/재부팅을 단정하지 않고 boot UUID를 우선한다.

첫 명령을 실제 snapshot과 동일한 실행 context에서 수행하고 그 canonical root로 `<canonical-project>`를 치환한다. Sandbox와 승격 host context의 `/mnt/d/project`/`/mnt/d/Project` case가 다를 수 있다. 절대경로를 사용하며 안전 검증을 우회하지 않는다. [최신 boot-only 기준점](evidence/wsl-monitor-off-latest-readiness-20261004/snapshot.json)은2026-10-04T03:20:45.550301+00:00/동일boot/uptime60607.81초/Interop enabled이며 runner 미추적이다. [이전 live snapshot](evidence/wsl-monitor-off-readiness-20261004/snapshot.json)의20:01 세 runner live는 역사 관찰이고 해당8h는 이후 완료됐다.

명령실패·부분쓰기·출력directory 존재만으로 snapshot 완료를 인정하지 않는다. Exit0와 완전한 bounded JSON을 확인한다. 실패시 기존 output을 지우거나 덮어쓰지 말고 새 고유 이름으로 재시도한다.

Sandbox namespace의 `/proc` 부재는 host 프로세스 종료 증명이 아니다. 2026-10-03T19:59:23Z sandbox에서 세 runner absent였지만20:00:00.840544Z host 읽기에서는 같은 boot/동일 start_ticks의 세 runner live와 WSLInterop enabled를 확인했다. Same boot+archive가 현재 watching이면 authorized host read-only 조회와 먼저 대조하고, 대조 불가 시 관측 제한으로 기록한다. 부재만 보고 runner를 종료·재시작하지 않는다.

| 관측 | 다음 판단 |
|---|---|
| boot 다름 | 옛 실행은 사라진 것으로 분류하되 terminal 없이 성공으로 처리하지 않는다. 옛 PID 번호로 signal/cleanup하지 않는다. |
| 같은 boot, identity-matches-live | 해당 role만 생존 관측. 진행/완료/자손 소유권은 별도 자료가 필요하다. |
| 같은 boot, reused-pid | 현재 PID는 이전 소유 프로세스가 아니다. 신호를 보내지 않는다. |
| absent 또는 zombie | 먼저 host 관측 범위를 대조한다. 확인 후에도 완료/중단은 terminal 증거로 구분한다. |
| observation-unavailable 또는 권한/파일/JSON 오류 | 관측 불가로 기록한다. live/완료 또는 restart 필요로 추정하지 않는다. |

collector는 download_runner/guardian_runner/archive_owner 세 role만 확인한다. 모든 자손의 정리 증명이나 현재 unit/cgroup 검증을 대신하지 않는다. boot ID와 journal에 접근해도 종료 요청자·OOM·절전 원인이 자동으로 증명되는 것은 아니다.

`observation_scope.pid_namespaces`는 관측 metadata이며 host_visibility_verified=false이면 전체 host 가시성을 확인한 것이 아니다. Launch 없는 snapshot에는 owned_processes가 없고 process_tracking 미요청이다. watchdogfalse는 이번 프로젝트가 설치하지 않았다는 계약이며 host 전체 서비스 감사 결과가 아니다. Windows executable_present도 eventlog 접근성 증명이 아니다.

## 3. 8시간 terminal과 부분 완료를 구분한다

archive의 scope_terminal_proofs와 download-result.json, runner-result.json, guardian-result.json, archive-result.json을 각 범위로 확인한다. 보관된 파일 size/SHA가 archive 기록과 일치하는지 확인한다. 파일이 없거나 truncated/아직 watching이면 그 범위는 미검증이다. runner/status가 live이고 planned_seconds=28800이어도 성공이 아니다.

기존8h는 [독립 terminal](evidence/soak-terminal-independent-20261004/result.json)/[root 검토](evidence/soak-terminal-independent-20261004/root-review.json)에서 두scope완료·소유11PID종료·DB/artifact/provenance를 확인했다. 옛 archive의 interrupted는 [terminal read/copy race](../work-items/S03-soak-archive-terminal-snapshot.md)와 구분한다. 충돌시 원시SHA와 새terminal evidence를 대조하고 완료시험을중복실행하거나이전archive를덮어쓰지않는다.

완료 인정에는 각각 실제 28800초 수행, 성공 exit, 원 DB의 최종 상태 및 재열기, 소유 자손 정리, archive terminal, 실행 파일과 입력 provenance가 필요하다. 두 범위 중 하나만 완료됐으면 그 증거는 유지하고 나머지만 중단/미완료로 기록한다. 이전 boot 약 5시간59분과 새 boot 시간을 합산하지 않는다. 현재 시험은 headless 합성 engine+SQLite/guardian이며 GUI·실제 영상·네트워크·인증 검증으로 확대하지 않는다.

SQLite는 시험이 terminal인지 먼저 확인한다. 활성 DB에 자동 복구·쓰기·일반 quick_check를 실행하지 않는다. 이전 감사의 mode=ro&immutable=1 SELECT는 제한된 관측이며 활성 WAL/동시성 무결성 검증을 대신하지 않는다. DB/sidecar 원본을 기존 archive로 덮어쓰지 않는다.

## 4. 실행 파일과 source snapshot을 확인한다

launch.artifacts의 immutable /tmp copy 경로, 크기, SHA256, native device/inode를 기준으로 비교한다. target/debug나 target/release의 현재 파일로 당시 실행 파일을 대체하지 않는다. 새 build 전에 실행 중인 copy와 source manifest를 보존한다. 이 실행의 CPU SHA는 `166946017de0b3d0589bf3d02bd864d004d90f3457ff2e314fe5e10a5f865417`이며 최신 제품 전체 검증의 증거가 아니다.

WSL 재부팅으로 /tmp copy/log/fixture가 없어졌다면 유실로 기록하고 프로젝트 archive의 보존 범위만 인정한다. 현재 경로에 새로 생긴 파일을 옛 inode나 SHA의 증거로 사용하지 않는다. 이전 DrvFS deleted-executable의 /proc/exe stat/직접 FD bytes 불일치는 [실행파일 감사](soak-resume-audit-20261003.md)에 기록돼 있다. PID/start 일치나 pathname hash만으로 loaded code의 SHA를 확정하지 않는다. 이후 Linux /tmp immutable 사본 계약을 유지한다.

## 5. 재개 범위를 정한 뒤 실행한다

이미 성공한 단위/CLI/OS 검증은 당시 source/binary SHA와 scope를 붙여 유지한다. source가 변경된 부분, 중단된 명령, terminal이 없는 장기 시험을 별도 작업으로 기록하고 실제 재검증한다. /tmp 소실만으로 프로젝트 전체를 초기화하거나 정상 완료 자료를 재작성하지 않는다.

같은 boot에서 살아 있는 기존 소유 장기 시험은 새로 시작하거나 종료하지 않고 담당자와 조정한다. boot가 바뀌었거나 terminal이 없으면 새 시험은 새 temp/evidence와 정확한 ownership manifest로 시작한다. 기존 archive를 --destination으로 다시 지정하거나 archive collector를 무심코 재실행하지 않는다. 새 시험도 사용자 Documents/실제 영상 대신 합성 fixture를 쓴다.

작업 보고에는 현재 boot 관측 UTC, 각 scope 완료/중단/미검증, 보존한 source/binary SHA, 실행한 정확한 명령과 exit, 남은 OS·장기 gate를 적는다. 시스템 전원·보안·서비스 변경 및 watchdog 설치는 이 절차에 포함되지 않는다.

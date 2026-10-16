# S00: WSL 중단 읽기 전용 snapshot

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: soak_recovery_audit — 기존 snapshot scripts(동결), docs/validation/wsl-interruption-recovery.md, docs/validation/wsl-resume-checklist.md, 이 문서
- 선행 작업: wsl-interruption-20261003 원시 증거

## 사용자 결과

전원 설정을 바꾸지 않고 현재 boot와 명시 소유 soak PID/start 상태를 영구 프로젝트 evidence에 남겨 재접속 후 비교한다. WSL 자체가 종료된 동안 agent가 계속 실행한다고 주장하지 않는다.

## 계약과 범위

현재 boot/uptime, bounded journal 접근 가능성, WSLInterop metadata와 명시 launch manifest PID/start만 읽는다. 전원·절전·방화벽 변경, WSL restart/shutdown, desktop, 관리자 권한, process signal, 전체 환경/명령행/계정 출력은 없다. 신규 evidence만 create-only이며 기존 자료는 보존한다. Windows eventlog는 root hidden CLI host에서 별도 읽기 전용으로 조회한다.

## 인수 조건

- 현재 boot mismatch는 old PID를 읽거나 종료하지 않고 다른 boot로 표시한다.
- 같은 boot PID는 start_ticks와 zombie 상태까지 확인하며 absent/reused/live를 구분한다.
- journal 비가용·interop 비가용은 명시한 오류 코드로 기록하며 중단 원인은 unconfirmed이다.
- input/output symlink를 거절하고 bounded fresh result 하나만 기록한다.

## 검증과 handoff

읽기 관찰: 현재 boot `0bc0c937-eee9-4d90-95dc-31a2122b6352`, uptime 약 24796초. `journalctl --list-boots --no-pager -n 3`는 현재와 이전 두 boot를 반환했다. 이전 종료 요청자는 기존 observation에서도 unknown이다. Snapshot source 동결 후 root가 실제 create-only evidence 실행·결과 검증한다. Windows eventlog 접근/원인 확정은 아직 미검증이다.

후속 root 실제 Linux snapshot exit0: same boot이며 명시 세 runner의 PID/start가 live로 일치했다. 이는 종료 원인이나 향후 연속 실행 보장이 아니다. 기존 power 읽기 자료 AC=0/DC=600은 root 증거이며 이 도구는 설정을 바꾸지 않는다.

Windows collector source를 별도 구현했다. 고정 System provider/ID pair와 Lxss/Hyper-V Compute log만 읽으며 default 최근36시간, 선택1~48시간, log당 최대32개이다. `Get-WinEvent -FilterHashtable`에는 정확한 LogName/StartTime/EndTime 및 System ProviderName/Id를 넘긴다. 이벤트 Message/Properties/UserId/MachineName/security log는 출력하지 않는다. 없는 log·접근 오류·일치 이벤트 없음은 구분하고 예외 Message 대신 type/HResult만 기록한다. 새로운 private output directory의 create-only `windows-events.json`은 64KiB 상한이다. Host watchdog/task/service 설치·desktop·WSL restart·설정 변경은 없다. 개별 Windows eventlog API deadline은 외부 hidden CLI host가 담당한다.

Root 실행: `powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File <project>/scripts/windows_wsl_event_snapshot.ps1 -OutputDirectory <fresh-project-evidence> -Hours 36` (필요 시 UTC `-EndUtc <ISO8601>`). Agent는 Windows 실행을 하지 않았으며 sourcefreeze 뒤 root가 실제 조회와 원시 JSON을 검토한다. 이벤트 존재 자체를 WSL 종료 요청자나 절전 원인 확정으로 표현하지 않는다.

## root 실제 snapshot 검증

`python3 scripts/wsl_interruption_snapshot.py --output <fresh evidence> --launch <existing launch.json>` 실제 exit0. docs/validation/evidence/wsl-interruption-snapshot-first/snapshot.json에서 동일 boot ID, 이전 boot journal 접근 가능, 소유 download/guardian/archive runner 3개 exact start identity live를 확인했다. 종료 요청자는 여전히 미확정이며 Windows eventlog는 아직 미검증이다.

## 최종 통합 검증

Root가 Linux snapshot 및 Windows collector 실제 exit0와 원시 JSON을 검토했다. Windows System 이벤트4개, Lxss/Hyper-V-Compute 로그 미등록을 확인했다. AC절전0/DC600초이며 설정은 변경하지 않았다. 종료 요청자는 미확정이다. 단일 관찰·복구 준비 범위만 완료했으며 호스트 watchdog·재중단 방지는 구현하지 않았다. 증거: docs/validation/evidence/windows-wsl-event-first/root-review.json 및 wsl-interruption-snapshot-first/snapshot.json.

## 사용자 모니터 종료 운용 대비 후속 감사

범위는 WSL 문서 두 개와 이 workitem뿐이다. Report collector/제품/타 agent 파일은 변경하지 않았다. 새 snapshot/Windows 실행/전원 설정/WSL restart/watchdog 설치/runner signal·재시작은 없다. 기존 snapshot 함수를 import해 결과를 stdout로만 읽었다.

2026-10-03T19:59:23Z sandbox 읽기는 같은 boot에서 세 runner absent 및 WSLInterop 없음으로 표시했다. 2026-10-03T20:00:00.840544Z require_escalated 읽기는 같은 boot `0bc0c937-eee9-4d90-95dc-31a2122b6352`, uptime31520.29초, 세 PID254678/254679/254799의 start294064/294064/294241가 모두 일치 live, WSLInterop enabled/init를 확인했다. Root의 독립 host 조회도 동일했다. 따라서 첫 absent는 namespace 범위 차이이며 실제 종료가 아니다. 정상 host 관측을 대조하기 전 부재를 중단으로 단정하면 안 된다.

프로젝트 archive 읽기는 observed19:58:54.167067Z/statewatching, download elapsed28502.3178초/guardian28488.4060초/state running, 두 scope_terminal_proofs null이며 archive/download/runner/guardian terminal 파일은 아직 없었다. 현재8h는 live 미완료 관측이며 terminal 없이는 PASS가 아니다. Archive는 프로젝트에 지속 보존되고 새 snapshot/event collector/복구 체크리스트가 있으므로 사후 확인 준비는 있다. WSL 종료 동안 host 독립 기록·자동 재시작·연속 실행 보장은 없다.

실제 발견한 CLI case 제약도 문서에 보완했다. 이 host에서 script PROJECT.resolve는 `/mnt/d/Project/dlpad`이므로 소문자 alias를 그대로 --launch에 주면 relative_to 안전 검증에서 실패한다. Canonical project root를 먼저 읽고 그 절대경로로 fresh output/launch를 구성한다. Script source는 이번 감사에서 바꾸지 않았다.

Root 후속 영구 snapshot은 `wsl-monitor-off-readiness-20261004/snapshot.json`,2026-10-03T20:01:27.463083Z, 승격 read-only exit0이다. 동일 boot/세 role exact start live/WSLInterop enabled를 확인했다. Root 실행 context는 lowercase PROJECT라 uppercase 입력의 relative_to 실패 후 실제 해당 context 경로로 재시도했다. 문서는 대소문자를 고정하지 않고 같은 context에서 script canonical root를 읽게 수정했다. Relative output/launch는 script 계약상 지원하지 않는다. 마지막 준비 감사는 문서만 보완했고 시스템 변경/watchdog 설치/runner 종료·재시작은 없다. 원인은 여전히 미확정이며 개발·Cargo 후속은 root가 별도 계속한다.

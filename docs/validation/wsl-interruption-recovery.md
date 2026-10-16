# WSL 중단 관찰과 복구

WSL 내부 에이전트와 soak는 WSL VM이 종료되면 실행을 계속할 수 없다. 절전을 끄고 모니터만 끄는 운용을 예정하더라도 현재 자료만으로 재중단 방지나 원인을 확정할 수 없다. 호스트 독립 watchdog은 설치하지 않았다. Windows/WSL update·수동 shutdown·호스트 종료 등은 구분할 증거가 필요하며 현재 shutdown requester는 unconfirmed이다. 이 작업은 시스템 전원·절전·보안 설정을 바꾸지 않는다.

기존 [중단 관찰](evidence/wsl-interruption-20261003/observation.json)은 boot `794988eb-539b-437e-8396-330738b565bb`에서 `0bc0c937-eee9-4d90-95dc-31a2122b6352`로 변경됐고 이전 종료 journal 시간이 기록됐음을 보여준다. 종료 요청 주체는 unknown이며 agent가 WSL을 종료했다는 근거는 없다. 2026-10-04 읽기 조사에서도 현재 boot는 `0bc0c937…`이고 이전 boot journal 목록에 접근할 수 있었다. 종료 원인을 추측해서 확정하지 않는다.

## 새 snapshot

먼저 `python3 -c "from pathlib import Path; print(Path('scripts/wsl_interruption_snapshot.py').resolve().parent.parent)"`로 같은 실행 context의 canonical project root를 읽는다. Sandbox와 승격 host context에서 `/mnt/d/Project/dlpad`과 `/mnt/d/project/dlpad`이 다르게 보고될 수 있으므로 대소문자를 고정하지 않는다. `<canonical-project>`를 실제 같은 context에서 읽은 값으로, `<fresh-name>`을 기존에 없는 이름으로 바꾼다. Collector는 absolute path를 요구하므로 output/launch 상대경로를 직접 넘기지 않는다.

`python3 scripts/wsl_interruption_snapshot.py --output <canonical-project>/docs/validation/evidence/<fresh-name>`

기본 명령은 boot-only이며 개발프로세스 live를 주장하지 않는다. 현재 실행의 명시 ownership manifest가 있을 때만 `--launch <canonical-project>/docs/validation/evidence/<owned-run>/launch.json`을 추가한다. 이미 완료된8h launch를 현재 개발 추적용으로 쓰지 않는다. 실패/부분출력은 snapshot 완료가 아니며 exit0와 완전한JSON을 확인하고 새이름으로retry한다. 기존출력을삭제/덮어쓰지않는다.

출력은 새 directory에만 생성하며 기존 evidence를 덮어쓰지 않는다. Boot/uptime·최대 3개 boot 목록·이전 kernel journal 접근 여부/record 시간·WSLInterop 등록 여부·명시 runner PID/start 상태를 기록한다. PID/start 일치도 종료 원인 증명은 아니다. 다른 boot이면 옛 PID를 조회하거나 signal을 보내지 않는다. 전체 환경·명령행·계정·kernel 메시지·Windows 이벤트 Message는 수집하지 않는다.

Boot UUID를 기준으로 비교한다. estimated_boot_utc는wallclock−uptime추정이라journal UTC와차이가날수있다. Namespace metadata는host전체가시성증명이아니며 observation-unavailable/absent를host조회없이중단으로단정하지않는다. watchdogfalse는프로젝트미설치계약이고host전체서비스감사를뜻하지않는다.

재접속 후 새 snapshot을 만들어 boot와 uptime을 비교한다. Boot가 바뀌면 기존 실행 결과를 terminal 증거 없이 통과 처리하지 않는다. 기존 evidence의 완료 proof와 SQLite 상태를 각각 확인하고 중단·미검증 범위만 새 임시 fixture로 재실행한다. `/tmp` 산출물은 WSL 재시작 후 사라질 수 있으므로 프로젝트 evidence의 원시 자료를 기준으로 검토한다. 기존 `.git`과 사용자 데이터는 보존한다.

## Windows eventlog 읽기 계약

Root가 기존 hidden CLI host를 통해 PowerShell argv를 직접 실행한다. `powershell.exe -NoProfile -NonInteractive -Command <fixed-read-only-query>`이며 앱·desktop·관리자 실행·서비스 설치는 없다. 알려진 `System`, `Microsoft-Windows-Lxss/Operational`, `Microsoft-Windows-Hyper-V-Compute-Admin` log만 각각 `Get-WinEvent -ListLog <name>` 접근 여부를 확인한다. 가능한 log에는 `Get-WinEvent -FilterHashtable @{LogName=<name>;StartTime=<UTC interruption window>;EndTime=<UTC end>} -MaxEvents 32`로 제한하고 `TimeCreated.ToUniversalTime(), Id, ProviderName, Level`만 JSON으로 출력한다. System은 ID 41/42/107/109/506/507/6005/6006/6008/1074로 추가 제한한다. Message/Properties/UserId/MachineName을 출력하지 않는다. 미존재·접근 거절은 availability로 구분하며 성공한 조회 자체를 원인 확인으로 표현하지 않는다. 실제 Windows 접근은 root 실행 전 미검증이다.

호스트 독립 주기 기록이 필요하면 별도 user-only watchdog의 수명·저장 상한·종료 계약을 검토해야 한다. 현재 snapshot은 단일 관찰 도구이며 자동 재시작·background Windows service가 아니다.

## 실제 수집 결과 (2026-10-04)

[root Windows 관찰](evidence/windows-wsl-event-first/root-review.json)은 읽기 전용 collector exit0, System 관련 이벤트4개, Lxss/Hyper-V-Compute 두 로그 미등록을 확인했다. UTC/ID/provider/level만 수집했고 사용자 데스크톱은 그대로다. 이벤트만으로 이전 WSL 종료의 요청자나 인과관계를 확정하지 않는다. [Linux snapshot](evidence/wsl-interruption-snapshot-first/snapshot.json)은 같은boot 및 소유3runner의 시작시각 일치 live를 기록한다. 현재 [전원 타이머 읽기](evidence/wsl-power-readonly-first/root-review.json)는 AC절전0/DC600초, AC/DC최대절전0이다. 설정 변경은 없다.

## 모니터만 끄는 운용 대비 (후속 읽기 감사)

아래 live/readiness 기록은 **2026-10-03T20:01:27Z 이전의 역사 관찰**이다. 이후 두 8시간 시험은 완료됐으며, 현재도 그 runner들이 실행 중이라는 뜻이 아니다. 최신 확인된 terminal 상태는 다음 절에 기록한다.

2026-10-03T20:00:00.840544Z host 범위 조회는 기존 boot와 세 소유 runner PID/start 일치 live를 확인했다. 직전 sandbox에서는 `/proc` namespace 제한으로 세 PID absent·WSLInterop 없음이었으므로 sandbox 결과만으로 종료나 재시작 필요를 판단하지 않는다. 마지막 archive19:58:54Z는 watching/두 terminal proof null로 아직8시간 완료가 아니다. 영구 프로젝트 archive, create-only snapshot, Windows bounded event collector, [재접속 체크리스트](wsl-resume-checklist.md)를 준비했으며 runner·설정은 변경하지 않았다.

모니터가 꺼진 뒤 작업이 지속되는지 또는 WSL이 다시 중단되는지는 이 단일 관찰로 보장하지 않는다. 재접속 후 먼저 boot와 관측 범위를 확인하고 archive terminal을 대조한다. WSL 종료 동안 내부 agent는 기록하지 못하며 host watchdog은 없다. 터미널/agent 연결만 끊긴 경우와 VM boot 변경을 구분하고, 동일 boot live runner는 종료·중복 재실행하지 않는다. 필요한 경우 root가 기존 hidden CLI host로 `scripts/windows_wsl_event_snapshot.ps1 -OutputDirectory <fresh-native-evidence> -Hours 36`을 실행해 종료 시점 주변의 제한된 이벤트를 새로 보존한다. 원인 불명은 그대로 미확정으로 남긴다.

Root의 최신 [영구 readiness snapshot](evidence/wsl-monitor-off-readiness-20261004/snapshot.json)은2026-10-03T20:01:27.463083Z 승격 read-only 실행 exit0, 동일 boot/세 runner exact start identity live/WSLInterop enabled를 기록한다. 이 실행 context에서는 lowercase project root가 맞아 uppercase 경로의 첫 relative_to 거절 후 같은 context의 실제 PROJECT 경로로 재시도했다. 초기 sandbox false absent는 실제 중단으로 판정하지 않았다. 설정 변경·서비스/스케줄러/watchdog 설치와 runner signal·재시작은 없다.

## 이후 확인된 8시간 terminal 완료

[독립 terminal 감사](evidence/soak-terminal-independent-20261004/result.json)와 [root 원시 자료 대조](evidence/soak-terminal-independent-20261004/root-review.json)는 동일 boot의 합성 download 28801.956초, guardian 28804.093초 완료를 확인했다. Download runner는 exit0, guardian은 descendants_reaped=true이다. 승격 host 읽기에서 소유 11개 PID/start의 종료를 확인한 뒤 terminal DB를 read-only/immutable로 검증했으며 Downloaded 상태·재열기·17-byte 합성 artifact·부분 파일 및 원본/copy SHA를 대조했다. 이는 Linux headless engine/SQLite/direct guardian 범위이며 GUI·실제 영상·네트워크·인증이나 최신 제품 backend의 8시간 검증으로 확대하지 않는다.

기존 archive의 20:03:26Z `interrupted`는 그 문자열만으로 WSL 중단을 의미하지 않는다. 당시 collector가 terminal proof를 먼저 읽고 뒤늦게 나타난 completed runner 파일을 복사하면서 생긴 read/copy race가 확인됐으며, 기존 archive를 덮어쓰지 않고 새 terminal evidence를 보존했다. [별도 collector 회귀 작업](../work-items/S03-soak-archive-terminal-snapshot.md)은 합성 시험으로 이를 수정했다. 이 문서 갱신을 위해 soak를 재실행하거나 프로세스를 종료하지 않았다.

현재 목적이 이미 끝난 이 시험의 상태 보완이라면 새 readiness snapshot은 필요하지 않다. 완료 증거는 프로젝트에 보존되어 있고 옛 runner의 absent는 정상 종료 관찰이다. 사용자가 자리를 비운 동안 **현재 개발 세션/WSL boot**의 관찰 기준점을 새로 원하면 `새 snapshot` 절의 create-only 명령을 실행할 수 있다. 재접속 후에는 그때의 새 snapshot이 필요하며, 옛 PID 생존만으로 현재 개발 활동을 판정하지 않는다. 이전 readiness snapshot을 현재 uptime/live 증거로 재사용하지 않는다.

기존 [재접속 체크리스트](wsl-resume-checklist.md)의 boot 우선 비교, host 관측 범위 대조, 원시 증거 보존 및 terminal별 검토 절차는 그대로 사용할 수 있다. 다만 그 문서의 readiness 링크 역시 과거 live 관측이다. WSL이 종료된 동안 내부 collector/agent는 기록하거나 자동 복구할 수 없고 호스트 독립 watchdog은 설치하지 않았다. 모니터만 끄는 운용이 재중단을 방지한다고 보장하지 않으며 종료 요청 주체는 계속 미확정이다.

## 현재 운용 기준점

Root가 새 create-only [host 읽기 snapshot](evidence/wsl-monitor-off-readiness-current-20261004/snapshot.json)을 실제 exit0로 생성했고 원시 JSON을 독립 대조했다. 관측 UTC는 2026-10-03T20:54:18.282247Z, boot는 `0bc0c937-eee9-4d90-95dc-31a2122b6352`, uptime35100.62초, WSLInterop enabled이다. 이전 boot kernel journal 접근도 가능했다. 재접속 후 새 boot/uptime을 비교할 기준점이다.

이번 snapshot은 launch/runner 추적 없이 생성되어 개발 프로세스나 agent의 live·향후 지속 실행을 증명하지 않는다. Windows eventlog 접근도 이번 Linux snapshot에서 다시 시험하지 않았다. Shutdown requester는 unconfirmed, host_watchdog_installed=false이며 기록 공백·자동 복구 한계는 그대로다. 설정·서비스·watchdog 설치·runner 종료/재시작은 없다.

## 수집기 관측 경계 보강

후속 수집기는 UUID의 정확한 형식을 검사하고 현재 관측자의 PID namespace native metadata를 기록한다. Metadata가 있어도 host 전체 visibility를 자동 확인한 것으로 간주하지 않는다. 권한 오류·잘못된 stat·입력 상한 초과는 프로세스별 `observation-unavailable`로 구분한다. 같은 boot의 absent/zombie/reused PID는 정상 완료와 중단을 구분하지 못하며 terminal 증거를 대조한다. Boot가 바뀌면 옛 PID를 읽지 않고 `different-boot-not-inspected`로 기록한다. Launch 없는 snapshot은 process tracking을 요청하지 않았다고 명시한다.

이 개선은 현재 부팅 안에서의 단일 관찰 도구다. Windows 고정 이벤트 collector는 그대로 사용할 수 있지만 주기 호스트 기록 기능이나 재중단 알림은 없다. 실제 중단 후에는 에이전트가 멈추므로 사용자의 재접속/호스트 독립 관찰 없이 스스로 감지·자동 재개하지 못한다. [Synthetic 관측 회귀](../work-items/S00-wsl-snapshot-observation-boundaries.md)는 실제 WSL 재부팅 시험을 대신하지 않는다.

개선 후 root의 [새 host snapshot](evidence/wsl-monitor-off-boundaries-20261004/snapshot.json)은 실제 exit0, UTC2026-10-03T21:59:25.361039Z/동일 boot/uptime39397.53초/Interop enabled이며 원시 JSON을 독립 대조했다. PID namespace metadata가 있어도 host_visibility_verified=false이고 absent는 shutdown proof가 아님을 명시한다. Runner 추적은 요청하지 않아 개발 프로세스 live나 완료된8h 재실행을 주장하지 않는다. Root 독립 synthetic6tests도 PASS했다. 이는 단일 관측 준비의 인수이며 Windows 재실행·실제 reboot/중단·모니터off 이후 연속성·watchdog 시험은 아니다.

## 사용자 모니터 종료 전 새 기준점

[새 baseline](evidence/wsl-monitor-off-user-baseline-20261004/snapshot.json)을 실제 exit0로 생성하고 root가 JSON을 대조했다. UTC2026-10-03T23:57:28.192508Z, 동일 boot0bc0c937, uptime47189.72초, WSLInterop enabled이다. 이전 kernel journal 조회는 exit1/available=false로 관측 제한이다. 수집기 synthetic6tests PASS. [작업 기록](../work-items/S00-monitor-off-current-baseline.md)에 범위를 기록했다.

재접속 시 새 boot UUID와 비교한다. 같은 boot도 agent/터미널만 종료될 수 있으므로 실행별 terminal 증거를 따로 확인한다. 이번 baseline은 runner 미추적이며 화면 종료 이후 연속성·자동 재개를 보장하지 않는다. WSL 내부 수집기는 VM이 꺼진 동안 관찰할 수 없다.

## 이번 세션의 모니터 종료 기준점

[새 세션 baseline](evidence/wsl-monitor-off-session-baseline-20261004/snapshot.json)을 root가 exit0와 원시 JSON으로 확인했다. UTC2026-10-04T01:03:18.321736+00:00, 동일 boot0bc0c937, uptime51532.95초, Interop enabled이다. 수집기6회귀 PASS. [이번 작업](../work-items/S00-monitor-off-session-readiness.md)에 범위와 실패·재시도 결과를 기록했다. 재접속 후 새 boot UUID를 우선 비교하고 실행별 terminal 증거를 확인한다. 화면 종료 이후 지속 실행이나 자동 재개를 증명하는 기록은 아니다.

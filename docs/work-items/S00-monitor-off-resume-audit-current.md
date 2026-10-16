# S00: 모니터 종료 후 현재 개발 세션 재접속 판정 감사

- 상태: scoped-review-complete
- 분류: validation
- 우선순위: P0
- 요구사항: R08
- 담당/수정 범위: stress_recovery; 이 신규 문서만
- 선행 작업: wsl-resume-checklist, wsl-interruption-recovery, 현재 owned workspace runner

## 사용자 결과

모니터만 끈 뒤 재접속했을 때 WSL boot 변경, 세션 연결 종료, 실행별 완료를 혼동하지 않도록 현재 체크리스트의 구체적인 누락을 정리했다. 모니터 종료가 WSL 중단을 방지하거나 개발이 계속된다고 보장하지 않는다.

## 계약과 범위

읽기 대상은 기존 WSL 체크리스트/관찰 문서, scripts/wsl_interruption_snapshot.py, scripts/rust_workspace_validation.py다. 변경 파일은 이 문서 하나다. 새 snapshot·Windows 전원/event 조회·프로세스 변경·서비스/watchdog 설치·Cargo 실행은 하지 않았다. 기존 완료 증거와 이번 읽기 감사를 구분한다.

## 인수 조건

현재 기준점의 UTC/boot/runner 추적 여부를 과거 증거로만 인용한다. 재접속 때 새 snapshot과 실제 terminal을 요구하며, 없는 자료는 미검증으로 둔다. 이전 PID·완료된 8시간 실행·launch prepared 상태로 현재 live/완료를 주장하지 않는다. 구체적인 빠진 계약을 후속 항목으로 기록한다.

## 현재 자료 대조

이전 사용자 baseline은 `evidence/wsl-monitor-off-user-baseline-20261004/snapshot.json`이며 문서상 UTC2026-10-03T23:57:28.192508Z, boot0bc0c937, uptime47189.72초, runner 미추적이다. 이전 kernel journal unavailable도 기록됐다. 이는 그 시점 관찰이며 이번 감사 시점 새 관측이 아니다. 오래된 readiness 세 runner live는 이미 끝난 8시간 실행의 역사다.

기존 체크리스트는 boot 우선, sameboot PID/start 대조, sandbox/host namespace 제한, PID 재사용, terminal 없는 성공 금지, immutable binary와 /tmp 유실, 부분 scope 보존을 이미 다룬다. 해당 내용을 새 결함처럼 중복 등록하지 않는다.

## 실제 빠진 재접속 판정 항목

1. **최신 baseline 선택:** 체크리스트의 직접 기준점 링크는 이전 boundaries snapshot이며 최신 사용자 baseline은 관찰 문서 후반에 있다. 재접속자는 임의의 옛 링크 대신 사용자가 자리를 비우기 직전 실제 생성된 baseline의 UTC/boot/관측 context를 선택해야 한다. 새 비교 snapshot 없는 상태에서는 이후 중단 여부를 판정하지 않는다.
2. **현재 workspace runner의 추적 범위:** 새 runner의 launch.json은 archive_owner에 runner를 넣으므로 기존 snapshot --launch로 runner만 추적 가능하다. operation별 `<name>-launch.json`의 command PID/start는 기존 snapshot이 읽는 세 role에 포함되지 않는다. runner live가 Cargo live/진행/성공의 증거가 아니며 이 누락을 explicit하게 기록해야 한다. 후속 collector 보강 없이는 operation JSON·log·terminal 결과와 직접 소유 identity 조회를 별도로 대조한다.
3. **prepared와 actual 실행 구분:** runner의 launch state prepared 및 operation waiting-for-durable-exec-gate는 실제 Cargo 실행을 뜻하지 않는다. --execute=false도 결과가 prepared다. 재접속 시 result.json의 workspace_checks_complete, source_unchanged 및 각 operation의 passed/exit_code/collector_failure를 확인한다. 동일 source 기본/CPU 7명령과 두 version probe가 모두 끝났다는 계약도 대조한다. ignored_native_acceptance_complete=false는 native 인수 완료로 바꾸지 않는다.
4. **연결 종료와 worker 종료 구분:** sameboot 상태에서 채팅/CLI 응답이 없더라도 owned runner는 남아 있을 수 있다. 반대로 runner absent도 정상 terminal일 수 있다. result 파일이 없거나 operation만 partial이면 중단/미검증으로 분류하고 새 실행 전에 소유 identity/terminal을 대조한다. 연결 재개 자체가 background agent의 지속 실행 증거는 아니다.
5. **재접속 결과의 비교 파일:** baseline과 새 snapshot 두 파일의 경로·UTC·boot 및 operation terminal별 결론을 새 create-only 문서/JSON으로 묶는 명시 절차가 없다. 후속 통합 담당자가 이 비교 결과를 기록해야 다음 세션이 옛 baseline을 현재로 재사용하지 않는다.
6. **원인 판단의 시간창:** boot 변경 또는 작업 중단이 관찰돼도 화면 off 시점과 인과를 연결할 수 없다. 사용자 화면 off/복귀 시각은 별도 진술로 기록하고, 기존 bounded Windows 이벤트 도구를 재실행할 때만 그 시간창의 availability/metadata를 새 증거로 보존한다. 이전 이벤트 조회 성공이나 AC/DC 타이머 값을 이번 중단 원인으로 재사용하지 않는다.

## 후속 작업과 미검증

통합 담당자는 자리를 비우기 전 새 baseline을 실제 생성했다면 그 정확한 파일을 재접속 기준으로 명시한다. 재접속 후 새 snapshot→boot/context 비교→현재 실행 launch/terminal/source 대조→scope별 완료/중단/관측불가 기록 순으로 진행한다. operation PID 추적 확장은 별도 구현 작업이며 이 감사에서 collector를 수정하지 않았다.

모니터 off 이후 연속성·실제 WSL 재중단·종료 요청자·Windows 전원 이벤트의 현재 접근성은 미검증이다. 내부 collector는 VM 종료 동안 기록하지 못하며 host watchdog은 설치하지 않는다. 설정이나 프로세스 변경으로 원인 불명을 우회하지 않는다.

## 검증과 handoff

rg/sed로 기존 문서와 두 수집기 계약을 대조했다. 실제 snapshot/전원/프로세스/Cargo 실행 0회다. 문서 검사 결과는 완료 보고에 첨부한다. 기존 결과를 덮어쓰지 않았다.

## Root의 새 세션 기준점 대조

감사 종료 직전 root가 생성한 `evidence/wsl-monitor-off-session-baseline-20261004/snapshot.json`을 실제 읽었다. UTC2026-10-04T01:03:18.321736+00:00, boot0bc0c937-eee9-4d90-95dc-31a2122b6352, uptime51532.95이며 process_tracking은 not requested다. 재접속 비교 기준은 이 새 파일로 갱신한다. 이 관찰도 개발 runner 생존이나 이후 연속성 증거가 아니다. 최신 기준점 기록의 누락은 root의 S00-monitor-off-session-readiness.md/복구 문서 갱신으로 보완됐으며, operation PID 추적과 실제 재접속 비교는 후속으로 남는다. 문서 검사 485개 PASS를 확인했다.

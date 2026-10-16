# S00: 모니터 끄기 이후 재접속 관찰 감사

- 상태: done
- 분류: review
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: metrics_recovery — 이 신규 문서만
- 선행 작업: 기존 Windows event snapshot, WSL 재접속 체크리스트; root 최신 Linux baseline 및 기존 collector 회귀

## 사용자 결과

사용자가 모니터만 끄고 비운 뒤 재접속했을 때 실제 중단, 관측 공백, agent 세션 종료를 분리한다. 이번 작업은 읽기 감사이며 모니터/전원/WSL/서비스/GUI/설정을 변경하지 않는다. 기존 collector를 재구현하지 않는다.

## 읽은 계약과 현재 기능

[scripts/windows_wsl_event_snapshot.ps1](../../scripts/windows_wsl_event_snapshot.ps1)은 System의 고정 provider/event ID와 Lxss Operational, Hyper-V Compute Admin 로그를 읽고 각 최근32건을 64KiB 이하 새 JSON에 저장한다. 메시지·계정·호스트 이름은 저장하지 않으며 unavailable/no-matching-events/query-failed를 구분한다. 기본 조회 구간은 종료UTC 기준36시간이고 1~48시간이다. 사용자 전용 ACL과 fresh create-only 출력 계약이 있다.

[wsl-resume-checklist.md](../validation/wsl-resume-checklist.md)는 Linux boot UUID, 같은 부팅의 PID/start identity, sandbox/host 관측 차이, terminal/source 증거 및 옛 PID 신호 금지를 이미 규정한다. 오래된8h live 관측과 완료 terminal을 구분한다. root가 만드는 최신 baseline을 역사20:01/21:59 관측과 대체 혼용하지 않는다.

## 부족한 관측과 해석 제한

- 각 로그32건 상한의 포화/누락 여부와 EventRecordID를 저장하지 않는다. 여러 사건이 있는36시간 조회에서 중요한 이전 event가 밀렸는지 확인할 수 없다. 32건이면 완전한 시간 구간이라고 보고하지 않는다. 필요한 경우 사건 주변으로 EndUtc/Hours를 좁혀 새 출력으로 다시 수집하고 이전 결과를 보존한다.
- System은 정해진 provider/ID만 포함한다. 다른 provider에서 기록한 원인, WSL 배포판 종료, service 종료 요청, host process crash/OOM을 모두 포괄하지 않는다. 로그 이름이 미등록/비활성이거나 query-failed면 원인 부재가 아닌 관측 불가다. 로그를 활성화하는 설정 변경은 이 감사 범위 밖이다.
- ID/시간/level만 있고 message/data가 없다. User32 1074가 있어도 요청 process나 요청자를 확정할 수 없고 Kernel-Power41도 WSL 종료 요청자를 특정하지 못한다. 안전한 요약으로부터 계정/명령을 추측하지 않는다.
- 관측UTC와 구간 종료UTC는 있으나 전후 host boot identity, clock 보정, 이벤트 지연 및 logger gap을 증명하지 않는다. Windows 시간과 Linux estimated_boot_utc를 직접 같다고 가정하지 않는다. Linux actual boot UUID와 원시UTC를 함께 기록한다.
- host_watchdog_installed=false는 이 프로젝트가 설치하지 않았다는 계약이다. host 전체의 외부 서비스 부재나 끊김 없는 관찰 증거가 아니다. WSL 내부 수집기는 VM 종료 중 관찰/개발을 계속할 수 없다.
- 현재 collector는 단발 snapshot이며 모니터 OFF 순간, 전체 부재 시간, 자동 재접속 또는 agent 자동 재개를 검증하지 않는다. 파일이 생겼다는 사실만으로 exit0/완전한 JSON/장기 시험 완료를 인정하지 않는다.

## 재접속 판정 계약

| 실제 관측 | 인정 가능한 범위 | 추가 확인 |
|---|---|---|
| Linux boot UUID 동일, owned PID/start live | 그 role의 현재 생존 | terminal/진행 증거 및 host namespace |
| Linux boot UUID 변경 | 이전 VM 실행은 연속 실행 아님 | Windows 전원/WSL 로그와 중단 명령의 별도 기록 |
| UUID 동일, agent만 부재 | VM 재부팅이 확인된 것은 아님 | agent 세션/owned child 종료와 terminal |
| sleep/resume 또는 standby event | host 전원 전환 기록 | WSL boot/runner 전후 상태; 모니터만 껐다는 증거로 확대 금지 |
| host shutdown/restart event | host 사건 후보 | event 요약만으로 요청자·WSL 원인 확정 금지 |
| 로그 없음/미등록/비활성/조회 실패 | 관측 한계 또는 해당 구간 일치 없음 | Linux baseline과 비교; 정상 연속 실행 추정 금지 |

모니터만 꺼졌다는 사용자 행동은 절전 정책이 발동하지 않았다는 증거가 아니다. 반대로 전원 로그가 없다는 사실로 모니터 OFF가 WSL을 종료시켰다고 판단하지 않는다. source/immutable binary와 actual command terminal은 부팅 관측과 별도로 보존한다.

## 인수 조건과 다음 단계

root 최신 baseline의 실제 UTC/boot/관측 context를 먼저 저장한다. 재접속 시 fresh Linux snapshot과 fresh Windows event snapshot의 정확한 명령·exit·조회 구간·로그 접근성을 기록한다. boot가 바뀌면 옛 PID를 읽거나 signal하지 않는다. 같은 boot도 host 가시성 및 PID/start를 먼저 대조한다. 중단된 작업은 terminal 없는 범위만 새 evidence로 재개하며 이미 완료한8h를 자동 재실행하지 않는다.

이번 감사는 실제 모니터 OFF 이후 재중단을 관측하지 않았다. 연속 host watchdog, 자동 agent 재개, 종료 요청자, 실제 전원 전환 원인은 미검증이다. 수집 상한/metadata 보강은 필요성 제안이며 이 문서에서 구현하거나 시스템 설정을 변경하지 않는다.

## 검증과 handoff

실제 수행: 기존 두 파일 전체 읽기, 신규 감사 문서 작성, 문서 검사. Rust/Cargo/native/GUI/전원/WSL 변경 및 새 collector 실행은 없음. 최신 baseline과 기존 collector 회귀의 실제 결과는 root 담당 evidence로만 인용한다.

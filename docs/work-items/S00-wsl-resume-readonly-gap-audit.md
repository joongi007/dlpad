# S00: WSL 읽기 전용 재개 체크리스트 누락 감사

- 상태: completed (읽기 감사; 체크리스트 수정 별도)
- 분류: review
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 파일: soak_recovery_audit — 이 문서만
- 의존성: snapshot/test, recovery 문서, observation-boundaries 및 실제 boot evidence

## 범위와 인수 조건

현재 재접속 체크리스트와 새 snapshot 계약의 차이를 읽기 감사한다. 실제 boot 관측과 종료/개발프로세스 미관측을 구분하고 남은 수동 재개 절차를 기록한다. 시스템·설정·watchdog·자동재개·Rust·Cargo·다른 문서는 수정하지 않는다. 독립 Python synthetic tests만 실행할 수 있다.

## 기존 준비에서 확인한 것

수집기는 boot UUID를 먼저 검증하고 launch boot가 다르면 옛 PID를 읽지 않는다. 같은 boot에서는 명시 PID/start를 대조하지만 namespace가 host 전체라고 주장하지 않는다. 권한·malformed stat는 observation-unavailable이고 launch 없는 출력은 process tracking 미요청을 명시한다. 출력은 프로젝트 fresh directory/create-only,64KiB,고정 journal 명령별5초이며 환경/commandline/account/message를 보존하지 않는다. 자동 중단 알림·host watchdog·자동재개는 없다.

실제 [새 host snapshot](../validation/evidence/wsl-monitor-off-boundaries-20261004/snapshot.json)은 UTC2026-10-03T21:59:25.361039Z/동일 boot/uptime39397.53/Interop enabled만 입증한다. host_visibility_verified=false이고 runner 미추적이다. [완료8h 감사](../validation/evidence/soak-terminal-independent-20261004/result.json)는 별도 terminal 증거이며 snapshot의 성공 exit를 현재 개발 프로세스 live나8h 재실행 PASS로 표현하지 않는다.

## 재접속 체크리스트의 후속 보완 항목

이번 담당은 이 workitem만 수정하므로 아래 항목은 체크리스트 owner의 후속 최소 문서 수정이다.

- 현재 체크리스트의 '최신 실제 readiness' 링크는20:01 과거live다. 최신21:59 boot-only 링크를 함께 제시하고 완료8h independent terminal/root-review를 연결한다. 옛 launch running/absent를 현재 개발 상태 판단에 쓰지 않는다.
- `--launch` 없는 boot-only 명령을 기본 준비/재접속 기준점으로도 제시한다. 실제 현재 소유 프로세스 목록이 없을 때 과거8h launch를 의무 인자로 넣으면 완료시험부재를중단으로오인할수있다. 현재개발runner를추적하려면그실행의새명시ownershipmanifest와별도담당검토가필요하다.
- 관측표의 absent/zombie행은 authorizedhost에서확인한sameidentity관측인지먼저구분하고, observation-unavailable은새상태명으로표시한다. Namespace metadata와 host_visibility_verified=false를 함께 읽는다. Sandbox absent만으로프로세스종료/재시작하지않는다.
- archive-result interrupted와 original8h terminal완료가충돌하면 [collector race 작업](S03-soak-archive-terminal-snapshot.md)/freshterminal원시대조를 먼저한다. 이전archive를덮어쓰거나완료시험을중복재실행하지않는다.
- `estimated_boot_utc`는 wallclock−uptime 계산이라 clock correction/suspend 시 journal의 시작 UTC와 다를 수 있다. Boot UUID를 재부팅 구분의 우선identity로 쓰고 timestamp차이를shutdown원인으로단정하지않는다.
- Snapshot의 host_watchdog_installed=false는 이번프로젝트가watchdog를설치하지않았다는계약표시다. 시스템전체scheduler/services를감사한결과라고확대하지않는다. Windows collector executable_present도eventlog접근성PASS가아니다.
- Freshoutput생성/쓰기실패는스냅샷미완료이며directory존재만으로인수하지않는다. 기존출력을삭제/재사용하지말고새이름으로retry하며완전한JSON과exit를읽는다. 프로젝트case는동일실행context의canonicalroot를쓴다.

## 수동 재개에 필요한 순서

재접속후기존source/evidence/workitem담당을보존하고새boot-onlysnapshot을create-only로만든다. Boot변경이면옛PID에는신호를보내지않고원terminal/provenance를읽는다. Sameboot라도현재실행을추적하지않았다면개발활동은미관측으로보고하고root의소유manifest/commandterminal을별도로확인한다. 실패/중단한명령과이미완료한scope를나눠새fixture/새evidence에서필요부분만재실행한다. 자동복구나모니터off연속성을보장하지않는다.

## 실제 검증과 잔여

`python3 -m unittest discover -s scripts -p test_wsl_interruption_snapshot.py`: exit0,6tests PASS,0.002초. 기존synthetic UUID/bootmismatch/권한/namespace경계만다시확인했다. 새로운hostsnapshot·Windows·reboot·서비스·Rust·Cargo실행은없고기존raw evidence는읽기만했다. 이번완료는누락감사/수동handoff이며체크리스트owner의후속문서갱신과현재개발프로세스명시manifest는잔여다.

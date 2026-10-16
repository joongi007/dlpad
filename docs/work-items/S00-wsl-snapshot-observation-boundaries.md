# S00: WSL snapshot 중단 관측 경계 보강

- 상태: completed (단일 관측 준비 범위)
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R11
- 담당 파일: soak_recovery_audit — scripts/wsl_interruption_snapshot.py, scripts/test_wsl_interruption_snapshot.py, docs/validation/wsl-interruption-recovery.md, 이 문서
- 의존성: 기존 snapshot/event collector와 terminal 감사, root host 실제 조회

## 범위와 인수 조건

Boot UUID 검증과 관측 namespace metadata를 보강하고 PID 부재/재사용/읽기 실패를 WSL 중단으로 자동 판정하지 않는다. 완료된 과거 launch가 현재 live 선언이 아님을 출력에 명시한다. Synthetic mock 시험으로 다른 boot일 때 oldPID 읽기 없음, UUID 거절, namespace/권한오류 관측 제한을 확인한다. 전원·보안·서비스·watchdog·자동재시작·Rust·GUI는 변경하지 않는다. Windows event collector는 고정 log/bounded/privacy 계약을 읽기 확인하고 이번 작업에서 실행하지 않는다.

## 검증과 한계

`python3 -m unittest discover -s scripts -p test_wsl_interruption_snapshot.py`: exit0, 6 tests PASS, 0.002초. Boot mismatch와 malformed UUID에서는 PID read가 없고 case-normalized boot/정확 start identity를 확인한다. Local absent는 host shutdown proof가 아니며 permission/malformed 입력과 namespace 비가용을 명시한다. Windows collector의 fixed logs/provider IDs/max32/64KiB/create-only/privacy 계약은 그대로이며 실행하지 않았다.

Source freeze 이후 root가 새로운 host read-only create-only snapshot을 실행할 수 있다. 개선된 collector의 실제 host JSON은 아직 미검증이다. 이전 최신 snapshot20:54:18Z는 이전 script source에서 생성된 자료로 보존한다. 완료된 기존8h는 terminal 증거로 분류하고 옛 launch의 running을 현재 생존으로 보지 않는다. 호스트 watchdog·알림·자동재개·WSL 종료 중 기록은 구현하지 않았다.

Root 독립 synthetic6tests PASS 및 실제 create-only `wsl-monitor-off-boundaries-20261004/snapshot.json` exit0 이후 원시 JSON을 다시 대조했다. UTC2026-10-03T21:59:25.361039Z, 같은 boot, uptime39397.53초, Interop enabled이다. namespace metadata를 기록하되 host_visibility_verified=false/absence_is_not_wsl_shutdown_proof=true를 유지하며 runner 추적 없음이 명시됐다. 위 pending 문장은 실행 전 기록이고 이 후속 검증으로 단일 관측 준비 범위만 완료됐다. Windows 재실행·실제 reboot/중단·모니터off 연속성·watchdog·자동재개는 미검증/미구현이다.

# S00: CPU paged preview 통합 idle 비용

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R01, R08, R11
- 담당/수정 범위: metrics_recovery — 전용 cost workitem/evidence/doc만
- 선행 작업: root frozen CPU SHA166946017de0b3d0589bf3d02bd864d004d90f3457ff2e314fe5e10a5f865417

## 사용자 결과

같은10초 baseline에서 paged preview 통합 후보의 앱 perPID RSS와 idleCPU를 관측한다.

## 계약과 범위

기존 accessible_performance_probe.py read-only 한 번. private Xvfb/DBus/임시folders 및 기존malgun폰트 읽기만 사용한다. source/probe/Cargo/다른 소유 시험 프로세스를 변경하지 않는다. root가 두 headless8h soak를 launch중이며 별도 공식 toolinstall GUI가 병행될 수 있는 host load를 명시한다.

## 인수 조건

exact byte size/SHA 보존, 앱 perPID10초표본과 cleanup 확인. 단일코어1% 미만/80MiB 제안예산 비교를 기록하되 전체 기능·대형보고서 peak·8h·다른OS를 대신하지 않는다.

## 검증과 handoff

실제 명령과raw evidence는 docs/validation/cpu-paged-cost.md를 따른다. exit0,10.013초 표본 CPU0.4993%/RSS54080KiB/12402040bytes/SHA보존, owned GUI/bus/fixture cleanuptrue. ancillary hostload 및 좁은 baseline 범위를 기록했다. GUI를 SIGTERM으로 회수하는 probe이며 정상 사용자 CloseWait 인수는 아니다.

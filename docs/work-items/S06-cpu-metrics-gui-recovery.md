# S06: WSL 재시작 뒤 CPU 지표 GUI 복구 검증

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R01, R06, R08, R11
- 담당/수정 범위: metrics_recovery — scripts/cpu_metrics_probe.py, 전용 cpu-metrics 문서/evidence
- 선행 작업: root 공유 UI 연결과 frozen Linux release 3a1bded7…

## 사용자 결과

대상 볼륨 여유와 선택 인터페이스 지표의 성공·실패 입력을 격리 Linux 화면에서 확인한다.

## 계약과 범위

사용자 데스크톱/클립보드/실제 미디어/외부 네트워크 없이 private Xvfb/DBus와 임시 폴더만 사용한다. 공유 source/Cargo와 이전 장시간 시험 프로세스는 변경하지 않는다. 기존 raw first 결과와 WSL 중단된 inputfix 산출물을 보존한다.

## 인수 조건

statvfs와 반올림 표시 일치, lo 실제 관측, invalid 선택의 이전값 보존, 없는 인터페이스 측정 불가, BackSpace 뒤 actual Text.GetText가 빈 값인지 확인한 off 동작, 반복 관측 및 정상 종료/cleanup/hash 보존. 단위 시험과 다른 OS 전체 R01 및 장시간 kernel stall/CloseWait 지속은 별도 인수다.

## 검증과 handoff

두 fresh run exit0/PASS. 최종 cpu-metrics-wsl-recovery-exact는5개 actual Text.GetText 값, statvfs 반올림 비교, 정상 종료0.153초, owned cleanup 및 source/binary hash 불변을 기록했다. 실제 명령과 raw 링크는 docs/validation/cpu-metrics.md를 따른다. 지속 CloseWait/다른OS/FFmpeg 및 peak 비용은 미검증이다. 공유 소스 통합 검증은 root 담당이다.

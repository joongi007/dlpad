# S06: CPU 볼륨·인터페이스 지표

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R01, R06, R08, R11
- 담당/수정 범위: library_identity — 신규 accessible_metrics.rs 및 전용 문서
- 선행 작업: 기존 engine.metrics API; root module/Candidate 연결

## 사용자 결과

대상 볼륨 여유와 명시적으로 선택한 인터페이스 전체 트래픽을 초당 최대1회 관측한다. 미지원·실패·3초 초과 stale는 측정 불가로 표시한다.

## 계약과 범위

단일 worker, command/result channel1 및 최신 설정 한 개. revision과 관측시각으로 이전루트/인터페이스 결과를 차단한다. UI thread filesystem/network I/O 없음. interface64 ASCII chars, root OS 표현32768bytes 상한. 빈interface는off, invalid 선택은 기존 설정 보존. 처리 ProgressFeed는root가연결한다. worker stopack/panic fixed unknown; 새deps/Cargo/schema 변경없음.

## 인수 조건

invalid/stale/revision/root상한/workerpanic/stop 회귀. 실제OS지표·GUI·성능은 최초 연결 후별도 검증. single syscall 중 즉시취소를 보장하지 않는다.

## 검증과 handoff

owner 파일 own rustfmt 뒤 freeze 및 root 최초 컴파일을 요청한다. 미실행 시험은 완료로 쓰지 않는다.

## GUI 담당 범위 추가

신규 scripts/cpu_metrics_probe.py 및 전용validation은 library_identity 담당이다. ownedXvfb/privatebus/temp media에서 actual statvfs와 roundedUI 비교, blank off/invalid 기존선택 보존/lo실측/없는interface unknown/관측주기/종료를 시험한다. 사용자desktop/clipboard/외부network 없음, root READY 전 GUI실행 금지. 외부값 지표는 실제측정과 미지원None을 구분한다.

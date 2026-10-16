# S00: CPU idle 비용 회귀 진단

- 상태: done
- 분류: bug
- 우선순위: P0
- 요구사항: R01, S00
- 담당/수정 범위: diagnostic_cache; 전용 workitem/validation/새 evidence prefix만. 기존 performance probe/source 읽기전용
- 선행 작업: root fixed CPU522e, 모든 다른GUI/Cargo시험 종료

## 사용자 결과

idle CPU<1% 제안 기준을 실제 같은 loop로 재측정하고 UI polling/worker wake/부하노이즈 가설을 분리한다. budget을 완화하지 않는다.

## 계약과 범위

diagnose 스킬과 docs/agents local tracker 계약. ownedXvfb/privatebus/temp/freshdata only, 사용자desktop/audio/clipboard 없음. 기존 scripts/accessible_performance_probe.py 읽기전용. root가제품 activation callback 수정담당. source/binaryhash 및ownedcleanup 기록. baseline1회,패치후동일loop1회로측정범위고정.

## 인수 조건

같은10초warm idle loop,clockticks/elapsed/RSS와원결과보존. 최초1.0985%를성공으로쓰지않음. 재현이안되면노이즈가능성과실제표본한계를명시. source변경은root담당/agent무변경. 최종same-loop회귀수치와원증거/결정반영.

## 검증과 handoff

baseline측정준비. 기존root accessible-performance/cpu-parity1.0985%/53024KiB. ranked 가설: (1)250ms activation repaint 제거시idle낮아짐,(2)IPC worker50ms polling을wake-driven으로바꾸면threadCPU감소,(3)다른부하제거한동일binary 재실행만으로낮아지면측정노이즈. 각가설은baseline뒤root단일변수패치로검증하며source만으로확정하지않음.

## 실제 회귀 결과

root callback-only후보 b8bbf93a2cc4b05dfe202edfe1c92fca6aff7566b694eb63f4427df7d7a87409에동일probe1회 actual exit0,10.012622s idle0.099874%/RSS52800KiB. baseline반복0.499357% 대비감소/target<1 유지.단일표본 노이즈한계/기존1.0985실패원시보존. rootruntimeIPC callback시험1개/clippy/release별도PASS보고.제품/probe무변경/모든소유GUI종료. cpu-idle-regression-validation.md/comparison.json.

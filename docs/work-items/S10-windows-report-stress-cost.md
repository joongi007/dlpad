# S10: Windows 대형 보고서 비용 검증

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R01, R13
- 담당/수정 범위: root는 이 문서·후속 collector 통합·실행·독립 검증, metrics_recovery는 /tmp/windows_report_stress_cost_plan.md 읽기 감사 초안만 담당한다.
- 선행 작업: 대형 보고서 GUI 저장·페이지 시험, 원 Job 비용 collector, 고정 Windows CPU release.

## 사용자 결과

대형 진단 보고서를 표시하고 저장할 때 앱 비용을 실제 Windows에서 측정한다.

## 계약과 범위

Rust 네이티브·격리 desktop·합성 fixture·원 프로세스 HANDLE과 Job identity를 유지한다. 제품 source와 Cargo 변경은 root가 조정한다. idle 비용을 stress peak로 표현하지 않는다. 사용자 화면 및 클립보드는 사용하지 않는다.

## 인수 조건

원 app/Job의 working set·peak working set·CPU를 보고서 생성 및 페이지·저장 구간에서 수집한다. 측정 구간과 관측 비용, 놓친 순간 표본의 한계를 기록한다. source/binary SHA와 자연 종료·Job0·desktop 정리를 독립 검증한다.

## 검증과 handoff

현재 collector 재사용 읽기 감사 중이며 비용 인수는 미완료다. 먼저 기존 bounded 메모리 수집 API와 대형 report 시나리오의 연결 초안을 검토한다.

root가 staging SHA와 실제 diff를 검토하고 scripts/windows_s10_stress_cost_probe.cs 및 scripts/windows_s10_stress_cost_native_probe.py를 게시했다. 원 app/Job HANDLE을 non-inherited duplicate로 보존하고 Start 직전부터 terminal UI까지50ms/최대2000회/95초 원app 표본을 수집한다. OS lifetime peak WS와 sampled private/WS를 분리하고 Job CPU 차이를 기록한다. 전체 Job 메모리는 미측정이며 UIA로 유도된 앱 작업과 짧은 표본 누락을 명시한다. 기존90초worker·5초sampler join·오류시FAIL·원 cleanup을 유지한다. 새 Rust/dependency/lock 변경은 없다. 첫 JSON actual native compile/실측을 시작하며 통과는 아직 미확정이다.

첫 JSON 실제 시험 windows-report-stress-cost-json-20261005-resume은 기능·수집기 실행을 통과했다. root가537개 native표본의 최댓값·시간순서·stage·source SHA·JSON/Failed DB·원Job0/private desktop을 대조했다. 구간33.27초, sampled app WS85721088B/private51146752B, OS lifetime peakWS87851008B, appCPU9.44%/JobCPU9.58%이다. idle1%/80MiB는 이 활성 보고서 구간의 PASS/FAIL 기준으로 적용하지 않는다. wholeJob메모리·짧은표본누락·quietHost는 미검증이며 stress 자체 예산은 별도 미정이다. PNG 및 나머지 두scenario 비용 인수는 후속이다.

Markdown 비용 actual시험은533표본/33.05초 sampledWS85835776B/private52072448B/OSlifetimepeak93528064B를 관측했고 root가 raw extrema·source·Markdown·DB·cleanup을 대조했다. 첫 preview 비용 시험은 Start 전 합성 cookie경로 입력에서 windows의 n 한 글자가 빠져124/125 exactreadback 실패했다. 비용표본 없음이며 원실패와 강제Job정리를 보존한다. 원 preview 기능 실패로 확대하지 않는다. root는 글자별 실제 prefix 확인(최대10×25ms·재입력없음)을 추가해 다음글자와겹치는 관측조건을 줄인다. 원90초worker 상한은 유지하며 corrected fresh재시험한다.

Corrected preview-prefix fresh 시험도 nativePASS이며 root가328표본/20.28초의 source SHA·극값·원DB·first/next/last 및 PNG·Job0/private desktop을 확인했다. sampledWS85692416B/private52805632B/OSlifetimepeak87064576B, appCPU10.40%/JobCPU10.63%였다. 글자prefix 검증 이전124/125입력 실패는 보존하며 missingchar원인확정은 하지 않는다. JSON/Markdown 실제PNG도 root 확인 완료다. 세scenario의 좁은 비용 관측은 완료했으나 전체Job메모리·quietHost·stress 자체예산/장시간·clipboard는 미완료다.

후속 root 담당 새수집기 scripts/windows_s10_job_peak_cost_probe.cs/.py: 원duplicatedJob HANDLE의 info9 PeakJobMemoryUsed/PeakProcessMemoryUsed를baseline/50ms/terminal에서수집하고query실패는전체FAIL로기록한다. 원app WS/private/OSpeakWS와Job committed peak를구분하며wholeJob RSS나개별순간child추적을완료로주장하지않는다. 원상한·privacy·Job정리조건을유지하고header reveal을최신ScrollItem→focus→bounds 순서로교정한다. [Microsoft 원계약](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_extended_limit_information)의상시peak추적을확인했으며실제나온값과측정구간은후속native결과로판정한다. native product/Rust/dependency 변경0.

2026-10-05 Job committed peak 3scenario actual native PASS와 root·독립 raw review 완료. JSON533표본/33.370초/Job peak54,484,992B/export1,841,855B; Markdown533표본/33.298초/63,025,152B/export1,933,102B; preview310표본/19.378초/53,329,920B/236페이지 경계를 확인했다. evidence/windows-s10-job-peak-{json,markdown,preview}-20261005-resume 각각 root-review.json/independent-review.json에 source535동일·원파일전SHA·Failed1/attempt1/artifact0·원naturalJob0·desktop정리를 기록했다. 초기 query성공과peak통합은 확인했지만 baseline 숫자는 별도 serialized되지 않아 첫sample을exactbaseline으로 취급하지 않는다. baseline raw 보강과 실제 native query 실패 경로 후속 시험을 진행한다. wholeJob RSS/quietHost/active budget/reader/장시간은 미검증이다.

후속 baseline/negative 전용 collector4files를 root diff검토 후 게시했다. baseline 원info9값/JobCPU ticks와 terminal 숫자를 boundedraw schema3에 보존한다. negative는 sample info9에만 fixedinvalidHANDLE을 넣은 시험 전용파일이며 외부command/environment override0, raw statusFAIL/exit2/original passedfalse를 유지하고 expectednegative만 별도로 인정한다. 인수에는 측정성공false·원 fault·자연close/Job0·MTA desktop복원·privateDesktopclose/inputSame·source/immutableSHA를 모두 요구한다. 제품코드/Cargo 비용변경0. 실제C#compile/native는 이제 수행한다.

Baseline JSON 실제 PASS/533표본: 40,558,592B→54,898,688B. root가 원시 극값·Job CPU ticks 계산·summary 일치를 확인했고 독립 검토도 PASS다. negative 실제 Win32 6/sample0/statusFAIL/exit2/measuredfalse와 자연 WM_CLOSE·MTA/private 복원·source 일치를 root와 독립 검토에서 확인했다. expectednegative만 PASS이며 원 native 실패는 보존한다. collector baseline/raw 실패 보고 게이트는 완료했고 Job RSS/quietHost/active budget/장시간 게이트는 미완료다.

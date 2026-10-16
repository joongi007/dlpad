# S10 CPU 진단·MPD 펼침 상태 GUI

- 상태: done
- 분류: verification
- 우선순위: P0
- 요구사항: R13, R08
- 담당/수정 범위: stress_recovery; scripts/cpu_diagnostic_expander_probe.py, 이 문서 및 전용 evidence/validation. 제품/shared harness/Cargo 변경 금지.
- 선행 작업: root accessible collapsing metadata·vendor 연결과 새release READY

## 사용자 결과

작은 실제 Failed 보고서 및 로컬MPD 첨부의 펼침/접힘 상태를 native 접근성으로 구분하고 실제 action으로 전환한다. 진단 열람/첨부는 원job과attempt를 바꾸지 않는다.

## 계약과 범위

AGENTS/README/PRODUCT/CONTEXT/R13/R08/ADR0004/DESIGN과 기존 cpu diagnostics/stress harness 및 locked atspi-common State enum을 읽었다. Expandable bit9/Expanded bit10. 기존 cpu_report_stress_probe.widget/scroll_outer/state를 readonly import하며 DLPAD_TEST_REPORT_STRESS는 제거한다. 새 temporary appdata/media/privateXvfb/DBus/합성tool만, 사용자desktop·clipboard·실영상·network·auth 금지.

## 인수 조건

exact rootREADY SHA, 실제 Failedjob/rev/attempt snapshot, diagnostic header closed/open/closed/reopen metadata, nativeattach한작은MPD0 header same toggles. 실제 visible role43/extents/Enabled 상태/action과 closed/open state기록. UI문구만찾아PASS하지않고 bits와원job/attemptDB를검증한다. 입력MPD before/afterhash 및owner 정상exit/process cleanup/SHA를보존한다. unit이GUI증거를대체하지않는다.

## 검증과 handoff

collector준비중이며 binary실행은root 새READY 뒤로보류한다. 기존8h PID/기타agent 소유파일수정·하위spawn없음. 다른OS/전체readerUX/최대report는별도scope다.

## 완료 근거 (608eeeb… Linux 한정)

root 새READY 뒤 실제current-1 exit0/PASS. 두header role43 nativeAction/GetState의Expandable 항상true/Expanded false,true,false,true와 작은MPD본문을 확인했다. 최초Failedjob/revision3/1Attempt/options와DB exact일치/integrity, MPDsourcehash불변, 정상exit/SHA유지/ownedPID·runtime부재 직접확인. [원시검증과미검증](../validation/cpu-diagnostic-expander-validation.md)을 따른다. 전용collector/docs만변경했으며 다른OS/animation모든frame/전체reader사용성은별도릴리스게이트다.

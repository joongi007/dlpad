# S01: 주기 재확인의 CPU 분산 실험

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: root 전용testfixture/실행·원결과·계약판정·문서; soak_recovery_audit /tmp native test 초안만
- 선행 작업: 최신전체/GUI CPU실패·freshquery비용0.328초/1만·relativeAPI이득없음

## 사용자 결과

주기 재확인의 전체fresh검증을 유지하면서 비상호작용 작업의 비용을 분산할 수 있는지 측정한다.

## 계약과 범위

제품30초hint·1%/10초/60초/80MiB예산은변경하지않는다. 먼저기존private Engine FnMut checkpoint로experiment만실행한다. 모든파일fresh조회·원ID/root검증·whole transaction게시·취소rollback을유지하고sleep은10ms이내slices로쪼갠다. 1msactivewall/250ms계획은singlethreadwall기반근사이며nativeI/O interrupt나hard CPU보장이아니다. 느린전체sweep는hint30초와실제게시지연을분리해보고한다. 워크로드빈도가같으면pacing만으로평균CPU를줄이지못하므로이실험을현재P0해결로주장하지않는다.

## 인수 조건

원1만fresh조회eventualcommit/aliasinside-outside원size/mtime/ID·DB재열기·bytes, paced sleep중Stop/typed supersede→원DB보존·별도fastretry, 원실행시간·모든10초측정·query/sleep상한 보존. 제품채택에는watchtriggerreason·boundedcommand선점·빠른실이벤트retry·starvation·장시간SQLitewriter영향 및추가전체/GUI검증이필요하다.

## 검증과 handoff

전용/tmp test 초안작성중이며현재default제품과reconcile주기는그대로다. ARCH의30초는hint이고hardcommit SLA는아니지만latency정책의실질변경은실험결과와명시ADR없이는채택하지않는다.

### 실제 첫 실험 종료

Windows native fmt(의도한 source format), fixture 포함 Clippy, release Stop/supersession 시험, explicit ignored 전체 fresh scan 시험이 exit0이다. format collector의 source-mutated/passed=false 원값은 보존한다. 전체 10,001파일 scan138.417초/CPU0.75초/80,025checkpoint/13,797sleep이며 원 alias13B·ID·DB 재열기·bytes를 유지했다. 평균 약0.542%지만 30–40초 원구간CPU1.24995%로 예산 미달이다. 기존 빠른 scan0.328초보다 총CPU가 늘어 이10ms polling pacing은 제품에 채택하지 않는다. root 원결과 검토와 독립검토는 추가 GUI나 앱 StopACK 증거가 아니다. [원증거](../validation/evidence/windows-periodic-pacing-experiment-20261005-continue/result.json). 이후 추가 fixture로 인해 이전541소스 전체검증은 당시 snapshot이며 최신전체를 재실행한다.

### 이벤트 알림 대안 실험

root가 default-off event_wake_pacing_experiment fixture를 게시했다. 10ms polling 대신 보호된 interruption flag+Condvar의 epoch/sample/180초deadline 대기를 사용한다. Stop/Supersede는 flag set 후 notify하여 대기 직전 신호 손실을 막는다. 동일1ms/250ms 계획과 모든freshquery/원ID/wholecommit을 유지하고 제품정책은 변경하지 않는다. 첫INSERT 뒤 실제대기/notify rollback, 1만 전체원bytes·alias13·DB재열기·모든10초CPU표본을 actual Windows에서 실행 중이다. 새로운대기전략만 바꾸며 느린완료를30초SLA/GUI예산통과로 표시하지 않는다.

### 이벤트 대기 실제 결과

Windows fixture포함Clippy·두앱페이지각18·Stop/supersession rollback1·원ignored전체시험이exit0이다. eventwake전체scan167.825초/CPU0.640625초/wait666회이며모든완전10초원표본은1%미만이다. root가readonlyDB10001행/무결성·원1만payload·alias13·sentinel을확인했고독립검토와원Job/private/input도대조했다. 기다림횟수는줄었지만빠른scan0.328CPU보다비용이높고완료지연도늘어제품에는미채택이다. 이것은GUI·동일30초workload budget통과가아니다. [원검토](../validation/evidence/windows-event-wake-pacing-experiment-20261005-continue/root-review.json).

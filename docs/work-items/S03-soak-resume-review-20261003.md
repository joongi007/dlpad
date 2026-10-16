# S03: 기존 8시간 합성 시험 재개 검토

- 상태: completed
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R08, R10
- 담당/수정 범위: guardian_fault; 이 신규 작업문서, docs/validation/soak-restart.md, 기존 soak-restart-20261002 evidence만
- 선행 작업: S04-soak-restart 및 기존 download13723/guardian64092/archive21052

## 사용자 결과

일시중지 뒤 기존 장시간 실행이 실제 살아있는지·종료했는지 정확히 확인하고, 실행 중간 상태와 8시간 완료 증거를 구분한다.

## 계약과 범위

host boot/PIDstart/실제 status/result/execterminal 읽기만 먼저 수행한다. 제품/Cargo/공유문서/GUI 수정 없음. 기존 실행 재시작·종료는 terminal과소유권 검토 후root합의 전금지. 관측timeout은restart근거가아니다. 사용자desktop/Documents/미디어/계정은사용하지않는다. 별도S10stress준비는새담당범위승인전이작업에혼합하지않는다.

## 인수 조건

두run+archive 정확boot/PIDstart와terminal결과를bounded JSON에기록; 완료result/exitcode/cleanup/DB증거없으면PASS미주장. 진행중이면live와남은monotonic시간/증거scope기록. 원증거와archive상한유지. root에실제결과와미검증gate전달.

## 검증과 handoff

초기host관측: boot794988eb-539b-437e-8396-330738b565bb 유지. download18393.03초/guardian18391.46초running,9PIDstart모두MATCH. archive6202/start48872watching,305historyrows/523644B. terminalresult아직없음. 실제execpoll와최신독립observation 후문서갱신예정.

재개review완료(8h시험완료아님): 세execpoll stillrunning,host정확identity검증+resume-review JSON+soakdoc갱신. terminalexit/result없으므로8h gate미완료. 재시작/종료권한새합의없음/실행변경0.

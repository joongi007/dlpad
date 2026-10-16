# S10: Windows 정상 크기 report 최종 수집 준비

- 상태: completed
- 분류: validation
- 우선순위: P0
- 요구사항: R13, R10, R11
- 담당/수정 범위: soak_recovery_audit — scripts/windows_cookie_gui_probe.ps1(준비 감사), 이 문서
- 선행 작업: normal report scroll-action 실패 증거 및 root/metrics ancestor scroll 수정

## 사용자 결과

960×640 private GUI 재시험 collector와 native runner가 실제 새 immutable binary·source 입력·로그를 보존할 준비가 됐는지 감사한다. Geometry 교집합을 실제 preview 글자 가시성 PASS로 바꾸지 않는다.

## 계약과 범위

기존 private PNG/body 가시성 판정은 root 담당이고 collector REVIEW_REQUIRED를 유지한다. Cargo/GUI 실행, 제품/공유 파일 변경, 사용자 desktop/clipboard/global input은 없다. Native runner와 final session56854 evidence를 읽기만 하고 기존 원시 자료를 덮어쓰지 않는다.

## 인수 조건

- 현재 collector의 glyph pending/actual-visible false 및 순차 header/export bounds proof를 확인한다.
- Native runner의 fresh output/SHA/immutable/tool/private host/timeout/입력 source provenance 계약을 확인한다.
- 최종 로그/inputs의 실제 보존과 완료 여부를 계획 또는 실행 중 상태와 구분한다.

## 검증과 handoff

Collector source SHA `0d9a9f953ab8281b07efa9689e5b50b022c8f1b94ffad6f6d95c9b17451bf30c`를 읽기 확인했다. Preview geometry intersection은 true일 수 있어도 actual_visible=false/glyph pending/root image review와 final reachability=false를 유지하며 auth 결과는 REVIEW_REQUIRED이다. Private image에 글자가 없으면 root가 가시 PASS로 인수할 수 없다. Header/export 실제 bounds/IsOffscreen proof, readonly=true/TextPattern/privacy 조건은 유지한다. Script source는 이번 준비 감사에서 수정하지 않았다.

`/tmp/dlpad-windows-cookie-gui-report-scroll-action-native.py`를 읽었다. 기존 runner는 old immutable CPU SHA `5c614db7275d3eaba2cab5a1429dceef7a055504251bf46d22a6f8482e31326e`와 이미 존재하는 scroll-action output에 고정돼 새 시험에 그대로 재사용하면 안 된다. Host는 owned Job/deadline/stdin 취소를 제공하고 source/tool/binary pre/post SHA를 기록하지만 `.local-tools/windows/logs/check.log`/check-inputs.json은 다음 operation에 덮이는 위치다. Native runner의 모듈 본문은 즉시 실행되므로 import해서 준비만 확인하지 않는다.

Root 승인 후 현재 `.local-tools/windows/logs/check-inputs.json`을 먼저 대조했다. Old operation argv OutputDirectory=windows-cookie-gui-auth-report-scroll-action, mode download-failure/두 verify switch, source SHA `4c38231c491b274ec36d168030fdddff5e15abafe46dab2f5948708257490932`, before=after/unchanged=true/exit1이며 해당 result binary SHA와 readonly visibility 실패가 일치했다. 이에 두 원시 파일만 기존 evidence 아래 아직 없는 이름으로 create-only 복사했다. Source log 변경이 없고 복사 bytes 동일함을 확인했다. check-inputs.json은2111B, check.log는630B이다. 이는 이전 scroll-action 실패의 원시 보존이며 session56854 최종 workspace PASS 증거가 아니다. Session56854의 명령·최종 입력 ledger/terminal 로그 보존은 root 별도 확인이 필요하다.

새 runner 계약: root가 ancestor-fix 후 새 immutable CPU와 immutable synthetic tool copy의 SHA를 명시하고 새 고유 evidence directory를 사용한다. 현재 tool은 live target/debug/examples path이므로 다음 operation은 독립 copy가 필요하다. 현재 script SHA/host SHA/새 CPU/tool SHA를 launch 전·후 모두 기록하고 pipeline source manifest도 root가 보존한다. Normal960 시험에는 report height option 없이 -VerifyReport/-VerifyReportReachability를 명시한다. Controller stdout/inputs를 operation terminal 직후 fresh evidence로 옮기고 다음 native 실행 전에 raw 보존을 확인한다. Source mutation이나 pre/post mismatch면 새 source 결과로 재인수해야 한다. Root가 직접 private screenshot body·preview 부분 글자·전체 export/DB/privacy/sentinel·owned cleanup을 검토한다. Agent는 Cargo/GUI 실행을 하지 않았다.

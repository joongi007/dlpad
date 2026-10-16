# S02: guardian의 쿠키 reader 시작 허가

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: root — process.rs, process/guardian.rs, tests/cookie_guardian_gate.rs, 이 문서
- 선행 작업: split I/B socket 교환31시험

## 사용자 결과

명시 gated guardian은 Begin index/nonce 일치 전에는 외부 도구를 생성하지 않는다. EOF·부분 frame·잘못된 nonce·deadline은 실행 없이 거절한다.

## 계약과 범위

전용 internal marker로만 선택한다. 기존 ungated argv/guardian 정책을 유지하며 gate뒤에 control drain thread와 tool spawn을 시작한다. 실제 service launcher I/cookie prepare wiring과 receipt 생성은 별도 연결이다. native process test는 이번ownedhelper/socket/temp 합성툴만 사용한다.

## 인수 조건

실제 helper에서 B전 toolstarted0/B후toolstarted1, wrongnonce/EOF no tool, stdout/controlFD 상속거절·소유종료·bounded waiting. 전체 engine회귀와clippy통과.

## 검증과 handoff

root 구현·native시험 진행 중.

Root 실제 helper 빌드와 Cargo cookie_guardian_gate 시험 exit0: native2+codec4 총6 PASS. 실제B전도구미실행/B후실행/RGC+EOF, wrongnonce/partialEOF는GU+exit126/도구미실행, stdin null 및 control socket 미상속을 확인했다. [증거](../validation/evidence/cookie-guardian-begin-gate/native-review.json). user-service 실제gate/I생산/receipt writer/쿠키adapter는 별도연결이다. strictclippy 진행중.

후속 전체engine회귀는 기존 guardian integration의 moved-group leader에서 TerminalUnconfirmed1개로exit101. 원시 engine-first.log를 보존했고 고정driver/helper로 반복진단을 별도작업에 배정했다. 새gate native2/codec4 및strictclippy는통과했으나 전체인수전완료상태를유지하지않는다.

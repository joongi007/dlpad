# S02: 시작 허가 소켓 교환 실제 검증

- 상태: done
- 분류: test
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: root — cookie_reader_models.rs, 이 문서, 검증 evidence
- 선행 작업: reader_gate 구현 동결

## 사용자 결과

실제 Unix 소켓에서 durable bind callback 성공 전에는 시작 허가 B를 보내지 않으며 deadline·취소·잘못된 frame을 거절하는 기반을 검증한다.

## 계약과 범위

모듈을 Cargo integration 시험에 연결한다. 제품 guardian/launcher/쿠키 journal 연결과 native cgroup 증명은 아직 별도이며 이 시험으로 완료를 주장하지 않는다. Cargo dependencies 추가 없음.

## 인수 조건

정상 순서와 socket timeout 복원, bind 실패 및 deadline 후 B0, 부분 frame과 nonce mismatch, EOF/상한/취소 실패를 실제 소켓으로 확인한다. 기존 journal/model 회귀도 통과해야 한다.

## 검증과 handoff

Root Cargo 결과 대기.

## Root 실제 통합 결과

현재 실제 소켓 gate·journal·cgroup 관련26개 시험 PASS. 디스크 bind→B→receipt 검증→durable Resolve→reopen까지 하나의 임시 폴더·Unix socket 시험으로 연결했다. 최초 fixture의 mode와 retainedlock 오류는 수정하고 원시 실패를 보존했다. 현재 소유 시험프로세스의 실제 cgroup은 native pinned FD로 nonempty 거절1개 PASS. 제품 launcher/nativeempty receipt 생성은 아직 미연결이며 합성 identity/receipt를 실제 service proof로 확대하지 않는다. clippy 후속 검증 진행 중.

Root 후속 strict engine all-targets clippy exit0. 이 작업은 소켓/journal/cgroup 기반 통합 시험 완료이며 제품 launcher gate 연결 전체는 미완료다.

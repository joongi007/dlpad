# S03: 미리보기 관리 도구 실행 수명

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R03, R05, R06, R08, R11
- 담당/수정 범위: root app worker/workspace/catalog 통합; engine media_preview.rs 후보 분리 담당; schema/Cargo.lock 변경 없음
- 선행 작업: 관리 실행 guardian·owned Deno cache 및 bounded preview worker

## 사용자 결과

관리 도구를 미리보기에 사용하는 동안 설치/제거와 실행 수명을 연결한다. 원 cache 정책을 yt-dlp에 전달하고 외부 도구의 데이터를 소유 cache처럼 처리하지 않는다.

## 계약과 범위

각 preview request마다 ManagedExecution journal scope를 만들고 원 reader 종료 확인 후 finish한다. worker 전체 수명으로 scope를 붙잡으면8 reader slots를 소진하므로 금지한다. canonical 또는 lexical 관리 경로의 검증 실패를 direct 실행 fallback으로 바꾸지 않는다. 관리 I/O는 worker에서만 한다. 미확인 cleanup/finish 실패는 sticky fatal이며 원 pending marker를 보존한다. 새 dependency 없음; request마다 proof·guardian 시작 비용은 현재 후보에서 다시 측정한다.

## 인수 조건

연속9회 이상 seek·취소·deadline·관리 root 변경·동시 mutation 거절·ambient Deno cache 차단·원본 외부 파일 보존·Stop ACK와 미확인 cleanup marker 보존을 검증한다.

## 검증과 handoff

2026-10-11 구현 중. 이전957 비용/GUI 증거는 새 관리 경로 검증으로 소급하지 않는다. scope creation busy는 명시 실패이며 조용한 direct fallback은 하지 않는다. download 시작 전 preview ACK 대기 UX는 통합에서 확인한다.

2026-10-11 root 추가: worker panic의 stop ACK를 process cleanup 성공으로 오인하지 않도록 독립 atomic uncertainty를 유지한다. 취소/ACK 뒤에도 OwnershipUnconfirmed가 sticky이며 신규 실행을 차단하는 회귀를 추가했다. 실제 신규 검증 전이다.

실제 원 pending marker가 있는 begin_execution 실패를 OwnershipUnconfirmed로 전파하도록 보강했다. 기존 오류 API는 생성 전후를 구분하지 않으므로 lock contention도 보수적으로 terminal 처리한다. `/tmp/dlpad-preview-begin-test.py abandoned_original_scope_rejects_preview_as_ownership_unconfirmed` 실제 1passed/0failed, 원 marker bytes 보존과 후속 실행 거절을 검증했다. 원 실행 process 누수가 있었다는 주장은 아니다.

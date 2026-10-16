# S01: 실제 worker 대기 방식의 감시 비용 대조

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: root 새 evidence fixture/원실행/통합; metrics_recovery /tmp fixture 초안
- 선행 작업: 단일 root 관측과 sleep 대조, 실제 GUI 비용 측정

## 사용자 결과

제품 CPU 초과의 원인을 정책 변경 전에 실제 worker 대기 방식으로 구분한다.

## 계약과 범위

ADR0002와 100ms·1초·30초 정책 유지. 제품 코드/공유 schema/Cargo.lock 수정 없음. fixture에서 live mpsc Sender를 유지한 명령 없는 recv_timeout과 기존 sleep 대조의 차이를 측정한다. fixture 작업과 native 실행은 분리한다.

## 인수 조건

같은 600회·한정 시간·CPU 공식·가상 데스크톱·소유 프로세스 자연 종료·합성 파일 보존을 확인한다. channel disconnect/즉시 반복과 timeout을 구분한다. 단회 비용을 제품 GUI 인수로 확대하지 않는다.

## 검증과 handoff

실제 측정과 원 guard 검토를 완료했다. GUI 성능 인수와 manifest 명시 before/after 핀의 미검증 범위는 아래 기록한다.

### 실제 결과와 증거 범위

lock/clippy/release/experiment4명령exit0. control600Timeout/명령0/sender유지/65.381초/CPU0ticks(해상도이하),watch600회60.263초/0.05186%다. root는원1만payload·sentinel·현재source/CPU/fixture_source핀을검증했다. 독립공식/Job/source감사도일치했다. 복사된inputs의이전experiment_source/manifest핀은역사필드이며수정하지않았다. 현재manifest는원archivedbase에서package이름만바꾼byte와동일하나before/after명시핀은없으므로그guard는미검증이다. 후속quietworker는freshinputs에source/manifest모두before핀을기록한다. 제품대기비용원인이나GUI인수통과로확대하지않는다.

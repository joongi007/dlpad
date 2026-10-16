# S01: 같은 배경 목록의 불필요 화면 전달 생략

- 상태: in-progress
- 분류: performance
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: root crates/app/src/accessible_library.rs·공유인터페이스·통합/GUI·문서; metrics_recovery /tmp 전용회귀; soak_recovery_audit /tmp 소비자/volume호환성 검토
- 선행 작업: engine 정확동일 목록 최적화와 동일UI 관측 감사

## 사용자 결과

목록이 그대로인 배경 재확인은 파일과 DB 확인을 계속하되 같은 화면 결과로 앱을 불필요하게 깨우지 않는다. 사용자 갱신·위치 변경의 응답과 오류·감시 상태 변화는 계속 전달한다.

## 계약과 범위

마지막 성공적으로 큐에 전달한 actual root·rows/error·health와 자율 snapshot이 정확히 같을 때만 send/wake를 생략한다. serial/root 이벤트는 항상 전달하며 Full/stop/disconnect에서는 마지막성공키를 갱신하지 않는다. Snapshot/UI/마지막키의 bounded128행은 Arc<Vec>로 공유하여 큰 경로의 복제 비용을 늘리지 않는다. 쓰기만 하고 표시/소비되지 않은 library private free와 중복 volume query를 제거한다. 표시되는 볼륨 정보는 기존 MetricsPanel의 독립root revision/관측을 유지한다. schema/deps/watch/재확인 주기와 실제scan/cancel/ACK 정책은 바꾸지 않는다.

## 인수 조건

자율 동일 결과의 queue/wake 생략, 행/시간/ID/root/health/오류 변화 전달, 동일 명령 ACK, Full/stop/disconnect와StopACK, Next뒤첫페이지 반환·실제scan실패/원DB보존 회귀를 검증한다. 최신fmt/clippy/test/release와 같은GUI60초×3회 CPU/메모리 원기준을 실행한다. UI 비용 감소 원인을 코드만으로 확정하지 않는다.

## 검증과 handoff

read-only 소비자 검토로 기존 private free의 읽기/표시가 없으며 Apply→output→다음frameMetricsPanel.set_root 독립볼륨 관측 경로를 확인했다. 실제 구현/검증은 후속이다. 이전GUI3회 평균CPU0.833/0.651/0.599%와 메모리는PASS지만 strict10초구간은FAIL이므로 전체P0완료로표시하지 않는다.

2026-10-05 root가 Arc 페이지·마지막 성공 전달 키·private free 제거를 구현하고 독립 읽기 검토를 받았다. 오류→정상과 동일경로의 크기/시간/ID 변화 회귀도 통합했다. 첫 Windows native 컴파일은 기존 rollback 시험의 Vec/Arc 비교 불일치로 exit101이었다(원 evidence windows-library-ui-delivery-20261005-continue). 비교를 dereference로 수정하고 새 evidence에서 재검증 중이다. 계획을 시험 완료로 표시하지 않는다.

후속 windows-library-ui-delivery-fixed-20261005-continue에서 `cargo test -p dlpad --locked --features cpu,library-scan-closewait-fixture accessible_library:: -- --test-threads=1`은 두 접근성 후보 각14회귀를 통과했다. StopACK/실제 scan 중 취소도 포함한다. 같은 feature의 app all-targets Clippy -D warnings도 통과했다. source guard와 원 Job 자연종료·private desktop 종료·input desktop 불변을 확인했다. 문서 검사716개 PASS. 전체workspace와 새GUI는 별도 진행 중이다.

최신 default CPU 전체 `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked --features cpu -- -D warnings`, `cargo test --workspace --locked --features cpu`, `cargo build --workspace --release --locked --features cpu`가 모두 PASS. evidence windows-library-ui-full-20261005-continue/root-review.json에서540소스·원Job4개 자연0·default profile marker 부재와 CPU SHA 3f9d8825f1674b1c682094506f8774df208494dc10ccf1e0655f62d06b8e3667를 대조했다. 새GUI 측정은 진행 중이다.

GUI windows-library-ui-loaded-thread-60sec-20261005-continue 세회차 평균CPU0.728060/0.650116/0.805898%는 모두 원평균<1% 목표를 충족했고80MiB working-set/private 메모리도 PASS. strict10초구간은 세회차 BUDGET_FAIL 그대로 보존한다. source540/CPU 해시 불변·WM_CLOSEexit0·Job자연0·private 종료/input불변, 각DB readonly integrity/10000행 및 native총30000파일 original3bytes를 root-review.json/files-receipt.json으로 대조했다. 구간예산 후속은 별도이며 본작업 전체상태 in-progress를 유지한다.

독립담당의 후속검토도 원540source/4native receipts/3DB를 대조했고 전체30000native 증거와9개원바이트표본을 확인했다. independent-review.json을 root가 통합했다. strictwindowFAIL과thread attribution미확정은 유지한다.

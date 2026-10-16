# S00: Windows 복귀 listener 유휴 event 대기

- 상태: done
- 분류: bug
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root engine/activation/windows.rs, CONTEXT.md, docs/ARCHITECTURE.md, docs/adr/0002-runtime-and-storage.md와 evidence; 하위 soak_recovery_audit 읽기 전용 /tmp 감사
- 선행 작업: 미디어 작업면 최종905 whole와 실제3×60초 비용 실패

## 사용자 결과

유휴 상태의 Windows 숨김/복귀 listener가 20ms마다 깨어나는 동작을 제거하면서 복귀 요청과 종료 취소를 보존한다.

## 계약과 범위

원 named pipe/overlapped/stop Atomic/OwnedHandle을 유지한다. 서버 connect 대기에서 원 완료 event와 새 stop event를 함께 기다린다. stop을 완료보다 우선하고 Pending Drop의 원 I/O cancel/drain과 token identity 정리를 보존한다. 외부 프로토콜/schema/Cargo 변경 없다. Library/metrics worker의 별도 비용과 UI/라이브러리 기능은 이 수정으로 성공이라고 주장하지 않는다.

## 인수 조건

실제 pending ConnectNamedPipe에서 120ms 연결 없는 구간의 native wait call은1회이며 stop event로 즉시 취소/drain한다. 종료·동일 사용자 nonce 복귀·잘못된 ACK·deadline 실패 회귀를 Windows native로 통과한다. 이후 whole와 새3×60/18window 비용을 실행하며 기준을 완화하거나 실패 자료를 지우지 않는다.

## 검증과 handoff

미디어 cost-final은18개 구간 중2개가1.098700%/1.103177%로1% CPU 기준을 넘었다. 모든 sampled WS는80MiB 이하였다. 원 thread snapshot에서 restore/library/metrics 활동을 확인했지만 thread churn 때문에 정확한 operation attribution은 미확보다. 소스에서 실제 server Pending::finish의20ms polling을 확인했다. 이 수정이 전체 CPU 초과를 해결하는지는 실측 전이다.

실제 red: 원20ms poll로 wait4회/기대1회가 실패했다(`windows-msvc-restore-idle-red-20261007`). green: 원 완료와stop event를 WaitForMultipleObjects로 기다려1회이며120ms 후stop/cancel/drain이 통과했다(`windows-msvc-restore-idle-green-20261007`). 테스트 counter는 cfg(test) 전용이며 제품에는 counter/추가 thread가 없다. 전체 검증은 event-idle whole 진행 중이다.

최신 event-idle whole906/0/16, source608,fmt/clippy/test/release모두exit0. idle/partial peer의bounded timeout과원serverDrop cancel·same-user nonce 복귀 실패 회귀가 함께 통과했다. CPU binary9,205,760B로직전보다1,536B증가,server당event handle1·추가thread0·Cargo.lock불변. 새 cost는같은3×60/18조건의첫 round를통과했으며나머지실행중이다.

최종 scoped 검증: 새 event-idle 비용18창 모두 CPU<1%/WS<80MiB, 최대0.936669876%·73,371,648B. root가 원30k파일·3DB·각 창 계산·3PNG를 검토했고 독립 검토도 통과했다. 이전2개 초과 자료는 보존했다. 유휴20ms poll 제거와 동일 조건 통과를 확인했으며 모든 비용의 인과 원인 해결이나 전체P0 완료로 확대하지 않는다.

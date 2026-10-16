# S01: 실제 주기 스캔 완료·Next·CloseWait

- 상태: done
- 분류: verification
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: root engine library.rs, app library_scan_closewait_fixture.rs·accessible_library.rs, 새 scripts/windows_library_periodic_functional_probe.cs·windows_library_periodic_seed.cs·runner·evidence; soak_recovery_audit /tmp C# 초안
- 선행 작업: periodic 예산 후보 전체 검증/기존 소유capability CloseWait

## 사용자 결과

실제 자연 주기 스캔이 완료되는지, 진행 중 다음 페이지·중단 후 종료가 원 목록과 명령 응답을 보존하는지 검증한다.

## 계약과 범위

기존 기본 OFF feature에 명시 marker·서로 다른 원 inherited event4개(started/release/committed/workerAck)를 추가한다. 시작은 실제 첫 TEMP INSERT, committed는 원 transaction 성공 반환 이후만 신호한다. 기존 manual3cap 계약을 유지한다. cap은 argv/HANDLE_LIST로만 전송하며 환경/사용자 파일스위치는 없다. 합성 owned tmp/private desktop만. 제품 Cargo/lock/schema 변경0.

## 인수 조건

자연 Periodic의 first INSERT 관측 선행 대기는75초 상한이며 실제 관측 대기시간을 별도 기록한다. 관측start후180초 commit 인수는 유지한다. Next원명령 후page2/serial응답·preemption rollback 구분. 자율 periodic 중 WM_CLOSE 자동 StopCloseWait의 제어비활성·release전ACK없음→release·workerACK·자연Job0 순서 및 priorDB/payload보존. 원 실패·race inconclusive 보존. GUI60초CPU 측정은 별도다.

## 검증과 handoff

이작업의done은2ms/500ms당시후보의실제3case와원DB/payload보존을대조한좁은인수다. 최신threadCPUcounter후보의completion/Next/Stop은별도재검증중이다. 표준runner순수11회귀와case/binary/root/CAP/sentinelbinding보강은통합했다.

### 구현과 실제 실행 이력

현재 /tmp C#초안만 작성했고 root 통합·컴파일/native/원DB 증거검토를 이어간다. 이전 manualCloseWait PASS를 periodic으로 일반화하지 않는다.

root 통합에서 C# argv marker/fixture 순서와 completion용worker기한90→260초를 교정했다. 기본OFF Windows workspace Clippy·app tests(원cap/manual회귀 및4eventcommit검사 포함) PASS. 원syntax/Clippy실패는 별도 evidence에 보존했다. GUI native는 아직 실행 전이다.

2ms/500ms 후보의 실제 completion118.773초·Next 관측시점후2.030초page2·Stop gate전ACK0/제어비활성→release후ACK/자연exit0를 검증했다. root가 각before/after DB1만행(rowid/root/path/size/mtime/present/full128 identity)·integrity·원1만payload/cap/sentinel을 대조했다. 모든nativeID 재열기는 하지 않았다. [completion root 검토](../validation/evidence/windows-periodic-functional-native-fixed-20261006-continue/root-completion-review.json), [Next root 검토](../validation/evidence/windows-periodic-functional-priority-20261006-continue/root-next-review.json), [Stop 최종 결과](../validation/evidence/windows-periodic-functional-stop-green-20261006-continue/result.json).

원 observer 컴파일/seed argv 실패, outer의 잘못된id컬럼 검증, 수동prompt/숨긴Download control을 요구한Stop 실패는 보존했다. 완료와Next의옛generic closewait_inflight flag는Stop직접관측으로 해석하지 않는다. actualserial 숫자는원UIA로직접읽지않았고페이지2/정확원명령·원worker serial회귀를함께확인했다. 새pacing knob의완료·성능은별도인수다.

재현 가능한CLI scripts/windows_library_periodic_functional_native_probe.py (--project/--output/--case)를 추가했다. fixturePE만 보관하고 default release로 승격하지 않는다. pure 옵션 실패4시험(외부/기존/심볼릭부모/경로상위/잘못된case·누락) PASS. root가 CI docs job에 연결했고 remoteCI는 미실행이다. 현재 soak_recovery_audit는 이Python runner·4tests 두 파일의 가독성/실패scope만 담당하며 root는 그 두 파일을 수정하지 않는다. C#/Rust/Cargo/공유문서는root전용, 하위native0이다.

### 표준 runner의 결과 검증 보강

독립 read-only 감사로 공통 REVIEW_REQUIRED만 확인하면 다른 scenario·누락된 case별 proof가 통과하는 공백을 확인했다. root가 expected scenario/정확4caps/정수exit0와 완료≤180000ms·Next≤5000ms·Stop의release전ACKfalse/controlsdisabled를 DB 읽기 전에 검증하도록 통합했다. bool/형식오류·누락·범위초과 거절을 포함한 순수9시험을 python3 -m unittest discover -s scripts -p test_windows_library_periodic_functional_native_probe.py로 실제 통과했다. 역사적 C# actual 실행의 실패를 뜻하지 않으며 최신 표준CLI native는 별도 재검증 예정이다.

표준CLI보강후원binary/observerSource/ownedroot/CAP/sentinelbinding을case별결과와대조한다. 합성payload읽기는WSL no-follow원descriptor의fstat/type/size/최대4bytes읽기로교체해반복pathnameRPC를줄였으며WindowsnativeID재열기증거로표현하지않는다. no-follow미지원orchestrator는실패한다. symlink/디렉터리/누락/잘못된bytes·원target보존을포함한순수11회귀가실제PASS다. 최신threadcounter후보의표준CLIcompletion인수를현재실행중이다.

### first INSERT 관측 기한과 제품 기한 구분

2ms/0.4% completion123825ms 및Next2039ms/원보존은PASS지만Stop의기존45초 firstINSERT 선행조건은미도달FAIL이다. 실제단독native2ms phase의preflight만12.3028초였고기본watch주기30초와합치면42.3초로45초시험상한의여유가작다. 제품180초전체scan/원Next5초/StopACK5초/원strictCPU1%는변경하지않고collector선행대기만75초로명시해first_insert_observer_wait_ms/bound_ms를성공·실패모두기록한다. 기존45초原FAIL을보존하며선행신호이후Stop·ACK·실제rollback·원보존을새source핀에서다시검증한다. 아직실제지연원인이나새StopPASS를확정하지않는다.

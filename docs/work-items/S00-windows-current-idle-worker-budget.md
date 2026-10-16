# S00: 현재 Windows 대기 worker 비용 초과 교정

- 상태: done (current worker/CloseWait scoped 검증; 전체 P0와 분리)
- 분류: performance
- 우선순위: P0
- 요구사항: R01, R08, R10, R11
- 담당/수정 범위: root — 공유 계약·통합·최종 Windows 검증·문서; soak_recovery_audit — /tmp readonly periodic budget/worker 개선 초안. 프로젝트 Rust 파일은 root 검토 뒤 범위를 확정하며 현재 수정하지 않는다.
- 선행 작업: 지표 host timer Windows 전체790/0/13, 별도진단808/0/13·same-value 중단복귀와 idle후 frame0 실제 관측.

## 사용자 결과

Windows 목록이 열린 대기 상태에서 freshness·파일 변경 반영·종료 응답을 보존하면서 기존 CPU1%/메모리80MiB 예산을 충족한다.

## 계약과 범위

current 기본 CPU91217a80… source579의 3×60초18구간 중17PASS/1FAIL을 보존한다. 실패는 round3/window5 CPU1.5614283916947795%, wholeJob1562500ticks/10.006863초이며 메모리는18구간PASS다. main0, 관측된 library781250ticks·metrics312500ticks·appearance156250ticks 등이 있지만 threadchurntrue/measuredfalse여서 전체원인 확정으로 표현하지 않는다. 현재 periodic2ms/500ms/180초 checkpoint, 지표1초·stale3초·appearance 경로와 Windows thread churn의 실제구현을 읽고 독립 초안을 준비한다. budget/warmup/firstwindow를 완화하지 않고 변경·취소·StopACK·DB rollback·데이터보존·180초기한을 유지한다. source/schema/Cargo/lock은 현재 동결이다.

## 인수 조건

최소원인 후보를 재현/분리하고 실제공유경로 회귀를 확인한다. 채택하는 변경의 최신 Windows 전체4검사와 현재기본3×60초18고정구간을 root/독립 검토한다. 실패/quantization/churn/unattributed를 보존하며 다른binary PASS를 소급하지 않는다. 현재 숨김복귀·Next128 및fixture보존 인수는 별도로 진행한다.

## 검증과 handoff

실제 evidence/windows-msvc-metrics-freshness-default-cost-real-20261006-continue는 collectorFAIL/nativeFAIL을 유지한다. 세 앱은 자연exit0/Job0·private/input복원을 확인했고 상세fixture 및 독립18구간 검토는 진행중이다. 성능 해결/전체P0 완료는 아니다.

readonly cadence 조사에서appearance idle recv_timeout100ms와metrics50ms가실제측정없이도깨우는경로를확인했다. 원CPU값의원인확정은아니다. 독립 초안 범위로 soak_recovery_audit는 accessible_settings.rs의 bounded StopWake 통지를 /tmp에서준비한다. root가projectfile을통합하며 기존latestaccepted persistence drain·busy/pending/StopACK·오류보존을유지한다. idle wait만 event-driven으로바꾸되 stop atomic을먼저set후nonblocking wake, fullqueue는원write를보존하며이미wake조건, disconnected는terminal로확인한다. UI blocking/I/O/newthread/dependency/schema/lock추가금지. empty/full/stop-before-wait/drain/error/panicACK의실패시험후실제currentwhole/native18을다시실행한다. 아직채택/성능해결로쓰지않는다.

root는원/stage SHA대조후appearance 단일파일 StopWake 초안을 통합했다. 추가3실패회귀 및기존acceptedlatestdrain 회귀를 실제Windows에서검증한다. Host due dispatch7회귀·testfixture2파일도S00-metrics-freshness-native-timer 계약으로원SHA대조통합했다. 3Rustfile source map이현재790/808 proof와달라졌으므로과거GUI/비용 인수를새binary에소급하지않는다. 새fmt/전체4검사를수행한다.

metrics50ms idle polling의 별도 후속초안은 metrics_recovery 담당 /tmp accessible_metrics.rs로분리한다. 현재통합된실제 due fixture/API를기준으로bounded Selection|StopWake와 stop-before-wait/fullqueue/disconnect/panicACK·selectionrevision·1초관측/stale3초/Stop100ms를유지한다. root가전체검증뒤채택·통합하며worker I/O/fullchannel overwrite/blocksend/newthread/dependency/schema/lock 변경은금지한다. appearance파일은soak_recovery_audit가독립후속검토하고두담당파일이겹치지않는다. 아직metrics StopWake는제품에통합하지않는다.

appearanceStopWake+Host due7회귀 통합의새Windows 전체fmt/clippy/test/release 실제exit0 및root raw4/정확현재source579/artifact검토 PASS다. 실제803passed/0failed/13ignored, CPU571ce7152eda87de25be1e6e3cd6cd5dc88ca8a565c0c4df4aa12cf98739a66e/8,711,680B, CLI97217440…/841,728B 유지. new appearance3시험은두harness에서6회, Host7은CPUharness에서7회 통과한다. 의존성/lock/schema변경없으며binary크기는직전91217…와동일하다. 실제currentGUI/18구간 인수전이며metrics StopWake초안은아직별도다.

root는현재module 원SHA와metrics StopWake stage SHA를대조하여통합했다. observation/privateworker loop를주입 seam으로분리하지만관측I/O·완료Sample.at·원next 재예약순서는유지한다. 테스트는완료timestamp가아닌실제scheduled due를기록해1초기한보존을검증한다. 새3실패시험과appearance의stop-before-wait/Drop immediateabort경계1개를추가했다. 기존Host7+metrics8+settingsacceptedlatestdrain/panic은유지한다. 새2Rustfile map을전체4검사·현재GUI/18구간에서검증하며803/571ce… 근거를새소스에소급하지않는다.

combined StopWake 전체4검사 실제811/0/13·CPU9aa25d5d…8,710,656B(직전571ce…보다1024B감소) root/독립 PASS다. 그러나독립source검토에서 UI mailbox poison분기가atomic만set하고StopWake를안보내 idleACK가최대1초늦어질수있음을발견했다. root는원poison시험에원capacity1 Receiver의StopWake 실제수신 assertion을추가해미수정제품경로의red를실행하고같은경로stop_worker재사용으로교정한다. 정상freshness check에서는selection명령/I/O를하지않고terminal poison 오류에만bounded 비차단 Stop 통지를허용한다. 이예외를CONTEXT/ADR/원timer계약에반영하며 새source전체4검증·native 인수를다시수행한다.

원poison StopWake assertion은실제Cargoexit101로red재현했다. 원privateJobnatural0/forcedFalse/input불변과원로그를보존했다. root가refresh_mailbox의terminal poison분기만stop_worker로교정했고정상selection/샘플/기한코드는유지했다. 새전체4검사에서동일assertion의green과전체회귀를검증한다.

poison교정의latest Windows전체4검사는실제811/0/13·rootPASS며원red의같은StopWakeassert가두harness에서green이다. CPU342b652b1c77167d90c87f4b24443f138e1cd2ca4735820e80222412a76bbc83/8,710,656B, CLI97217440…/841,728B, dependency/schema/lock 변경없음. 원9aa25…와크기는같다. 최신default3×60초18구간을단독실행중이며source를동결한다.

최신 current CPU342b652b…의 기본3×60초18구간은18PASS, 최대CPU0.7813238194744639%, WS71,712,768B다. root/독립raw·pins·3화면과30,000합성files/3readonlyDB의count/integrity/SHA불변을 확인했다. 과거17PASS/1FAIL은보존한다. 같은소스진단whole은829/0/13·raw4PASS다. 실제metrics stall은321.4GiB→Unknown→같은321.4GiB, 복귀1065ms·원threadresume/naturalJob0을확인하고root가Unknown/resumed화면을직접검사했다. 현재background/restore와scan CloseWait실측이남아전체P0완료는아니다.

current기본CPU342…의실제background scopedPASS(root화면·동일worker3448/creation134357469526782590·heartbeat12→18·원job/attempt1 및Cancelled DB/integrity/SHA보존), loadedlibrary scopedPASS(root Next128/manual/invalid화면·readonlyDB10000/identityformat/count·collector원10000payload·독립직접bytes검토)까지통합했다. 이background모드는explicitcancel/hiddenCloseACK 검증으로확대하지않는다. scan중CloseWait release/timeout 인수는별도빌드진행중이다.

currentsource의실제CloseWait release3656ms/timeout20009ms두case를root/독립인수했다. 각원ACK전20회alive·ACKfalse, actualfirstobservation, 원3manualresetevents, ACK후naturalexit0/Job0/privateinput불변, 원DBold1행/path/size/nativeidentity 논리보존 및newobserv미게시를확인했다. root는두PNG/readonlyarchiveDBintegrity·SHA불변/원8files와archive SHA를직접대조했고원fixture를삭제하지않았다. 현재값은기존featureCPU59305…에결합된scope이며새설치검사source의일반OS/physical/전체P0인수로확대하지않는다.

# CPU 라이브러리 panel 구현 범위

2026-10-03. 신규 accessible_library.rs는 기존 library/location/watch/metrics 엔진만 재사용한다. root의 Candidate/module/CloseWait 연결은 완료됐고 Linux 실제 boundedpanelGUI·CloseWait 및 Linux/WindowsCPU CLI를 검증했다. 범위는 아래 현재 검증 결과를 따른다.

단일worker가 위치 준비·canonical containment·Store 정합성/page·WatchSession·여유용량 I/O를 소유한다. command/result sync_channel은 각각capacity1, UI page는128rows, refresh intent는한bit, worker watch는fixed marker다. UI poll은 결과1개만읽고 nativeworker가새snapshot/stopack에만egui repaint를요청한다. UI busy/stopwait때만100ms deadline으로ack를확인한다. UI thread join/Drop blocking을사용하지 않는다. Drop은 stopflag를설정하고현재scan이끝난후worker가자기자원을정리한다.

위치 Apply는 기존 engine location::prepare를사용하여 app-data overlap·symlink·invalidabsolute 및 persistedselection 정책을유지한다. 위치prepare실패시기존root/rows/cursor를보존한다. 위치prepare가성공했으나후속scan이실패하면persistedroot와UIroot는새root이고이전rootrows는비운다. root가서로다른항목을혼합하지않는다. 같은rootscan실패는기존rows와SQLite관측을보존한다. 시험/fixture의Default는DLPAD_TEST_DOCUMENTS없으면실제Documents로fallback하지않는다.

request_stop은새명령·refresh를차단하고현재scan완료후ack한다. resultchannel이full이면worker는20ms마다stop을확인하여UI의결과drain없이종료할수있다. filesystem/SQLite호출중즉시중단은보장하지않는다. 현재 reconcile_library_with_cancel에 같은Signals.stop을전달하고directory/DB경계취소시rollback한다.

작성한9개회귀: 260합성파일의128/128/4 keyset, disconnect실패시UI/DB보존, rejectedApply overlap·기존파일보존, successfulApply rootcommit·postStopcommand차단, fullsnapshotchannel undrained stopack, fixtureDefault Documentsfallback거절, root/data overlap양방향거절, worker panic 종료ack와boundedfailure표시, cancelledscan DB관측보존. 9개회귀는rootLinuxCPU와실제WindowsCPU에서통과했다. 담당파일 `rustfmt --edition 2024 crates/app/src/accessible_library.rs` exit0.

남은범위: 전체reader사용성·IME·정확peakRSS/전체성능시나리오·Windows/macOSlibraryactualGUI·R03전체. Linux boundedpanel/rootinput/Next/watch/CloseWait actualGUI는아래증거로확인했다. free bytes는worker관측만유지하고새중앙catalog문구없이는임의UI문구를추가하지않는다. 새domain/schema/Cargo/catalog변경없음.

worker Entry는 catch_unwind와 StopAck RAII guard를사용한다. worker자원이정리된후정상/오류/panic 모두stopped+wake를보장하고panic payload/path는UI에전달하지않으며fixed catalog실패를표시한다. 테스트hook의합성panic도동일wrapper를사용한다. process abort/OS강제종료는이협조ack범위가아니다.

후속연결: root가제공한 Store::reconcile_library_with_cancel(root,&AtomicBool)에scan을연결했다. Refresh/Next/Apply/nativewatch모두같은Signals.stop을전달하여directory/DB경계에서협조취소하고관측transaction을rollback한다. 단일filesystem/SQLite호출중즉시취소는보장하지않는다. 협조취소회귀를실제로실행해통과했다.

실제통합검증추가: root Linux CPUentry20tests 및Clippy 통과보고, WindowsGNUCPU전체Clippy/test/release exit0을 docs/validation/windows-cpu-library.md와원시증거에보존했다. panel9회귀는Windowsaccessible/CPU양binary통과. 현재CLI결과와실제GUI결과의binary/inputscope를구분한다.

## 현재 검증 결과

rootLinux fmt/CPU Clippy/default193/CPU242/fullrelease 통과. test-onlyfixture수정후에도 생산CPU SHA8fb4a67c27ff89fd883ff6fddaa9136a5fc36c62d74c049665543a513ab06986/10437144bytes 유지. Windows최신전체CPU lint/test/release exit0/167passed1ignored, 원시자료 windows-cpu-diagnostics-fixturefix/.

실제Linux합성10000file의목록/keysetNext/외부addrename삭제수렴/invalidroot보존/Tabfocus는cpu-library-gui-validation.md. RSS51000→56332KiB는해당185442binary의click→commit관측이며첫paint/peak/new8fb4성능으로확대하지않는다. 새8fb4 actualCloseWait busyentry→namedStopExit→lock중생존→해제후exit0는cpu-library-closewait-validation.md. 초기buttonenabledproxy실패및scrollcollector원인은원시보존했다. 완료는이boundedLinuxpanel범위이고3OS/R03전체/P0전체성공이아니다.

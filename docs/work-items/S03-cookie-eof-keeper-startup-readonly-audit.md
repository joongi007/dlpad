# S03: Cookie EOF keeper 초기 실패 읽기 감사

- 상태: completed (source 읽기 감사; native 초기실패 인수 별도)
- 분류: architecture-review
- 우선순위: P0
- 요구사항: R08, R10
- 담당/수정 파일: soak_recovery_audit — 이 문서만
- 의존성: root/stress user_service.rs 및 root guardian.rs sourcefreeze, 실제 native 결과

## 범위와 인수 조건

StartTransientUnit accepted 이후 GetUnit/Invocation 확인 전 오류에서 EOF keeper/control RAII가 tool 미실행과 유한 회수에 주는 근거를 읽기 검토한다. Debug-only nonce 고정/hook fixture의 최소 계약을 제안한다. 제품/tests/scripts/Cargo/native 변경·실행과 하위 agent는 없다. 소스 논거와 실제 native 증거를 구분하고 cleanup proof를 완화하지 않는다.

## Source 보장과 한계

`run_gated`는 private pipe2(O_CLOEXEC)의 read end만 keeper stdin에 전달한다. Sole write end는 launcher의 OwnedFd 지역변수이며 manager/tool에 전달되지 않는다. Start message/reply는 start 함수 종료 시 drop하고 성공 뒤 passed control alias와 keeper_input도 명시 drop한다. 오류/return/unwind이면 remaining keeper_writer와 control writer는 RAII drop한다. Launcher 강제종료에서는 OS가 sole writer를 닫는다. keeper는 readonly/owned FIFO를 fstat/fcntl로 확인하고 byte를 받으면 오류, EOF이면 정상 종료한다. Cookie/receipt FD를 keeper에 전달하지 않는다.

Start reply accepted 이후 `get_unit`/Invocation에서 실패하면 ReaderService RAII가 아직 생성되지 않은 경우가 있다. 이때 이름만 가지고 StopUnit을 호출하는 안전하지 않은 정리는 하지 않는다. Pipe EOF와 guardian control 종료가 독립 안전수단이다. Service RAII가 만들어진 이후 Drop은 같은 Invocation일 때만 stop_owned/wait_cleanup/unref한다. Bus drop은 sd_bus_unref이며 connection-owned AddRef 해제/manager-side GC 완료 시각은 실제 관측해야 한다.

Guardian은 cookie gate `wait_begin_only`가 성공하기 전 external tool Command.spawn이나 drain worker를 만들지 않는다. Launcher는 keeper/native membership/원 cgroup pin 및 Invocation을 검증한 뒤에야 ReaderIdentity를 app에 전달하며 자신은 B를 읽지 않는다. 따라서 identity 전달 전 초기 오류가 제대로 EOF를 전파하면 tool0가 예상된다. `launch_service` Err 경로는 LU 발신 후 EP와 socket Shutdown::Both를 수행하며 product app도 시작 실패 control alias를 닫는다. 단순 descriptor drop과 socket alias shutdown의 효과를 혼동하지 않는다.

Keeper는 reader의 Wants/After, keeper 자체는 cookie slice Requires/After이며 별도 app.slice에 속한다. **Wants/After는 keeper 종료만으로 reader를 자동 중지하는 BindsTo가 아니다.** Tool0 논거는 Begin 이전 실패/control EOF이고 keeper는 slice pin 수명/종료를 위한 별도 자원이다. TimeoutStart10초/TimeoutStop2초/guardian Begin10초는 각 단계 상한이며 전체 manager GC가 정확2초 안에 완료된다는 증명이 아니다. Accepted job 실행 지연·bus reference 해제·unit inactive·GC를 native fixture의 유한 관측 deadline으로 확인해야 한다.

현재 읽은 source에서는 after-start/after-get-unit hook이 일반 run에만 있고 run_gated에는 아직 없다. 기존 native3/4/2 결과를 gated 초기 accepted-stage 실패의 실제 증거로 확대하지 않는다. Natural 정상 GC와 초기 오류 EOF 수명은 별도 fixture가 필요하다.

## Debug-only 초기 실패 fixture Proposed 계약

- Gated path에만 기존 exact after-start/after-get-unit hook을 선택적으로 삽입한다. after-start는 reply accepted 뒤/GetUnit 전, after-get-unit은 원 unit path 수신 뒤/Invocation 전이어야 한다. 앞 identity/begin 교환은 아직 일어나지 않았음을 확인한다.
- Fixture가 /dev/urandom128-bit nonce를 생성하고 debug executable subprocess에만 명시 nonceoverride를 전달한다. 정확32 lowercase hex/nonzero만 허용하고 unit/service/slice/keeper prefix는 제품이 구성한다. Release는 환경 변수를 읽지 않으며 override가 default run/backend 선택이나 production receipt에 영향을 주지 않는다.
- Nonce를 사전 고정하면 fixture는 service/slice/keeper 정확3이름만 조회할 수 있다. Preexisting unit이 하나라도 발견되거나 원 Invocation/PID-start를 확인하지 못하면 failclosed한다. StartTransientUnit mode=fail을 유지한다. Precheck와 start 사이 경쟁에서도 기존 unit을 사용/stop하지 않는다.
- Hook 관측 창을 유한 debug-only 지연으로 확보하고 별도 observer가 exact unit/Invocation/MainPID-start/native cgroup과 keeper readonly pipe기동을 관측한다. App source/root의원cookie 기록·sentinel hashes를 보존하고 tool-start marker/calls가0인지 실제 확인한다. 조용히빨리끝나관측못하면 PASS가 아니라 미검증이다.
- Error 반환 뒤 owner/helper exit, exactowned pidfd 종료, service/keeper/slice상태와 GC를 bounded polling한다. Unit/pathabsence만으로 cookie cleanup을 승인하지 않는다. Cookie journal Intent/Bound는 적절한 actual receipt 없으면 preserved/deferred여야 한다.
- Native 시험의 emergencycleanup은 관측한 정확Invocation/ownedpidfd에만 한정한다. 원 cleanup 실패를 fixturecleanup으로 성공처럼 바꾸지 않고 둘을 분리 기록한다. 기존 user units/manager/8h/desktop은 건드리지 않는다.

## 비용·담당·검증

Nonce override와 gated hook은 root 제품 owner의 debug-only 최소 변경, runtime observer/fixture는 별도 담당 파일로 조정한다. Cargo/schema/new dependency가 필요하지 않으며 production thread/queue/logger를 추가하지 않는다. Keeper는 이미 도입된 별도 process/unit이고 early-failure 시험은 이를 새 아키텍처 비용 없이 검증한다. Debug fixed nonce는 실제 account/cookie 비밀이 아니지만 입력을 unitname으로 그대로 쓰지 않고 제한한 suffix로만 사용한다.

이번 감사는 user_service.rs/guardian.rs와 root 출력상한 evidence를 읽고 전용 문서만 작성했다. Rust/fixture/scripts/Cargo/native 실행은 없다. 초기 error actualtool0/naturalGC proof는 아직 인수되지 않았으며 source 논거로 실제 PASS를 대체하지 않는다.

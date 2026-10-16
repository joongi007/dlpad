# S00: 고정 도구 runner의 실제 timeout·gate 이전 취소

- 상태: done (고정 run(fmt) 두 실제 예상 실패 인수; 전체 workspace 별도)
- 분류: native-regression
- 우선순위: P0
- 요구사항: R08, R11, R13
- 담당/수정 범위: stress_recovery; scripts/rust_workspace_validation.py는 구체결함이 있을 때만, 이 신규 문서와 전용fresh evidence
- 선행 작업: source3839ea owned runner, root 실제 Rust PATH와 source freeze 조정

## 사용자 결과와 계약

고정 run(fmt)만 실제 project Rust PATH로 실행한다. timeout은 tiny positive deadline, 취소는 spawn후 G 이전 CANCEL_REQUESTED=true로유발한다. 임의executable/shell/fakeCargo/전체workspace7check재실행없음. 모든 신호는 ownboot/PIDstart/sessiongroup 대조와 Popen-owned Childreap 범위다. root Rust module/Cargo integration에 간섭하지 않는다.

## 인수 조건

원 collector 결과는failure여야 하며timeout/취소가PASS로바뀌어서는안된다. distinct acceptance는expectedfailure와ownidentity/reap/원Proc부재/groupempty 및gate권한없음을 실제원시로확인한다. source beforeafter 불일치는기록하고실행proof와구분하여전체PASS로승격하지않는다. gate이전취소는native child exists를증명하되 Cargoexec0을별도 확인한다. rawlog/durablelaunch/result의상태를구분한다.

## 검증과 handoff

root PATH/sourcefreeze READY 대기. 작은deadline검증 자체는최대10초와boundedcleanup2+8초로제한하며기존8h/외부PID/사용자desktop에접근하지않는다. 아직actualfailure regression을PASS로보고하지않는다.

## 실제 환경·원시·결과

root가 source freeze와 project-local tool 환경을 제공했다. PATH앞에ROOT/.local-tools/cargo/bin, RUSTUP_HOME=ROOT/.local-tools/rustup, CARGO_HOME=ROOT/.local-tools/cargo를적용하고실제고정run(fmt)을호출했다. runner source3839ea7e378e6de800ed9964e18fbf457c5cd45ef7bccb8a8ef6c31d107e86eb는수정하지않았다. fresh evidence `rust-workspace-runner-failure-native-first`에fixture launch, 각 fmt-launch/fmt-result/log 및별도acceptance를create-only/durable로남겼다. 실제collector작업이실패해야인수를통과하는fixture이며이를fmtPASS로계산하지않는다.

timeout budget0.001초: actual Child715238/start4825416/group=session715238. TimeoutError, collector passed=false/exit-15, exactboot+identityverified→TERM, 실제Childreap/proc부재/groupempty/terminalconfirmed. gateG는전송됐지만Cargo exec와fmt본체실행은독립관측하지않았으므로실제fmt검증완료라고주장하지않는다.

cancel은CANCEL_REQUESTED=true를run호출전preset하되run이실제Popen을수행한뒤G이전의체크에서중단했다. actual Child715239/start4826417/group=session715239. InterruptedError, collector passed=false/exit-15, exactboot+identityverified→TERM→실제reap/proc부재/groupempty/terminalconfirmed. gateG0이므로worker는Cargoexec권한을받지않았다. Cargoexec0과signal이이번ownedgroup에만적용됐음을원시와코드위치로대조했다.

두case sourcebeforeafterunchanged=true. fixture실행exit0은두expectedfailurecleanup인수가맞았다는뜻이며두원collector결과는실패다. elapsed5.07/5.05초는durablemanifest/toolhash·finalsource관측·cleanup을포함하며0.001초budget을전체기록완료시간으로표현하지않는다. 원root전체workspace7check증거는별도보존하며본작업은그재실행이아니다.

## 반복용 고정 fixture 계약

Python에서동결모듈을import한뒤위project-local환경만설정한다. runner identity/boot와sourceSHA를freshfixturelaunch에기록한다. 두case별freshdirectory에서 `CANCEL_REQUESTED=False; run("fmt", out, 0.001, runner, boot)`와 `CANCEL_REQUESTED=True; run("fmt", out, 10, runner, boot)`를호출하고finallyflag를False로복구한다. 두원결과는passed=false, FailureKind각TimeoutError/InterruptedError, cleanup identity_verified/terminal_confirmed=true, Childreap후/proc부재및groupempty/sourceunchanged를assert한다. nativePID숫자만다른프로세스에신호권한으로사용하지않는다.

별도executable/shell/가짜Cargo/source변경/전체workspace/subagent/GUI는없다. root에게두Child terminal을통보하여Rustsourcefreeze해제가능상태를전달했다. 추가코드결함은이인수에서재현되지않아수정범위를확장하지않았다. productiontimeout/실제nativeignored수명/WSL중단자체복구완료는이시험의범위가아니다.

Root 독립 대조: acceptance JSON의 두 expectedfailure/collectorpassed=false 및 원cleanupidentityverified/terminalconfirmed를 확인했다. 같은 kernelboot에서 원PID715238/715239의 /proc이 현재부재임을 host읽기로 재확인했다. 신호/재시작 없이 검토했고 rawtimeoutCargoexec미관측을 유지한다.

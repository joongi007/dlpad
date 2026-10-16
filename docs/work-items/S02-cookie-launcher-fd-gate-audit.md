# S02: launcher FD/gate 제품 연결 읽기 감사

- 상태: done
- 분류: review
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: metrics_recovery — 이 문서만; process/guardian/user_service/codec/gate/journal 읽기 전용
- 선행 작업: journal26Cargo/native1/clippy rootPASS; optional same-boot gate 계약

## 결론과 현재 구현 범위

root 구현 선택은 app Intent→launcher의 native identity I→app durable Bound→guardian exact B→tool spawn/R, launcher native-empty proof→durable receipt→Unref→LC다. 현재 code에는 이 제품 연결이 없고 pure protocol/gate/journal만 별도 시험되었다. 아래 인터페이스를 조정하고 actual native crash/FD 인수를 거쳐야 한다. Proposed/구현 선택을 릴리스 채택·P0 완료로 표현하지 않는다.

가장 중요한 차이는 현재 wait_unit의 known_group이 PathBuf라는 점이다. native cgroup inode/mount를 FD로 pin하지 않는다. empty_group은 events와 path가 NotFound이면 empty로 허용하고 wait_cleanup은 ControlGroup 빈 문자열만으로도 성공한다. 이 legacy cleanup 결과를 **그대로 durable receipt의 exact pinned-empty proof로 쓰면 안 된다.** receipt writer는 별도 엄격한 pinned identity 검사에서 성공해야 한다.

## FD capability와 exec/systemd 경계

현재 guardian::command의 pre_exec는 control FD 하나만 CLOEXEC를 해제한다. gated command는 control/receipt-writer/record-lock 각각 소유 FD≥3, 서로 다른FD 및 receipt/record 서로 다른 nativeID를 검증하고 한 launcher exec에만 explicit allowlist로 상속해야 한다. startup 직후 세 FD의 CLOEXEC를 복구한다. receipt writer는 F_GETFL의O_ACCMODE=O_WRONLY, regular0600/currentUID/nlink1/devino를 expected capability와 대조한다. record lock은 원 record inode/0600/link1과 같은OFD flock 수명용이며 arbitrary record write API를 제공하지 않는다.

user_service StartTransientUnit는 현재 message의 StandardInputFileDescriptor=control, stdout1,stderr2만전달한다. sd-bus는FD를message안에서복제하므로call후messageunref와원controldrop를보존해야EOF가걸린다. receipt/record FD를D-Bus property에추가하지않는다. launcher환경snapshot에새path/nonce를추가하지않으며recordpath를argv/log에넣지않는다. guardian/service/tool의/proc/self/fd 실제allowlist에서receipt/recordFD0개를확인해야한다. RustCLOEXEC검사만으로이실제인수를대체하지않는다.

launcher main-process 종료시FD가닫혀도service는user-manager소유다. app과launcher동시crash는EOF/cgroup정리와receipt부재보수보존으로판단한다. launcher kill이service확인이라고추정하지않는다. root의8-slot boundedreaper는그대로보존하고deferred Child가receipt를쓰지못한경로를nextstartup완료로확대하지않는다.

## capability bootstrap와 한 명의 reader

현재 launcher는control을읽지않고guardian만read한다. nonce/boot/intenttoken/receiptID는FD만으로복원할수없으므로gated launcher bootstrap에boundedCapability C packet 전달이필요하다. 최소안은 **StartTransientUnit 전** launcher가control에서exact C frame 하나만읽고검증한뒤reader역할을끝내는것이다. app은Intent sync 후C를먼저보내고I를기다린다. serviceStart후launcher는B/T/K를절대읽지않는다. root가대신별도boundedbootstrappipe를선택할수있지만새FDallowlist/닫기인수가필요하다.

serviceguardian은expected index/intentnonce를알아야B를검증한다. gatedinternalmarker에boundedindex/nonce를명시전달하거나별도pre-toolbootstrap을정의해야한다. nonce는credential내용/path가아니지만rawdiagnostic에노출하지않는다. launcher가같은guardian endpoint에서nonce를write하면그byte는guardian이아닌app쪽에가므로그방법으로guardian에Capability를전달할수없다.

serviceguardian의ControlReader는**B후**시작한다. preB에서wait_begin_only(stream,expectedindexnonce,deadline)을유일reader로호출하고T/K/EOF/invaliddomain은toolspawn0/fixedfailure다. 기존ControlReader는T이외byte를K로해석하므로B검증전시작하면25B Bframe을소비하고tool을종료시키거나gate를우회한다. wait_begin이성공한뒤남은T/K가같은stream에서기존reader로전달되도록frame정확길이만소비한다.

현재reader_gate의combined send_identity_then_wait_begin은같은역할이I송신과B수신을하는시험용helper다. **launcher에그대로연결하면guardian과B competingreader가된다.** 제품최소추가API는send_identity_only와wait_begin_only로분리한다. app receive_identity_then_begin은유지할수있다. nongated도구/metadata/version은observerNone으로기존경로를보존하고ProcessSpec필수field를추가하지않는다.

## native identity I 생성 인터페이스

user_service::run은현재Start→GetUnit→Invocation→wait_unit에서뒤늦게MainPID/cgroup을관찰한다. gated용capture_gate_identity는I전에서다음증거를확보해야한다.

1. 같은ownedunit object 및InvocationID16이전후일치, activeguardianMainPID>1.
2. MainIdentity::capture의startticks/pidfd를보유하고readmembership전후matches_live.
3. bounded /proc/PID/cgroup의unifiedmembership이managerControlGroup과같음.
4. canonical ownedunit suffix의cgroup를no-follow componentwalk로열고native dev/ino/mount identity를pin; bytespath≤512이고rootcgroup가아님.
5. pinnedgroup에서guardianactive존재와manageridentity를재확인하고canonicalfullidentitydigest를계산한다.

추천형식 GatedServiceIdentity { unit/path/Invocation, main:MainIdentity, group:PinnedCgroup, reader:ReaderIdentity }. 기존wait_unit의known_pid/group를새pin으로시작하도록파라미터화하거나gated별도wait함수를둔다. I의receiptFileID/boot/index/nonce/intenttoken은bootstrapCapability에서만취하고currentobserved값으로expectedID를교체하지않는다. appjournal.bind는capabilityexact일치와canonicalSHA256을재검증하고file/directorysync완료후B를쓴다.

I가부분write되거나appbind가실패하면B0이므로toolspawn0이어야한다. guardianpreB는R/GC/GU를정상으로쓰지않아launcherI frame과interleave하지않는다. setuperrorGU/LU는여전히fixedtypedunknown으로보존하며임의terminalframe을I로해석하지않는다. I 송신완료후app이B를보내야guardian이R/GC를쓰므로정상ordering을실제trace로인수한다.

## receipt용 native-empty proof와 terminal 순서

prove_reader_empty(gatedIdentity)는sameownedunit/sameInvocation/terminal inactive-or-failed/noactivejob와**같은pinnedcgroup**의populated0+procs empty를전후재검증한다. stringpathNotFound/ControlGroup빈문자열/unitGC만으로성공하지않는다. activechildsubgroup도populated값으로덮고mount/nativeidentity교체는보존한다. pinnedgroup이삭제된후empty증거를읽을수없으면독립유효증거없는receiptwrite는거절한다. pathnamerename/pathabsence를nativeinode죽음으로추정하지않는다.

legacy empty_group의NotFound成功과wait_cleanup의빈ControlGroup成功을gatedreceiptproof에서재사용하지않는다. stop_owned→wait_cleanup가Ok여도receipt는이별도proof함수성공시에만쓴다. 실제systemd가guardian종료직후cgroup를제거해pinnedproof기회를잃는경우에는receipt를못쓸수있다. 이가정은native실험으로확정하고manager가원래pin을제거한상황의안전한독립증거없이는보존한다.

proof성공→fixed256receipt write_all+sync_all→sameInvocation再確認→Unref→LC 순서다. receiptwrite/sync실패는fixedLU/typedunknown이며recordphase를Resolved로먼저바꾸지않는다. app은GC+LC+realEOF/Childreap/drain완료를execution성공으로계속확인한다. cleanup-onlyreceipt는guardianGU/missingGC후에도독립nativeempty가입증되면sameboot복구용으로유효할수있지만currentexecution성공으로바꾸지않는다.

## deadline/cancel와 rollback 없는 실패

전체appabsoluteInstant를Rustrepr바이트로다른process에보내지않는다. childdeadline이필요하면검증된CLOCK_MONOTONIC timestamp 또는boundedrelativebudget를별도정의하고각launcher/guardian phase에finitecap을둔다. app전체deadline이취소되면같은소유socket종료가preB gate를깨고기존typed StartupUnconfirmed/Deferred를유지한다. app B0를보냈다고추정하여readerpending를clear하지않는다. 각read/write는20ms/remaining이며C/I/B partialprefix를잃지않는다.

appwrite-halfclose는controlalias전체에적용되며guardianEOF→cleanup를유발한다. launcher/guardian側의정상SHUT_WRITE는GC/LC를차단하므로금지한다. 신뢰할수없는setupfailure에서만기존aliasSHUT_RDWR를유지한다. lateK/reset!=EOF최소수정과boundedControlReader수명을gated경로에서도보존한다.

record Intent/Bound sync나receipt sync는kernel정체까지hardtimeout으로보장하지않는다. bindcallback후deadline/cancel을재확인해B를보내지않으며partialcommit은journalpoison/copy보존이다. partialunlink/marker/journal기록실패를원자적rollback으로보고하지않는다. existingownerunknownentry/capacity128/Windows recovery Unavailable는그대로다.

## actual native 인수

- preB worker marker0, serviceguardianPID/Invocation/pinnedcgroup active를먼저확보; bind前/後/Bpartial/R前appSIGKILL의actualproof를분리.
- FDallowlist는launcher의receipt/recordexpectedID와guardian/tool의0개상속을actualprocfd에서검증. callerunrelatedpipe/file/sentinel보존.
- I前identityfail/nonce/index mismatch/cancel/expireddeadline/partialC/I/B: toolspawn0또는typedunknown/copy보존,foreignunit Stop/PID신호0.
- 실제fast0/42/126,normalexit/cancel/timeout/escapedchild,GU후nativeemptyreceipt,launcherSIGKILL/managerunavailable: onlyexactproofwritesreceipt.
- receipt sync後Unref前appcrash→unitGC→nextstartup anchoredrecord/receipt만으로copy정리; sync前crash/partialreceipt/changedmount/pathrename/unitreuse는삭제0.
- actualsystemd cgroup제거timing과PinnedCgroupread가능성을원시기록해absentgroup정책을검증. currentPathBuflegacyPASS로durableproof완료를대체하지않는다.
- root immutablebuildinputSHA/driverSHA/defaultparallelCargo/nativeprivateGUI/source/DBsecretabsence/FDcounts를별도통합한다. 사용자desktop/실계정/실영상/기존8hunit은시험대상이아니다.

## 실행과 한계

실제수행은sed/rg소스읽기와이문서작성뿐이다. code/Cargo/manager/process/기존evidence를수정하거나실행하지않았다. root가공유API와nativepin/receipt proof를선택하고실제검증한뒤제품기능완료를판단한다.

## 역할 분리 구현 동결

root 요청으로 reader_gate.rs에 launcher 전용 send_identity_only(&ReaderIdentity, &mut UnixStream, deadline, cancel)와 service guardian 전용 wait_begin_only(index, nonce, &mut UnixStream, deadline, cancel)를 추가했다. launcher 함수는 I만 쓰며 B를 읽지 않는다. guardian 함수는 유효한 index와 nonzero nonce를 먼저 검사하고 exact B 수신이 성공해야 permission을 반환한다. 실제 tool spawn은 이 함수가 하지 않고 root의 제품 연결에서 성공 반환 뒤에만 허용해야 한다.

기존 send_identity_then_wait_begin은 두 함수를 재사용한다. 같은 absolute deadline을 넘기며 각 교환에서 원래 socket read/write timeout을 복구한다. GateError 및 관련 함수의 visibility는 pub(crate)로 통일했다. 기존 부분 수신, 1024B 상한, 20ms timeout, 취소 우선과 고정 오류를 유지한다. 제품 helper는 stream을 복제하거나 global shutdown하지 않는다.

실제 UnixStream pair 시험 3개를 추가했다. fixture에서 endpoint를 복제해 launcher I writer와 guardian B reader 역할을 모델링하고 durable bind 관찰 뒤에만 permission을 받는 순서를 확인한다. partial B deadline, 잘못된 nonce, EOF, zero nonce에서는 permission을 반환하지 않는다. 만료된 I 송신은 0byte이며 fragment로 나눈 유효 B는 수신할 수 있다. 기존 시험 4개는 보존했다. 이는 actual service/tool gate 시험을 대신하지 않는다.

source SHA256 `6e75ee4298ad87b30d57762bc3421db202fc9ab9cc3798947a4fcf8bc41cdda9`, rustfmt exit0. Cargo는 아직 실행하지 않았고 root가 수행한다. 기존 process/guardian/user_service/schema/Cargo를 수정하지 않았다. 현재 제품 gate와 receipt 작성 연결은 여전히 별도 진행이며 P0 완료로 보고하지 않는다.

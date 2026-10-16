# Linux systemd native 보호 backend 계약 검토

이 문서는 **Proposed 구현 계약 검토**다. [작업 계약](../work-items/S03-systemd-native-backend-contract.md), [준비 조사](systemd-user-readiness.md), [CLI prototype](systemd-user-transient-prototype.md), [no-env 후속](systemd-execstartex-prototype.md)를 읽고 현재 process.rs/process/guardian.rs와 대조했다. 제품backend/채택/전체P0 보호 완료가 아니다.

## 현재 증거와 구현 경계

원시 CLI prototype는 EOF/guardian SIGKILL 두 case에서 소유guardian/setsid child/leaf 회수·빈cgroup을 확인했지만 literalargv가 변형되어 overall=false다. 별도 ExecStartEx prototype는 같은7개 `$VAR`/`${VAR}`/embeddedVAR/공백/한글/%n/quote literal표본을 보존했고 두 case overall=true, cleanup0.050/0.054초였다. 이Python/regular-log 실험은 실제제품R/T/K, stdout pipe EOF, launcher status proxy, app+guardian 동시죽음이나 두job을 증명하지 않는다. 원시실패를 후속성공으로 덮지 않는다.

현재 engine은 guardian command의 직접Child와 stdout/stderr를소유하고 앱control에서10초 내R을 읽는다. 취소는T→200ms→K→Child.wait이며 Ownership.Drop도직접child waitpid만한다. guardian은tool직접부모/subreaper로현재R/T/K/EOF와toolExitStatus를유지한다. 새service MainPID를직접waitpid하려는변경은불가능하므로 root공유계약은다음과같아야한다.

## 직접child와 service 역할

앱→직접child native launcher→StartTransientUnit→manager직접child service guardian→tool/자손 구조다. `std::process::Child`는launcher이며stdout/stderr와wait/try_wait 계약을유지한다. launcher는MainPID가아니다. 별도marker로현재앱/adjacenthelper의internal entry에서 launcher/service guardian을구분하고 일반GUI인자를변경하지않는다. service guardian은기존supervise를재사용하되 transport초기화만분리한다.

StartTransientUnit은고유nonce unit name, mode=`fail`, Type=exec, KillMode=control-group, SendSIGKILL=true, Restart=no, RemainAfterExit=false, 명시 유한TimeoutStartUSec/TimeoutStopUSec를설정한다. ExecStartEx=`a(sasas)`의단일guardian 실행과 `no-env-expand` flag를사용한다. rawsystemd-run래퍼/보호없는PGID fallback은없다. no-env 미지원은시작거절이며대체escaping으로성공을꾸미지않는다. 공식 [v249 Ex setter](https://raw.githubusercontent.com/systemd/systemd/v249/src/core/dbus-execute.c)와 기존실험이필드/flag지원근거다.

## FD 소유와 EOF

| 경계 | 소유·해제 계약 |
|---|---|
| 앱 | control socket의app끝만유지. guard끝을launcher spawn뒤즉시닫음. stdout/stderr read끝은기존drains만소유. |
| launcher spawn | guard끝의실제fd>=3만pre_exec에서CLOEXEC해제. fixed3 덮어쓰기/exec-error pipe 충돌금지. stdout1/stderr2는기존piped write끝. appcontrol끝/다른jobfd는inherit금지. |
| D-Bus message | StandardInputFileDescriptor=h(controlfd), Output=h(1), Error=h(2). append_basic(h)는FD를복제하므로call후message/reply즉시unref. launcheroriginal control/write1/write2도즉시닫고unit조회중붙잡지않음. |
| service guardian | 전달된FD0가socket인지검증하고 F_DUPFD_CLOEXEC로privatefd>=3 복제. FD0닫기/단일소유이전으로중복끝을남기지않음. 별도marker로이transport만허용. |
| guardian→tool | 기존toolstdin=null; control원본/readerclone 모두CLOEXEC. stdout/stderr만inherit. tool이appendpoint를inherit할수없음. |
| terminal | status/cgroup증거capture뒤unitreference/message/bus/FD해제→launcherterminal→앱drainEOF 순서. manager/unit에retained stdFD가남는지실제pipe시험필수. |

FD0복제를하지않고기존descriptor파서를0으로재사용하면toolstdin=null 전환및socket중복소유위험이있다. RAII OwnedFd/UnixStream은한rawfd당한owner만생성한다. sd-bus append_basic `h`는int주소, string은UTF8 NULterminated pointer다. 실패는negativeerrno이며fddup도message lifetime에속한다. [v249 append_basic API](https://raw.githubusercontent.com/systemd/systemd/v249/man/sd_bus_message_append_basic.xml).

## unit identity와 terminal status

소유단위는random nonce unit name+동일bus usermanager identity+InvocationID+검증된guardianMainPID/startticks+ControlGroup이다. 임의전체unit/PID목록검색/이름prefix일괄kill은금지한다. StartTransientUnit reply jobpath는start접수이지R성공/terminal증거가아니다. 조회는그unit의선택property만bounded하게읽는다. InvocationID가바뀌거나MainPID/cgroup불일치면해당호출은failclosed하고재사용unit을stop하지않는다.

빠른정상종료와CollectMode GC가status를삭제하는경쟁을막으려면create시AddRef=true가필요하다. terminal상태를읽고unitcgroup정리를확인한뒤Unit.Unref한다. [v249 run.c](https://raw.githubusercontent.com/systemd/systemd/v249/src/run/run.c)의directstdio/--wait 경로도AddRef를사용하며 [unit wait source](https://raw.githubusercontent.com/systemd/systemd/v249/src/shared/bus-wait-for-units.c)는Ref/Unref와PropertiesChanged/GetAll을조합한다. 단순create뒤Ref 호출은fast-exit사이에unit이없어질수있으므로동일하지않다. 새로운native 구현에서이계약은아직검증되지않았다.

launcher는같은InvocationID의ExecMainCode/ExecMainStatus 및Result/ActiveState/SubState/빈cgroup을확인한다. CLD_EXITED면guardiantool proxy exitcode, CLD_KILLED/CLD_DUMPED면유효signal만default복원/raise하여기존Child ExitStatus.signal을보존한다. TERM으로전체launcher를먼저죽여service성공을가정하지않는다. systemdResult=timeout/oom-kill/startfailure/managerdisconnected/identityfailure는normaltoolExited0이아니며typedbackendfailure를기존worker고정오류로전달한다. stdout/stderr에launcher JSON/status/debug를섞지않는다. 별도boundedfailurechannel 또는내부exit marker mapping은root결정이며exit126만으로정상tool126과backend126을혼동하지않는시험이필요하다.

## literal 입력·환경·동적 ABI

D-Bus s필드는UTF8이므로nonUTF8 OsString/path 및NUL은lossy변환/shellescaping 없이typed오류로실행전거절한다. `$`,percent,공백,quote,한글은no-env flag와exactargv배열로보존하며argv0/executable절대경로도분리확인한다. executable/WorkingDirectory의systemd별specifier처리는도구argv와별도로시험해야한다. prototype가%n argv성공했다고경로specifier성공으로확대하지않는다. 이제약으로Linux모든filesystem경로지원완료라고주장하지않는다.

user service는caller환경을자동상속하지않는다. 기존ProcessSpec에는env/cwd가없으므로root가명시bounded snapshot/전달정책을정해야한다. 제안initialallowlist는HOME/PATH/LANG/LC_ALL/LC_CTYPE/TZ/TMPDIR와필요XDG/CA 경로값이다. 값/총bytes/count상한,UTF8/NUL검증,실제HOME/PATH의해석과cwd특수문자를합성시험한다. 인증cookie/token/real account/proxycredentials를unitDescription/Environment/property/debug에넣지않는다. 테스트전용fixture env는명시opt-in으로별도주입한다. 이allowlist는현재제품환경호환성을증명하지않았으며root채택전외부FFmpeg/JSruntime/설정계약대조가필요하다.

기존libc의dlopen(`libsystemd.so.0`,RTLD_NOW|RTLD_LOCAL)/dlsym과공식v249 sd-bus C signatures를좁은private 모듈에서사용하는Proposed대안이다. 새Rustdependency는0이지만runtime조건/807936B 관측hostlibrary를제품배포물크기0으로혼동하지않는다. loaderhandle은함수pointer/RAII bus/message/slot보다오래산다. 누락symbol/library/ABI/invalidbus/권한failclosed이며user환경의LD_LIBRARY_PATH등library injection 경계는rootreview필수다. 가변인자C API대신append_basic/open_container/close_container/read_basic같은정확signature를사용한다. sd_bus_error C layout/free와nullableoutputs를공식header로검증하고negativeerrno를양의ioerror로만복원한다. libsystemd내부OOM/symbol불일치/rustallocation 상한시험을추가한다.

## 유한 종료와 실패경로 인수

sd_bus_call timeout은microsecond값으로각phase의남은deadline을명시하며0 default/infinity를사용하지않는다. [v249 call API](https://raw.githubusercontent.com/systemd/systemd/v249/man/sd_bus_call.xml). 초기제안은start call5초/R10초/stop2초+확인grace3초다. RuntimeMaxUSec는fixture45초를제품8시간에복사하지않고ProcessSpec.timeout의checked변환+정리여유와대조한다. 취소200ms는기존T→K간격이지manager call이즉시중단된다는보장이아니다.

현재Ownership attach 실패와Drop 및cancel의Child.wait는무한대기할수있다. launcher가bus wait에서끝없이멈추지않게각phase deadline/managerdisconnect/ownunitstopack를설계하고worker유한반환을실제확인한다. StartTransientUnit timeout은서비스가만들어졌을수있는모호결과다. nonce조회/identity확인된부분service정리와FD해제를시도하고실패는보호미완료로보고한다. manager부재/no-env/권한/OOM시새tool이시작되지않는것이기본이며이미시작된경쟁은무조건미시작으로표현하지않는다. unit정리확인불능이면임의PIDkill로우회하지않고범위를기록한다.

필수 actual native 인수는다음과같다.

- 정상exit0/nonzero/signaled/fast-exit≤1ms을반복해정확Child status와stdout/stderr framing/EOF 확인. 지속stdout·pipeholder·setsid/fork/doublefork는상한drain을유지한다.
- R/T/K/EOF와start/readiness/exec/missinghelper/library/symbol/bus/no-env/FDclosed/dupOOM/permission 실패. FDcount·message대조로다른jobcontrol이inherit되지않는것확인.
- guardianSIGKILL, appSIGKILL, app+guardian동시SIGKILL, launcherSIGKILL, launch중app소실. exactpidfd/startticks/InvocationID/cgroup membership/종료·빈cgroup 및worker유한반환 직접확인.
- 두job동시실행에서하나만cancel/crash하고다른job·소유무관sentinel 생존확인. 이름collision modefail/InvocationID변경/querytimeout은무관unit정리를하지않음.
- managerdisconnect/OOM/delegation거절/uninterruptible syscall/logoff/sleep는성공범위와실패범위를분리. 독립manager cgroup이동시죽음에도정리하는것을실증하기전backend보호채택하지않음.
- 합성download→cancel/pause/failure/DBpublication/recovery와실제8h메모리증가, launcher/managerCPU·RSS/시작지연·패키지런타임조건을별도측정. cgroupfork상속은악의적사용자권한migration탈출의보장이아님.

이번review는문서읽기/원시JSON대조/공식v249source 조회만수행했다. 새unit생성·서비스stop/kill·cgroupwrite·제품source/Cargo/기존8h프로세스변경은없다. root는sharedinterfaces·error/statuschannel·환경전달·retainedFD/GC·deadline을확정한뒤구현파일을독점배정해야한다. 단순weakfallback 도입으로R08/S03를완료시키지않는다.

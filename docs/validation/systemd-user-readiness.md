# Linux 사용자 systemd/cgroup 준비 상태와 guardian 후보

[작업 계약](../work-items/S03-systemd-user-readiness.md), [실제 host 관측](evidence/systemd-user-readiness-20261003/observation.json), [도구·공식 source 근거](evidence/systemd-user-readiness-20261003/source-review.json). 2026-10-03 UTC12:45, boot0bc0c937-eee9-4d90-95dc-31a2122b6352. 이 문서는 **읽기 전용 조사와 Proposed 후보**이며 아키텍처 채택·제품 보호 구현·crash 인수 완료가 아니다.

## 실제 host 준비 상태

systemctl/systemd-run은249.11-0ubuntu3.22다. 선택 property show와 --user is-system-running은exit0이며 user manager running이다. UID1000의 /run/user/1000은0700, bus와systemd/private는 실제 UID1000 socket이다. 현재 subprocess의 cgroup은/non-systemd이고 해당 디렉터리/procs는root-owned로현재사용자가쓸수없다.

host cgroup2 mount는rw/nsdelegate다. 이는 기존 [app subreaper 검토](../work-items/S03-app-subreaper-fallback-review.md)의 sandbox ro view와 다르며 과거 기록을 틀렸다고 바꾸지 않는다. system manager의 특정 user@1000.service 조회는ActiveState=active/SubState=running/Delegate=yes/ControlGroup=/user.slice/user-1000.slice/user@1000.service/KillMode=mixed다. 해당 사용자 cgroup 디렉터리, cgroup.procs, cgroup.subtree_control은UID1000이며metadata의writeaccess=true다. 위임된controllers는memory/pids다. root·non-systemd cgroup writeaccess=false이며 사용자 cgroup의cgroup.kill도root-owned/writeaccess=false다.

따라서 사용자manager를 통한소유transientservice 실험후보는기존readonly근거만으로배제하지않는다. 그러나 metadata/os.access와manager연결은실제unit생성·FD전달·cgroup정리권한 성공을증명하지않는다. cgroup.kill 파일존재는직접쓰기권한이나앱죽음시자동정리증거가아니다. 이번조회는선택manager/user@UID property만이며전체unit목록·다른앱process목록·환경·credentials는출력하지않았다.

## systemd-run 후보와 FD 계약: Proposed

설치된249의help에--user/--pipe/--wait/--quiet/--service-type 옵션이있다. 공식v249문서는--pipe가caller stdFD를서비스에그대로전달하고--wait가unit비활성화까지기다린다고설명한다. Type=exec는execve성공후start성공이므로simple의fork직후성공보다강하지만guardian의실제R readiness를대체하지않는다. [공식 systemd-run v249](https://raw.githubusercontent.com/systemd/systemd/v249/man/systemd-run.xml).

v249 run.c의ARG_STDIO_DIRECT는StandardInputFileDescriptor/Output/Error를DBus h형으로전달한다. 앱의socketpair에서guardian쪽끝을launcherstdin에지정하고service guardian이FD0을UnixStream으로받는후보가소스계약상가능하다. 앱쪽control끝은앱만보유하고CLOEXEC/불필요FD차단을유지해야EOF가owner소실을뜻한다. FD는예전숫자3이자동보존된다고가정하지않으며양방향R/T/K와EOF를합성으로검증해야한다. --pty/--shell은후보에쓰지않는다. [공식 run.c v249](https://github.com/systemd/systemd/blob/v249/src/run/run.c).

서비스parent는앱이나launcher가아닌user manager다. MainPID는guardian을직접ExecStart한실제guardian이어야한다. launcher만죽이는것은서비스정리증거가아니다. 현재guardian은자신의직접tool child에PDEATHSIG를설정하므로그부모관계는유지할수있지만app Ownership의Child/waitpid는launcher와guardian을분리해조정해야한다. 외부wait(-1)/unknownPID귀속을도입하지않는다. 기존프로세스adapter를systemd-run한겹으로싸기만해서정리완료로표시하지않는다.

## MainPID 종료와 cgroup 정리: Proposed

공식service계약은main process가자연종료하거나죽어도정지동작을수행한다. Type=exec의guardian MainPID가SIGKILL되면서비스실패/정지와소유cgroup정리로이어지는후보이며guardian 자체subreaper가죽는기존경계를보완할수있다. Restart=no/RemainAfterExit=no와정확MainPID를검증해야한다. [공식 service v249](https://raw.githubusercontent.com/systemd/systemd/v249/man/systemd.service.xml).

KillMode=control-group은해당unit의남은process를정지시정리하며TimeoutStopSec후SendSIGKILL로강제종료한다. 무관한사용자process나현재8hscope를대상으로하지않고고유한앱소유unit이름·ControlGroup만사용해야한다. 위user@1000.service의KillMode=mixed를실제job unit값으로재사용하지않는다. 매job에KillMode=control-group/SendSIGKILL=yes/Restart=no/RemainAfterExit=no/유한TimeoutStopSec를명시적으로설정·조회하는후보이며이번에는설정하지않았다. [공식 kill v249](https://raw.githubusercontent.com/systemd/systemd/v249/man/systemd.kill.xml).

setsid/새PGID/fork도같은cgroup membership을유지하는것을실제로확인해야한다. 앱+guardian이동시에죽어도독립user manager가살아있으면같은후보경로로정리될가능성이있지만이번에는실험하지않았다. user manager까지죽는경우,직접cgroup탈출,권한실패,uninterruptiblekernelwait와logout/sleep보장은별도다. 단지cgroup을만들었다고앱소실시자동정리나모든고아방지보장이라고보고하지않는다.

## literal argv와 환경의 추가 선행 조건

서비스는caller환경을전부자동상속하지않는다. root가필요한최소환경·작업폴더·절대도구경로를명시적으로전달하고credentials를unitdescription/명령미리보기/로그에노출하지않는계약을결정해야한다.

v249 run.c는ExecStart a(sasb)로argv를전달하지만execute.c는NO_ENV_EXPAND가없으면replace_env_argv를호출한다. 로컬249help에는--expand-environment=no가없다. 따라서 `$VAR`/`${VAR}`·공백·따옴표·percent·한글·줄바꿈이현재제품의literal argv와같이보존되는지를별도검증하기전에는이wrapper를제품에채택하지않는다. no-shell이어도manager의환경치환이없다는뜻은아니다. v249 API/escaping·정제된설정전달프로토콜등가역적대안을root가검토하며사용자인자를거부하는축소된기능으로P0를통과시키지않는다. [공식 exec source v249](https://raw.githubusercontent.com/systemd/systemd/v249/src/core/execute.c).

## 다음 합성 실험과 검증 범위

root가소유임시폴더/고유unit/argv·FD계약을정한뒤다음별도작업으로unit생성/readiness/MainPID+ControlGroup 확인,정상끝·취소·guardianSIGKILL·appSIGKILL·동시SIGKILL·fork/setsid/pipeholder·두job+무관sentinel·missinghelper·startup/cgroup권한실패·literalargv를시험한다. exactowned descendants 종료/빈cgroup/유한wait/worker반환·무관sentinel생존을직접대조한다. 새로운준비불능이면해당보호후보만실패·차단하고나머지개발을계속한다. admin/cgroup mount권한/globalservice/security설정변경은허용된범위가아니다.

이번실제명령: systemctl --version, systemd-run --version, systemctl --user show --property=Version --property=ControlGroup, systemctl --user is-system-running, systemctl show user@1000.service의선택5property. 모두exit0이며argv/raw출력은observation.json에있다. 추가systemd-run --help도exit0다. hostmetadata·공식source읽기만수행했고transientunit/service생성·cgroupwrite·stop/kill/restart·기존8hPID/서비스변경은없다. guardian SIGKILL P0릴리스게이트는여전히미완료다. 비용은추가Cargo의존성0/기존systemd도구재사용후보이며per-joblauncher/managerRSS·시간·실제정리비용은아직미측정이다.

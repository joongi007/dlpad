# S03: terminal control socket reset 읽기 검토

- 상태: done
- 분류: review
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: metrics_recovery — 이 문서만; process.rs/guardian.rs/user_service.rs/terminal.rs 읽기 전용
- 선행 작업: bounded terminal GC/GU·LC/LU protocol, startup 전체 deadline

## 관측과 가설의 구분

root의 full engine oversized case는 간헐적으로 TerminalUnconfirmed였으나 exact single 재실행은 PASS였다. root socketpair 모델에서는 GC가 queued인 채 peer가 unread K를 보유하고 close하면 GC 뒤 ECONNRESET104가 나왔다. shutdown Read/Write/Both도 해당 unread 상태 자체를 없애지 못했다. 이 모델은 실제 실패 원인이 late K라고 확정하는 증거가 아니다. 현재 typed 오류가 errno/partial terminal frame을 숨기므로 실제 실패의 내부 trace가 추가로 필요하다. reset을 EOF로 처리하는 parser 완화는 제안하지 않는다.

## 실제 코드 경로

process.rs의 cancel/timeout/outputlimit 분기는 T를 보낸 뒤200ms grace에서 child.try_wait가 Some이면 break하지만 그 다음 K를 무조건 보낸다. final child status를 얻은 뒤에도 normal/error outcome과 무관하게 poststatus K를 보낸다. 그때 guardian은 이미 GC를 보냈거나 종료 중일 수 있다. signal의 BrokenPipe만 성공으로 취급하므로 successfully queued late K도 이후 terminal read의 reset 후보가 된다.

guardian supervise는 cleanup group SIGKILL/direct child wait/reap_descendants/status 결과가 모두 성공하면 GC를 쓴다. 그 뒤 호출자가 process exit한다. socket clone을 읽는 control thread는 detached이며 종료 여부/큐 소비를 기다리지 않는다. launcher는 같은 guardian endpoint alias를 유지하여 GC 이후 service/cgroup 확인과 LC를 쓴다. app terminal::confirm은 Child reap 후 exact GC 또는 GCLC와 진짜 read EOF만 허용하고 reset은 typed 미확인이다. 이 안전 조건은 유지한다.

## 가장 작은 app-side 수정 계약

1. Linux grace에서 child.try_wait Some을 기억하고 이미 회수되면 K를 생략한다. 아직 unreaped이면 기존 owned control K를 한 번만 보낸다. Linux poststatus의 unconditional K는 제거한다. guardian의 성공 status/GC 이전에 이미 전체 group/directchild/descendant cleanup이 끝나므로 회수된 guardian에 K를 추가하는 것은 종료 증거를 강화하지 않는다. Windows TerminateJobObject poststatus 정책은 이 Linux socket 수정과 별개로 유지한다.
2. 취소/timeout/outputlimit의 **마지막** T/K 뒤 app endpoint Shutdown::Write를 Child final bounded wait 이전에 수행한다. socket read 방향은 유지하여 GC/LC를 계속 받는다. 정상 경로에서 already-reaped status를 확인한 뒤에도 Write half-close는 가능하지만 그 시점에는 추가 command를 보내지 않는다. half-close 실패는 typed 미확인으로 분류하고 일반 성공으로 숨기지 않는다. early error/unwind의 기존 alias Shutdown::Both는 보호 cleanup 수단으로 유지한다.
3. app normal execution R 직후에 Write를 닫으면 guardian이 EOF를 K로 해석해 tool을 즉시 죽이므로 금지한다. command phase와 terminal phase를 명확히 나누고 정상 실행 중 취소 채널을 계속 유지한다. terminal read 시작 뒤 command writer를 다시 사용하지 않는다.
4. Unix shutdown은 같은 endpoint의 clone에도 적용된다. app lifetime alias에도 write EOF를 적용해야 하며 단순 원 descriptor drop으로 해결하지 않는다. app half-close는 반대 endpoint의 launcher GC/LC write를 막지 않는다. guardian/launcher endpoint의 SHUT_RD/SHUT_RDWR를 정상 종료에 추가하면 app에서 오는 command를 버리거나 서비스 LC write까지 닫을 수 있어 금지한다.

## guardian reader 수명 보완과 deadlock 경계

reader JoinHandle을 보유하고20ms read timeout 또는 nonblocking poll 및 terminal-drain flag를 사용해 cleanup 완료 시 queued command를 bounded하게 소비할 수 있도록 한다. control reader만 socket의 command 방향을 읽는다. terminal drain 중 읽은 K는 이미 완료된 cleanup 이후 numeric PID를 다시 신호하는 근거가 아니다. 새로운 thread/queue를 추가하지 않는다.

**모든 정상 종료에서 EOF까지 join하는 변경은 현재 caller 순서와 deadlock한다.** app은 child exit를 먼저 기다리고 그 뒤 terminal read/half-close하므로 guardian이 정상 process exit 전에 app EOF를 필수로 기다리면 둘 다 대기한다. 최소 보완은 stop flag+read timeout으로 bounded queue drain/reader 종료를 관리하고 app이 terminal phase 이후 late command를 보내지 않는 계약과 함께 적용하는 것이다. cancel 경로는 app의 선행 Write half-close가 실제 EOF를 제공한다. normal natural exit는 poststatus K 제거가 late writer를 없앤다. 상대방이 계속 임의 byte를 보내거나 read thread가 bounded 종료하지 못하면 terminal/protection failure로 보존하고 무한 join하지 않는다.

control-thread drain을 cleanup proof의 일부로 엄격하게 만들려면 GC를 먼저 보내고 app이 GC를 실행 중 관찰한 뒤 Write half-close하는 동시 terminal observer가 필요하다. 이는 현재 Child-reap-before-confirm 계약보다 큰 변경이므로 첫 최소 수정에 포함하지 않는다. GC를 썼다는 이유만으로 reader 종료 실패를 숨기거나 EOF 이전 reset을 성공으로 바꾸는 안은 금지한다. 가능한 순서/실제 kernel 동작은 dedicated synthetic fixture로 확정한다.

## service 경로

service guardian과 launcher는 같은 endpoint의 writer지만 command reader는 guardian 하나다. app Write half-close→guardian command EOF→tree cleanup→GC→manager 같은 Invocation/cgroup empty 확인→LC→launcher alias close 순서다. launcher endpoint를 app처럼 Write half-close하면 LC까지 차단하므로 역할을 혼동하지 않는다. guardian의 read clone이 process exit 전에 끝나도 launcher alias는 유지돼야 한다. app은 GCLC 뒤 EOF를 끝까지 확인하고 GU/LU/extra bytes/reset은 실패한다.

startup preR error의 EP/LU 및 alias Shutdown::Both는 정상 terminal handshake와 별개다. rooted same-boot cleanup-only receipt가 나중에 native empty를 증명할 수 있어도 현재 process terminal reset을 execution success로 바꾸지 않는다. receipt 실패/미검증 cookie는 보존한다.

## diagnosis 및 인수 조건

- 제품 공개 error/log는 고정 유지하고 owned fixture의 전용 raw trace에만 timestamp/command byte kind/Child reaped bool/guardian GC write 결과/reader exit reason/terminal errno·수신 길이를 기록한다. URL/argv/cookie/env는 기록하지 않는다. 실제 full-engine 실패와 같은 immutable snapshot으로 원인을 재현한다.
- socketpair 최소 모델은 GC queued+late unreadK에서 reset 실패를 유지한다. 수정 caller 순서인 final K→app Write close→peer consume EOF→GC/GCLC→close는 실제 EOF를 보여야 한다. reset!=EOF parser 회귀는 보존한다.
- immediate tool exit0/42/126, oversized output, cancel duringT grace와grace 안 이미reaped, timeout, stdout/stderr writer-held helper를 default parallel 조건에서 반복한다. 실제 command trace에 poststatus K0/중복K0과 exact terminal EOF를 확인한다. 시험 직렬화/임의 sleep 확대/슬롯 확대는 원인 수정이 아니다.
- direct 및 actual owned user service 모두 exactGC/GCLC+EOF, pipe EOF, exactPID/startticks/Invocation/cgroup cleanup과 unrelated sentinel 보존을 인수한다. 기존 guardianSIGKILL/missing frame/manager failure typedunknown 시험은 약화하지 않는다.
- guardian reader 종료 fixture는 정상 natural exit에서 app writer가 아직 열려 있어도 무한 join하지 않고, cancellation에서는 app half-close 후 실제 EOF를 소비한다. continuous foreign byte stream/reader error는 유한 실패다. raw native probe가 GC를 읽고 나중에 socket을 닫는 현재 순서도 별도 검증한다.

## 실행과 한계

실제 수행은 sed/rg/cat의 코드 읽기와 이 문서 작성뿐이다. Rust/Cargo/기존 evidence/process/manager를 변경하거나 시험하지 않았다. root socket 모델은 주어진 관측으로 인용했고 독립 재현 PASS로 쓰지 않았다. root가 최소 app-side 수정과 필요 guardian 수명 보강을 선택·계측·실제 반복 검증한 뒤 완료 처리한다.

## guardian bounded reader 구현 handoff

root가 process.rs의 Linux lateKskip/grace-reapedKskip/finalstop Write half-close를 먼저 구현한 뒤 metrics_recovery가 guardian.rs만 수정했다. ControlReader는 기존 한 control thread의JoinHandle과stopflag를보유하고UnixStream read timeout20ms를설정한다. runtime timeout은중단신호가아니며계속기다린다. EOF는기존K의도,actualreaderror는고정readerfailure다.

cleanup 결과를얻은뒤 finish가stopflag를설정하고queuedbytes를소비한다. EOF 또는첫timeout이면reader가끝나고join은is_finished확인후에만한다. continuouswriter는1초drain절대상한으로실패한다. main finish도1초상한이며worker미확인시GU로반환하고무한join하지않는다. 정상naturalexit에서appwriter가열려있어도첫20ms idle에서끝나므로EOF필수join deadlock이없다. toolkill/reap 결과와readerfinish가모두성공해야GC다. Drop은같은stopflag만설정하고무한join/새thread를만들지않는다. readerthread는stop후자체deadline을갖고앱을종료시키는helperprocess수명밖으로확장하지않는다.

내부시험2개를작성했다. livewriter를유지한normalfinish가500ms이내끝나며JoinHandle을회수한다. 실제socketpair에서TKK queued→appWritehalfclose→readerEOF/input2→GC→directEOF 및launcheralias유지→LC→serviceEOF를검증한다. reset을EOF로취급하지않으며기존terminalparser는수정하지않았다. runtime최종20ms drain과finish polling의최대20ms scheduling비용이추가된다. nativeperformance근거는root가새snapshot으로별도갱신해야한다.

guardian executable resolver는등록된app가없으면adjacent dlpad-process-guard 또는deps부모target/debug helper를is_file만보고쓴다. cargo test -p dlpad-engine은이helper binary의현재source재빌드를보장하지않는다. root는explicithelperbuild/SHA입력확인후실제반복회귀를해야한다. controlledactualhelper실험은R→fasttoolGCqueued→lateK주입의reset과finalK→Writeclose→readerdrain→GC/EOF를대조하는것이다. 아직이실험을실행하지않았다.

source SHA256 `d9f4ddbdac34715bc5eaccb6971d37f8a94e58bccfc9cff1a0e334fca3d86186`, rustfmt exit0, Cargo미실행. root의최소수정후195PASS/3FAIL(metadata cap/outputlimit/100msstartup timeout)은당시snapshot근거로보존하며이번reader수정이실제원인을해결했다고아직주장하지않는다. process.rs/windows.rs/terminal.rs/Cargo수정없음. 전체원인확정과native/fullparallel회귀는root통합대기다.

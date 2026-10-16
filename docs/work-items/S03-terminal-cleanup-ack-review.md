# S03: terminal cleanup acknowledgment 계약 검토

- 상태: done
- 분류: review
- 우선순위: P0
- 요구사항: R06, R08, R11, R13
- 담당/수정 범위: metrics_recovery — 이 문서만; process.rs/process/guardian.rs/process/user_service.rs 읽기 전용
- 선행 작업: 명시3경로 OwnershipCleanupError 및 DownloadError::SessionCleanupDeferred

## 사용자 결과

도구가 정상적으로 exit126을 반환한 경우와 guardian 보호/정리 실패가 exit126으로 보이는 경우를 stderr 없이 구분해야 한다. 명시한 process3경로의 typed 분류는 이미 별도 작업이며, 이 문서는 추가 terminal 계약 제안이다. done은 검토 완료이며 제품 프로토콜 구현 완료가 아니다.

## 실제 코드 관측

guardian은 tool spawn 뒤 R을 보낸다. 앱→guardian 제어는 T/K이며 소유 앱 peer EOF는 강제 중단 의도다. supervisor는 leader 관측 루프 뒤 소유 그룹/아직 unreaped leader를 종료하고 child.wait(), reap_descendants()를 수행한다. reap_descendants는 자신의 bounded children 목록·WNOHANG·3초 deadline으로 descendants를 회수한다. supervise의 run/killed/reaped/status 중 하나가 Err이면 entry는 자유 stderr 오류와 exit126만 낸다. tool 자체의 genuine exit126도 Ok(status)→exit126이라 같은 숫자다. terminal control frame은 없다.

service backend는 guardianMainPID 및 InvocationID와 cgroup을 관측하고 ExecMainCode/Status를 launcher의 exit/signal로 proxy한다. wait_unit은 서비스가 terminal이고 cgroup populated=0 또는 제거이며 InvocationID가 같은 경우 정상 status를 반환한다. 그러나 guardian의 exit126 원인이 tool인지 supervisor 오류인지는 이 manager 상태로 알 수 없다. wait/stop/cleanup/Unref 오류 역시 launch_service Err→EP+SHUT_RDWR→entry126이며, R 이후 EP는 기존 readiness-only app reader에 의해 분류되지 않는다. guardian이 비정상 종료된 경우 manager cgroup 정리는 기대할 수 있어도 현재 app이 supervisor 성공을 확인하지는 않는다.

process의 typed StartupDeferred/StartupUnconfirmed는 outstanding 소유 Child를 실제 reaper에 이관한 경우다. OutputDrainInterrupted는 writer 소유 보호 미완료다. 이 세 가지가 아닌 모든 exit126/모든 I/O 오류를 같은 marker로 확장하거나 stderr 문구를 matching하면 정확한 소유 증거가 사라진다.

## 가장 작은 견고한 프로토콜 제안

기존 R, EP, app→T/K/EOF를 유지하고 소유 control endpoint에 고정2byte terminal 프레임 두 종류만 추가한다. versioned 문서 계약으로 정의하고 서로 다른 앱/guardian protocol revision 혼용을 허용하지 않는다.

| 프레임 | 발신자 | 의미 |
|---|---|---|
| GC | tool을 감독한 guardian | supervise 성공 및 자기 children 회수 확인. Child status는 별도 exit/signal로 유지 |
| GF | guardian | 보호/관측 단계 실패지만 자기 children 정리는 확인됨 |
| GU | guardian | 자기 children 정리 확인 불가 |
| LC | service launcher | 같은 invocation의 service terminal/cgroup 정리 확인 및 manager ref 처리 성공 |
| LF | service launcher | launcher 보호 단계 실패지만 해당 invocation/tr리 정리는 확인됨 |
| LU | service launcher | 초기 identity 미확인, manager/cleanup 실패 등 service 정리 확인 불가 |

C/F/U는 자유 텍스트/errno/PID/path/cookie를 포함하지 않는 단일 enum byte다. tool이 실행 상태126을 반환해도 GC와 exit126이면 genuine tool failure로 처리한다. GF는 tool126으로 오인하지 않는 고정 protection failure다. GU 또는 LU는 typed cleanup unconfirmed이며 lease 보존을 요구한다. 정리 확인 F와 미확인 U를 구분하지 않는 보수적 단일 실패 프레임도 가능하지만 그때는 안전한 오탐 보존 비용과 의미를 명시해야 한다.

direct backend는 G 프레임이 terminal 인수다. service backend는 guardian G와 launcher L의 두 단계가 필요하다. GC만으로 manager의 전체 cgroup/서비스 정리를 인수하거나 LC만으로 supervisor 성공을 추정하지 않는다. service G는 guardian exit 전에 쓰고 L은 guardian이 terminal인 뒤 쓰므로 정상 두 발신자의 데이터가 경쟁하지 않는다. 앱 parser는 backend 기대 프레임을 알고 있어야 하며 실제2byte write/partial receive도 처리한다. 같은 Unix stream을 둘이 read하여 acknowledgment를 서로 빼앗는 방식은 사용하지 않는다. launcher와 service는 동일 guardian-side endpoint를 가진 발신자이며 최종 인수 판단은 app-side reader가 한다.

프레임 최대2개/4byte와 기존 startup 프레임까지 유한 buffer를 둔다. 중복·미지 코드·역순·truncated·예상backend에 없는 프레임은 protocol/protection error다. 확인되지 않은 상태를 더 늦은 C 하나로 덮어쓰지 않는다. guardian/service endpoint는 모두 CLOEXEC로 tool에 전파되지 않으므로 tool stdout/stderr에 fake GC를 써도 인증되지 않는다. fatal signal handler에서는 소켓쓰기/할당/로그를 하지 않는다. 비정상 죽음은 missing frame으로 관측한다.

## 구현 수명과 유한 대기

supervise의 socket은 run/killed/reaped/status 결과를 분류하고 terminal G를 쓰는 순간까지 유지해야 한다. early setup 실패도 endpoint를 확보했다면 fixed failure를 보낼 수 있으나, child/service 생성 여부가 모호하면 U가 안전하다. leader wait의 blocking 경계는 이전 startup-cancel-deadline 검토와 별도 해결해야 한다. frame만 추가하면 그 대기가 유한해지는 것은 아니다.

launcher의 alias는 service terminal L을 쓰기까지 유지한다. partial Start 성공의 identity 조회 실패는 EP를 보내고 소유 socket SHUT_RDWR로 guardian EOF를 강제하지만, 이후 성공 acknowledgment를 보내겠다고 약속하면 안 된다. 이 path는 미확인으로 분류하고 lease를 보존한다. identity가 확정된 경우만 기존 StopUnit/InvocationID 재검증/terminal/cgroup proof를 사용한다. cgroup identity/evidence를 한 번도 확보하지 못했다면 없는 terminal property를 성공 증거로 확대하지 않는다.

app control은 readiness 이후에도 살아 있어야 한다. Child terminal까지 GC/GF/GU와 LC/LF/LU를 모으고, terminal 이후 pending byte를 EOF보다 먼저 drain한다. poll은 최대20ms로 cancel/deadline과 일관되게 관찰하고, terminal frame 최종 대기는 예시1초 absolute deadline으로 제한한다(launcher는 exit 전에 frame을 쓰므로 정상적으로 이미 queued 상태다). empty buffer/EOF에 busy loop하지 않는다. data가 없거나 socket/Child가 비정상 종료되면 missing acknowledgment typed 오류로 반환한다. signed tool exitstatus를 임의로 신호 실패로 재해석하지 않는다.

취소에서는 app peer를 먼저 닫으면 guardian EOF 정리는 시작되지만 app이 그 뒤 acknowledgment를 읽을 수 없다. 가능하면 T→K를 보내고 같은 peer를 terminal acknowledgment까지 유지해야 한다. forced app exit/partial-start SHUT_RDWR 경로는 예외적으로 acknowledgment 미확인으로 기록한다. 읽기 thread와 socket 소유권은 단일하며 새 unbounded thread/queue가 필요 없다. 이미 outstanding reaper Child가 있으면 acknowledgment 성공으로 덮어쓰거나 먼저 lease를 cleanup하지 않는다.

## lease와 recovery

GC+LC 및 launcher reap/pipe drain이 인수된 일반 성공·실패·취소 경로에서만 cookie lease를 정상 cleanup한다. missing/ GU/LU/typed process 미확인 경로는 DownloadError::SessionCleanupDeferred로 이어져 UI/session 소유자가 private copy를 보존한다. 인증 로그 생략 이후에도 이 typed 정보는 유지한다.

다음 시작 cleanup이 단순히 원래 app PID가 죽었다는 사실만 확인하면 살아 있는 guardian/service가 필요로 하는 복사본을 조기에 지울 수 있다. recovery 역시 실제 manager/guardian 소유 종료를 확인하거나, 확인 불가한 copy를 계속 보존해야 한다. nonce unit/InvocationID/startticks 같은 secret-free owner 참조를 private recovery marker에 포함하는 선택은 root/stress가 별도 저장 계약을 결정해야 한다. 실행 경로/cookie 값은 진단/공유DB에 넣지 않는다. 장기 보존의 용량·개수 상한 및 사용자 고정 오류는 별도 인수 대상으로 남는다.

## 인수 조건

- direct/service tool genuine exit126: GC(및 LC), 원 exit126 보존, tool failure이며 cleanup-unconfirmed로 오인하지 않음.
- 합성 supervisor run 실패+정리 성공: GF와 고정 protection error; raw stderr secret은 무관.
- guardian cleanup timeout/error: GU 또는 missing G, 실제 descendants/cgroup 증거와 미확인 분류 일치.
- guardian SIGKILL/service manager 확인 불가: missing G 또는 LU, 성공으로 확대하지 않으며 private lease 보존.
- GC 후 launcher cleanup 실패: GC+LU, LC 대신 단일GC로 성공 처리 금지.
- truncated/duplicate/역순/미지 frames 및 stdout fake frame: bounded parser failure, no secret payload/no busy polling.
- partial Start accepted 후 identity 실패: 소유 socket EOF cleanup 요청, startup EP 후 미확인typed·copy보존, 미확인 unit/PID 신호 없음.
- R 전후 및 terminal 전후 cancel/CloseWait: token 일관성, UI response, peer 수명, 실제 child/descendants/cgroup/pipe와 copy 수명 확인.
- outstanding reaper 이관: 원 Child 회수와 terminal 미확인 정보를 독립적으로 유지, 복사본 조기삭제 없음.
- 실제 native17 및 기존 public EP/noR2 회귀를 원시 evidence로 보존하고 새 protocol snapshot에서 재검증.

## 검증과 handoff

실제 수행은 sed/rg 읽기와 전용 문서 작성이다. 소스/Cargo/manager/실credential를 변경·실행하지 않았다. root가 프레임 이름·revision·backend 기대 상태·typed errors 및 guardian/launcher 수명 공유 계약을 결정한 뒤 별도 구현/시험을 배정해야 한다. 이 제안을 현재 제품의 검증 완료로 표현하지 않는다.

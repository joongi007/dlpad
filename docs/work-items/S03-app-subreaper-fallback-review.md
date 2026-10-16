# S03: 살아 있는 앱의 guardian 고장 fallback 검토

- 상태: done
- 분류: research
- 우선순위: P0
- 요구사항: R02, R04, R11
- 담당/수정 범위: library_identity — 이 신규 workitem만
- 선행 작업: ADR-0002 Proposed, guardian-fault 및 guardian-drain 실제 실패 증거

## 사용자 결과

앱이 살아 있을 때 guardian 강제 종료의 자손 누출을 보완할 후보 계약과 한계를 기록한다. 이는 read-only 분석 완료이며 제품 수정이나 보호 기능 완료가 아니다.

## 계약과 범위

읽은 제품 파일은 crates/engine/src/process.rs 및 process/guardian.rs다. 현재 guardian은 작업별 subreaper이고 직접 leader의 PDEATHSIG를 설정한다. 앱 Ownership은 anonymous control socket과 guardian PID만 보유하며 readiness는 1 byte R다. 제품에 작업 자손의 pidfd registry나 app subreaper는 없다. 현재 PID와 실제 fixture pidfd를 보유한 tests/guardian.rs 및 scripts/guardian_probe.py의 증거를 제품 API로 혼동하지 않는다.

기존 guardian-fault.md의 SIGKILL probe는 protection_passed=false/exit2이며 일반·setsid 자손 두 개가 생존했다. guardian-drain.md는 2초 이내 bounded 반환을 증명했지만 escaped 자손이 여전히 살아 있음을 확인했다. pipe drain 중단은 자손 정리 완료가 아니다. 기존 soak/source/Cargo/프로세스 실험은 수정하거나 실행하지 않는다.

## 안전한 후보 설계: Proposed

1. Linux 앱에서 어떤 작업 guardian보다 먼저 PR_SET_CHILD_SUBREAPER를 설정하고 실제 성공을 확인한다. guardian 내부 subreaper는 유지한다. 정상일 때 orphan은 가까운 guardian에 남고 guardian 고장 시 살아 있는 앱으로 입양되는 모델이다. 앱 process 전체에 영향을 주므로 모든 외부 spawn과 Child 회수 경로를 먼저 조사하고 중앙 registry로 합의해야 한다. UI thread에서 대기하거나 signal handler에서 할당·kill·wait하지 않는다.
2. 직접 생성한 guardian은 spawn mutex 아래 registry에 등록하고 unreaped child identity에서 pidfd를 취득한다. 최소 계약은 owner scope/guardian generation/Job ID/Attempt ID, pidfd와 직접 child wait 책임이다. 숫자 PID나 /proc start_ticks는 진단 보조이며 signal identity는 pidfd다. 다른 thread의 Child::try_wait/wait와 registry supervisor가 같은 자식을 reap하지 않도록 한 책임자로 통합한다. SIGCHLD SIG_IGN/SA_NOCLDWAIT 또는 외부 wait(-1)도 금지·검사 대상이다.
3. 앱의 여러 thread가 guardian을 spawn하므로 /proc/self/task/<tid>/children을 한 thread만 읽어서는 충분하지 않다. 모든 spawn을 전용 supervisor thread에 모으거나 앱 자체 task 목록·children을 bounded로 읽는 계약이 필요하다. /proc 전체 PID 검색은 하지 않는다. children 관측은 동시에 종료 중인 자식을 누락할 수 있어 완전한 일회 snapshot이 아니다. 단일 reaper 아래 unreaped 직접 자식임을 확인한 뒤 pidfd를 취득하고 이름별 wait를 수행한다. registry/child 상한 초과·권한·관측 실패는 신규 job 금지와 보호 실패 반환이다.
4. guardian pidfd readable 또는 control socket EOF/HUP를 출력 EOF/join보다 먼저 감지한다. 출력 pipe EOF는 외부 writer가 닫아야 발생하므로 guardian 생존 판단이 아니다. readiness socket은 tool exec에 CLOEXEC이므로 외부 도구가 guardian control socket을 보유하는 현재 계약도 유지한다. guardian 정상 exit와 고장 분류에는 별도 cleanup-complete ack 또는 확인된 owned child 회수 상태가 필요하다. 도구 exit code만 전달하는 현재 exit status로 완전 정리를 추정하지 않는다.
5. 고장 감지 즉시 신규 spawn을 막고 소유 job들을 Interrupted/보호 실패로 처리하는 fail-closed 후보를 우선한다. 앱으로 입양된 새 자식은 앱 ancestry는 증명되지만 이전 guardian/job 정보가 없어 단일 job에 자동 귀속할 수 없다. 여러 guardian 동시 고장, fork 직후 고장, setsid, 부모 조기 종료에서는 per-job pipe·PGID·argv·PID 파일·이전 관측 lineage만으로 귀속을 증명할 수 없다. 모든 작업을 멈추는 정책은 동시 job의 완료·공개도 차단해야 하므로 core/engine 공유 계약을 root가 결정해야 한다.
6. 미등록 입양 자식은 모든 앱 descendant가 작업 guardian 계열이라는 검증된 spawn 계약이 있을 때만 job 집합의 소유로 정리할 수 있다. 브라우저/파일 열기 등 non-job subprocess가 앱에서 만들어질 수 있으면 그 자손까지 죽이지 않도록 별도 supervisor/비작업 scope 계약이 필요하다. unknown child는 임의 kill하지 않고 보호 미충족으로 보존·오류 처리한다. per-job 등록된 pidfd가 있으면 해당 exact identity는 정리할 수 있지만 늦은 등록은 미등록 fork race를 해결하지 않는다.
7. 정확히 소유가 확인된 adopted child에만 pidfd_send_signal을 사용하고 이름별 wait로 회수한다. 새로운 입양을 반복 관측하되 bounded deadline/child count를 넘으면 성공을 주장하지 않는다. PID 재사용과 무관한 다른 process에는 signal하지 않는다. 무한 fork/종료 불가능한 kernel wait까지 userspace polling으로 완전 보장하지 않는다.

## 보장 범위와 대안

위 모델은 앱이 계속 살아 있고 중앙 spawn/reap 계약과 소유 입양 증거가 성립할 때만 후보이다. 앱과 guardian의 동시 SIGKILL에서는 두 subreaper 모두 없어지고 앱이 죽으면 pidfd를 닫는 것만으로 대상이 종료되지 않는다. 둘째 사용자 공간 watchdog만 추가해 동시 고장을 해결했다고 주장할 수 없다. kernel-owned delegated cgroup에 대해 사용자 session의 명시적인 scope·권한·종료 정책이 이미 제공될 때만 별도 검증 가능하다. cgroup.kill 존재 자체는 앱 프로세스 종료 시 자동 호출을 보장하지 않는다. 관리자/global cgroup/security 변경 대안은 이번 범위에서 제외한다.

현재 read-only 관측: /proc/self/status CapEff=0, NoNewPrivs=1, Seccomp=2; /proc/self/cgroup는 0::/이고 /sys/fs/cgroup의 cgroup2 mount는 ro다. 이는 현재 sandbox view이며 GUI 실행 환경의 위임 권한을 증명하지 않는다. cgroup 생성·systemd 호출·prctl·pidfd syscall 실험·signal·프로세스 생성은 수행하지 않았다.

## 인수 조건: 후속 구현·실험 필요

실제 guardian SIGKILL/fatal error에서 즉시 fork·setsid·출력 pipe holder·중첩 새 자손·동시 두 job·정상 다른 job·non-job sentinel을 시험한다. exact owned descendants 종료/이름별 회수, 무관한 sentinel 생존, bounded worker 반환, ready 전 고장, PID 재사용·빠른 exit와 동시 wait, registry 상한/관측 실패, 새 job 거절 및 partial 공개 금지, 종료 ack를 확인해야 한다. app 동시 SIGKILL 시험은 별도 실패 게이트로 유지한다. 기존 긴 soak를 이 설계 완료로 확대하지 않는다.

## 검증과 handoff

파일 읽기 및 /proc 상태·mount 정보만 조회했다. root에 안전한 단일-job 귀속 불가능성과 fail-closed 전체 job 정리 후보를 전달했다. 제품 source/Cargo 변경·실험 process·기존 soak 변경은 없다. 구현 승인 전 기존 guardian SIGKILL 출시 게이트는 열린 상태로 유지한다.

Linux primary 문서: [subreaper](https://man7.org/linux/man-pages/man2/PR_SET_CHILD_SUBREAPER.2const.html), [pidfd identity와 unreaped child 조건](https://man7.org/linux/man-pages/man2/pidfd_open.2.html), [thread children 관측 한계](https://man7.org/linux/man-pages/man5/proc_tid_children.5.html), [cgroup v2 위임과 cgroup.kill](https://docs.kernel.org/admin-guide/cgroup-v2.html). 링크를 읽고 API 계약을 검토했으며 실제 syscall 성공을 실행 증거로 주장하지 않는다.

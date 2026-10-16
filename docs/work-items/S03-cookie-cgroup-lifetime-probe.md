# S03: 정상 종료 뒤 cgroup pinned 파일 수명 실험

- 상태: completed (실험 관측 완료, receipt proof 불가 확인)
- 분류: native-experiment
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: stress_recovery; 신규 scripts/cookie_cgroup_lifetime_probe.py 및 이 문서, 전용 신규 evidence
- 선행 작업: launcher native integration preflight 및 reader_cgroup pin

## 사용자 결과

제품과 같은 AddRef=yes transient userunit에서 정상 종료 뒤 pinned dir/events/procs FD의 실제 읽기 가능성을 확인해 receipt 설계의 systemd cgroup 조기삭제 제약을 판단한다.

## 계약과 범위

명시 owned user bus에서 random dlpad-cgroup-lifetime unit 하나만 생성한다. 고정 /usr/bin/python3 sleep2 argv이며 runtime3초 상한, 외부 network/GUI/자격증명 없음. sd-bus 연결을 유지하고 정상 terminal 관측 후 동일 Invocation Unit.Unref로 해제한다. 다른 unit 검색/신호/StopUnit/설정/관리자/Cargo/Rust 변경은 없다. start 실패는 fallback 없이 기록한다. active PID/start/pidfd/membership와 cgroup2 mount/dev/inode를 확인하고 nofollow dir/events/procs를 pin한다.

## 인수 조건

정상 종료 전후 held FD read와 pinned dir 새 openat의 결과를 별도 기록한다. path absence를 empty로 인정하지 않는다. events4096/procs65536/기록256KiB/관측5초 상한, source SHA 및 owned PID 종료/unitGC/temp/FDbaseline 확인. empty 읽기 불가도 유효한 실험 관측이며 receipt 기준을 완화하지 않는다.

## 검증과 handoff

작성 중. 실제 worker는2초 뒤 정상 종료하며 최대3초 runtime으로 제한한다. root가 cgroup 삭제 시점과 reader_cgroup 현재 새 openat 방식의 결과를 대조해 설계를 결정한다. 본 실험은 실제 cookie/launcher gate 또는 crash recovery 구현 완료가 아니다.

## 실제 실행 결과

`python3 scripts/cookie_cgroup_lifetime_probe.py /mnt/d/project/dlpad/docs/validation/evidence/cookie-cgroup-lifetime-first` 실제 exit0. AddRef=true/RemainAfterExit=false, 같은 Invocation과 loaded inactive/dead 상태에서 ControlGroup은 빈 문자열이고 실제 cgroup path가 이미 삭제됐다. active 동안 고정한 events/procs FD는 모두 ENODEV(19), pinned dir에서 새 openat는 모두 ENOENT(2)였다.100ms 뒤 동일했다. held directory fstat의 dev24/inode16153/mount121은 남아 있지만 읽을 수 있는 empty 증거는 아니다. exact worker PID539449/start identity 부재·pidfd ready, same bus Unref 후 owned unit LoadState=not-found, FD4→4를 확인했다.

Root 제안에 따라 별도 소유 unit 속성만 바꾼 `python3 scripts/cookie_cgroup_lifetime_probe.py /mnt/d/project/dlpad/docs/validation/evidence/cookie-cgroup-lifetime-remain-first --remain-after-exit`도 실제 exit0이다. AddRef=true/RemainAfterExit=true, 같은 Invocation과 loaded active(exited),MainPID0인데 ControlGroup 빈 값/cgroup path 삭제/heldFD ENODEV/newopenat ENOENT가 동일하게 관측됐다.100ms 뒤에도 읽기 불가였다. saved worker PID/start 부재와 pidfd ready를 확인하고 동일 Invocation owned StopUnit→inactive/dead→same bus Unref→unit not-found 순서로 정리했다. FD4→4이며 외부 unit/프로세스에 신호를 보내지 않았다.

두 exit0와 passed=true는 관측/소유 자원 정리 완료이며 receipt empty proof 성공을 뜻하지 않는다. 두 결과의 held_empty/new_openat_empty는 모두 false다. 현재 Linux/WSL 사용자 manager에서 AddRef와 RemainAfterExit는 커널 cgroup 파일 수명을 보장하지 않는다. 부재를 empty로 인정하거나 receipt 기준을 완화하지 않았다. 현재 엄격 proof라면 이 경로는 receipt 부재·쿠키 보존이어야 한다. 새 실험은 다른 OS/manager 버전/실제 쿠키 B gate/OS crash/escaped reader 검증을 대신하지 않는다.

최종 script SHA `b32e9c4020d60d05e7add28a2629353622db99ab093b3fdb5495b8a71d1ed98c`로 동결 인계한다. 첫 실행과 true 옵션 추가 뒤 둘째 실행의 source SHA는 각 result.json에 따로 보존했다. Python AST parse와 두 실제 argv 실행을 수행했고 Rust/Cargo/기존 probe/manager 설정/기존8h/사용자 desktop 변경은 없다. 후속 설계 결정은 root 담당이다.

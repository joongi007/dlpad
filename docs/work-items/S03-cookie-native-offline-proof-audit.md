# S03: Cookie native offlined cgroup 증명 읽기 감사

- 상태: completed (읽기 감사; 대안 채택 아님)
- 분류: architecture
- 우선순위: P0
- 요구사항: R08, R10
- 담당/수정 범위: soak_recovery_audit — 이 문서만
- 선행 작업: root service cgroup directory/events/procs FD pin 실험

## 사용자 결과

고정 native cgroup FD의 ENODEV가 안전한 cookie reader 종료 증명에 포함될 수 있는지 공식 Linux v6.18 source와 반례로 검토한다. 기존 cleanup receipt 기준을 완화하지 않는다.

## 계약과 범위

Linux 공식 문서 및 Torvalds 공식 Git의 v6.18 source만 읽으며 제품 Rust/scripts·manager 실행·설정 변경은 없다. 현재 WSL kernel은 6.18.40.1이고 vanilla v6.18 검토가 WSL patch 동등성까지 증명하지 않는다. Path absence/foreign unit/GC는 단독 proof가 아니다. Native directory pin·InvocationID·pidfd·kernfs offline 조합은 Proposed 대안으로만 평가한다.

## 인수 조건

- cgroup destroy의 populated/child EBUSY 조건과 dying task 처리 차이를 source로 확인한다.
- Held kernfs FD ENODEV·fstat identity/link count와 namespace 반례를 구분한다.
- Original cgroup 제거와 모든 reader 종료 사이 충분성/불충분성을 명시한다.

## 검증과 handoff

## 결론과 반증

현재 관측인 `events/procs FD ENODEV + directory FD native identity 동일`은 원래 cgroup 제거의 **후보 증거**다. 이것만으로 cleanup receipt를 승인할 수 없다. 기존 `populated=0 + procs empty` 및 소유 identity/종료 확인 계약을 유지한다. 아래 조건을 추가로 증명할 수 있다면 original cgroup의 불가역 제거를 별도 Proposed proof로 검토할 수 있으나 이번 감사는 채택하거나 제품에 구현하지 않는다.

v6.18 `cgroup_destroy_locked()`는 cgroup mutex를 잡은 상태에서 populated와 online children을 각각 검사하여 EBUSY로 거절한다. 통과하면 online 상태를 내리고 추가 migration/child creation을 막은 뒤 directory를 제거한다. 따라서 **이 경로로 실제 제거된 original non-root cgroup**은 제거 시 살아 있는 구성원이 없었다는 커널 측 근거가 된다. 죽어가는 child와 controller references는 남을 수 있으며 이름 재생성도 가능하다. 객체 최종 해제나 모든 PID reap 완료를 뜻하지 않는다. [공식 v6.18 cgroup.c, 5758–5847](https://raw.githubusercontent.com/torvalds/linux/v6.18/kernel/cgroup/cgroup.c)

`kernfs_seq_start()`의 ENODEV는 active reference 획득 실패에서 나오지만 custom seq operation도 같은 errno를 반환할 수 있다. 따라서 syscall errno만으로 내부 원인을 식별할 수 없다. EOF나 이미 버퍼된 read 결과도 새 active 상태 판정이 아니므로 bounded rewind/read 실제 실패를 구분해야 한다. [공식 v6.18 kernfs/file.c, 125–174](https://raw.githubusercontent.com/torvalds/linux/v6.18/fs/kernfs/file.c)

`__kernfs_remove()`는 subtree를 deactivation한 뒤 unlink한다. 반면 `kernfs_show(false)`도 파일을 deactivation하지만 나중에 다시 활성화할 수 있다. 즉 일반 kernfs file의 ENODEV는 불가역 제거와 동의어가 아니다. 이 hide API는 directory에는 허용되지 않는다. [공식 v6.18 kernfs/dir.c, 1353–1464](https://raw.githubusercontent.com/torvalds/linux/v6.18/fs/kernfs/dir.c)

core `cgroup.events`와 `cgroup.procs`는 v6.18 base interface이며 procs의 threaded-subtree 거절은 EOPNOTSUPP다. core 두 FD가 함께 실패했다는 점은 일반 controller file 하나의 실패보다 강하지만, `cgroup_file_show()`라는 hide 경로도 존재하므로 모든 caller/WSL patch에서 core files의 hide/replacement가 불가능하다는 검토는 아직 완료되지 않았다. 이 문서는 hide를 실제 core-file 반례가 실행되었다고 주장하지 않는다. [공식 v6.18 cgroup.c, 4420–4431 및 4944–5131](https://raw.githubusercontent.com/torvalds/linux/v6.18/kernel/cgroup/cgroup.c)

held inode는 kernfs node reference를 보유한다. getattr는 directory link count를 `subdirs + 2`로 갱신한다. 따라서 dev/inode identity 유지와 nlink=2는 제거 후에도 가능하며, POSIX 일반파일의 nlink=0 판정을 이 filesystem에 가져오면 안 된다. Native identity는 **같은 객체를 계속 보고 있음**을 확인하는 데 쓰고 alive/dead 자체의 판정으로 쓰지 않는다. [공식 v6.18 kernfs/inode.c, 157–195](https://raw.githubusercontent.com/torvalds/linux/v6.18/fs/kernfs/inode.c)

## Reader 종료까지의 충분성 경계

공식 문서는 zombies가 procs에 표시되지 않고 directory 제거를 막지 않는다고 설명한다. cgroup membership 이동도 허용되므로 원래 cgroup이 비거나 제거되어도 이미 다른 cgroup으로 이동한 자손 reader가 살아 있을 수 있다. 알려진 guardian/worker pidfd 종료는 미관측 자손까지 증명하지 않는다. cgroup namespace/mount view가 달라지거나 동일 path가 재생성된 경우 path 부재/foreign unit/GC는 original FD 증명의 대체물이 아니다. [공식 v6.18 cgroup v2 문서](https://docs.kernel.org/6.18/admin-guide/cgroup-v2.html)

v6.18 `do_exit()`에서는 file table 정리가 `cgroup_exit()` 및 `exit_notify()`보다 먼저다. 따라서 zombie가 남는다는 이유만으로 그 task가 cookie 파일을 계속 읽는다고 추정해서도 안 된다. 다만 공유 file table의 다른 live task, 다른 cgroup으로 이전된 reader, 독립적으로 전달된 descriptor는 별도로 배제해야 한다. [공식 v6.18 kernel/exit.c, 900–924](https://raw.githubusercontent.com/torvalds/linux/v6.18/kernel/exit.c)

Proposed 충분 조건은 다음을 **모두** 입증하는 것이다.

- 기동 시 같은 InvocationID/소유 MainPID-start와 연결된 non-root cgroup2 directory를 pin하고, 그 directory에서 core 두 파일을 연다. 서비스 교체 후 새 path를 재조회한 결과를 원래 FD 결과와 섞지 않는다.
- mount/cgroup namespace 및 cgroup2 filesystem provenance를 보존하고, fd 재사용/close/reopen/잘못된 전달이 없음을 확인한다. 같은 fd 번호나 dev/inode 값만으로 이를 대신하지 않는다.
- held core FD의 bounded fresh read ENODEV가 해당 커널의 original cgroup 제거 경로에만 해당함을 source/capability 시험으로 입증한다. 일반 hide, 파일 교체, 임의 errno, view 변경을 제거로 간주하지 않는다.
- 모든 cookie reader가 original subtree에 계속 포함되는 계약을 별도로 보장하고, 알려진 process pidfd 종료·pipe EOF·동일 InvocationID ownership을 확인한다. 서비스 외부 migration/FD 전달 가능성을 배제하지 못하면 reader cleanup은 deferred로 유지한다.

이 조건하의 **실제 불가역 original cgroup 제거**는 이후 그 객체로 살아 있는 task가 들어올 수 없다는 추가 강점이 있다. 그러나 이번 관측 조합만으로 세 번째·네 번째 조건이 증명되지는 않았다. 현재 WSL 6.18.40.1 배포 source/config와 vanilla v6.18의 대응도 미검증이다. 이전 커널이나 이후 커널에 같은 판정을 자동 적용하지 않는다.

## 실행과 미검증

- 공식 primary browsing만 수행했다. kernel.org Git 웹 endpoint가 접근 실패하여 동일 프로젝트의 Torvalds 공식 Git v6.18 raw source를 사용했다.
- 제품 Rust/scripts, Cargo, 기존 cleanup receipt, manager/unit 및 시스템 설정을 변경하지 않았다.
- 실제 ENODEV/FD 동일성은 root 실험 관측으로 제공받았으며 이 감사에서 재실행하지 않았다. 새 runtime PASS 주장이 없다.
- 남은 결정: WSL exact source 대응 및 core hide/replacement caller 전수 검토, subtree containment 보장, FD carrier provenance 실제 인수 후 별도 ADR에서 채택 여부를 판단한다.

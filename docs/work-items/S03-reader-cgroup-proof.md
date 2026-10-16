# S03: 같은 boot reader cgroup의 pinned empty 증거

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R10
- 담당/수정 범위: stress_recovery; 신규 crates/engine/src/process/reader_cgroup.rs 및 이 문서
- 선행 작업: root reader typed protocol 계약

## 사용자 결과

쿠키 reader의 종료 증거에 사용할 cgroup을 native FD로 고정하고 cgroup2 및 mount/dev/inode identity와 실제 empty 상태를 확인한다.

## 계약과 범위

채택 interface는 pub(crate) PinnedCgroup::open(&Path), identity()->&CgroupIdentity, confirm_empty()->io::Result<()>이다. identity 필드는 mount_id/device/inode 및 /로 시작하는 canonical group relative_path이다. /sys/fs/cgroup root 자체는 거절한다. 각 경로 component openat nofollow/directory/cloexec, cgroup2 statfs, statx mount ID 확인. events 4096 bytes/procs65536 bytes 상한과 1초 읽기 deadline. populated0 및 정확히 빈 procs가 모두 필요하다. namespace 및 현재 path binding은 root caller가 별도 확인한다. 삭제/신호/shell/새 dependencies/기존 모듈 수정은 없다.

## 인수 조건

foreign filesystem, dot/symlink/noncanonical/oversize path, native identity 변화, missing/duplicate/invalid populated, partial 또는 oversize events, nonempty procs를 fail-closed 처리한다. parser unit과 실제 임시 foreignFS fixture를 제공하며 actual cgroup 생성 및 native 실행은 root 담당이다.

## 검증과 handoff

작성 중. Cargo 실행 및 module wiring은 root가 담당한다. 소유 source rustfmt 후 freeze 인계하며 scoped parser 시험은 실제 manager/boot/쿠키 전체 검증을 대신하지 않는다.

Root Cargo 연결에서 magic 비교 as _ 타입 추론 오류2개를 발견했고 양쪽 u64 비교로 수정했다. 기본26개 시험 PASS. 명시 ignored native 시험은 현재 소유 시험 프로세스 group을 읽기만 하며 실제 nonempty group을 empty로 승인하지 않아야 한다. 새 cgroup 생성·외부 group 제어는 없다. actual unit cleanup/gate는 별도다.

# S03: 사용자 데이터 디렉터리 단일 인스턴스

- 상태: implemented, 통합 검증 대기
- 분류: feature
- 우선순위: P0
- 요구사항: R08
- 담당/수정 범위: library agent — crates/engine/src/instance.rs, 이 문서
- 선행 작업: S01 데이터 디렉터리 확정, S02 supervisor; 격리 adapter 준비

## 사용자 결과

같은 앱 전용 데이터 폴더에 두 번째 앱이 시작하면 명시적 AlreadyRunning으로 거절한다. 첫 앱이 종료하면 OS 파일 잠금을 해제하고 다음 앱이 시작한다. stale PID를 읽거나 다른 프로세스를 종료하지 않는다.

## 계약과 범위

ARCHITECTURE, ADR-0002를 따른다. InstanceGuard::acquire는 canonical root와 owner marker를 확인하며 symlink/path traversal과 잘못된 marker를 거절한다. 생성한 marker는 고정 문자열이며 기존 marker를 덮어쓰지 않는다. marker가 없는 기존 data directory에 marker만 추가하므로 caller가 앱 전용 data root를 전달해야 한다. fs2 try_lock_exclusive의 file handle을 guard에 보존한다. Drop은 unlock하며 lockfile을 삭제하지 않는다. marker/lockfile 외 기존 데이터를 수정·삭제하지 않는다. lockfile 삭제 시 다른 inode의 동시 잠금이 가능하므로 cleanup 대상으로 삼지 않는다.

Second instance의 기존 창 활성화 IPC, OS 전체 single-instance, production app-data 조회와 crash recovery는 후속이다. 서로 다른 데이터 디렉터리는 독립 인스턴스를 허용한다. lock은 로컬 filesystem OS 동작을 전제로 하며 network filesystem 동작은 미검증이다. 악의적인 동시 symlink 치환을 막는 capability openat 구현은 후속이며 root는 사용자 전용 권한이어야 한다.

## 인수 조건

임시 데이터 폴더에서 중복 lock 거절, guard drop 뒤 재획득, 영구 lockfile 유지, 기존 잘못된 marker 보존, symlink lock 대상 synthetic media 보존. 두 file handle 시험은 process 간 OS lock의 단위 시험이며 별도 process와 Windows/macOS 실증으로 대체하지 않는다.

## 검증과 handoff

rustfmt 성공. Cargo는 주 담당이 fs2 0.4 manifest/lock 통합 뒤 실행한다. OS별 GUI 두 번째 실행과 실제 crash recovery는 미검증. fs2는 OS lock adapter 비용만 추가하며 binary 증가량은 통합 build에서 측정한다. App는 guard를 모든 worker 종료까지 유지해야 한다.

통합 검토 보완: OS `/tmp`·`/var`와 명시적 선택 ancestor alias는 canonicalization 전에 허용하되 선택된 최종 root symlink와 `..`는 거절한다. 소유 canonical root 아래 marker/lock/install 경로의 symlink 검사는 유지한다. 임시 alias-parent 성공과 final root symlink 거절 regression 테스트를 추가했다. Tool root의 nonempty adoption 거절과 기존 파일 보존은 유지한다. 실제 macOS 실행 검증을 뜻하지 않는다.

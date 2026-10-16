# S04: Managed executable lifetime and crash ownership

- 상태: proposed
- 분류: safety-contract
- 우선순위: P0
- 요구사항: R03, R05, R08, R13
- 담당/수정 범위: diagnostic_cache; 이 문서만
- 선행 작업: root ExecutionLease 정상 scope shared-lock foundation; guardian capability 후속

## 사용자 결과

실행 중인 앱 관리 도구의 파일은 다른 인스턴스나 관리 작업이 제거하지 못한다. 앱 crash/terminal 확인 실패에서도 live reader가 있을 수 있으면 제거를 보류한다. 정상 scope lock만으로 crash 보호 완료를 주장하지 않는다.

## 현재 읽기 근거

`tools/direct_download.rs:17–23` TransferLock Drop은 명시 unlock한다. Unix flock duplicated/inherited same file description은 unlock을 공유하므로 이 타입을 실행 lease로 재사용하지 않는다. `process/guardian.rs:192–225`는 현재 명시 소유 descriptor inheritance와 CLOEXEC 제어를 수행한다. 서비스 launcher/guardian으로 자식 정리 권한을 넘기고 원 control의 terminal frame을 확인한다. `process.rs:1026–1069`는 launcher reap/drain 종료와 terminal acknowledgement를 분리한다. `process/windows_guardian_entry.rs:89–107`는 허용된 원 handles를 받으며 이후 implicit inheritance를 차단한다. Windows Job 전체 종료·원 Job/process handle 증거는 PID 문자열과 구분한다. `download.rs:49–54,68–73`는 cleanup 미확인 오류를 SessionCleanupDeferred로 보존한다.

## 공유 계약 제안

1. **정상 foundation**: ExecutionLease는 관리 root의 검증된 원 lock File에 shared lock을 획득하고 원 file/root native identity를 유지한다. remove/install/select mutation은 같은 원 lock의 exclusive acquisition이 실패하면 작업을 거절한다. lease Drop은 close-only; 명시 unlock 금지. 외부 selected executable은 앱 관리 lease/삭제 권한 대상이 아니다. path prefix만으로 managed 판정하지 말고 원 root marker/native identity + package manifest/component identity/hash를 확인한다.
2. **reader scope**: 원 typed options에서 선택된 managed yt-dlp/FFmpeg/ffprobe/Deno 각각을 metadata/version 및 download 시작 전에 lease로 보호한다. 같은 root shared lease를 공동 유지할 수 있으나 도구 component별 original pin 검증은 생략하지 않는다. 재개는 저장된 원 snapshot 경로를 다시 검증하며 현재 UI 편집값/다른 active로 조용히 변경하지 않는다.
3. **Linux crash lifetime**: 검증된 원 lock descriptor를 기존 launcher의 explicit capability 목록에 별도 전달한다. guardian이 실제 child launch 이전부터 owned descendant cleanup+terminal EOF proof 이후까지 원 shared lock을 유지해야 한다. app parent가 죽어도 guardian alias가 살아 있으므로 close-only parent drop이 락을 풀지 않는다. 서비스 경로는 launcher exit 이후 실제 서비스 keeper에 original capability가 이동했음을 확인해야 한다. 임의 fd number/환경변수만으로 ownership를 주장하지 않는다. descriptor 전달 실패는 launch failclosed이고 original files는 보존한다.
4. **Windows crash lifetime**: Linux flock inheritance 의미를 Windows LockFileEx에 그대로 적용하면 안 된다. 기존 process creation allowlist에 원 managed lease handle을 전달할 수 있으나 락의 original locking-process 종료 이후 유지 의미를 실제 native 시험해야 한다. 보장 불가능하면 guardian 자신이 검증된 원 root lock에 shared lock을 획득하고 ready ack를 보내기 전 tool launch를 금지한다. parent가 unlock할 때까지 handoff 빈 구간이 없어야 한다. Job은 프로세스 cleanup authority이며 package 파일 삭제 authority가 아니다.
5. **TerminalUnconfirmed**: 정상 Rust scope가 반환되어 lease가 닫힌다고 reader 종료로 해석하지 않는다. guardian이 lease를 유지하는 경로 또는 durable original capability evidence가 없으면 failclosed 보호 gate를 남겨 remove/install mutation을 보류해야 한다. `mem::forget`에 의한 parent-process-only 락 유지도 crash 후 해결책이 아니다.
6. **durable 대안**: create-only marker에 root/lock/package original native IDs, nonce, guardian process creation identity, original receipt identity를 저장할 수 있으나 self-claimed IDs/PID/이전 persistedResolved/동일 boot boolean만으로 gate를 지우지 않는다. 원 lock·ACL/linkcount·namespace pins 및 실제 terminal receipts가 모두 일치할 때만 cleanup을 허용한다. 증거 누락/partial write/foreign/symlink/priorboot authority 미확인은 보존한다. 새로운 schema·receipt protocol은 root 조정 후 별도 수직 기능으로 구현한다.

## 담당 파일 및 의존성

root 조정 대상: tools 실행 lease 신규 모듈 및 native_files, remove/install/select lock 연결; process/guardian.rs와 service keeper FD handoff; windows_guardian_config/entry/capture inherited capability; download.rs의 owned session/SessionCleanupDeferred 연결; UI catalog failclosed 안내. 동일 파일 동시 수정 금지. guardian config/schema와 Cargo는 주 담당 소유. 이 읽기 감사는 해당 Rust 파일을 수정하지 않는다.

## 인수 조건

- 정상 실행 중 별도 owned process의 remove exclusive lock 거절; 종료 acknowledgement 후 허용.
- 원 FD duplicate 유지 시 parent lease Drop 후 exclusive acquisition은 계속 거절; TransferLock처럼 명시 unlock하면 실패하는 회귀 대조.
- app SIGKILL 뒤 live synthetic child/guardian이 원 파일을 읽는 동안 관리 remove 거절; owned guardian terminal cleanup 후만 허용.
- Windows 실제 caller process kill/Job descendant/handle close와 shared lock handoff를 native fixture로 검증; mock Linux pass를 Windows 완료로 확대하지 않음.
- terminal ack 손상/누락, launch handoff 실패, guardian crash, service launcher 조기 exit, wrong native ID/foreign marker는 파일 보존·gate 유지.
- 외부 source/sentinel/사용자 executable과 다른 package bytes 보존. Cancel/Pause/timeout 및 동일 saved snapshot resume에서도 보호 scope 유지.

## 검증과 handoff

읽기 감사 완료. 제품 코드·Cargo·프로세스 실행 변경 없음. 정상 scope foundation과 guardian/crash proof를 별도 gate로 구분하여 주 담당에게 전달한다. 위 proposed 구조는 실제 native 실험 후 채택/수정하며 현재 crash 보호 구현 완료가 아니다.

### Explicit process capability draft

`/tmp/dlpad-managed-execution-capability.rs`에 typed ExecutionCapabilities와 원 File의 close-only Arc clone, direct Linux command explicit FD allowlist/pre_exec CLOEXEC 제어/parent spawn pins, readiness 이전 lease holder 및 Confirmed/Deferred 소유 반환 초안을 작성했다. `Deferred { lease, error }`를 단순 drop하지 않고 keeper/reconciliation authority로 넘겨야 하며 임의 terminal 오류를 confirmed로 바꾸지 않는다. 아직 프로젝트 compile·실행을 하지 않았다.

실제 user_service.rs:1297–1326은 원 control을 StandardInputFileDescriptor로 systemd에 넘긴 뒤 launcher alias를 닫는다. 서비스는 같은 Unix control을 이미 갖는다. 가장 작은 정합안은 해당 검증된 원 socket에서 versioned bounded ancillary frame으로 SCM_RIGHTS lease 1개를 전달하고 서비스가 original lock ID를 대조·CLOEXEC·shared lock 보유를 확인한 ready acknowledgement를 보내는 방식이다. payload에는 filename/authority 환경변수를 넣지 않는다. CMSG truncation/extra FD/alias/잘못된 file type를 거절하고 받은 미사용 FD는 닫는다. 기존 cookie capability framing과 충돌하지 않도록 control frame 순서를 root가 통합해야 한다. 상호 handshake 완료 전 tool을 시작하지 않는다. 이 경로는 pathname 재개방/guardian ownlock 전략보다 원 open-description 권한을 유지하며 기존 Unix control을 재사용한다. systemd FD property 추가와 새 arbitrary filesystem 권한을 피한다.

Windows는 원 lock 공유 의미가 Linux와 다르므로 별도 guardian 자체 shared-lock 획득/원 identity 대조 후 ready handoff를 유지한다. typed config/allowlisted handle protocol은 root 담당이며 이 draft에 Windows 성공을 주장하지 않는다. root의 FD clone foundation native3tests PASS 전달은 정상 FD scope 증거이며 app crash/서비스 keeper 전체 proof로 확대하지 않는다.

# S02: launcher의 receipt 및 record FD 소유권

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: metrics_recovery — 신규 crates/engine/src/process/reader_receipt.rs 및 이 문서
- 선행 작업: fixed256 receipt codec, 제한된 writer FD 및 record flock 계약

## 사용자 결과

인증 reader의 종료 증거를 쓰는 launcher가 전달받은 앱 소유 파일만 사용하도록 제한한다. 이 독립 모듈은 아직 제품 호출부에 연결되지 않았으며 동일 부팅 복구의 완료를 의미하지 않는다.

## 계약과 범위

`ReceiptHandles::validate(writer: OwnedFd, lock: OwnedFd, &Capability, expected_record_dev, expected_record_ino)`는 FD를 소유 이관받아 즉시 CLOEXEC를 복구한다. 서로 다른 regular 파일, 0600, 현재 UID, 단일 hard link 및 정확한 native 식별자를 확인한다. writer는 O_WRONLY이며 receipt 식별자와 일치하고 lock은 journal이 사용하는 O_RDWR record 식별자와 일치해야 한다. nonblocking exclusive flock을 확인하며 동일 OFD 전달의 원본성은 호출자의 기존 record FD clone 전달 계약으로 보장한다. FD 검사만으로 동일 OFD의 역사적 원본성을 증명한다고 주장하지 않는다.

`write_receipt(&CleanupReceipt)`는 capability의 모든 필드가 일치할 때만 쓴다. 호출자는 그 전에 동일 소유 unit/Invocation과 고정한 cgroup의 종료 및 빈 상태를 독립 검증해야 한다. 이 모듈은 native 종료 증명을 생성하거나 검증하지 않는다. 256바이트 codec, 최대 32회 write 시도, 파일 sync를 사용한다. 부분 쓰기 실패 이후 같은 handles로 재시도하지 않는다. Drop은 FD만 닫으며 unlink하지 않는다. 오류와 Debug는 고정 문자열로 경로·토큰을 노출하지 않는다.

schema, Cargo, 공유 module 선언 및 제품 호출부는 root 담당이다. regular 파일 syscall과 fsync의 커널 정체까지 실시간 상한을 보장하지 않는다. 취소 및 protocol deadline은 외부 호출자가 제어한다.

## 인수 조건

임시 파일에서 접근권한·mode·hard link·receipt 및 record 식별자·동일 inode·capability 불일치를 거절한다. 정상 256바이트 저장 및 decode, 원본 record FD 종료 후에도 유지되는 flock과 마지막 handles 종료 후 해제, 부분 write 실패의 bytes 보존, Interrupted 최대 32회, CLOEXEC 복구를 시험한다. 실패 주입은 비공개 단위 시험 adapter만 사용하며 제품 환경 훅을 추가하지 않는다.

## 검증과 handoff

- `.local-tools/cargo/bin/rustfmt --edition 2024 crates/engine/src/process/reader_receipt.rs`: 실제 exit 0.
- 소스 SHA256: `985e7f7a608895a43f5c8b1147e7da832468722d4c65c2faf0bf9c9098f9bb77`.
- 단위 시험 4개 작성. Cargo 실행은 root 독점이므로 아직 실행 결과를 주장하지 않는다.
- 소스 동결 및 API를 root에게 전달했다. root의 module 연결·실제 Cargo 검증과 이후 native launcher 호출부 연결이 남았다.

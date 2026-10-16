# S02: 두 cookie reader의 순수 record 상태

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: metrics_recovery — 신규 cookies/reader_records.rs, 이 문서; root 요청으로 reader_protocol.rs identity_digest 보완
- 선행 작업: [reader codec](S02-cookie-reader-protocol.md), [gate 검토](S02-cookie-sameboot-reader-gate-review.md)

## 범위 및 인수

metadata0/download1 두 슬롯의 Empty/Intent/Bound/Resolved 전이와 version2 JSON encode/decode를 작성한다. intent token과 별도 native identity digest를 구분하고 exact capability/receipt fileID/nonce/Invocation/cgroup/digest를 검증한다. Resolved 재로드에는 실제 durable receipt를 caller가 다시 제공해야 한다. live owner lock과 anchored ownership 검증은 caller 책임이며 순수 cleanup_eligible로 파일 삭제하지 않는다.

record 모델 serialized4096B 상한과 최대 canonical path 두 개를 시험한다. 기존 cookies.rs의 완전한 cache/owner/record/marker schema에 통합한 전체 크기는 root가 추가로검증한다. 실제 fileI/O/durable commit/manager/credential를 실행하지 않는다. Cargo는 root 독점이다.

## 구현 source freeze

ReaderRecords는 begin(Capability), bind(ReaderIdentity), resolve(&CleanupReceipt), cleanup_eligible(), encode(), decode(bytes,[Option<&CleanupReceipt>;2]), slot()을 제공한다. metadata0의 valid receipt로 Resolved가 된 후 download1 Intent를 허용한다. 반복 begin/bind/resolve와 unresolved slot 덮어쓰기는 고정 오류다. Capability.digest는 intent token이고 별도 identity_digest는 full native binding이다. bind는 실제 canonical SHA256을 재계산해 path를 바꾼 채 제공 digest만 재사용하는 경우를 거절한다.

identity_digest의 canonical preimage는 `DLPAD-READER-IDENTITY-V1` + NUL domain, boot16/unit16/Invocation16, cgroup mount/dev/ino 각각LEu64, path byte countLEu16, exact UTF-8 path bytes다. intent token/receiptID는 별도 exact capability equality로 바인딩한다. identity_digest 함수는 아직 hash를 할당하지 않은 input을 받아도 나머지 identity를 검증하고 계산할 수 있다. native ownership 검증 없는 값의 hash 자체는 권한 증명이 아니다.

JSON v2는 고정 두 slots와 phase0/1/2/3, lowercase hex nonce/digest/boot/unit/Invocation, native 숫자 identity 및 canonical path다. unknownfields/잘못된version/zero nonce/oversize/truncated를 거절한다. v1을 v2로 자동 승격하거나 삭제가능으로 해석하지 않는다. Resolved reload는 caller가 독립 검증한 durable receipt를 다시 제공해야 하므로 JSON phase3만으로 cleanup_eligible을 만들지 않는다. pure resolved state도 anchored cookie/record/marker/fileID 및 live owner lock 확인을 대신하지 않는다.

4개 unit은 ordered two readers+reload missing proof, repeated transitions/path digest/capability/receipt mismatch, 두512B escaped path를 실제 JSON serialize하여4096B 이하확인과oversize, v1/partial/unknown/zero intent 보존을 검사한다. 실제 실행은 root 대기다. reader_records SHA256 `de97e663e2c8f5df09ddbde29471b6c50053300b8c6478d47da049998dfdad6c`; 수정된 reader_protocol SHA256 `d31afe8dde1df70b7ddd1bc7ff56ffffdba275ceb8c14c84a8b891c3969ad940`. rustfmt exit0/Cargo 미실행. module 선언과 pub(crate) protocol 타입 visibility는 root가 조정한다.

완전한 기존 recovery record의 cache/owner/cookie/record/marker 필드까지 합친4096B 및 atomic commit은 아직 미검증이다. root는 두 최대Bound slot과 실제basefield를 결합한 encodedsize 시험과 durability/crash 실패 인수를 추가해야 한다. 현 pure JSON을 recovery record로 바로 채택하거나 anchored disk commit 완료로 보고하지 않는다.

# S02: bounded cookie reader protocol codec

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: metrics_recovery — 신규 crates/engine/src/process/reader_protocol.rs 및 이 문서
- 선행 작업: [same-boot gate 검토](S02-cookie-sameboot-reader-gate-review.md)

## 범위와 사용자 결과

I identity/B begin/capability/cleanup receipt를 credential 없는 bounded binary로 인코딩한다. root가 optional observer/FD 전달/record binding/gate를 연결하기 전 제품 동작은 바뀌지 않는다. ProcessSpec/Cargo/공유 process 파일은 수정하지 않는다. I/O/process/env를 사용하지 않고 새 dependency/thread를 추가하지 않는다.

## 계약과 인수 조건

versioned magic와 domain별 frame, little endian 정수,1024B packet/512B canonical cgroup path/고정256B cleanup receipt, index0/1 및 nonzero nonce/identity digest/boot/unit/invocation을 검증한다. partial decoder는 유한 buffer이고 domain mismatch/overflow/truncation/invalid field를 fixed enum 오류로 반환한다. Debug는 redacted다. valid roundtrip/partial fragments/truncation/oversize/path/zero/domain mismatch/receipt identity mismatch 시험을 작성한다. 실제 파일/record4096B/manager/cgroup 증명은 root 통합 담당이며 pure codec PASS로 대체하지 않는다.

## 검증

구현 작성 후 rustfmt만 수행하고 source freeze를 root에 전달한다. module은 아직 제품에 연결하지 않는다. Cargo 실행은 root 독점이며 자체 실행 금지다.

## source freeze와 인터페이스

신규 reader_protocol.rs만 작성했고 module은 제품에 연결하지 않았다. 헤더는 DLPG magic/version1/domain I(identity),B(begin),C(capability),P(cleanup proof)/LE u16 payload 길이다. capability89B, begin25B, identity147B+path(최대659B), cleanup receipt256B다. fixed receipt padding은 전부0이어야 한다. Domain/typed identity/Decoder Debug는 redacted이며 오류 enum Display는 고정 문구다.

encode/decode_identity, encode/decode_begin, encode/decode_capability, encode/decode_cleanup_receipt, CleanupReceipt::from_identity/matches, Decoder::new/push/finish를 제공한다. Decoder push는 정확 complete frame 하나만 반환하고 추가 데이터/domain mismatch/1024B 초과를 거절한다. 완료 전 finish는 Truncated다. path는 absolute UTF-8이며 root/slash빈component/dot/dotdot/control문자를 거절한다. Unicode와 공백을 정규화하거나 shell 처리하지 않는다.

4개 내부 시험은 roundtrip/literal 및 redacted Debug, 실제1byte partialdecoder와모든prefixtruncation/oversize, path/zero/index/version/domain/padding 실패, 다른nonce/Invocation/receiptfileID mismatch를 확인한다. 아직 실행하지 않았다. rustfmt exit0, source SHA256 `75bb5f1cf401f945a5b6b3b6fadfff016df983fbdc2db393a2b38dc9e22fca6a`. Cargo/root공유source 수정 없음.

codec는 nonzero digest를 전달받을 뿐 digest를 계산하거나 native proof를 수행하지 않는다. caller는canonical full identity/path/receiptID/nonce를 digest에묶어 검증해야 한다. receipt.matches는그digest와고정fields동일성만확인하므로 caller가path를변경해놓고digest를재계산하지않는경우를이API가탐지한다고주장하지않는다. cleanupproofkind1은caller의sameownedunit/Invocation/pinnedcgroupempty검증이선행해야하며tool이임의codec를호출하는것을권한증거로인정하지않는다. 실제receiptwriteFDallowlist/durability/fileID/record4096/nativecrash/nextstartup인수는root별도연결이다.

## root 독립 모듈 검증

Pinned Rust1.94 toolchain의 임시 부모 모듈 wrapper로 실제 컴파일·실행하여 4개 시험 PASS. [증거](../validation/evidence/reader-protocol-standalone/result.json). 제품 연결·Cargo 통합·native gate·crash recovery는 미완료다. 최초 standalone crate root 컴파일의 pub(super) 문맥 오류는 wrapper로 수정했다.

## identity digest 분리 보강

root는 Capability.digest를 intent token으로 유지하고 ReaderIdentity/CleanupReceipt에 별도 identity_digest32를 추가하도록 결정했다. codec는 별도 digest의nonzero와receipt fullmatch를검증한다. 현재 identity wire는179B+path(최대691B), receipt는256B그대로며encode시padding전overflow를명시검사한다. reader_records의bind가canonicalfullidentitySHA256을실제로재계산한다. codec단독은계산하지않으며path만바꾼채같은providedidentity_digest를재사용하면matches가통과한다는시험으로caller책임을명시했다. intent token을fullnative/pathdigest로표현한앞선설명은이번구분으로대체한다.

현재 source SHA256 `d31afe8dde1df70b7ddd1bc7ff56ffffdba275ceb8c14c84a8b891c3969ad940`, rustfmt exit0/Cargo미실행. 이전standalone결과는이전digest구조snapshot근거다. rootpub(crate)visibility/제품module연결/새standaloneunit실행대기다.

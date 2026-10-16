# S02: Windows reader 순수 기록 기반

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: metrics_recovery — 신규 crates/engine/src/cookies/windows_records.rs 및 이 문서만
- 선행 작업: root의 S02-windows-reader-record-contract; root module 연결 및 실제 Cargo/Windows 검증

## 사용자 결과

Windows startup 회수의 순수 두 reader 기록과 완료 receipt 기반을 구현한다. 제품 journal/guardian/public recovery는 이 단계에 연결하지 않으며 이전 부팅 권한을 만들지 않는다. Linux 기록은 변경하지 않는다.

## 계약과 범위

WindowsAnchor는 volume_serial과 nonzero128bit file_id다. SessionAnchors는 원래 cache/owner/copy/journal과 두 receipt의 native anchor를 고정한다. WindowsReaderRecord::new의 record_digest는 원 recovery record bytes의 digest이며 mutable 슬롯 JSON에서 새로 만들지 않는다. session nonce16/digest32는 nonzero다. slot은 Empty→Intent→Bound→Resolved이고 metadata0Resolved 후 download1만 허용한다. 두 슬롯 nonce는 서로 다르고 nonzero다.

BoundIdentity는 index/nonce/원 guardian 및 tool creationFILETIME/identity_digest다. SHA256 domain과 sessionnonce/originaldigest/여섯 anchor/slot/nonce/두 creation 값을 canonical little-endian으로 해시하고 parse에서 재계산한다. PID/JobHANDLE/boot필드/경로는 없다. creation값은 원 volatile handle 검증의 실행시 evidence이며 startup 단독 삭제 권한이 아니다.

JSON은 unknown fields 거절, 최대4096B이고 내부 state validation을 거친다. CompletionReceipt는 정확256B, version/index/sessionnonce/originaldigest/bounddigest/COMPLETE magic 및 reserved0다. decode는 partial/oversize/domain/zero/reserved/mismatch를 거절한다. 모든 Debug와 오류는 고정 redacted 문구다.

## 공유 API

WindowsReaderRecord::new/parse/encode/begin/bind/resolve, 원래 anchors/session_nonce/record_digest/bound_identity getter를 제공한다. receipt_for는 기대 포맷 생성만 하며 native receipt 작성 권한을 부여하지 않는다. CompletionReceipt::encode/decode를 제공한다.

all_receipts_match(&[CompletionReceipt])는 실제 I/O 계층에서 별도로 읽고 원래 FileId/ACL/잠금을 검증한 receipt들을 받아 각 Bound/Resolved의 exactmatch를 확인한다. empty-only/Intent/누락/중복/foreign receipt는 false다. persisted Resolved 자체로 정리를 허용하는 무인자 API는 없다. resolve는 중복 전이를 거절하며 recovery의 반복 proof 검증은 all_receipts_match로 한다.

## 인수 조건

정상 두 reader JSON roundtrip과 정확 receipt를 검증한다. illegal order/reuse/동일nonce/foreigndigest가 상태를 변경하지 않는지 검사한다. zero anchor, unknown JSON,4097B,stored creation 변경으로 digest불일치,partialreceipt,reserved/domain/index/version 오류를 거절한다. Resolved라도 실제 receipt 없이 false이며 실제 정확 receipt가 있을 때만 true다. I/O provenance/ACL/FlushFileBuffers/guardian crash는 root 후속 수직 인수다.

## 검증과 handoff

초기 코드 준비는 /tmp에서 수행했고 root의 실제 timeout/cancel 시험 종료 및 source freeze 해제 신호 후 신규 파일만 게시한다. 개별 rustfmt 실제 exit0. 자체 Cargo/native 실행 없음. root가 module wiring/focused unit/nativeWindows compile을 수행한 후 실제 결과를 기록한다. 제품 회수 완료는 주장하지 않는다. 기존 serde/sha2만 사용하며 새 dependency/Cargo 변경은 없다. 두 슬롯, JSON4KiB, receipt256B의 상한을 유지한다.

## Root 실제 검증

첫 Linux focused4tests/fmt PASS, Clippy101 collapsible_if 원시는 windows-reader-records-linux-first에 보존했다. letchain수정 및 추가2개 parse실패시험을 반영한 SHA8616976b2eba109638bd9c2555f90bdb45b2e4b5d84ca93228f4552d59130bf9로 실제 Windows GNU native engine lib6tests PASS/0failed/0ignored/102filtered. Root가 records.log와 records.json의 exit0/privateDesktop/inputdesktopunchanged/sourcebeforeafter 및 현재입력SHA 일치를 독립 확인했다. 증거 windows-reader-records-native-first.

Linux 전체7검증+2도구버전은 windows-reader-records-workspace-second에서 진행하며 terminal만인정한다. 이 결과는 순수포맷기반이며 startup회수/guardian/실제receipt파일 권한 검증은 아직 미연결이다. 다음 journal 연결에는 durable원receiptFileId 두개를 추가해야하며 nativeI/O연결 전에 공유schema를 보강한다.

Root 후속 Linux 전체 검증 terminal: windows-reader-records-workspace-second/result.json statecompleted/7workspace+2toolversion 모두PASS, 입력 beforeafter+현재SHA 동일을 독립 확인했다. 이 전체검증은 receiptsanchor 추가 전 SHA861697... 범위다. 실제I/O 후속source는 별도 재검증한다.

후속receipts anchor스키마 및 journal연결의 Windows전체4검증은 windows-journal-default-tab-workspace-native-first/result.json에 보존했다. 기존6시험에 원receiptanchor변경거절도 포함하며 root는actual6PASS를확인했다. Linux통합은새prefix에서 terminal대조한다.

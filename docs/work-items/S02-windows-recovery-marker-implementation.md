# S02: Windows recovery marker 구현 초안

- 상태: native-foundation-verified; product integration pending
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: diagnostic_cache — /tmp/dlpad-windows-recovery-marker-draft.rs 및 이 문서만
- 선행 작업: S02-windows-public-recovery-contract, Windows native journal; 제품 연결은 root

## 사용자 결과

bounded immutable marker를 Windows engine 모듈에 통합하고 native 파일·ACL·원 journal 연결을 검증했다. 실제 public recovery는 미완료다.

## 구현 인터페이스

초안은 cookies/windows/recovery_marker.rs에 배치하는 형태다. `base_digest(nonce,cache,owner,copy)`는 fixed domain/version과 실제 원 IDs를 해시한다. `Marker::from_created(nonce,SessionAnchors,marker_id)`는 최종 여섯 original anchors와 marker own ID를 묶는다. digest 필드 자신은 해시에서 제외한다. `encode/decode/validate`는 deny_unknown_fields/4096bytes/nonce비영/원 ID비영·상호 distinct/base·payloaddigest 재계산을 검증한다. `read_marker`는 metadata 길이 확인 후 take(limit+1), 실제 길이 비교로 읽는다. Debug/오류는 고정 정제 문자열이다. 원본 경로·URL·쿠키 값은 저장하지 않는다.

`journal_seed()`는 원 SessionAnchors/nonce/baseDigest를 Journal::reopen에 넘긴다. `check_actual(sid,[cache,owner,copy,marker handles])`는 각 actual native FileID와 type/reparse/nlink/protected ACL을 원 기록과 비교한다. 이 함수는 일관성 검사일 뿐 삭제 권한·namespace 안전을 보증하지 않는다. DELETE API는 의도적으로 없다.

## 제품 import 순서

1. original ancestry/namespace pins와 private cache/owner/copy 실제 handles를 보유하고 copy를 sync한다.
2. CSPRNG session nonce와 실제 cache/owner/copy IDs로 baseDigest를 만든다.
3. Journal::create(baseDigest)를 호출해 actual journal·receipt 원 IDs와 lock을 보유한다.
4. protected SID descriptor로 marker CREATE_NEW, marker own actual ID 관측, final payload 생성·bounded write·sync·re-read 및 모든 actual identity 재검증.
5. 완성 marker 이후에만 initialized lease를 공개하고 reader intent를 허용한다.

Journal 생성 이후 실패에서 기존 cookies.txt-only cleanup을 호출하면 부분 파일 때문에 실패하거나 잘못된 범위를 추정한다. 초기 실패 RAII cleanup 권한과 journal 이후 deferred 보존을 명시적으로 나눠야 한다. CREATE_NEW 충돌 파일은 절대로 앱 소유로 채택하지 않는다.

## 미해결 증명·통합 사항

- Windows OwnedCopy는 현재 journal/marker를 보유하지 않는다. root가 retained original handles/seed/facade를 연결하고 strict allowlist를 다섯 entries로 전환해야 한다. 현재 windows journal 내부 receipt handles는 private이다. 초안을 네 handles로 수정했다. Journal::reopen은 journal_seed의 원 IDs로 journal 및 두 receipt actual identity를 독립 검증한다. 현재 관측 IDs를 원 IDs로 대체하는 연결은 금지한다.
- startup에서 marker의 selfclaimed original IDs를 actual 파일과 비교하는 것만으로 foreign 사용자 파일 삭제 권한이 생기지 않는다. importer가 실제 만든 durable original 기록과 private ownership 체인, 현재 namespace pins, strict allowlist, actual guardian receipt provenance·lock이 함께 필요하다. 동일 사용자 완전 위조를 SHA256만으로 방어한다고 주장하지 않는다. 강화된 retained DELETE handles와 namespace reopening 검증은 root 책임이다.
- all_receipts_match는 actual 최소 한 bound identity/모든 원 receipt를 요구한다. Empty/Intent/미완료 Bound/실제 receipt 없는 persisted Resolved는 보존한다. Windows 이전 boot authority가 없으므로 receipt 없는 priorboot 회수는 보존한다.
- marker CREATE_NEW 이전 crash, write 일부, sync 실패, marker완성 전 journal부분은 보존한다. marker자기ID/nlink/ACL 변경, original journal/receipt 교체는 거절한다. 파일 sync 성공을 Windows directory persistence 증명으로 확대하지 않는다.
- draft는 native API 부모 모듈 안에서 compile하지 않았다. encode/decode 순수 시험, 실제 NTFS native ACL/lock/receipt+product startup 삭제 시험은 root 통합 이후 필요하다.

## 인수 조건과 handoff

실제 생성한 서로 다른 FileIDs/nonce/digests의 roundtrip, unknown/oversize/NUL 없는 민감값 부재, digest 및 marker ownID 변경 거절을 시험한다. 실제 import 각 단계 crash·partial sync, 원 sentinel/foreign sibling·원 source 보존, reparse/hardlink/ACL·namespace 교체, 129+ scan128/truncated, lock busy, receipt 교체/한 개 누락, actual matching receipt 정상 회수, 삭제 부분 실패를 native fixture로 확인해야 한다. 단순 초안이나 synthetic 자기서술 기록을 product 회수 PASS로 쓰지 않는다.

실제 실행: current Windows OwnedCopy/journal/windows_records 읽기 감사, /tmp 단독 초안 작성, `rustfmt --edition 2024 /tmp/dlpad-windows-recovery-marker-draft.rs` exit0. 빌드·테스트·실제 삭제는 실행하지 않았다. root integration에 전달할 초안 작업 완료이며 P0 기능 완료가 아니다.

## 후속 native 시험 초안

`/tmp/dlpad-marker-tests-draft.rs`에 native TestFixture 기반 3개 시험 초안을 작성했다. 실제 private marker CREATE_NEW/write/sync/handle 검증을 실행하도록 설계하고 unknown/oversize/digest·nonce·ID tamper, marker namespace 실제 replacement와 원 marker sentinel 보존, owned marker ACL 실제 확대 후 거절·sentinel 보존을 시험한다. 제품 소스 및 TestFixture를 수정하지 않았으며 시험은 아직 compile/run하지 않았다. Windows-sys WRITE_DAC 위치는 기존 로컬 source에서 FileSystem 상수로 확인했다.

필요한 test-only bridge: 기존 TestFixture::create는 journal digest [2;32]를 사용하므로 marker 단독 consistency 시험에는 충분하지만 integrated Journal::reopen(baseDigest) 대조에는 부족하다. TestFixture::create_with_base_digest처럼 actual cache/owner/copy와 nonce로 baseDigest를 생성 전에 넣는 fixture API가 필요하다. production journal private receipt handles를 공개할 필요는 없다. Marker 테스트를 recovery_marker.rs 내부 cfg(test) module로 포함하면 private payload 변조와 windows 부모 helpers에 접근할 수 있다. 독립 module에 놓는다면 test-only 접근을 합의해야 한다. 파서 시험이나 matching 자기서술만으로 deletion authority PASS를 주장하지 않는다.

## Root 통합 범위

Root가 recovery_marker.rs와 Windows private module 연결 및 native 변조/교체/ACL 실패 3tests를 통합했다. 복구 marker consistency는 단독 삭제 권한이 아니다. 현재 fixture arbitrary digest는 production journal_seed와 다르므로 제품 import/recover와 integrated original journal reopen을 완료로 표시하지 않는다. Cargo/schema/새 의존성 변경 없음. 실제 native 검증 전 상태다.

## 실제 root native 검증

windows-cookie-marker-native-first에서 engine all-target Clippy 및 실제 marker3tests PASS. 후속 second에서 TestFixture baseDigest 생성/원 anchor journal reopen을 연결한 exact module4tests PASS/140 filtered, Clippy exit0. 실제 원 digest reopening 성공, 변조 digest 거절, receipt 없는 marker만으로 완료 증거 생성하지 않음을 확인했다. 두 operation source_unchanged/input_desktop_unchanged true이며 root가 raw log/JSON을 직접 확인했다. 실제 product import/recovery/삭제·appkill·전체 workspace 후속검증은 남는다. 새 Cargo dependency/schema 변경 없다.

# S02: Windows 제품 startup 회수 계약 읽기 감사

- 상태: done
- 분류: audit
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: diagnostic_cache — 이 신규 문서만
- 선행 작업: Windows retained-handle journal/guardian/WindowsReaderRecord, 주 담당 제품 wiring

## 사용자 결과

현재 Windows startup 회수는 아직 제품에 연결되지 않았다. 이 감사는 안전한 연결의 선행 계약과 실패 인수 조건을 제시하며 제품 회수 완료를 의미하지 않는다. Rust/Cargo/공유 source 및 실행 프로세스는 변경하지 않았다.

## 현재 소스 관측

- `cookies.rs::recover_sessions`는 Linux만 recovery를 호출하며 Windows는 Unavailable이다. `begin_gated_reader` 및 기존 reader journal facade도 Linux 경로다.
- `cookies/windows.rs::OwnedCopy`는 cache ancestry pins, owner/copy NativeFileIdentity 및 SID를 보유한다. `owner_valid`와 `validate_for_tool`는 실제 열린 handle과 현재 namespace의 identity, NTFS 고정 드라이브, reparse 부재, 파일 nlink1, protected 단일 current-SID ACL을 확인한다. 허용 owner entries는 현재 `cookies.txt` 하나뿐이다. 저널/receipt/immutable marker를 추가하면 이 allowlist와 cleanup을 함께 바꿔야 한다.
- Windows import는 copy를 CREATE_NEW로 생성·sync하고 실제 IDs를 보유하지만 journal/marker를 만들지 않는다. defer_cleanup은 삭제를 중단하고 handle/pins를 놓는다. 이름만 보고 재구성하는 복구 권한은 없다.
- `cookies/windows/journal.rs::Journal::create`는 session_nonce/record_digest/cache/copy를 외부에서 받는다. journal 및 두 receipt를 실제 생성한 뒤 SessionAnchors의 여섯 IDs를 구성한다. reopen은 caller가 ORIGINAL durable anchors를 전달해야 하며 관측한 현재 IDs를 원본이라고 대체하면 안 된다. actual handle identity/ACL 및 원 receipt pins를 검증하고 writer lock을 확보한다.
- WindowsReaderRecord identity_digest는 session_nonce, record_digest, 여섯 anchors, slot, reader nonce 및 guardian/tool creation time을 묶는다. `record_digest` 자체가 이 record의 자기해시라는 계약은 현재 없다. `all_receipts_match`는 receipt가 하나 이상이어야 하고 Intent를 거절한다. Bound/Resolved 모두 원본 actual receipt와 정확히 일치해야 한다. persisted Resolved만으로 권한을 만들지 않는다.
- Linux recovery는 immutable Record+Marker binding을 실제 handle IDs와 비교하고 bounded 128 candidate scan을 수행한다. same-boot는 실제 완료 receipt를 요구하며 별도 PreviousBootAuthority가 있다. Windows에 Linux boot 권한을 이식할 근거는 없다.

## 작성 순서 제안 — 아직 Proposed

### A: base digest와 최종 immutable marker 분리 (우선 실험 권고)

1. private cache/owner/copy 실제 handles와 원 IDs, CSPRNG session nonce를 확보한다. copy를 sync한다.
2. 명시 domain/version과 cache/owner/copy 원 IDs·session nonce로 base immutable digest를 계산한다. 원본 경로·URL·쿠키 값은 저장하지 않는다.
3. 기존 Journal::create의 record_digest에 base digest를 넣어 journal/두 receipt CREATE_NEW+sync를 수행하고 실제 원 IDs를 얻는다. Empty journal은 삭제 권한이 아니다.
4. immutable marker file을 CREATE_NEW로 열어 marker 자신의 실제 ID를 얻는다. canonical immutable payload(version/session/baseDigest/all6 originals/marker own ID)의 digest를 계산해 payload+digest를 bounded write+sync한다. digest는 자신의 저장 digest 필드를 제외한 domain-separated payload를 해시해 순환을 피한다.
5. marker를 검증한 뒤에만 reader intent를 허용한다. startup에서 baseDigest를 원 cache/owner/copy와 nonce로 재계산하고 marker payload/digest 및 marker 자신의 실제 ID와 journal 원 IDs를 모두 확인한다. Journal::reopen은 marker의 원 IDs/baseDigest를 받는다.

이 설계는 existing Journal API와 순환 없이 연결되지만 record_digest를 “최종 marker 전체 해시”라고 표현하면 안 된다. guardian identity_digest에는 baseDigest와 all6 anchors가 이미 포함되므로 receipt는 그 여섯 IDs를 묶는다. immutable marker는 최종 ownership binding이지 외부 파일에 대한 서명이나 사용자보다 강한 보안 경계가 아니다.

### B: initialized seed 생성 후 final digest bind

journal/receipt를 먼저 CREATE_NEW하고 외부 실행 불가능한 Uninitialized 상태로 보유한 뒤, 실제 IDs와 marker own ID로 final immutable payload digest를 결정하고 finalize API로 record_digest를 단 한 번 설정·sync한다. 이후에만 begin을 허용한다. 기존 Journal::create/new/validate는 nonzero digest와 initialized record를 전제하므로 별도 private builder/typestate 및 finalized 전 모든 handoff/reopen 거절이 필요하다. seed/finalize 사이 crash, 실패 rollback, digest 재설정 거절의 추가 인수 부담이 있다. 최종 digest라는 단일 명칭은 명료하지만 현재 API 수정량과 부분 상태가 A보다 크다.

둘 다 실제 root 실험·구현 후 채택한다. placeholder digest로 실행한 뒤 이미 bound reader digest를 바꾸는 방법은 금지한다.

## 삭제 권한과 startup wiring 인수 조건

- public recover_windows 경로와 startup worker 호출, summary scanned/recovered/deferred/rejected/truncated 구분이 필요하다. UI thread I/O 금지. 128 candidates+초과 한 개 확인으로 scan을 제한하고 전 entries를 메모리에 읽지 않는다.
- 이름·selfclaimed FileID·matching JSON/digest만으로 foreign 사용자 데이터를 삭제하지 않는다. import가 실제 보유했던 original marker/cache/owner/copy/journal/receipt IDs의 체인을 재검증하며 SID protected ACL, NTFS/reparse/nlink 조건, retained original handle/namespace pins, exact allowlist, exclusive journal lock을 함께 요구한다. 다른 SID/ACL/cache replacement는 보존한다. 동일 사용자에 의한 완전한 위조를 해시만으로 방어한다고 주장하지 않는다.
- 실제 원 receipt 모두 일치하고 최소 한 Bound/Resolved identity가 있어야 회수한다. Empty-only, Intent, unmatched Bound, persisted Resolved이나 실제 receipt 없음/교체는 보존한다. 순수 persisted same-boot boolean은 권한이 아니다. Windows prior-boot native authority가 없으므로 receipt 없는 prior-boot도 보존한다. 실제 receipt가 모두 일치하는 경우에도 원 ownership 검증은 생략하지 않는다.
- 삭제 직전 전체 identity/allowlist/ACL을 다시 검증하고 검증된 DELETE handles의 disposition으로만 삭제한다. foreign entry가 있거나 일부 삭제가 실패하면 원본/source/cache 외부 sentinel을 보존하고 정확한 부분 실패를 보고한다.
- crash before marker create/write/sync, partial marker, partial journal, receipt 없이 Intent, finalization 전 상태는 보존한다. 완성 marker 이후 guardian EOF/terminal receipt 사례만 독립 OS 시험으로 권한을 증명한다.

## 실패 경로 인수 목록

1. 실제 Windows product import→guardian terminal→앱 종료→startup recover로 원 copy만 회수; 원 source/foreign sentinel bytes·IDs 보존.
2. Empty/Intent/unresolved Bound/Resolved-without-actual-receipt, wrong nonce/baseDigest/slot/creation-time/digest, 한 receipt만 완료는 보존.
3. 원 file 교체·hardlink·junction/reparse·ACL 확대·다른 SID·cache/owner rename·marker own-ID 교체·journal lock busy·foreign sibling/receipt는 보존.
4. malformed/unknown-field/oversized marker·journal·receipt, partial write/sync 실패는 bounded read 후 거절·보존.
5. 129+ 후보 및 foreign 큰 디렉터리에서 scan128/truncated와 메모리 상한 확인; 외부 arbitrary nested 탐색 없음.
6. prior-boot receipt 없는 synthetic 기록은 보존; 실제 reboot 권한 실증 전 이를 완료로 표시하지 않는다.
7. 각 생성 단계 crash와 cleanup 실패 뒤 재시작, actual native guardian receipt writer 독점 및 원 file pin 확인.

## 검증과 handoff

실행한 것은 rg/sed/cat에 의한 현재 소스 읽기 감사뿐이다. 핵심 파일: cookies.rs, cookies/windows.rs, cookies/windows/journal.rs, cookies/windows_records.rs 및 Linux recovery Records/create/recover_one. Rust/Cargo 변경·빌드·native GUI·실제 삭제 시험은 실행하지 않았다. 문서 감사는 완료, 제품 wiring/immutable marker/native 회수 인수는 미완료다. root가 A/B 실험을 근거로 채택하고 전용 native 실패 경로를 통합 검증한다.

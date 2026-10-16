# S03: Windows journal 잠금 명시 인계

- 상태: in-progress
- 분류: fix
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: metrics_recovery — cookies/windows/journal.rs 및 이 문서만
- 선행 작업: 실제 복제handle 잠금 미유지3회; root Windows guardian gate 및 shared records 계약

## 사용자 결과

Windows lock 복제본이 부모 잠금을 유지하지 못한 실측에 따라, journal 앱 쓰기 권한과 guardian 잠금 권한을 명시적으로 구분한다. 제품 crash recovery 연결은 root 후속 통합이며 이 기반만으로 완료라고 보고하지 않는다.

## 계약과 범위

root 순서: durable Intent→native suspended prepare→app durable Bound→app unlock→guardian own lock ACK 및 originalBound 검증→Begin→Resume. helper lock 이후 app Bound 쓰기 방식은 사용하지 않는다. 기존 Linux 정책/schema/Cargo는 변경하지 않는다. 현재 root source freeze 해제 GO 전 구현을 게시하지 않는다.

locked bool은 잠금 권한을 표현하며 validate/commit/read/다음전이/receipt 판정은 locked=true를 요구한다. release_for_guardian(index,expectedBound)는 원래 native anchors/state 및 정확 pendingBound를 검증하고 sync_all 후 unlock한다. 성공하면 locked=false이며 앱 begin/bind/resolve/all_receipts_match를 거절한다. 잠금 복제본은 권한 증명이 아니다.

reacquire는 비차단 try_lock 성공 후 original 전체native anchors/ACL/no-reparse 및 cached original Bound 기록을 다시 검증한다. Busy는 lockedfalse로 남기고 caller의 유한 기한 안에서만 재시도한다. 검증 실패는 다시 unlock하고 poisoned 보존하며 쓰기/정리 권한을 복구하지 않는다. unlock 실패도 성공으로 보고하지 않는다. Guardian은 자신의 file handle로 실제 잠금을 얻고 같은 original Bound를 검증한 뒤에만 ACK한다.

root가 schema 형식 변경 없이 pending_bound_identity(index)->Option<BoundIdentity> (Bound만)를 추가하기로 합의했다. release_for_guardian은 이 getter로 exact expected Bound를 검증한다. persistedResolved나 생성한기대receipt를 실제receipt 권한으로 대체하지 않는다.

## 인수 조건

native fixture는 앱 unlock 동안 Bound/noactualreceipt를 scanner가 관측해도 삭제할 수 없음을 확인한다. guardian의 별도 open/owntry_lock Busy, 앱 unlocked mutation거절, guardian unlock 후 app reacquire, actualreceipt resolve를 검사한다. record/ACL/nativeidentity tamper 시 reacquire가 실패하고 bytes를 보존한다. 원래 FileId·확실한 handle lifetime·유한기한은 실제 Windows에서 인수한다.

## 검증과 handoff

현재 단계는 읽기 설계와 작업 계약 기록이다. root Linux49197 검증 중으로 journal Rust/Cargo/native를 실행·변경하지 않았다. GO 이후 최소 수정/회귀를 작성하고 source SHA 동결 후 root 실제 native 및 전체 검증으로 인계한다. 사용자 desktop/계정/시스템/전원 설정 변경 없음.

## 구현 게시 및 source 동결

root Linux49197 실제 terminal0/source equal 확인과 GO 후 journal만 구현했다. lockedbool로 validate/commit/actualreceipt/record() 읽기 권한을 검증한다. record()는 Result를 반환한다. release_for_guardian은 pending_bound_identity의 exact expectedBound와 file sync를 확인하고 unlock하며, unlocked 앱은 전이·receipt 판정·authoritative record 읽기를 거절한다. reacquire Busy는 다시 시도할 수 있지만 기타 잠금/원본 검증 실패는 보존한다. 검증 실패 후 획득했던 잠금은 해제하고 poisoned를 고정한다.

추가 native 시험2개: explicit_handoff_blocks_mutation_scanner_preserves_and_reacquires, reacquire_tampered_record_unlocks_and_poison_preserves. 첫 시험은 실제 별도 exacttest child가 원 Bound 기록으로 Journal::reopen/owntry_lock 후 markerACK를 보낸다. 부모는 해당 child 생존과 app Busy를 확인하고 Begin에 해당하는 fixture release 후 child originalreceipt write/sync 및 original Child terminal을 확인한다. 그 뒤 app reacquire/actualreceipt resolve를 검사한다. child 미진입/0test/timeout은 marker가 없어 실패하며 RAII ownedchild kill/유한reap를 사용한다. 제품 guardian entrypoint 통합은 아직 아니다.

개별 rustfmt 실제 exit0, journal SHA256 50b2c1b084c7bc01fcea6808628633ad4ed768e52f5cfb5b8d76db5b3df7ce1b. 자체 Cargo/native는 실행하지 않았고 root 실제 native/full 검증 전이다. 공유 records getter는 root 담당이며 schema 형식 변경은 없다.

Root 실제 Windows native-first: `cargo test -p dlpad-engine --lib --locked cookies::windows::journal::tests -- --nocapture` exit0, journal6test PASS(별도 child 자기회귀1PASS 포함), sourceunchanged/input desktop unchanged. raw로그에서 경쟁scanner 보존/별도child자체잠금·receipt작성/reacquire변조보존을 독립 확인했다. 제품 helper/Job/public 회수 인수로 확대하지 않는다.

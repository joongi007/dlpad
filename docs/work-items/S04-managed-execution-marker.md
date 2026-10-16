# S04: Durable managed execution pending marker

- 상태: in-progress
- 분류: safety
- 우선순위: P0
- 요구사항: R03, R05, R08, R13
- 담당/수정 범위: diagnostic_cache; crates/engine/src/tools/execution_marker.rs, 이 문서만
- 선행 작업: root exclusive/shared ExecutionLease + guardian FD transport/completion proof

## 사용자 결과

앱 crash 또는 불확정 terminal 뒤관리작업이pending기록을발견하면무기한보류한다. startup자동회수는후속이며기존불명기록은삭제하지않는다.

## 계약과 범위

pending는symlink_metadata로존재여부만읽고어떤type/content여도true. create는caller의exclusive원rootlock전제,원rootpinclone/create_new nonce고정record77bytes미만/file sync; Unixroot sync. 실패초기화기록보존. clear는reservedownedterminalcompletiononly,Drop자동clear없음. 원root/file nativeID/linkcount/exactbytes검증, Unix원rootfd-relativeunlinkat, Windowscreation부터DELETEhandle+FILE_SHARE_READ로foreignrename/writer배제후원FileDisposition 사용. 외부tool/PID/비밀authority추측없음.

## 남은 native 한계

Unixfstatat→unlinkat은동일사용자의악의적namespace교체에원자적조건부unlink가아니다. 전용root와exclusiveapp협력lock은다른협력관리작업을배제하지만비협력sameuser공격race를제거하지않는다. 동시foreignrename 전체방어를완료로주장하지않고rootintegration에서gate판단필요. Windowsdeletepending은원handleclose후완료하므로caller가clear후marker객체를drop해야한다. Windows디렉터리flush는현rootpinREAD_ATTRIBUTES권한으로실행하지못해파일FlushFileBuffers만증명하며rootdurability추가native검증필요. 원root/markerpin불일치/내용손상/link는보존한다.

## 인수 조건

drop후pending보존/createexisting실패; confirmedclear후해제; 내용변조/foreignreplacement/symlink/hardlink clear거절및sentinel보존. 초기write/sync실패보존,unsupportedOSfailclosed. 상기테스트는새module내작성. guardian/crashproof그자체는별도root검증이다.

## 검증과 handoff

단독source작성/fmt후rootcompile/scopedtests대기. Cargo/module선언/기존파일무수정. 실제Windows삭제/ACL및appcrash전체시험미검증.

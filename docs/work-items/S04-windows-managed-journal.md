# S04: Windows managed reader native journal

- 상태: in-progress
- 분류: safety-storage
- 우선순위: P0
- 요구사항: R03, R05, R08, R13
- 담당/수정 범위: diagnostic_cache; 신규 tools/windows_managed_journal.rs, 이 문서만
- 선행 작업: root pure managed records + Windows private security helper/module declaration; root immutable marker/process binding

## 사용자 결과

원nativehandles/strictSIDACL에묶인create-only관리journal과8receipt를보관하고single-use reader Intent/Bound를sync한다. 자동회수/삭제권한은노출하지않는다.

## 계약과 범위

입력원root/marker/executionlockFile 및nonce/baseDigest; actual12anchors생성. 고정 .execution-readers.json와.execution-reader-0.receipt..7만CREATE_NEW,protectedcurrentSIDSD/noinherit/noreparse/regularnlink1/contained 검증. journal16KiB/receipt256B;모든원pin/recordbytes재검증. root/lock는existingdefaultACL을수정하지않고원ID/no-reparse/nativepath로검증;marker/journal/receipt는protectedSIDACL필수. lockrelease는Boundcommit뒤명시원journalFile unlock; guardian자체원lock획득은별도handoff. copiedFile을잠금권한으로간주하지않는다. Drop/partial실패는files보존. cookieanchor/type직접재사용없음.

## 인수 조건

create/reopenoriginalanchors/Intent-bind-resolveactualreceipt; metadata/ACL/nativeIDreplacement/tamper/oversize/busy 실패;불명receipt/Resolved-only자동삭제없음;sentinel보존. actualWindowsrootfixture는currentSIDprivateSD로만생성;기존사용자권한변경없음.

## 검증과 handoff

create/reopen/begin/bind/release/reacquire/resolve/allactualreceipts 구현 및4nativefixture테스트 작성. ownfmtPASS;실제Windowscompile/test root대기. Cargo/sharedmarker/lib/process변경없음. nativeWindowscompile/test는root담당이며미실행을성공으로보고하지않는다.

### 주 담당 통합 검토

주 담당이 native module 선언을 연결하고 `JournalError`의 Debug 구현에 남은 이전 타입 이름을 수정했다. 하위 담당의 포맷 통과만으로 Windows 컴파일 통과를 주장하지 않는다. 실제 Windows 전체 검증은 아래 후속 실행 결과로 판정한다. 자동 회수 및 프로세스 연결은 여전히 후속 통합 대상이다.

Windows 첫 통합 수집: fmt 통과, Clippy에서 READ_CONTROL의 잘못된 Security 모듈 import로 실패했다. 실제 windows-sys 선언(Storage::FileSystem)을 확인하여 import를 수정했다. 실패 증거 windows-managed-journal-integrated-first는 보존한다.

주 담당 실제 Windows GNU CPU 통합 검증: `python3 scripts/windows_current_workspace_probe.py .../windows-managed-journal-import-retry --execute --seconds 1800`의 fmt·전체 Clippy(-D warnings)·workspace test·release 모두 통과했다. 주 담당이 result.json과 원 로그, 최종 소스 SHA를 독립 대조했다. native journal 4개 실패 보존/잠금/원 receipt 시험도 통과했다. 자동 회수·실제 reader guardian 연결은 아직 미완료이며 합성 receipt 시험을 실제 Job 종료 증거로 사용하지 않는다.

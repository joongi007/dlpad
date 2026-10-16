# S04: 관리 실행의 durable 변경 보류 연결

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R08, R13
- 담당/수정 범위: root — tools.rs, execution_lease.rs, direct_download.rs, accessible.rs, managed.rs, catalog.rs; diagnostic_cache — execution_marker.rs 및 자체workitem
- 선행 작업: shared 실행lease·process cleanup미확인typederror 및 originalmarker 삭제capability

## 사용자 결과

관리 실행전에durablependingmarker를기록하고정상reader정리확인뒤만제거한다. 앱crash/미확인정리/marker변경에서marker를보존하고다른manager의설치·선택·제거를거절한다. 외부도구는이marker를생성하지않는다.

## 계약과 범위

managerexclusive→createonlysyncmarker→sharedlease순서이며Drop은marker를지우지않는다. 기존mutationlock은pending존재시ExecutionPending반환한다. appworker는원options/resume저장snapshot의canonical관리path를확인한경우만시작한다. SessionCleanupDeferred및cleanup_is_unconfirmed에서finish금지,모든확정실행반환뒤만finish한다. 자동pending회수·guardianFDhandoff/Windows 실제crash는후속이다. 보류중외부path다운로드는가능하고관리mutation은failclosed한다.

## 인수 조건

normalfinish후mutation허용,drop/marker변조/미확인종료후manager재열기에도mutation차단,원외부source/다른package보존. worker통합및strictClippy/scopedtests결과를기록한다.

## 검증과 handoff

현재통합중이며실제검증전완료로표시하지않는다.

### 첫 실제 검증

기반marker/lease7시험PASS. worker최초CPU시험은80PASS/1FAIL로fixture의기존yt-dlp입력이비어app.options()검증이거절됐다. 실행기능오류로취급하지않고fixturetool값을추가한다. 원결과를기록하며제품pending통합완료를아직주장하지않는다. marker자동회수·guardianhandoff·Windowsactualpendingcrash는별도다.

### 실제 scoped 통합 결과

workerfixture보정후CPUentry81PASS,marker/lease8PASS(명시finish/abandon후manager재열기pending거절포함),CPUworkspacealltargetClippyexit0. 정상discovery실패는현재worker가marker를clear하고다시reservation가능한것을확인했다. SessionCleanupDeferred는Drop시기록보존하도록연결했지만실제guardian미확인/crashGUI수직검증은후속이다. Windowscurrentworkspace는실행중이며Unixfstatat→unlinkat동일사용자namespace경합/Windowsdirflush한계는marker작업문서에유지한다.

### Windows 최초 실제 실패와 수정

Windowsfmt/Clippy는exit0였으나CPUworkspace test는marker파일생성에서InvalidInput(creating/truncating requires write access)로실패했다. access_mode(FILE_GENERIC_WRITE)만으로RustOpenOptions의내부writeflag가설정되지않은것이원인이어서read(true)/write(true)를명시했다. 기존실패원시execution-pending-windows-workspace-first를보존하며새evidence에서재검증한다.

### Windows 재시도 실패의 실제 원인 정정

execution-pending-windows-open-retry도 같은InvalidInput으로 실패했다. 원소스를 재확인한 결과 앞선 문자열 치환이 실제코드와 일치하지 않아 read/write flag가 추가되지 않았다. 이번에는 원문 일치1개를 assert한 뒤 Windows OpenOptions에 read(true)/write(true)를 실제 추가했다. 앞선 수정 주장을 정정하며 두 실패 원시를 보존한다. 새 execution-pending-windows-open-flags-retry에서 전체4명령을 재검증한다.

### Windows 수정 재검증 완료

execution-pending-windows-open-flags-retry의 fmt/Clippy/test/release 실제4명령은 모두exit0/PASS다. root가 각 JSON의 source before/after 일치와 현재파일 SHA를 독립 대조해 불일치0개를 확인했다. Linux execution회귀8개도 재실행해PASS, 문서563개PASS. 첫 Linux 재실행은 sandbox target/debug/.cargo-lock read-only로 실패했으므로 승인된 프로젝트 쓰기 환경에서 재실행한 결과를 사용한다. WindowsGUI/실제관리reader crash/자동pending회수는 여전히별도게이트다.

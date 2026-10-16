# S07: 이번 준비 트랜잭션의 원 파일만 명시적으로 정리

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R11
- 담당/수정 범위: root — app_install.rs 공유 outcome/failure/remainder와 payload.rs 읽기 pin visibility·문서·통합·native 검증; soak_recovery_audit — app_install/windows_caps.rs 원 생성 HANDLE DELETE/disposition primitive; diagnostic_cache — 새 app_install/removal.rs coordinator; metrics_recovery — 독립 native fixture 초안
- 선행 작업: source553 로컬 prepare/inspect CLI 전체4검사·31+2 tests·실제11명령·원nativeID대조·windowless cancel/controlled expiry·foreign/partial inspect PASS

## 사용자 결과

같은 프로세스가 이번에 생성하고 계속 보유한 PreparedBundle을 명시적으로 버릴 수 있다. 설치 준비 후 취소·실패 복구의 기반이며 재시작 뒤 기존 경로에서 소유권을 추정하지 않는다. 개발 CLI의 자동 cleanup과 정식 cold uninstall은 제외한다.

## 계약과 범위

discard_fresh(bundle,cancel,deadline) -> Result<DiscardOutcome,DiscardFailure>. 새 원 root/child 생성 시에만 DELETE 접근권한을 추가하고 original HANDLE의 FileDispositionInfo를 사용한다. path 재개방에 삭제권한을 부여하거나 recursive delete/권한 상승/ACL 수정/POSIX share 우회를 하지 않는다. 원 전체 ID/hash/ACL/exact namespace preflight 실패에는 삭제0으로 보존한다. 삭제는 비원자적이다. disposition 성공 수는 storage 영구 삭제 증명이 아니며 Drop은 새 삭제 호출을 하지 않는다.

DiscardFailure는 원 root와 아직 승인하지 않은 원 file/optional marker capabilities 및 immutable original record를 opaque remainder로 보유한다. retry는 이 동일 remainder만 consume하며 경로/JSON을 adoption하지 않는다. accepted original objects는 close로 삭제가 실현될 수 있다. 취소·기한은 다음 disposition을 막으며 이미 승인된 삭제를 복원/재생성하지 않는다. marker는 payload 뒤, root는 원 빈 directory에만 마지막 disposition을 적용한다. unknown 파일이 중간 삽입되면 원 객체 외에는 건드리지 않고 실패 및 remaining caps를 보존한다.

기존 reader의 share 호환성이 DELETE access 추가로 바뀔 수 있다. 관측자는 원 WRITE/DELETE 권한을 공유하도록 명시해야 하며 읽기 전용 inspection은 writable bundle이 살아있을 때 Busy를 유지한다. 신규 dependency/lock 변경 없음. 기존 Windows FileSystem API만 사용한다. 버전·domain/schema 변경 없음.

## 인수 조건

실제 synthetic PE/notice의 fresh discard, outside source/sentinel/unknown 보존; preflight extra/changedbytes/root replacement/refuse0; 단계 사이 cancel·deadline·injected IoFailure의 accepted/remaining 구분과 원caps retry; unknown race·external read handle pending에는 foreign/remaining 보존; delete 권한 없는 original HANDLE refusal/no path escalation. 실제 namespace에서 original root가 사라짐을 관측하되 storage wipe·cold uninstall을 주장하지 않는다. 기존 prepare/inspect 및 CLI 원 source read/캡 공유 회귀를 포함해 전체 검증한다.

## 검증과 handoff

원 prepared record는 durable 삭제 권한이 아니다. root가 공유 contract를 확정한 뒤 파일별 병렬 구현하고 actual Windows whole/native 검사를 직렬 통합한다. helper source/compiled pins와 실제 결과를 별도로 기록한다.

통합 검토: 기존 transaction::verify_current의 내부 고정30초/취소false 호출을 재사용하지 않고 caller 취소·기한이 적용되는 전체 원 record preflight로 통일했다. 각 disposition 전 남은 payload를 재해시하므로 최악64개 항목/총256MiB에서 반복 읽기 비용은 보수적으로16GiB 이하이며 O(N²)이다. chunk64KiB·메모리 상한은 유지하고 기한 초과 시 원 remaining capabilities를 보존한다. 원 cap 공유와 root replacement·DeletePending은 실제 Windows 시험으로 판정한다. Rust std 기본 File::open을 share3으로 단정하지 않으며 명시 observer SHARE_ALL은 계약 명확화다.

2026-10-06 실제 중간 인수: source555 Windows fmt/Clippy/test/release4PASS·app_install40actual tests. CPU371b3fc0…/7,917,568B·크기증가0. 실제CPU fixture 첫 실행은 fixture OpenOptions create_new에write옵션누락으로 InvalidInput 실패했고 원 실패 evidence를 보존했다. 수정 후 별도 fresh evidence에서4bundles 정상/foreign/취소/기한 및 원remainder retry3files+root disposition·관측namespace삭제/outside notice/sentinel/sourceSHA보존 PASS, root와독립 검토를 수행한다. 신규 rootDELETE cap에 맞춰 CLI 관측 rootpin을 SHARE_ALL로 명시한 격리windowless cancel/controlled expiry2case도 원prefix/source/foreign보존·자연Job0PASS. fixture수정포함최종전체4검사와 기존preparefixture호환인수는 진행 중이다.

증거: [전체 최초](../validation/evidence/windows-app-fresh-discard-workspace-20261006-continue/root-review.json), [첫 fixture 실패](../validation/evidence/windows-app-fresh-discard-real-cpu-20261006-continue/collector-result.json), [fresh retry](../validation/evidence/windows-app-fresh-discard-real-cpu-retry-20261006-continue/root-review.json), [CLI 원제어](../validation/evidence/windows-local-install-cli-discard-control-20261006-continue/root-review.json). root replacement 회귀는 kernel rename거절분기와 실제대체분기를 구분한다; 해당 unittest 성공을 실제대체 발생 증거로 승격하지 않는다.

최종 scoped 인수: fixture수정포함 source555 Windows 전체4검사 PASS 및 root/독립검토. 기존 실제CPU prepare fixture도 별도프로세스에서 marker719/원ID·SHA/cursor17/정확3namespace/foreignsource 보존 PASS. 정상·실패·원cap retry fresh discard 인수 완료이며 cold uninstall/설치등록·updater는 완료가 아니다. [최종 전체](../validation/evidence/windows-app-fresh-discard-fixture-fix-workspace-20261006-continue/root-review.json), [준비 호환](../validation/evidence/windows-app-prepare-discard-real-cpu-20261006-continue/root-review.json).

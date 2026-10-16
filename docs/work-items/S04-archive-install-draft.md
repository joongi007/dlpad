# S04: local archive package 설치 실제 Rust 초안

- 상태: done
- 분류: prototype
- 우선순위: P0
- 요구사항: R05, R11
- 담당/수정 범위: diagnostic_cache — /tmp/dlpad_archive_install_draft.rs 및 이 신규 문서
- 선행 작업: root ArchivePackage 및 실제 bounded_zip7회귀/공식 Deno 검증; 제품 연결은 root

## 사용자 결과

명시 local archive를 source 읽기 전용으로 pin하고 component를 추출·sync·검증·비활성 publication하는 코드 초안을 작성했다. 설치가 active DB를 변경하지 않는다. 프로젝트 source/Cargo/vendor/runner는 동결을 유지했다.

## 실제 코드 대조

기존 direct_download::OwnedDirectory는 Unix dev/ino 기반 files를 추적하지만 nonUnix same_identity=false라 불확실 잔여물을 보존한다. path 기반 unlink/rmdir에 race gate가 남는다. multi-component native publication helper로 그대로 복사하면 부족하다. 기존 import는 manifest.json/executable 두 파일과 activeDB 자동전환을 묶으므로 새 설치 요구와 다르다.

staging_recovery는 MAX_STAGES8/MAX_STAGE_BYTES512MiB/visitedentries2048/evidence16KiB, PID/boot/start 및 original file identity/checkpoint digest로 종료 owner만 회복한다. archive reservation(추출component+두manifest)은 예산 재사용 가능하나 direct knownfiles/evidence 구조를 component set으로 확장해야 한다. 설치 전체128 제한은 installed per-tool128와 같지 않으므로 root bounded total package inventory API가 필요하다. source archive는 사용자 파일이므로 owned stage bytes나 cleanup 대상에 포함하지 않는다.

## 초안 인터페이스와 구현

`ToolManager::install_verified_archive(&mut self,source:&Path,package:ArchivePackage,cancel:&AtomicBool)->Result<InstalledPackage,ToolError>`; InstalledPackage{package,directory,components{name,path,bytes}}. SourcePin은 canonical plain path/actual NativeFileIdentity/regular·exactlen을 보유하고 반복 재검증한다. package.validate→inventorycap→reservation→새 ownedstage→각component create_new→extract_verified_zip→sync→원handle mode(Notice600/실행·보조700)→checkpoint→archive-package.json/fullpackage 및 compatibility manifest.json 작성→source전체hash 재확인→packagefull검증→create-only whole-dir publication→staging기록 정리 순서다. 무조건 path rename/copy로 native 권한 문제를 덮지 않고 root bridge를 명시했다.

compatibility manifest는 main Executable의 SHA를 사용하며 archive SHA를 넣지 않는다. single InstalledTool은 set 전체검증 후에만 main을 반환해야 한다. remove/selection 현재 strict twofiles는 package.json/notice/auxiliary를 foreign로 거절하므로 package-aware adapter가 필요하다. 설치만으로 version실행 성공을 의미하지 않으며 Deno --version native argv/timeout/ownedprocess 검증은 후속이다. supply trust/noticegate도 자동 해제하지 않는다.

## root가 구현해야 하는 구체 bridge

- verify_package_inventory_capacity(128,2048): bounded 전체 package inventory, unknown nested failclosed.
- OwnedPackageStage::create/create_file/verify_file/checkpoint_complete/checkpoint_metadata: native dir/file original IDs/retained handles, Unixprivate mode 또는 Windows protectedSID ACL, create_new/reparse보호, <=8components+2metadata evidence 및 예약bytes.
- verify_source_digest: publication 직전 complete source streaming SHA/exactsize/cancel/deadline. 같은 ID와 길이만으로 내용 불변을 주장하지 않는다.
- verify_complete/publish_create_only: 모든 원 component hashes·exactknownentries·부모 namespace pins 및 native nooverwrite 전체dir공개. finalpublication crash/state journal 필요.
- original_staging_path/commit_preserve 및 실패Drop: ownership불명은 보존. publication 이후 forget_staging DB 실패는 이미 공개된 비활성 package를 보존하고 recoverable journal 오류로 보고한다. 기존 active 유지. finalpath 원인 모르면 삭제 금지.

이는 실제 Rust control-flow 초안이며 bridge에 가짜 안전 구현·unimplemented 실행을 넣지 않았다. 현재 통째로 compile할 수 있는 제품 코드라고 주장하지 않는다.

## 실패 fixture 계약

owned temp root/source/foreignsentinel만 사용한다. valid stored/deflate single Deno와 aux+notice package를 설치해 exact content/permission/fullmanifest/active unchanged 확인. archive hash/CRC/componenthash/size/경로 및 cancel/deadline 오류, source ID·내용 교체, symlink/reparse/hardlink/foreignnested, output write/sync/diskfull, checkpoint crash/미기록stage보존, capacity8/512MiB·inventory128/2048, publishcollision/동시namespace교체/DBforget failure에서 원 source/sentinel/이전active bytes 보존을 확인한다. 각stage lifecycle/native originalID pin을 확인하며 nonUnix 증거부족을 Linux PASS로 대체하지 않는다.

## 검증과 handoff

기존 tools/direct_download/staging_recovery 실제 source를 읽고 임시 Rust 코드를 작성했다. `rustfmt --edition 2024 /tmp/dlpad_archive_install_draft.rs` exit0. Cargo compile/native install/GUI/source 삽입은 실행하지 않았다. root bridge/타입 합의 및 실제 실패 인수 후에만 제품 완료로 전환한다.

# S01: 소유 임시 저장 위치의 Windows 생성 권한 거절

- 상태: done (Windows owned ACL 실패 경로 한정)
- 분류: test
- 우선순위: P0
- 요구사항: R02, R11
- 담당/수정 범위: soak_recovery_audit 초안; root 통합·실행·검증 — crates/engine/examples/windows_location_acl_fixture.rs, 본 문서, 전용 evidence
- 선행 작업: Windows location 공개 API 실제 시험, root source freeze session92486 종료 및 root SHA 검토

## 사용자 결과

선택한 저장 위치를 실제 Windows ACL이 거절해도 이전 저장 설정과 원 합성 sentinel이 보존되는지 검증한다.

## 계약과 범위

기존 location 예제·R02·ADR-0002/ARCHITECTURE 및 windows_session_cookies의 currentSID/SDDL API를 읽는다. 새 TempDir 하위에만 currentSID protected DACL을 적용한다. 원 디렉터리 HANDLE의 READ_CONTROL/WRITE_DAC 권한과 원 DACL·protected flag를 유지하고, FILE_ADD_SUBDIRECTORY만 명시 DENY 후 currentSID GA를 허용한다. deny ACE는 자손에 상속하지 않는다. global ACL/실제 Documents/기존 사용자 파일·GUI·시스템 설정은 수정하지 않는다. 기존 windows-sys feature만 재사용한다.

## 인수 조건

native fs::create_dir가 ERROR_ACCESS_DENIED(5)를 실제 반환해야 하고, 같은 경로의 공개 location::prepare_with_default Selected도 동일 오류를 반환해야 한다. 실패 target 부재, 이전 선택 Restore 및 원 sentinel bytes 보존을 요구한다. original HANDLE의 security descriptor/DACL을 원 protected 상태로 복원하고 그 뒤에만 TempDir 정리한다. 복원 실패는 성공으로 처리하지 않고 owned fixture를 보존한다. 주입 PermissionDenied로 대체하지 않는다.

## 검증과 handoff

프로젝트 Rust/Cargo/native 실행 금지; /tmp standalone Windows example로 준비하여 root 검토 후 게시·실제 빌드·hidden CLI 실행을 맡긴다. 현재 실제 Windows 인수 미검증이다. original ACL restoration RAII는 오류 경로의 best-effort이며 명시 restore 성공 검증이 최종 인수다.

초안 SHA `b52e5c342ccef6ff020c273e86e3421dd0b6e3b0083de928bb000101a7f891ed`, 소유 사본 `rustfmt --edition 2024` exit0. native 원 권한으로 ACL을 복원한 뒤 원 DACL bytes/protected flag를 다시 조회해 비교한다. 복원 오류에는 TempDir를 keep하여 무조건 정리하지 않는다. 복원 후 같은 공개 위치 API의 생성 재시도와 TempDir::close 성공까지 요구한다. root에게 검토·게시·실행 handoff했으며 Windows compile/native API 결과는 아직 없다.

## root 실제 실행

SHA검증 후 crates/engine/examples/windows_location_acl_fixture.rs에 통합했다. Windows에서 cargo run -p dlpad-engine --example windows_location_acl_fixture --locked exit0 PASS. root는 actual JSON의 direct/publiclocation raw error5, savedroot/sentinel 보존, exact original ACL 복원, 복원후재시도와 ownedTempRemovedtrue를 확인하고 windows-location-acl-20261004-resume/root-review.json에 기록했다. 기존Documents·시스템ACL 변경0. workspace 전체회귀는 진행중이다.

후속WindowsGNU CPU workspace fmt/Clippy/test/release 네명령 모두PASS; root가source_final과현재SHA·각exit0·releasebinarySHA를확인했다. windows-acl-integrated-workspace-20261004-resume/root-review.json. 이 ownedACL 실패경로 작업만 완료이며 실제OneDriveprofile/unknownvolume/전체R02인수는별도다.

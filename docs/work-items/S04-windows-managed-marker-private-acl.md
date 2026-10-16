# S04: Windows 관리 실행 기록의 private ACL

- 상태: in-progress
- 분류: safety
- 우선순위: P0
- 요구사항: R05, R08, R13
- 담당/수정 범위: root — tools/execution_marker.rs 및 이 문서
- 선행 작업: 공통WindowsSID/ACLhelper의native기존cookie회귀

## 사용자 결과와 계약

원create-only execution marker는현재사용자SID의protected단일ACE로생성하고retainedHANDLE로검증한다. manager root 전체기존ACL을바꾸지않고프로젝트밖자료권한을변경하지않는다. 파일생성securitydescriptor는새helper의PrivateDescriptor이며delete/read/write/originalreparsepin권한은기존정책유지한다. verify/clear에서currentSID와원markerACL을재검증한다. 원marker/root nativeIDs는기존v2와함께유지한다. ACL은reader종료증거가아니며자동회수권한을부여하지않는다.

## 인수 조건

Windows실제create/normalfinish/drop/impersonation/ACL변조거절및외부sentinel보존. 기존unknown/v1pending은보존. 새반환오류는fixed이며SID/path노출없음. failedwrite/verify원marker보존. 원helper회귀검증종료뒤구현한다.

## 검증과 handoff

현재조건부다음작업계약이며구현완료/PASS아니다. Windowssourcefreeze종료뒤진행한다.

### 구현

공통helper통합Windows4checksPASS뒤marker를원CREATE_NEW/privateSD/비상속HANDLE로생성하도록변경했다. create/verify/clear에서원HANDLE protectedACL/currentSID를검증하고실패기록은보존한다. root전체ACL은변경하지않는다. Windowsactual회귀는다음collector에서검증한다.

### 첫 native 통과와 실패 경로 추가

windows-managed-marker-private-acl-first fmt/Clippy/test/release 모두exit0/PASS. 이후 원 temporary marker만 WRITE_DAC HANDLE로 NULL/unprotected ACL로 의도적으로 변경하고 clear거절/원기록bytes/foreignsentinel보존을 확인하는 Windows 전용 회귀를 추가했다. 새 시험 실제 결과는 다음 collector에서 검증한다.

### ACL 변조 회귀 첫 실패

windows-managed-marker-acl-tamper-regression는fmt/Clippy통과후test에서SetSecurityInfo가ERROR_ACCESS_DENIED5로실패했다. 원scope207PASS/1FAIL/2ignored이며변조성공을검증하지못했다. 시험용HANDLE에READ_CONTROL도명시하여native재검증한다. 실제제품보호실패또는성공으로단정하지않고원raw보존한다.

### ACL 변조 Windows 재검증 통과

시험용WRITE_DAC HANDLE에READ_CONTROL추가후원WindowsSetSecurityInfo변조성공/clear거절/기록과foreign파일보존회귀PASS. windows-managed-marker-acl-tamper-readcontrol-retry 전체fmt/Clippy/test/release 모두exit0/PASS 및root각sourceafter/currentSHA대조일치. 첫ERROR_ACCESS_DENIED5실패원시는보존한다. ACL보호는자동회수/reader종료증거완료가아니다.

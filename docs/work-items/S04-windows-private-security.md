# S04: Windows retained-handle private security helper

- 상태: in-progress
- 분류: safety
- 우선순위: P0
- 요구사항: R03, R05, R08, R13
- 담당/수정 범위: diagnostic_cache; crates/engine/src/windows_private_security.rs, 이 문서만
- 선행 작업: root cfg(windows) lib module declaration/cookie wrapper integration

## 사용자 결과

cookie와managed execution binding이동일한원WindowsSID/protectedACL 검증을공유한다. 임의파일open/delete/ACL변경권한은노출하지않는다.

## 계약과 범위

cookies/windows.rs45–250 Local/owned/sid_string/current_sid/Descriptor/acl을그대로복사후도메인중립io에러로매핑한다. pub(crate) current_sid, PrivateDescriptor::new/attributes, verify_acl만제공. fixedPermissionDenied/Unsupported메시지에SID/path없음. threadimpersonationreject/processTOKEN_USER선택, securitybuffer65536/SID68/string256,protectedDACL/ownerexactSID/ACEexact1/noflags/fullaccess/exactSIDlength 검증유지. publicdescriptor입력에는255ASCII SID형태상한을추가해SDDL주입/무한allocation을거절하며유효currentOS SID정책은완화하지않는다.

## 인수 조건

root WindowsstrictClippy/nativecookie회귀로원동작유지,적절currentSIDprivatefile성공/외부ACE·owner·inheritedunprotectedACL·impersonation거절. nativehandleLocalAlloc/tokenRAII 유지. rootcookie wrappers는Unsupported를Unavailable로,다른보안오류를CleanupOwnershipChanged로매핑하여기존계약유지. Linux단위통과는Windows검증아님.

## 검증과 handoff

신규모듈단독fmt후rootcfg Windowscompile/native회귀대기. 기존cookieRust/lib/Cargo변경없음. integration전일시중복은승인범위이며root기존함수제거후공유helper사용한다.

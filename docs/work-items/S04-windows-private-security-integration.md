# S04: Windows 원본 보안 검증 공통화 통합

- 상태: in-progress
- 분류: refactor
- 우선순위: P0
- 요구사항: R05, R08, R13
- 담당/수정 범위: root — lib.rs, cookies/windows.rs 및 이 문서; diagnostic_cache — windows_private_security.rs
- 선행 작업: Windows 관리 pending 복구 계약

## 사용자 결과와 계약

기존쿠키SID/strictACL/impersonation거절로직을공통helper로옮겨관리reader원본보안검증에도재사용한다. CookieImportError매핑과private Descriptor API는wrapper로유지하며파일open/delete권한은새helper에추가하지않는다. SID/경로/비밀없는고정오류를사용한다. root가공유module과기존cookie연결을담당한다.

## 인수 조건

WindowsGNUstrictfmt/clippy/test/release및기존nativecookieACL/reader/recovery회귀통과. Unsupportedsecuritycontext는CookieUnavailable,다른ownership검증오류는기존changed로유지. 관리pending자동회수구현완료로표시하지않는다.

## 검증과 handoff

원함수와helper를root읽기대조했다. Windowsactual회귀는통합뒤실행한다.

### 최초 실제 Windows 결과

fmt通過뒤Clippy는원cookie에서공통helper로옮긴Threading import가남아unusedimport오류로실패했다. 보호동작시험전compile실패이며원raw windows-private-security-integrated-first를보존한다. 해당unusedimport를제거한새retry로전체검증한다. 문서577개PASS.

### 고정된 Windows 공급 버전 재검증

windows-ffmpeg-native-version-frozen-retry actual두버전명령exit0/정확version일치/private desktop보존/binary전후SHA일치/sourceunchangedPASS. root원result검토완료. 보안helperimportretry Windows전체검증은실행중이다.

### Windows 원회귀 전체 통과

windows-private-security-integrated-import-retry fmt/Clippy/test/release전부exit0/PASS및각sourcebefore/after일치를rootJSON대조로확인했다. 원cookie/reader/recoverynative시험이포함된workspace검증이며이후markerprivateACL추가는별도snapshot인수를진행한다. 최초unusedimport실패raw는보존한다.

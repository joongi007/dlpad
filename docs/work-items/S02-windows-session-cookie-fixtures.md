# S02: Windows 세션 쿠키 native ACL 합성 회귀

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: soak_recovery_audit — crates/engine/tests/windows_session_cookies.rs 및 이 문서
- 선행 작업: root Windows prepare_session_cache/import_session/lease API 연결

## 사용자 결과

Windows에서도 현재 사용자만 허용하는 세션 복사본을 일반 도구 방식으로 열고 확인된 종료 뒤 정리한다. 확인 없는 reader와 소유권·ACL 변경은 복사본을 보존한다.

## 계약과 범위

임시 source/cache/sentinel과 합성 Netscape 값만 사용한다. 현재 SID/process token 및 소유 tmp 파일 security descriptor를 기존 windows-sys feature로 읽는다. ACL broaden 시험은 이번 fixture 파일만 변경한다. 실제 계정 생성/사용자 프로필/관리자 권한/권한 필요한 symlink/외부 credentials를 사용하지 않는다. cookies.rs/windows.rs/process.rs/Cargo/다른 tests는 수정하지 않는다. Cargo/Windows 실행은 root가 담당한다.

## 인수 조건

- literal copy/private path/source 불변, ordinary std File read/write 및 cleanup, cache/owner/copy의 actual currentSID owner와 protected 단일 allow DACL.
- reader confirm은 정상 정리, 확인 없는 reader와 defer/Drop은 복사본 보존과 lease 무효.
- extra file/current-user+Everyone ACE broaden은 failclosed, source/sentinel과 변형 fixture는 자동 삭제하지 않음. NTFS hardlink 생성 성공은 기존 identity 검증/cleanup 거절·copy/link 보존을 시험한다. OS 오류32로 생성이 거절되면 foreign 부재·copy exact bytes·lease 정상·reader 확인·정상 cleanup/source/sentinel 불변을 검사하고 hardlink 검출 성공으로 표현하지 않는다. 다른 오류는 실패다.
- Windows만 cfg하고 새 dependency/feature 없이 작성. Linux compile만으로 Windows 인수 완료를 보고하지 않음.

## 검증과 handoff

4개 tests 작성과 소유 rustfmt 완료, sourcefreeze 상태다. Actual process TokenUser SID와 GetNamedSecurityInfo/GetAclInformation/GetAce로 owner currentSID·SE_DACL_PROTECTED·단일 currentSID allowACE 및 full-access mask를 확인한다. 실제 ordinary std read/write→reader confirm→cleanup, pending/defer Drop 보존, extra/NTFS hardlink/ACL Everyone read 추가에 validate+cleanup 거절, existing broadened cache SDDL 불변을 시험한다. 모든 변경은 소유 tempfile에 한정되며 symlink/새 계정/admin 작업은 없다. 새 feature/dependency를 추가하지 않았다.

Root의 actual Windows runtime session18167은 exit101, fixture4 중3PASS/1FAIL이다. [최초 원시](../validation/evidence/windows-cookie-runtime-first/test.log)를 보존했고 root는 입력 불변을 확인했다. 정상 literal/read-write/cleanup/privateACL, pending/defer 보존, existing cacheACL 거절은 통과했다. mixed extra/hardlink/ACL test는 hardlink 생성의 OS32 sharing violation에서 실패해 후속 copy ACL 확대 case는 아직 실행되지 않았다. 기존 Windows Job3/lib process6 PASS는 별도 범위다.

OS32만 정확히 처리하는 분기를 추가했다. copy/source/sentinel 불변·lease validate/reader confirm/normal cleanup을 모두 assert하고 해당 loop만 continue하여 다음 ACL mode가 반드시 실행된다. Hardlink 생성 성공 분기는 기존 rejection/preservation 인수를 유지하고 다른 오류는 panic한다. stdout에는 credential/path/SID 없이 hardlink의 created/refused32 및 copyACL broadened 고정 marker만 기록하므로 root의 `--nocapture` 실제 실행에서 경로를 구분할 수 있다.

소유 rustfmt/sourcefreeze 완료 후 root 실제 Windows 재실행을 기다린다. 하위 담당은 Cargo/Windows를 실행하지 않았다. 새 분기는 아직 미검증이며 Linux cfg 제외를 Windows PASS로 표현하지 않는다.

# S07: 등록된 개발 묶음의 재시작 읽기 검사

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R11
- 담당/수정 범위: root — app_install.rs export·문서·통합·native; soak_recovery_audit — 새 app_install/registration_reopen.rs; metrics_recovery — /tmp 별도프로세스검사 fixture 초안
- 선행 작업: fresh registration 최종실제인수, schema/원ReadRoot·ReadFile 고정(현재원558 전체4검사49회귀PASS, actualCPU등록시험 진행중)

## 사용자 결과

새 프로세스에서 명시된 등록 폴더의 record/seal·ready·payload 원 IDs와 SHA·privateACL·정확namespace를 읽기전용으로 검사한다. 성공도 삭제·업그레이드 권한으로 승격하지 않는다.

## 계약과 범위

inspect_registered(target,cancel,deadline) -> RegisteredInspection. 원Readonly ReadRoot/ReadFile만 이용하고 기존FreshRoot를 adopt하지 않는다. record64KiB/seal4KiB/ready64KiB/payload64KiB chunk·64개/256MiB 상한. strict typedrecord/sealcanonicalbytes/원readyID/fullmanifest/entryIDs/hash/PE·namespace+조상전후검증. RegisteredInspection.record/seal/path/executable/verify_current는읽기cap보유; Drop은삭제없음. 기존prepare/reopen/CLI계약변경없음. colddelete/registry/PATH/activepointer/updater는제외.

## 인수 조건

실제 fresh프로세스에서 priorfresh등록 actualCPU 원IDs/bytes/canonicalrecordseal/정확5namespace검사; livewriter Busy·wrongdomain/schema/trust/nonce/digest/root·fileIDs/foreign/changedbytes·cancel/deadline는변경0; verify실패도 원caps보존. source/media/sentinel전bytes불변. actualWindowswhole4와root독립검증. unsigned consistency가historicalauthority라는주장없음.

## 검증과 handoff

초안 담당들은현재native source freeze를유지하도록 /tmp초안만작성한다. root가선행실측완료후프로젝트게시·shared export통합한다. 현재미완료.

통합실제기록: 첫source560 fmt/Clippy PASS, 시험은 samebytes 새파일의 ACL검사에서 PermissionDenied로 FAIL(406pass/1fail/7ignored). 이를파일ID거절성공으로기록하지않는다. 새합성교체파일만 existing PrivateDescriptor/currentSID와 CreateFileW CREATE_NEW originalreturnedHANDLE로생성하도록fixture보완했다. 기존ACL변경·production검증완화없음. 원ID!=새ID/원객체outside유지/동일bytes/보호ACL+readonlyopen성공 후 InvalidData를요구한다. 최종실행중이다.

후속진단정정: 두번째동일FAIL의 원인은 identity-only observer HANDLE이 FILE_READ_ATTRIBUTES만 보유하고 GetSecurityInfo용 READ_CONTROL이 없었기 때문이다. 첫FAIL을실제상속DACL상태관측으로단정한것은근거가부족해정정한다. ACL검증은 신규 생성의원GENERIC_READWRITE HANDLE(READ_CONTROL포함)에서하고 identityobserver는원ID조회만사용한다. production ReadRoot.open_file의실제protectedACL검증과ID일치를먼저확인한뒤 InvalidData를요구하는조건은유지한다. targetednative를먼저검증한뒤전체검사를진행한다.

최종 scoped 인수: source560 Windows 전체4검사·app_install53actual회귀 PASS. CPU19c49321…7,917,568B·크기증가0. 별도fresh프로세스에서prior source558 registeredCPU4888…의원5namespace·root/ready/record/seal/payload IDs/SHA/PE/정확canonicalsealbytes·typed inspect/verify취소·기한4회/원exe callercursor37 보존PASS. outside notice/sentinel/원payload 모두root와독립검토에서보존확인. colddelete/active설치/GUI전체인수는아니다.

증거: [최종 전체](../validation/evidence/windows-app-registered-reopen-read-control-workspace-20261006-continue/root-review.json), [targeted4객체ID교체](../validation/evidence/windows-app-registered-reopen-id-targeted-20261006-continue/result.json), [별도 실제복사본](../validation/evidence/windows-app-registered-reopen-real-copy-20261006-continue/root-review.json). source560 두번째failed ACLfixture·포인터typefail 원결과도보존했다. 다음 [개발 CLI연결](S07-windows-registered-local-cli.md) 통합진행.

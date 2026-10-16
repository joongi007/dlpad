# S03: OS native 파일 identity 공통 계약

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: 주 담당; engine/file_identity.rs·lib.rs·publication 공유 schema·library.rs/library/identity.rs, library_identity 담당 file_identity/windows.rs 및 전용 Windows 시험
- 선행 작업: Unix 조건부 library identity; publication journal v5

## 사용자 결과

Windows에서도 이름과 크기만으로 파일을 같은 결과로 추측하지 않고 OS 파일 identity를 관측한다. 공개·복구와 외부 rename 연결에 활용한다.

## 계약과 범위

NativeFileIdentity는 serde enum Unix(device,inode) / Windows(volume_serial,file_id16bytes)다. from_file은 호출자가 소유한 열린 File에서 관측한다. 경로 containment와 reparse 거절은 별도 필요하며 identity 자체는 영구 콘텐츠/출처 증명이 아니다. shared publication schema·모듈 선언은 주 담당만 변경한다. 다른 OS 검증은 별도다.

## 인수 조건

같은 파일의 hardlink/rename identity 유지, 교체된 파일 구별, 실패는 identity 없음으로 fail-closed. 경로와 OS 소유권 검증 없이 삭제/완료 권한을 주지 않는다. 테스트는 임시 synthetic files와 Windows 전용 hidden host에서 실행한다. migration 호환성과 실제 publication 통합 시험 이후에만 해당 기능 완료를 표시한다.

## 검증과 handoff

구현 중. Windows 하위 helper와 주 담당 공유 인터페이스를 분리한다.

### 공유 publication 계약 연결

새 publication JSON version2는 NativeFileIdentity enum을 기록한다. 이전 version1 Unix (dev,ino) tuple은 deserialize 때 같은 Unix enum으로 변환하여 복구한다. SQLite adapter v5의 기존 JSON column을 유지하며 새 DB migration은 필요 없다. Unix identity는 O_NOFOLLOW로 연 File에서 읽으며 콘텐츠 identity로 표현하지 않는다. Windows은 secure helper 실증/연결 전 None fail-closed를 유지한다. cargo test -p dlpad-engine --lib download::publication:: --locked:3개 통과, partial crash/교체·충돌/legacytuple복구 포함.

### Windows publication 연결

Windows identity(path)는 final reparse를따르지않는regular handle을열고 handle finalpath가저장된canonicalpath와정확히일치할때만 nativeidentity를반환한다. Planv2에Windows volume/file ID를저장하며partial2artifact공개복구·동일크기교체거절·반복복구·matchingstaging정리를시험하는Windows전용unit을추가했다. 실제Windows실행은담당자에게요청했으며실증전완료로표기하지않는다. ancestor path삭제/링크의same-userTOCTOU와OneDrive/cloudreparse정책은남는다.

### identity 조회 실패의 공개 차단

prepare는nativeidentity관측실패를오류로반환하며fresh publication도persistedidentity가None이면거절한다. 이전에는non-recovery경로에서None끼리같다고완료될수있어강화했다. 손상/legacy nullidentity 계획의fresh/recovery모두final공개없음과staging보존을시험한다. 확인불가OS를크기만으로성공처리하지않는다.

### Windows 파일 목록 연결

기존Unix identity 문자열은변경하지않는다. Windows regular no-reparse handle과관측크기/수정시각을대조한뒤windows:v1:volume_serial:16bytehex관측을저장한다. Windows전용 외부rename rowid/identity유지시험을추가했으며실제실행은담당자에게요청했다. DBcolumn과migration은변경없다. 실패시None이며기존조건부유일성/경로점유검증을유지한다. Windowszero fileID지원여부는helper담당이추가검증한다.

### Windows 실행에서 발견한 시험 helper 제한

Windows clippy가libraryrename시험의row_id helper가Unixcfg에만있음을발견했다. helper를Unix/Windows공통cfg로수정하고hardlink중복·교체가과거기록을잘못이동하지않는시험도canonicalroot로두OS에연결했다. 실제실패로그는보존하며native4검증을다시요청한다.

### legacy 단일 journal Windows 복구 연결

기존planned_*단일결과도Windows regular no-reparse handle/nativeidentity와저장canonicalfinalpath를대조해복구한다. 같은크기외부파일을Downloaded로승격하지않는다. Windows전용SQLitereopen/idempotent복구/hardlink성공/foreign동일크기거절시험을추가했으며native실행을요청했다. Unixlegacy복구는기존정책을유지한다.

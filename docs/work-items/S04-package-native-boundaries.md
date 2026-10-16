# S04: package 설치 native 소유권·동시성·상한 통합

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R08
- 담당/수정 범위: root — tools.rs/native_files.rs/direct_download.rs/staging_recovery.rs/selection.rs/install_capacity.rs/공통문서; 하위 archive_install 및 package_inventory 결과 통합
- 선행 작업: boundedZIP 검증, package 공유계약

## 사용자 결과

설치가 기존 active를 유지하며 원 소유 디렉터리/파일을 교체한 foreign 자료를 지우지 않는다. 관리 archive/direct 설치가 같은 root lock과 전체상한을 공유한다.

## 계약과 범위

원 dir/File NativeFileIdentity를 보유하고 경로·type·reparse/no-follow를 대조한다. stage cleanup은 원 files만하며 filepin을각각닫아 Windows namespace 삭제를 완료한다. 기존journal v1을 유지하고 v2 component+2metadata≤10 기록을 추가한다. perroot 설치 lock, 전체install128 및fsentry2048, 기존stage8/512MiB를 적용한다. 새 Cargo dependency 없으며 기존nativeFileID API를 재사용한다.

## 인수 조건

source/foreign/prioractive 보존, 정상 비활성 설치 및 전체set 선택, dir/file/root 교체와 lock/capacity 실패 경로. Linux/Windows 실제 compile/test와공통검증. unknown crash leftovers는보존하며 부분publication/삭제를원자rollback으로표현하지않는다.

## 검증과 handoff

처음 도구 회귀는42PASS1FAIL1ignored: packagefixture AUX가Windows예약명이라 InvalidManifest. probe로교정한 뒤focused3PASS 및 root 전체47PASS1ignored를 확인했다. rootdir/file swap 원 소유권 회귀2개도 실제통과했다. 이후 lock/capacity/rootpin 및 제거 연결 변경은 후속 검증 중이며 이전PASS로소급하지않는다. runtime 실제version·공식trust/license·archiveHTTPSdownload·GUI·최종publication crash/3OS는별도게이트다.

## 추가 통합 범위

archive/direct는 동일 설치 lock을 공유하고 전체root의install128/fsentry2048 상한을 적용한다. manager 초기화의 원 root 디렉터리 pin을 marker/DB 생성 전 확보해 이후 namespace 교체를 거절한다. 신규 package_removal 모듈을 기존 remove(InstallManifest)에 연결하며 활성·foreign·내용변조와 원DELETEcap 권한 실패는 preflight에서 거절한다. delete는 부분실패 시 이미 삭제한 파일을 복원했다고 주장하지 않는다. 문서·코드 담당이 동결 보고한 뒤 root가 통합하며 source concurrent edit은 하지 않는다.

## 동시성 및 목록 비용 보강

select/remove/localimport도 동일root 설치 lock을 공유하며 검사→active 전환/삭제의 manager간 경합을 막는다. Windowsreadonly 파일은 삭제cap preflight에서거절한다. 목록은 metadata/nativeIDs/type/이름/실제size만읽고 package payload를전체hash하지않는다. explicitselect/activeuse/remove는전체component hash검증을유지한다. 128개×256MiB를목록worker가불필요하게읽는경로를분리했다. 원 root복제marker·전체설치128·두managerlock회귀3개를추가한 tool53PASS/1ignored 및 CPU전체Clippy0를확인했다. metadata목록분리/metadatareservedname는새후속source로 별도전체검증한다.

## 현재 전체 검증 관찰

2026-10-04 Linux 전체 collector는 기본 fmt/Clippy/test/release 및 CPU Clippy를 종료했고 CPU test 실행 중이다. Windows fresh guard build는 package_removal.rs의 FILE_DISPOSITION_INFO.DeleteFile에 정수1을 전달해 bool 타입 오류 E0308로 exit101했다. [실제 로그](../validation/evidence/managed-package-native-guard-first/build-guard.log)를 보존한다. Linux source freeze 종료 후 bool로 수정하고 새로운 두 OS snapshot에서 전체 검증한다. 현재 Windows package 구현을 검증 완료로 보고하지 않는다. 공통 문서 검사는529문서 PASS였다.

## Windows 분기 교정

Linux first snapshot 전체9명령 PASS/source_final 현재SHA 일치를 수정 전에 root가 확인했다. Windows DeleteFile bool 수정 후 guard build exit0, 전체collector fmt PASS 뒤 native_files.rs needless_return Clippy 오류로 중단했다. 두번째 Linux collector는 원runner PID/start/boot 대조 후 SIGINT 정상 취소하고 terminal failed를 확인했으며 PASS로 집계하지 않는다. Windows 전용 분기의 return을 제거한 최종 소스로 새 전체 검증한다.

## Windows 도구 회귀 교정 완료 범위

원 directory pin 시험은 열린 자식 writer를 갖는 Windows 디렉터리 rename 제한과 분리했다. 비어있는 원 디렉터리의 retained native pin을 유지한 rename→foreign 동일경로 교체→검사 거절→양쪽 데이터 보존을 실제 수행한다. 원 file pin의 same-name 교체는 별도 기존 회귀로 유지한다. [Windows 실제 회귀](../validation/evidence/managed-package-native-directory-pin-isolated/tools-test.log)는35PASS/0FAIL이며 ignore를 추가하지 않았다. payload readcap 수정 당시 Linux 도구53PASS/1ignored도 확인했다. 후속 HTTPS/sharedtransfer 변경은 이 근거에 포함되지 않으며 새 통합 검증한다.

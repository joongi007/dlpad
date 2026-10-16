# S04: Windows package read capability와 rename 공유 감사

- 상태: done
- 분류: review
- 우선순위: P0
- 요구사항: R03, R11
- 담당/수정 범위: metrics_recovery — 이 문서만
- 선행 작업: root 실제 Windows171PASS/7FAIL/2ignored 및 sourcefreeze70098

## 실제 원인과 snapshot

file_identity/windows.rs SHA8ce82270ad4f174355b1a0b22ced1aacc2cedcb54a25f58b69aff169a9d394c8의116줄은 read(true) 이후 access_mode(FILE_READ_ATTRIBUTES)를 지정한다. 명시 access_mode가 일반 read 권한을 대체하므로 해당 handle은 metadata를 조회할 수 있지만 FILE_READ_DATA payload 읽기 권한이 없다. native_files::open_regular 및 package_inventory::open_checked가 이 metadata helper를 사용한 뒤 Read/hash를 시도하여 OS5가 발생하는 경로는 실제 접근 계약과 일치한다. 성공하지 않은 Windows package 시험을 Linux 통과로 대체하지 않는다.

## 최소 root 수정 계약

기존 metadata-only open_regular_no_reparse의 이름·권한 계약을 보존하고 read_regular_no_reparse 또는 명시 access mode 내부 helper를 분리한다. payload용은 GENERIC_READ/FILE_GENERIC_READ를 요청하고 create/truncate/write/delete를 요청하지 않는다. final OPEN_REPARSE_POINT, regular disk type/reparse/nlink1/native FileId 검증과 원 directory ensure_regular_under를 유지한다. native_files read/hash·package record/component parsing에 readcap을 연결하고 ID-only 관측은 metadata cap을 유지한다. 일반 File::open follow fallback이나 cfg(ignore)는 채택하지 않는다. hash 후 original FileId와 현재 path identity 검증도 유지한다. source 전후 bytes/hash 및 외부 source 보존을 실제 Windows에서 확인한다.

## directory rename OS5의 별도 경계

native_files.rs SHA c090a99dab67612803e81e593914f6384fc28930d059d52e20024666b9b30077의 open_directory는 read(true)+BACKUP_SEMANTICS+OPEN_REPARSE_POINT이며 share mode 명시가 없다. Rust OpenOptions Windows 기본은 READ/WRITE/DELETE 공유이므로 현재 코드만 보고 share_delete 누락을 확정할 수 없다. direct_download.rs SHA5a4313cf288f84a54ce3d137581def3cdf5957f8f488248edb4445a02e79f97b의475 원directory replacement 시험은 OwnedDirectory.directory_pin과 files의 write-only clone을 유지한 채 rename한다. directory 자체의 폭넓은 GENERIC_READ 권한과 retained child writer/Windows rename 상호작용을 payload OS5와 분리해야 한다.

root는 디렉토리 pin을 필요한 FILE_READ_ATTRIBUTES 또는 FILE_LIST_DIRECTORY metadata 접근으로 최소화하고 명시 FILE_SHARE_READ|WRITE|DELETE를 적용하는 후보를 실제 native 회귀로 판단한다. payload File pins의 실제 desired access/share flags와 직전 OS error를 별도 기록한다. pin을 그냥 닫아 원 identity를 잃거나 replacement 시험을 건너뛰지 않는다. 원 handle 유지 rename→동명 foreign directory 생성→owned.check 거절→Drop foreign 및 moved original 보존의 기존 엄격 인수는 그대로다. Linux에서 rename 성공한 결과는 Windows 성공 증거가 아니다.

## 실제 native 인수

readcap에서 합성 payload bytes/hash 읽기 성공과 metadata cap의 동일 작업 OS5를 구분한다. final symlink/reparse/ancestor containment/비regular/hardlink/같은 이름 교체를 거절한다. package fullset inventory/select/remove의 actual Windows 회귀와 DB active 보존을 재실행한다. Windows allDELETE preflight는 다른 의미의 접근 제한이며 기존 heldhandle 거절·삭제0 시험을 유지한다. directory와 file replacement sourcepin 시험을 실제 Windows에서 실행하고 외부 synthetic sentinel 바이트를 보존한다. root 전체workspace native 재검증과 sourcebeforeafter SHA가 통과하기 전 결함 해결로 마감하지 않는다.

## 검증과 handoff

읽기 전용 감사와 source SHA 수집만 수행했다. Rust/Cargo/프로세스/OS 설정은 수정·실행하지 않았다. read capability 문제는 구체 코드 계약상 결함이고 rename 원인은 후보를 분리한 상태다. root가 소스수정 및 실제 Windows 재검증을 담당한다.

## 통합 담당 교정과 실기 결과

root는 metadata opener 자체를 바꾸지 않고 native_files payload opener에서 original attributes-only pin 및 no-reparse read-data handle의 동일 native ID를 대조했다. package metadata/hash 조회도 이 read capability를 사용한다. 원 directory pin은 FILE_READ_ATTRIBUTES와 명시 share read/write/delete로 최소화했다. Windows 실제 도구 회귀는34PASS1FAIL: 이전 archive 설치3개/package 선택1개/제거2개 실패가 해소됐다. 남은 original_directory_pin_preserves_replacement_directory는 열린 자식 파일을 가진 디렉터리의 rename 단계에서 ERROR_ACCESS_DENIED5다. 원본 보존 및 Windows namespace 공유 동작을 실제 회귀로 분리해야 하며 ignore/통과로 처리하지 않았다. [로그](../validation/evidence/managed-package-native-read-capability-first/tools-test.log)를 보존한다. 같은 소스 Linux 도구 회귀는 실행 중이다.

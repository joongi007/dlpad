# S04: 명시 local archive 비활성 설치

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R11
- 담당/수정 범위: diagnostic_cache — crates/engine/src/tools/archive_install.rs 및 이 문서
- 선행 작업: root native_files/OwnedDirectory/package checkpoint; package_inventory 검증

## 사용자 결과

명시한 source archive를 보존하며 검증된 component set을 비활성으로 설치한다. active 전환은 select_package에서 명시적으로 수행한다.

## 계약과 범위

install_verified_archive는120초/native sourcepin/preposthash/boundedZIP, component create_new+sync+원handlemode, checkpoint schema2, 두manifest, create-onlyhardlink publication을 수행한다. 전체디렉터리원자공개가 아니며 partialfinal crash gate 남음. source/외부/prioractive 보존. Cargo/sharedsource수정없음.

## 인수 조건

stored/multicomponent/inactive/select, cancel/hash/collision/foreign/DBcheckpointfail 보존시험. actualruntimeversion실행/HTTPS/catalog/GUI후속.

## 검증과 handoff

실제 source READY/frozen, 담당 파일 rustfmt exit0. native_files nofollow source pin과 전체 archive SHA를 추출 전·공개 전 재확인하고 120초/cancel을 적용한다. original stage의 component+두metadata마다 schema2 checkpoint를 남긴다. 파일은 원 output handle로 sync/mode 적용 후 readonly pin으로 교체한다. 최종 directory는 create_new이고 hardlink마다 원내용을 재검증한다. stage 원 links를 Drop 정리한 뒤 package_inventory의 strict nlink1/fullset 검증을 수행한다. 검증 통과 후 keep=true로 공개본을 보존하고 staging DB를 정리한다. 그 DB 정리 실패는 완성 비활성 package 보존+오류이며 active는 변경하지 않는다.

시험 4개를 작성했다: stored 및 multi-component 설치/명시 select, cancel/archivehash/existingdestination 및 active 보존, partialcomponenthash/DBcheckpointfail source·sentinel 보존, Unix source symlink 거절. 테스트 select_package 계약은 실제 &ArchivePackage로 재검토 후 수정했다. 실행 결과는 root 통합 검증 대기다.

한계: per-tool installed128/기존stage8·512MiB/scan2048을 재사용하며 전체root package128 정책은 root 후속이다. hardlink 다중 파일 공개는 whole-directory 원자 transaction이 아니다. final부분 crash의 durable publish journal/회수 gate는 남는다. Unix mode700/600을 적용했으나 Windows 도구 디렉터리 protected ACL 정책은 기존 helper 수준이며 별도 인수다. arbitrary runtime version 실행·공급 trust/catalog·HTTPS archive download·관리 GUI·3OS는 이 local inactive 설치로 완료하지 않는다.

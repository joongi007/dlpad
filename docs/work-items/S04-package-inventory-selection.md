# S04: 전체 package inventory와 명시 재선택

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R03, R11
- 담당/수정 범위: metrics_recovery — tools/package_inventory.rs 및 이 문서만
- 선행 작업: root ArchivePackage/InstalledPackage 및 module 연결

## 사용자 결과

설치된 archive package의 main/auxiliary/notice 전체 실제 크기·SHA를 다시 확인한 뒤에만 활성 버전을 재선택한다. archive hash를 main executable hash로 혼동하지 않는다.

## 계약과 인수

package_installed는 archive-package.json 부재만 None이며 malformed는 오류다. 두 manifest는64KiB 상한이다. package components+manifest.json+archive-package.json만 허용하고 최대10개와 한 overflow 검사로 foreign 항목을 거절한다. Unix nofollow/singlelink/native identity와 hash 뒤 원 path identity를 확인한다. Windows final reparse 거절 native open을 사용한다. 전체 최대256MiB worker-only I/O이며 cancel/deadline API가 없는 별도 inventory 계약이다. select_package는 전체 검증 후 active DB transaction을 갱신하고 실패시 active를 변경하지 않는다.

## 검증과 handoff

구현 중이며 Cargo/native는 root 전담이다. 제품 공유 schema·기존 selection·삭제 API는 수정하지 않는다. 회귀시험과 source READY 후 root 통합 검증이 필요하다. 외부 concurrent namespace 변조 전체 차단은 별도 native directory pin 계약과 구분한다.

## 첫 통합 실패와 fixture 교정

root focused tools 실행은42PASS/1FAIL/1ignored였다. full_set_reselection fixture의 AUX 이름이 Windows 예약 이름 금지 schema에 의해 InvalidManifest로 거절됐다. 이는 정책이 맞으며 시험 이름만 probe로 교정했다. native_files 원 directory pin 및 root selection 연결 코드는 보존했다. 최초 실패를 성공으로 덮어쓰지 않는다.

교정 후 실제 명령은 project-local PATH/CARGO_HOME/RUSTUP_HOME을 지정한 cargo test -p dlpad-engine --locked tools::package_inventory::tests -- --nocapture이며 exit0, 실제 focused3PASS다. 처음 두 시도는 rustc PATH 부재 및 sandbox target .cargo-lock read-only로 시험 실행 전 실패했다. 승인된 writable 실행으로 재시도해 성공했다. Windows native/fullworkspace 성공으로 확대하지 않는다. 최종 모듈 SHA25670e4b567eaf603eead0bf1a79715e2300a813cf98cec3d3e2809744a2bbbaac8이며 root native_files 통합은 유지했다.

## 목록 비용 경계 보강

recorded_package(path)는 directory/native IDs/type/exact names/두 manifest64KiB/actual component sizes/main 실행 권한만 확인하며 component contents는 읽지 않는다. 공통 verify_package_inner의 verify_contents=false를 사용한다. select_package와 package_installed는 verify_contents=true를 유지하여 전체 component SHA를 재검증한다. 따라서128개×256MiB package 목록이32GiB contents hash 읽기로 확대되지 않는다. root acquire_install_lock 추가는 보존했다. 같은 길이 변조 probe는 metadata inventory 성공/실제 선택과 package_installed 실패를 분리하는 회귀를 보강했다. foreign 항목은 metadata 목록도 실패한다. source 포맷만 수행하고 focused Cargo는 root 조정을 기다린다.

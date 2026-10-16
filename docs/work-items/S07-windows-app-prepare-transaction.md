# S07: 새 Windows 앱 폴더의 검증된 준비 트랜잭션

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R11
- 담당/수정 범위: root — crates/engine/src/app_install.rs 공유 PreparedRecord/PreparedBundle/PrepareFailure·module·scripts/windows_app_prepared_launch_probe.cs·문서·전체 검증; soak_recovery_audit — app_install/windows_caps.rs original child 생성/검증; diagnostic_cache — app_install/transaction.rs 새 coordinator; metrics_recovery — 독립 검토 및 root 승인 후 새 example fixture 담당
- 선행 작업: source548 설치 foundation 전체/native PASS; 진행 중 native 프로세스 없음

## 사용자 결과

명시한 로컬 manifest와 열린 source 파일을 새로운 Windows 전용 폴더에 bounded copy·hash·sync한 뒤 prepared record를 확정한다. 기존 설치·사용자 data/media를 변경하지 않는다. 실제 설치 UI/등록·active pointer·updater·upgrade·uninstall은 후속이다.

## 계약과 범위

prepare_new(manifest, files, target, cancel, deadline)는 descriptor를 재검증하고 원 HANDLE의 deny-write/delete read source pins를 유지한 뒤 원 root-relative CreateNew child HANDLE에 64KiB 단위로 복사한다. source·child 원 native ID/size/SHA·privateACL·exact namespace를 재검증한다. 성공은 PreparedBundle(private 원 root/child/record caps와 immutable view)이며 activation/trust가 아니다. ready metadata는 install-ready.json의 create-only <=64KiB PreparedRecord(domain/schema/root native ID/version/target/development flag/정확 entry size/hash/native ID)로 쓴다. flush는 power-loss directory durability의 증명이 아니다.

실패는 PrepareFailure(kind, target hint, optional 원 root cap, 원 child caps)를 반환하고 일부 생성된 원 객체를 보존한다. prefix/path를 삭제 권한으로 사용하거나 recursive cleanup하지 않는다. root 생성 후 검증 실패에는 원 cap을 얻지 못했을 수 있음을 공개한다. unknown/replaced child·기존 target·기존 marker는 덮어쓰지 않는다. 새 Cargo/schema dependency 없음; 공유 DTO는 root가 정의한다. 모든 I/O는 blocking adapter이며 UI thread 밖에서 사용한다.

## 인수 조건

실제 합성 PE·notice 및 실제 CPU copy/SHA/native ID·prepared record·전후 sentinel 보존, 기존 target refusal, 취소/기한의 mutation 전후 구분, 원 생성 child path 교체물 채택 금지, wrong source·extra namespace·write/hash 실패 보존을 시험한다. prepared current inspection은 원 handle/identity/hash/namespace를 재검증한다. actual initial launch는 copy 인수 뒤 separate private desktop 시험이며 같은 CPU의 복사는 version/schema upgrade 인수가 아니다.

## 검증과 handoff

shared API를 확정해 파일 담당별 병렬 구현한다. native build/test/GUI는 root가 source freeze 후 직렬화한다. 정상 Git 부재와 기존 .git 보존을 유지한다. signed installer/legal/clean-machine·durable startup recovery·정확 제거는 후속 게이트다.

최초 통합 검사: fmt PASS, Clippy는 unused original identity accessor와 PrepareFailure의 큰 Err(216B)로 FAIL하여 test/release 미실행이다. root가 반환 전 관측 identity와 원 cap identity를 직접 대조하고 failure의 retained root를 Box로 보유하도록 조정했다. 원 native capability 수명/보존 의미는 유지되며 failure당 작은 heap allocation 한 개가 추가된다. 원 실패 evidence를 보존한다. create_file이 native 생성 뒤 내부 검증에 실패하면 해당 child cap는 반환되지 않을 수 있으므로 retained_file_cap_count는 전체 생성 수가 아니다. 원 파일은 자동 삭제하지 않으며 target_hint는 권한이 아니다.

Actual launch observer는 기존 private settings collector를 같은 assembly에서 호출한다. 준비 fixture의 자연 종료로 writable caps가 닫힌 뒤, 원 copied exe의 SHA와 FileIdInfo(volume+16B)를 read-only SHARE_READ handle로 전후 확인하며 같은 pin을 두 실제 시작·테마 저장 재시작 동안 유지한다. helper는 global input/visible fallback을 추가하지 않는다. 동일 CPU copy의 재시작이며 version/schema upgrade나 installer 전체 완료가 아니다.

최종 통합: source550에서 Windows GNU fmt·Clippy(all targets, -D warnings)·workspace test·release build PASS, app_install 실제 22 tests PASS. evidence `windows-app-prepare-boxed-workspace-20261006-continue`, `windows-app-prepare-real-cpu-20261006-continue`에서 실제 7,917,568B CPU 복사 SHA f1a9c1ad13d65fa46f4cd08536bd96519e31c17fe68a49e64a397eba128bfb28, 정확한 3파일 namespace·719B marker·원 native ID·source/notice/sentinel·네 실패 인수를 root와 독립 검토했다. 이전 CPU 대비 크기 증가 0B, Cargo.lock unchanged.

`windows-app-prepared-private-launch-20261006-continue/root-launch-review.json`: 복사 원 exe 두 private 시작·자연 exit0·Job0·input desktop 불변 PASS. root가 실제 selected/restart PNG의 밝게 표시와 읽기 전용 SQLite integrity ok/theme Light 및 hash 불변을 확인했다. marker/payload 전후 보존 PASS. 별도 독립 검토는 PNG 제외 native receipts/DB를 확인했다. 실제 버전 업그레이드·설치 등록·정확 제거·서명·clean-machine 검증은 미완료다.

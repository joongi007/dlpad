# S07: Windows 원 HANDLE 기반 새 파일 공개 adapter

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R11 (ROADMAP S07 로컬 업그레이드·rollback)
- 담당/수정 범위: root 인터페이스·통합·검증; diagnostic_inputs는 /tmp 후보; crates/engine/src/app_install/windows_caps.rs, crates/engine/src/app_install/active_selection.rs, 이 기록
- 선행 작업: S07-local-upgrade-publication-experiment-20261012의 NT nt20/nt24 좁은 성공

## 사용자 결과

로컬 업그레이드에서 기존 설치 파일을 덮어쓰지 않고 소유 staged 파일을 새 revision 이름으로 공개할 수 있는 Rust 기반을 만든다. 전체 업그레이드·rollback 사용자 동작은 후속 통합이 필요하다.

## 계약과 범위

원 FreshRoot/CreatedFile HANDLE과 ACL/native identity를 재검증하고 단일 leaf의 create-only NT rename을 사용한다. collision과 검증 실패에 원 bytes를 보존한다. 공유 schema/Cargo/dependency 변경 없음. 소유권 검증·기존 share 정책을 약화하지 않는다. pathname 덮어쓰기 fallback, 기존 generation 삭제, updater 네트워크는 제외한다.

## 인수 조건

실제 Windows Rust 회귀에서 원 staged identity를 유지해 fresh publish 성공, 기존 이름 collision 거절·양쪽 bytes/ID 보존, malformed leaf 거절·원 caps 보존을 확인한다. 원 root 변경/권한 실패는 실패한다. sync_all과 rename만으로 power-loss durability를 주장하지 않는다. postpublication 검증 실패는 namespace 상태 불확실로 보존하고 삭제하지 않는다.

## 검증과 handoff

`CreatedFile::publish_fresh_name(&mut self, root: &FreshRoot, new_leaf: &str) -> Result<(), FreshNamePublishError>`는 기존 private active selection 생성 경로에 통합했고 아래 실제 Windows Rust 회귀와 전체 검사를 실행했다. C# 실험은 이 Rust adapter 검증과 별개이며 전체 버전 업그레이드·rollback은 아직 미완료다.

- root는 기존 active selection 생성에 production caller를 연결한다. canonical record를 active-selection.pending에 작성·검증하고 원 HANDLE로 final 이름을 공개한다. 기존 schema1/marker ID/retained failure caps를 유지하며 cleanup·delete 권한을 추가하지 않는다. 짧은 leaf도 native struct 최소 크기를 보장한다. 전체 업그레이드·rollback은 아직 아니다.

- 실제 Windows targeted89604 exit0/source_unchanged=true: app_install131회귀 성공. Unicode·한 글자 최소 ABI 길이·연속 새 이름 공개·원ID/bytes·외부 mutation 거절·collision/malformed 보존 및 기존 선택 기록 생성/취소/실패/cold inspection 포함. 취소·hook 실패는 final 이름 부재와 pending 원본 보존을 강화했다. 미완료 bytes를 final로 공개하지 않는다.
- 기존 private active selection production caller에 연결해 dead_code 억제 없이 사용한다. prepublication exact pending namespace를 먼저 확인하고 NT call 뒤 실패는 원 객체를 그대로 보존한다. 기존 schema/Cargo/의존성 변경 없음. 전체 검사와 별도 CLI 인수는 남아 있다.
- fresh-publication 전체43337/독립91460 exit0:43 harness/1157성공/0실패/30ignored 및 fmt/clippy/test/release 원로그·소스·실행 파일 대조 통과. 그 snapshot의 CPU9,708,032바이트, local-install978,432바이트. 이후 source-first UI 전체82075/독립86696에서도 동일 engine 통합을 포함해1157/0/30 통과했다. 현재 별도 새 payload CLI 인수와 postpublication OS 실패 주입, 반복 업그레이드·rollback은 남아 있다. 전체 P0 완료 증거로 사용하지 않는다.

- 현재 UI 설치 refresh가 exact5 payload와 새 manifest/binding을 생성했다. root가 최신1165 whole에 고정하여 /tmp/dlpad-fresh-publication-cli-smoke/collect.py로 새 소유 bundle의 prepare→cold inspect→동일 root AlreadyExists 거절을 검증한다. 기존 bundle/사용자 데이터를 변경하지 않으며 upgrade/rollback·중간 취소·rename 전 ID 관측의 완료를 주장하지 않는다. 담당 제품 파일은 root 전용인 기존 adapter 범위이며 이번 단계는 CLI 검증·이 문서 갱신이다.

- fresh CLI smoke72899 exit1. 실제 prepare/cold inspect는 성공했다. 동일 root 거절은 kind=AlreadyExists이지만 mutation_may_have_occurred=true여서 false 기대에 실패했다. 원 result/log/bundle을 보존하며 assert 이후 native ID/namespace 재비교는 미실행이다. diagnostic_cache가 /tmp 후보로 제품 계약과 collector 기대를 조사한다.

- diagnostic_cache의 읽기 조사: 원 factory가 생성 호출 직전 mutation=true를 설정하는 보수적 계약이다. 제품이 기존 root를 덮어쓴 증거는 없다. kind만으로 false로 낮추면 생성 성공 후 ACL/ID 오류를 오판할 수 있어 원 FILE_CREATE의 정확한 STATUS_OBJECT_NAME_COLLISION에 한정한 typed exclusive_collision 후보를 준비했다. 원 smoke 실패는 유지하며 후보 compile/native 인수는 아직 수행하지 않았다. 후보: /tmp/dlpad-existing-root-refusal-20261012/HANDOFF.md.

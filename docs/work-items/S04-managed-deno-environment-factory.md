# S04: 관리 Deno 원 환경 factory staging

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R05, R08, R11, R12
- 담당: metrics_recovery
- 수정 파일: `/tmp/dlpad-managed-deno-environment-factory/runtime_environment.rs`, `/tmp/dlpad-managed-deno-environment-factory/tools/managed_deno_classification.rs`, 이 문서
- 선행 조건: 실제 cache 원 native helper lock handoff PASS, 관리 Deno exact package classification

## 범위와 인수 조건

프로젝트 Rust/Cargo/shared 선언은 수정하지 않는다. root가 VerifiedManagedDeno reexport와 정책 wire/capture 연결을 별도로 담당한다. policy private binding과 Arc authority는 원 cache Mutex 및 실제 package proof를 보유한다. Clone은 Arc를 유지하며 PartialEq는 path/version/binding만 비교한다. 권한 상태가 같다는 주장이 아니다.

Windows 공개 ToolManager factory는 exact managed classification Some에만 원 cache를 생성·재검증하고 from_owned로 policy를 구성한다. external None은 cache 생성 0이다. 내부 malformed cache는 보존하고 실패한다. from_owned는 원 cache binding과 실제 package proof root를 대조하며 원 pins 수명을 policy에 유지한다. encode/immutable wire getter는 authority Mutex를 다시 잠가 capture guard와 deadlock을 만들지 않는다.

기존 new(path)는 binding/authority None으로 pure/direct tests를 유지한다. 생산 caller는 원 owned factory를 사용해야 하며 path constructor만으로 소유권을 주장하지 않는다. meaningful synthetic Windows factory 회귀를 작성하되 native/Cargo 실행은 root 통합 이후 수행한다. 실제 다운로드/EJS와 원 app crash는 기존 별도 인수를 대신하지 않는다.

## 구현 인계와 비용

실제 staging 사본에 private `binding: Option<CacheBinding>` 및 Windows `authority: Option<Arc<EnvironmentAuthority>>`를 추가했다. authority는 원 `Mutex<OwnedDenoCache>`와 원 `VerifiedManagedDeno`를 보유한다. `from_owned(cache, verified)`는 원 cache verify/binding, package 원 root pin nativeID, root path와 cache path를 대조한다. 원 cache root가 같은 문자열인 교체 파일이어도 package original nativeID와 다르면 거절한다. `VerifiedManagedDeno::root_identity()` read-only getter는 이 exact 비교만을 위한 staged 추가다.

immutable `cache_binding()`, `authority()`, `selected_executable()` getter를 제공한다. policy validate는 bounded binding/path 형식만 비교하고 authority Mutex를 잠그지 않는다. 실제 실행 직전 원 authority 확인/잠금 gate는 root capture 통합에서 담당한다. PartialEq는 cache_path/policy_version/binding만 비교하며 Arc/동적 lock 상태의 equality를 주장하지 않는다. Debug는 기존 고정 redacted 문자열을 유지한다.

`ToolManager::prepare_managed_deno_environment(&self, selected, cancel)`는 Windows 공개 factory다. classify None일 때 cache 생성 없이 None을 반환한다. Some이면 manager 원 root pin에 cache를 생성/reopen하고 원 package를 다시 verify한 뒤 authority 정책을 반환한다. 실패는 기존 fixed ToolError::UnsafePath로 변환하며 Cancelled는 기존 classify/verify 경로에서 유지한다. native cache Busy와 잘못된 owner record를 권한 완화로 재시도하지 않는다. 긴 classification/full hash 및 cache IO는 UI thread가 아닌 기존 worker에서 수행해야 한다. package 재검증은 기존 10초 예산/원 component 상한을 사용하며 추가 dependency는 없다. Arc policy clone은 원 proof와 잠금 수명을 유지하고 cache 내용을 복제하지 않는다.

새 synthetic Windows 3회귀는 external None/cache 없음, immutable binding/path 및 Arc clone 후 원 cache Busy/마지막 drop 뒤 재획득, partial owner record 보존/cancel cache 생성 없음, foreign-root package/cache pairing 거절을 포함한다. 기존 pure Debug 회귀는 binding None getter도 확인하여 Linux test 빌드의 cfg(test) API를 사용한다. 기존 fixture executable은 합성 4 bytes이며 이 factory 시험은 Deno 실행·version·EJS 호환성을 주장하지 않는다.

실제 실행한 명령: `.local-tools/cargo/bin/rustfmt --edition 2024 --config skip_children=true /tmp/dlpad-managed-deno-environment-factory/runtime_environment.rs /tmp/dlpad-managed-deno-environment-factory/tools/managed_deno_classification.rs` — exit 0. Cargo/native 실행은 0이다. 프로젝트 두 원 source SHA는 staging 시작과 마지막 비교에서 동일했다. `/tmp/dlpad-managed-deno-environment-factory/manifest.json`에 원·현재 프로젝트·staged SHA 및 bytes를 기록했다.

- runtime 원 SHA: `a79f6933ba9286ee108860440aa7f9b8162d0f3da187e7a92dd93847a5b9498d`; staged SHA: `f8ed3b442bd25c82de4f6952be297601ea6d50a28eb48a249478db359018b09f`.
- classification 원 SHA: `74a90c009283776185a723cf93b29a81e193605bd2f9e2c1dfa87ec38c7fa440`; staged SHA: `a6164fc5666a426eb3680df7ff4ea81b85f21eda161868ca9bf0358b0d07d2a4`.

현재 상태는 READY staging 인계이며 제품 게시·module reexport·wire/getter 연결·native compilation·실제 factory 회귀·다운로드 적용은 주 담당 통합 후 검증한다. 이전 native independent helper cache 잠금 PASS를 이번 factory 새 코드 성공으로 확대하지 않는다.

## 최신 source 재실행 확정

[최신5case](../validation/evidence/windows-managed-deno-product-corrected-20261004T121739Z/result.json)는 fresh helper/example build와 성공·paired-cookie·version-mismatch·metadata-failure·cancel 모두 exit0이다. root-review.json은 각 operation source SHA가 현재 source와 같고 원 receipt magic/256bytes/SHA, owner SHA, pending 부재를 다시 확인했다. 최신 Windows workspace 네 명령 PASS 근거와 결합해 이 합성 제품 연결 인수는 완료다. 실제 Deno/EJS/site·GUI·부모kill·공급/패키징은 후속 P0로 남는다.

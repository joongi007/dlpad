# S07: 검증 단계마다 journal을 기록하는 새 개발 bundle

- 상태: done (실제 단계 coordinator API 범위)
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — 공유 API·schema·통합·최종 검증; metrics_recovery — /tmp coordinator 계약/초안만.
- 선행 작업: journal readonly CLI·실제 explicit exit 인수. 실행 선행 인수 전 프로젝트 coordinator를 변경하지 않는다.

## 사용자 결과

새 bundle의 실제 검증 단계와 journal evidence를 순서대로 기록하며 중간 실패에서 원 bundle/journal 자료를 보존한다.

## 계약과 범위

AGENTS/README/PRODUCT/CONTEXT/R11/ADR0005 및 fresh user bundle·journal capability 계약을 읽는다. 검토 파일은 crates/engine/src/app_install/user_bundle.rs, install_journal.rs, 후속 분리 coordinator module과 facade·CLI·fixture·문서다. 현재 /tmp 독립 설계만 허용하며 실제 담당 파일/API는 root가 합의 후 확정한다. 완료된 bundle을 만든 뒤 단계 기록을 backfill해서 실제 중간 이력으로 표현하지 않는다. 원 bundle/journal 실패 capability를 보존하고 Drop 자동 삭제·과거 crash 소유권 복원·자동 resume/remove는 제공하지 않는다. 공유 Cargo/lock/schema 변경은 root 조정이다.

## 인수 조건

Prepared/Registered/Receipt/Active 각각 실제 검증 직후 journal을 기록한다. journal 기록 실패·bundle 생성 실패·취소·기한·foreign namespace에서 원 데이터/원cap을 보존한다. 완료 판정은 실제 journal과 current bundle의 manifest/active 대조로 검증한다. stage fingerprint는 caller evidence이며 historical ownership·권한이 아니다.

## 검증과 handoff

/tmp 계약 조사 단계, 코드 통합·실제 tests/native·제품 installer/UI 연결은 미완료다. 필요 private seam을 식별하고 현재 API를 유지하는 수직 단위를 root가 검토한 뒤 실제 실패 경로·Windows 전체 검증을 실행한다.

선행 explicit exit 인수를 완료했다. root 공유 계약 검토에서 PreparedBundle legacy verifier가 false cancel/새30s를 사용하는 것을 확인했다. 새 경로에는 transaction.rs의 private controlled verifier(caller c/deadline)가 필요하고 기존 public API는 유지한다. journal original writer는 이전 기록 bytes와 canonical을 대조한 뒤 다음 단계를 기록하고 최종 성공에서도 재검사하도록 install_journal.rs를 보강한다. 담당 파일을 journal_bundle.rs, user_bundle.rs, transaction.rs, install_journal.rs, app_install.rs와 해당 tests로 확정한다. Noop 기존 경로는 추가 단계 SHA/I/O/직렬화를 생략한다. root가 공유 seam을 조정하고 전체 검증한다.

actual coordinator 통합 source585 Windows 전체4검사825/0/13을 root raw 검토했다. 원 writer bytes 변형 시 다음 phase 미생성·원cap 보존, 실제4phase 후 Other fault에서 prefix/후속stage없음/양cap 보존, preCancel/deadline/wrongSHA/existingroot, actual success의 cold current binding이 통과했다. CPU1891b4…8,710,656B(크기동일)·CLIa7e6f3…910,848B(+14,336B), GNU/lock 보존. 이어 actual phase 직후 cancelAtomic을 설정하고 hook은 Ok로 반환하여 실제 다음 coordinator checkpoint가 Interrupted를 반환하는 4case 회귀와 실제 publicfactory nativeexample를 통합했다. 기존 경로는 observer None이며 추가 IO/serialize 없음. 새 source586 전체/native 검증은 진행 중이다.

새 source586 Windows 전체4검사826/0/13을 root raw 검토했다. 4개 실제 phase 동기화 직후 cancelAtomic 설정→hook Ok→다음 실제 coordinator checkpoint Interrupted와 양cap/정확prefix/후속stage미생성/source보존을 확인했다. CPU5bcc9f…8,710,656B·CLIc286ef…910,848B, GNU/lock 보존. 실제 앱 복사 native fixture의 example release와 raw/ID/SHA/canonical 원 기록 검토를 이어서 수행한다. 새 파일 담당은 crates/engine/examples/windows_journal_aware_bundle_fixture.rs, scripts/windows_msvc_journal_aware_bundle_example_build.py, scripts/windows_msvc_journal_aware_bundle_collect.py다.

source586 실제 public create_journal_aware_bundle native 실행0을 인수했다. root 추가 C#7명령·native19관측에서 original journal/nested bundle/source ID·SHA, 실제4stage fingerprint·canonical chain·manifest binding, 양existingroot/취소/기한 foreign 보존을 대조했다. outer ID와 source input root는 현재 post-Drop 관측 범위이며 원 outer creator cap/historical 권한 증명이 아니다. 원 fixture·전체4검사·example producer와 독립 raw6 검토를 보존했다. 증거: docs/validation/evidence/windows-msvc-journal-aware-native-real-20261006-continue/root-review.json 및 independent-review.json. 실제 명령은 scripts/windows_msvc_journal_aware_bundle_collect.py와 /tmp root readonly ID 검토 스크립트에 기록했으며 컴파일 포함7개 원 argv/log는 root-native-review에 보존했다. CPU5bcc9f…/8,710,656B·CLIc286ef…/910,848B; Cargo.lock/의존성/feature 변경 없음. 새 coordinator는 명시적 호출 시에만 동작하며 상시 worker 없음. 정식 installer UI/업그레이드/외부 crash/powerloss 복구·역사 ownership·삭제 권한·최신 GUI 비용은 미검증이다. 다음 작업은 S07-windows-journal-aware-bundle-cli.md다.

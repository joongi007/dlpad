# S07: 설치 journal과 명시 bundle의 읽기 전용 CLI 대조

- 상태: done (개발 readonly CLI 범위)
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — 공유 계약·통합·검증·문서; metrics_recovery — /tmp CLI 초안만. 프로젝트 CLI 파일은 root만 수정한다.
- 선행 작업: journal bounded/native fixture 인수. 현재 초안만 병렬 작성하고 선행 인수 후 통합한다.

## 사용자 결과

개발 CLI에서 사용자가 명시한 journal과 bundle의 현재 일관성을 읽기 전용으로 확인한다. 기록으로 과거 소유권·완료·삭제 권한을 복원하지 않는다.

## 계약과 범위

AGENTS/README/PRODUCT/CONTEXT/R11/ADR0005와 S07 journal API 계약을 따른다. 담당 파일은 crates/engine/src/bin/dlpad-local-install.rs, scripts/windows_msvc_install_journal_cli_collect.py, docs/development/windows-local-install.md 및 이 기록이다. 기존 13명령의 동작을 보존한다. UI 설치·자동복구·upgrade·제거·launch는 별도 작업이다. Cargo/lock 변경은 root만 조정하며 현재 필요 없다.

## 인수 조건

명시 journal-root/bundle-root와 bounded timeout을 파싱한다. 누락·중복·상대/UNC·unknown·다른 명령 옵션·0/601 기한을 거절한다. 원 readonly capability와 actual manifest/active 대조를 수행하고 current consistency만 보고한다. 취소·기한·미완료/unknown/torn·다른 valid manifest/bundle을 완료로 인정하지 않는다. 실패에서 원 파일·native ID·SHA가 보존되어야 한다.

## 검증과 handoff

/tmp 초안 단계이며 통합/컴파일/native CLI 인수는 미완료다. journal native 선행 인수 이후 root가 diff를 검토하고 전체 Windows fmt/clippy/test/release와 실제 합성 fixture/원 파일 대조를 실행한다.

선행 bounded/native journal의 actual1명령+root 추가9명령과 원ID/SHA/canonical manifest-active 대조를 root/독립 인수했다. 특정 torn parser 원인은 증명했다고 확대하지 않는다. root가 staged CLI14의 원/변경 SHA와 기존13 parse 흐름·readonly current consistency 계약을 검토해 통합했다. parser unit/Windows전체/native 명령 검증은 아직 미완료다. 성공 dataset은 별도 pristine journal을 새 factory로 만들며 unknown 기록을 제거하거나 복원하지 않는다.

정상 CLI 시험용 cli-current는 새로운 factory journal의 원 5파일을 보존한다. 기존 complete의 foreign 항목을 제거/복원하지 않는다. 같은 현재 source의 whole와 실제 fixture proof를 요구한다. 수집기 초안의 Windows OsString 이름 해석과 bounded bundle namespace/원 proof 핀 검토를 root가 요청했고 통합 전 보완 중이다.

첫 CLI 전체 검사는 fmt0 뒤 clippy101로 실패했다. 새 parser 테스트가 다른 tests 모듈의 비공개 args helper를 참조한 오류를 root가 확인했고 동일 테스트 안에 helper를 정의했다. 제품 parser와 테스트 거절 조건은 변경하지 않았다. 원 result/raw clippy/root-failure-review를 보존하며 새 전체 검증을 실행한다.

helper 수정 후 source583 Windows fmt/clippy/test/release 전부 exit0, 819passed/0failed/13ignored를 root가 raw 대조했다. 앱 CPU5cf193…8,710,656B는 직전 journal API 빌드와 동일하다. CLIaf1245…896,512B로 직전 847,872B보다 48,640B 증가했으며 journal 파싱·native 검사 코드가 CLI에 연결된 비용이다. 새 의존성/feature/상시 worker는 없다. 실제 native CLI 인수와 현재 앱 GUI 비용은 미완료다.

실제 CLI 인수 완료: 현재 source583/819·MSVC 전체4검사 root/독립 PASS. 새 fixture 생성1명령, CLI12명령(성공2/예상실패10), root 추가10 native명령·32원ID 대조, 원24파일 SHA/bytes·pristine/canonical manifest-active chain·missingroot 미생성을 확인했다. 최초clippy101·부분PENDING·torn parser원인미확정은 보존한다. root-review/independent-review는 windows-msvc-install-journal-cli-real-20261006-continue에 있다. 정식 installer·UI·자동복구·crash commit/제거 권한과 전체P0는 미완료다. 다음은 explicit process exit 후 prefix 보존 시험이다.

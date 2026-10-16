# S07: Windows 앱 설치 journal의 다음 수직 기능

- 상태: in-progress (분리 초안·실험; 제품 통합 전)
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — 공유 API/schema·통합·최종 검증; metrics_recovery — /tmp journal 초안; soak_recovery_audit — /tmp 권한·실패 검토. 프로젝트 파일은 root만 수정한다.
- 선행 작업: fresh bundle와 readonly 재검사. 현재 readonly 재검사 실제 인수 종료 전 제품 통합하지 않는다.

## 사용자 결과

설치 도중 중단된 실행을 재시작했을 때 현재 상태를 식별하고 원 데이터·외부 설치를 보존한다. 이 작업에서는 기존 설치를 자동 채택하거나 추측 경로를 삭제하지 않는다.

## 계약과 범위

README/PRODUCT/CONTEXT/R11/ADR0005와 기존 fresh root·registration·receipt·active 계약을 읽는다. 실제 다음 구현 단위를 원 생성 capability와 crash 후 읽기 관측의 경계에 맞춰 실험한다. 저장 schema·공유 Cargo/lock 변경은 root 조정이며 초안은 /tmp에서만 작성한다. 과거 managed-tool journal을 앱 설치 권한으로 재사용하지 않는다. updater·공개 배포·서명 결제는 별도 범위다.

## 인수 조건

bounded 상태·단계별 실패·취소·기한·미완료/unknown namespace 보존을 실제 코드로 시험한다. journal 상태만으로 삭제/교체/실행 권한이 생기지 않는다. 원 ID/SHA·최종 publication과 불일치하면 실패를 보존하며 검증하지 않은 recovery를 완료로 표현하지 않는다.

## 검증과 handoff

공유 API를 root가 통합했고 Windows 전체 검증을 실행 중이다. 실제 native fixture와 crash 인수·제품 installer 연결은 미완료다. 실제 코드·실험 결과를 근거로 다음 단위를 선택한다.

## 초기 통합

readonly bundle 인수를 끝낸 뒤 root가 journal 초안과 공유 API를 통합했다. 고정 5개 파일·각 64KiB 상한, 원 HANDLE 기반 재검사, 단계 순서와 이전 SHA, 실제 bundle manifest 및 active record 대조를 구현한다. caller가 제공한 단계 fingerprint는 과거 실행 증명이 아니다. 파일 sync_all은 디렉터리 전원 장애 내구성을 증명하지 않는다. 원 bundle 외부 root ID, crash 후 삭제 권한, 자동 복구·publication·UI·CLI는 아직 제공하지 않는다. Windows 실제 컴파일과 실패 경로 검증은 진행 중이다. Cargo/lock 변경은 없다.

담당 파일: `crates/engine/src/app_install/install_journal.rs`, `crates/engine/src/app_install/active_selection.rs`, `crates/engine/src/app_install.rs`, 후속 `crates/engine/examples/windows_install_journal_fixture.rs`, `scripts/windows_msvc_install_journal_example_build.py`, CONTEXT.md, ADR0005 및 이 작업 기록. native fixture는 owned TEMP에서만 실행하고 현재 전체 검사 종료 후 통합한다. 전체 검사 중 source 변경을 금지한다.

API 통합 실제 검증: `PYTHONDONTWRITEBYTECODE=1 python3 scripts/windows_msvc_workspace_probe.py --configuration /tmp/dlpad-msvc-workspace-stdout-plan.json --output docs/validation/evidence/windows-msvc-install-journal-workspace-20261006-continue --reuse-owned-target`의 MSVC fmt/clippy/test/release 전부 exit0. root와 독립 raw 검토로 818passed/0failed/13ignored·source582 확인. CPU5cf193…8,710,656B·CLIc7ff11…847,872B로 크기 증가 0B, 새 의존성/feature/상시 실행 worker 없음. GNU/lock 보존. 이후 native fixture 추가로 source583이며 새 전체 검사를 실행 중이다. 이 결과는 실제 crash·installer 완료 증명이 아니다.

실제 fixture 수집 파일 `scripts/windows_msvc_install_journal_collect.py`도 root 담당에 포함한다. 현재 source583의 새 전체4검사818/0/13은 raw root 검토를 통과했고 실제 example release 빌드를 실행 중이다. 수집기는 whole/config/producer/native argv·자연 Job 종료·원/보관 SHA를 확인하며 최종 root/독립 TEMP 기록 검토 전 REVIEW_REQUIRED를 유지한다.

첫 실제 native fixture는 1명령 자연 Job0·current binding/typed4와 root 추가9명령 native ID·SHA/canonical chain 대조로 범위 PASS다. 원 manifest 직렬화 field 순서를 root 검토에서 수정한 경과를 보존했다. torn 일반 파일 ACL 거절과 snapshot 읽기 전 상한을 아직 증명하지 못한 한계를 root/독립 기록했다. 후속 fixture는 공개 factory의 원 보호 phase ID를 유지한 own-file torn 변형과 65537B 파일을 읽기 전 거절/커서17 보존 검사를 추가했다. 이전 증거를 덮어쓰지 않고 새 실제 검증을 실행한다.

보강 source583 Windows 전체4검사818/0/13·example release·실제 native 1명령은 exit0이며 fixture JSON에서 읽기 전 65537B 거절/커서17과 공개 factory 보호 phase ID 유지 변형을 확인했다. root/독립 파일 대조는 진행 중이다. 실제 명령은 `scripts/windows_msvc_install_journal_example_build.py --proof .../windows-msvc-install-journal-bounded-workspace-20261006-continue/result.json --output .../windows-msvc-install-journal-bounded-example-build-20261006-continue` 및 `scripts/windows_msvc_install_journal_collect.py --proof .../windows-msvc-install-journal-bounded-workspace-20261006-continue/result.json --example-proof .../windows-msvc-install-journal-bounded-example-build-20261006-continue/result.json --example .../inputs/example.exe --output .../windows-msvc-install-journal-bounded-native-real-20261006-continue`이다. 각 prefix/torn/unknown 파일을 보존한다. explicit child exit(73) 시험은 /tmp 초안만 있으며 실제 crash/recovery는 미완료다.

# S11: Windows 앱 업데이트 독립 회귀 workflow

- 상태: review
- 분류: test
- 우선순위: P0
- 요구사항: R10, R13
- 담당/수정 범위: release_audit — 신규 `.github/workflows/update-regression.yml`, 본 문서만
- 선행 작업: app-update-fixture 기본 OFF feature/bin과 ci_app_update_fixture.py 원 Job capture 구현. Root가 commit/push/dispatch와 통합을 담당한다.

## 사용자 결과

앱 업데이트의 signed A→B 전환·cold start·실패 복귀를 전체 앱/설치기/GPU 릴리스 빌드와 분리해 수동으로 진단한다. 이 결과는 릴리스 CI, 실제 GUI 설치 인수 또는 공개 준비 완료를 대신하지 않는다.

## 계약과 범위

기존 ci.yml의 pinned checkout/upload, 정확한 Rust toolchain, lock fetch, static CRT, CI 전용 MSVC copy와 `ci_app_update_fixture.py`를 재사용한다. Windows 2022에서 engine fixture만 strict Clippy/build하고 고정 7 case를 원 Job 아래 실행한다. workflow_dispatch만 허용하며 임의 인자·path·실행 명령 입력은 없다. 권한은 contents:read이고 공개·배포·비공개키·운영 서명·기존 workflow/Cargo/제품 수정은 없다.

TMP/TEMP, 시험 profile/library, case parent는 checkout의 무시된 `.local-tools/` 아래 실행 ID/attempt별 새 폴더로 한정한다. 실패 로그/receipt와 남은 합성 fixture는 보존한다. 숨김 경로 포함 업로드는 명시한 합성 runtime root/capture/compiler manifest에만 한정하며 운영 signing root는 포함하지 않는다. 자동 GUI/visible desktop fallback은 없고 headless native 수명만 검사한다. 같은 ref의 수동 실행은 취소하지 않고 순차 진행한다.

## 인수 조건

- push/PR/tag/reusable 실행 트리거 없이 명시적 workflow_dispatch만 존재한다.
- source checkout의 기존 wrapper가 고정 feature strict Clippy/build와 7 case를 수행한다. 원 Job0·원 로그/receipt 검사를 우회하지 않는다.
- failure에서도 independent captures와 합성 오류 파일을 artifact로 남긴다. 시험 실행파일/시험키가 제품 ZIP에 들어가지 않는다.
- 실제 hosted 실행 전에는 성공으로 기록하지 않는다. Root가 workflow 통합/실행과 원 결과를 검토한다.

## 검증과 handoff

신규 workflow를 작성했다. `yaml.BaseLoader`로 GitHub `on` 키를 보존해 dispatch 단독 trigger, 읽기 권한, action SHA, 인증 미보존, Windows runner, 원 Job wrapper 고정 phase, owned TMP/profile/library와 always artifact 보존을 검사하여 PASS했다. `python3 scripts/check_docs.py --source-only`는 1,138문서 PASS이며 local-only evidence/외부URL 검증을 뜻하지 않는다.

고정 실행은 `python scripts/ci_app_update_fixture.py --root "$env:GITHUB_WORKSPACE" --directory ci-update-captures --phase build --seconds 600`, 이어 `--phase run --owned-parent "$env:DLPAD_UPDATE_CASE_PARENT" --seconds 450`이다. 기존 capture의 원 Job0/readers/raw hash/정확한 CASE_PASS 정책을 그대로 사용한다. artifact에는 capture 원본, compiler copy manifest, 이번 실행의 합성 runtime root를 함께 보존한다. 다중 path upload이므로 artifact 내부 capture 경로는 `ci-update-captures/`이며 기존 단일 path main CI updater artifact와 구분한다.

Windows hosted 실행 시간은 아직 측정하지 않았다. cold 준비·다운로드를 포함한 3~10분 예상은 추정이며, phase 상한은 build600초/run450초(합17.5분), 전체 job은25분이다. 원격 실행/성공은 Root의 별도 commit/push/dispatch와 실제 결과 검토 뒤 기록한다. 이 workflow의 성공만으로 fixed7 전체 CI나 installer/GUI/metadata 공개 인수를 주장하지 않는다.

동일 디렉터리 publication 수정을 진단할 때 retained-parent·collision·source 교체 실패 회귀를 함께 실행하도록 engine `created_relative_file` lib test를 추가했다. 전체 릴리스 gate 대체는 아니다.

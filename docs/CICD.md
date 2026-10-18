# CI/CD와 배포 계획

## 현재 공개 상태 — 2026-10-19

사용자의 공개 승인에 따라 [Windows v0.1.2 preview](https://github.com/joongi007/dlpad/releases/tag/v0.1.2)를 제공합니다. [v0.1.2 태그 릴리스 실행](https://github.com/joongi007/dlpad/actions/runs/39138176097)은 정확한 source `9ed25f1297b59bfc922e0659279a56a227614302`의 전체 CI와 같은 실행 원본 설치형·포터블 ZIP 패키징을 연결합니다. 공개 업데이트 메타데이터의 Ed25519 서명 검증과 Windows Authenticode 코드 서명은 별개이며, 이번 preview의 실행 파일에는 Authenticode 서명이 없습니다. 설정 → 앱 업데이트 → 조회 채널의 **미리보기 버전**에서 사용자 선택으로 새 서명 버전을 다운로드하고 재시작합니다. v0.1.0 사용자는 v0.1.2를 한 번 수동 설치해야 합니다. 실제 검수와 제한은 [업데이트 인수](work-items/S11-update-activation-20261018.md)를 확인하세요.

고정 부모 실행 파일의 보안 패치·프로토콜/호환성 epoch·신뢰 키 교체는 현재 UI 업데이트 범위 밖이며 수동 설치가 필요합니다. 운영 비공개키는 GitHub에 공급하지 않고, 로컬 서명 후 공개 메타데이터만 게시합니다. 공개키는 `crates/engine/src/app_update/release-public-key.json`이며 내려받은 공개키 파일을 앱의 새 신뢰 기준으로 채택하지 않습니다.

### 이전 v0.1.0 공개 인수

사용자의 공개 승인에 따라 [소개 페이지](https://joongi007.github.io/dlpad/)와 [Windows v0.1.0 preview](https://github.com/joongi007/dlpad/releases/tag/v0.1.0)를 게시했다. [태그 기반 전체 검증·동일 실행 ZIP·draft 생성](https://github.com/joongi007/dlpad/actions/runs/39101008422)과 [릴리스 갱신 relay → main Pages 배포](https://github.com/joongi007/dlpad/actions/runs/39103421782)이 실제 성공했다. 설치형·포터블 ZIP, SHA256SUMS, release-binding.json의 원본 CI 일치와 인증 없는 공개 다운로드 해시를 대조했다. 이번 a98 원 ZIP의 격리 Windows 설치·재검증과 설치 앱의 Light/Dark·재시작·자연 종료도 Root가 확인했다. 코드 서명 없는 preview이며 운영 서명키·앱 자동 교체는 제공하지 않는다. [공개 인수 기록](work-items/S11-github-publication-20261018.md).

아래의 날짜별 실행 기록은 당시 상태를 보존한다. 정식 서명·전체 자동 업데이트 목표와 현재 preview 공개를 구분한다.

## 현재 제공되는 자동화

`.github/workflows/ci.yml`은 PR/push/수동 실행에서 문서 링크·필수 문서·Rust 도입 계약을 검사한다. manifest가 없으면 native 검증이 skip임을 요약에 남긴다. manifest 도입 시 정확한 toolchain과 lockfile을 요구하고 Windows에서 fmt/clippy/test/release build를 실행한다. Linux/macOS는 `workflow_dispatch`의 `include_other_platforms` 선택으로만 실행하며 현재 Windows 우선 개발 범위와 구분한다. 각 OS build log는 artifact로 보존한다.

2026-10-17 사용자 요청으로 실제 dlpad의 빈 .git을 삭제 없이 초기화하고 origin을 https://github.com/joongi007/dlpad.git 으로 연결했다. Windows Git 기존 인증의 ls-remote는0/refs없음이었다. 초기 소스 commit `9955a95f3cbc80d5b922d73fc3414276bf7bbd2d`의 main push가 실제0으로 완료됐고 origin/main tracking·clean status와 원격 동일SHA를 확인했다. [첫 hosted Actions](https://github.com/joongi007/dlpad/actions/runs/38942190611)는 push 이벤트로 시작됐으며 이 기록 시점 in_progress/결론미정이다. 결과 기록을 위한 docs 후속 commit은 새 실행으로 갱신될 수 있으므로 [현재 Actions](https://github.com/joongi007/dlpad/actions/workflows/ci.yml)에서 최신 head를 확인한다. 기존 Windows 로컬 whole37은1,685 PASS이며 원격 Actions 성공으로 대체하지 않는다. workflow 파일 존재와 실제 CI 운영 성공, 공개 CD를 구분한다.

Git에는 소스·문서·native assets·vendor·lock을 넣고 로컬 증거/ZIP/도구/cache/계정 설정을 제외한다. CI 문서 검사는 `python3 scripts/check_docs.py --source-only`를 명시한다. 이 모드는 source export의 제외 정책에 해당하는 링크만 따로 집계하며 원격에서 runtime 증거를 검증했다고 표시하지 않는다. 기본 로컬 검사는 그대로 엄격하다. [소스 checkout 계약](SOURCE-CHECKOUT.md), [초기 연결 작업](work-items/S11-github-initial-push-20261017.md).

문서 검증은 Python stdlib만 사용한다. Rust 도입 후 실행 명령은 AGENTS와 같다. 처음에는 선택된 기본 feature 구성만 테스트하고, renderer 등 feature 조합은 S00 결과에 따라 명시적 matrix로 추가한다. 무조건 `--all-features`를 사용해 상호 배타적 backend를 동시에 켜지 않는다.

## 최초 hosted 실행의 실패와 수정 검증

첫 docs job19명령과 Windows source/format은 실제 통과했으나 run38942396783의 Lint는125로 실패했다. 원 artifact는 Clippy 자체0·두 로그reader종료·원Job자손5초내자연종료미확인/강제정리를 기록했다. Cargo 오류로 바꾸어 설명하지 않는다. 기존 통과 로컬 CI와 같은 dev/test debug0·incremental0 설정과 상세 실패/잔류프로세스/로그 tail 출력을 추가하고 실제 새 hosted 실행에서 검증한다. 테스트 assertions·overflow·release 프로필과 자연Job종료 게이트는 유지한다. 실제 c7 재실행38946356371도 Clippy0 뒤 vctip.exe 잔류로 실패했으므로 프로필 일치만으로 해결됐다고 보지 않는다. 다음 수정은 원 설치·registry를 보존하고 runner 임시 폴더의 byte-identical MSVC bin 복사본에서만 vctip.exe를 제외한다. 공식 [Developer PowerShell](https://learn.microsoft.com/en-us/visualstudio/ide/reference/command-prompt-powershell?view=vs-2022)의 원 INCLUDE/LIB를 사용하며 CC/CXX/AR/linker와 PATH는 복사본을 선택한다. 복사는 최대4096파일/개별512MiB/전체2GiB로 제한하고 실제 bytes/hash를 ci-msvc-copy.json artifact에 기록한다. Rust/앱 의존성·배포 크기 비용0이며 runner 임시 디스크·복사 I/O만 추가된다. 실제 새 hosted 완료 전에는 해결로 표시하지 않는다. [실패 작업](work-items/S11-hosted-ci-lint-125-20261017.md).

## 실제 hosted 전체 통과 — 2026-10-17

[run39040643416](https://github.com/joongi007/dlpad/actions/runs/39040643416), source commit5d4cf73bf2e642c378e564ff7eae2ae51fedf430의 docs와 Windows2022 native job이 모두 성공했다. fmt·workspace CPU Clippy/test·workspace release·대체/CPU/GPU renderer·installer/companion compile·source/lifecycle binding과 graph/artifact 기록을 실행했다. 원 test stdout은 주 harness1684PASS/0FAIL/43ignored와 별도 filtered child1PASS다. 7개 capture 모두 원 Job 자연종료/worker0/reader종료·강제정리0을 확인했다.

첫125는 vctip.exe 잔류였고, 그 뒤22FAIL은 동일 폴더 publication에 불필요한 target-parent reopen 및 marker 시험의 주입 방식 문제였다. 원 설치를 보존한 CI-private MSVC와 원 file HANDLE의 same-parent rename으로 수정했다. [종료125](work-items/S11-hosted-ci-lint-125-20261017.md), [Windows 공유 실패·최종 증거](work-items/S11-hosted-windows-file-sharing-20261017.md). 이 성공은 실제 GUI/별도 native fixture43개·서명·정식 패키징·공개 CD 완료가 아니다. 이후 문서-only commit의 재실행 상태는 위 source 검증과 구분해 Actions에서 확인한다.

## 목표 파이프라인

| 시점 | 수행 | 결과/게이트 |
|---|---|---|
| PR | 문서·format·lint·unit·adapter fixture·Windows release compile(다른 OS는 수동 선택) | 코드 검증, 실패 시 merge 불가 |
| main | 위 검사 + 원본 입력 archive + 설치형/포터블 후보 ZIP | 비서명 preview 후보14일, 원본 입력7일 보존, 정식 서명 릴리스 아님 |
| nightly | migration·장시간 soak·프로세스 cleanup·memory sample | 회귀 보고; 실제 사이트는 별도 선택 job |
| v* tag | 버전 일치·전체 검사·target별 build·SBOM·digest·패키징 | draft release 후보 |
| release 환경 승인 | 서명·notarization·설치/업데이트/제거 시험 | 서명 검증된 artifact만 공개 |
| publish | checksum·라이선스·변경 내역·지원 OS 표 게시 | 동일 검증 artifact 승격, 재빌드하지 않음 |

2026-10-18 `.github/workflows/release.yml`에 Windows preview draft 자동화를 구현했다. 정확한 버전 태그→reusable CI 전체 성공→동일 SHA/run artifact ID 다운로드→실행 파일 재빌드 없이 notice·runtime·원 로그 검증→설치 ZIP/포터블 ZIP→미공개 draft 순서다. 실제 v0.1.0 hosted 패키징·draft·공개를 검증했다. 정식 서명·업데이트 교체 인수는 별도다. 인증서·저장소·패키지 형식과 앱 entrypoint가 확정되기 전에 비어 있는 릴리스를 생성하지 않는다. 최초 공개 전에는 maintainer가 release 환경 권한·승인 정책을 설정한다.

## OS와 패키지

| target | 초기 패키지 후보 | 필수 확인 |
|---|---|---|
| Windows x86_64 MSVC | zip portable + per-user installer | 코드 서명, C runtime, 한글/긴 경로, Job Object, uninstall tool 선택 |
| macOS aarch64 + x86_64 | 각각 .app/.dmg, 필요 시 universal | Developer ID, hardened runtime, notarization/stapling, Intel/ARM 실기기 |
| Linux x86_64 GNU | tar archive 우선, AppImage 또는 deb 검토 | 최소 glibc baseline, Wayland/X11, font/DBus/tray, 배포판별 실행 |

Windows/Linux ARM64는 후속 후보. 현재 CI host matrix는 target 지원 선언과 다르다. 예컨대 macOS host 한 개의 build로 Intel과 ARM 모두를 검증했다고 주장하지 않는다. 설치 패키지·OS 최소 버전·서명된 외부 도구 실행을 release matrix에서 추가 검증한다.

## 공급망과 비용

Actions는 commit SHA에 pin하며 업데이트 PR로 변경한다. 기본 권한은 contents:read, fork PR에 signing secret을 주지 않는다. PR 코드에 비밀을 주는 pull_request_target 실행은 쓰지 않는다. release 자격 증명은 별도 environment에 한정한다. 외부 도구 다운로드 manifest에도 target·version·digest를 pin한다.

S04/S07에서 cargo audit/deny 또는 동등 검사, license inventory, SBOM, 외부 바이너리 라이선스·출처·해시 검증을 추가한다. FFmpeg 빌드 옵션에 따른 재배포 조건은 실제 공급물을 기준으로 확인한다. signing·notarization에는 외부 계정/인증서 준비가 필요하다.

릴리스 시험은 깨끗한 환경 설치 → 선택 도구 설치 → 샘플 다운로드 → 앱 업데이트 → DB migration → 제거(미디어 보존)를 수행한다. rollback은 이전 앱과 DB schema 호환성을 확인하며 migration 전에 backup을 만든다. 관리 도구 업데이트는 앱 업데이트와 별도로 rollback 가능해야 한다.

도구를 제외한 앱 크기만 보여주지 않고 최소 설치, 추천 도구 포함, 선택 구성요소별 설치 크기를 게시한다. CI cache는 성능 최적화일 뿐 재현성을 대체하지 않는다.

## 소개 사이트 (2026-10-18)

`site/`는 Rust 제품 UI와 별개의 정적 소개 페이지다. `scripts/build_site.py`는 명시한7개 파일만 새 출력 폴더에 생성하며 source/evidence/cache를 통째로 배포하지 않는다. 공개 GitHub Releases에서 installer·portable asset 쌍과 정확한 저장소/파일명/크기를 검사하고 stable을 우선한다. stable이 없으면 공개 preview라고 명시하며 draft는 다운로드로 노출하지 않는다. 방문자의 브라우저는 자체 `release.json`만 읽으며 GitHub API·외부 폰트·분석 서버에 요청하지 않는다.

`.github/workflows/pages.yml`은 PR/main에서 사이트를 빌드하고 artifact로 검수할 수 있게 한다. `DLPAD_PAGES_ENABLED=true` 저장소 변수를 명시적으로 켠 뒤 main 자동 배포와 공개 Release 이벤트의 다운로드 갱신이 동작한다. GitHub Pages source는 Actions와 github-pages environment로 설정한다. 사용자의 공개 승인 후 저장소 변수와 workflow Pages를 활성화했고 실제 소개 페이지·preview Release 게시와 main 배포의 다운로드 갱신을 검증했다. 후속 태그는 draft 검수 후 게시한다. 최초 published 이벤트의 tag 배포 거절을 main 갱신 relay로 교정했고, 실제 수동 relay와 그 토큰이 요청한 main 배포를 검증했다. 후속 새 태그의 published 자동 연결은 다음 릴리스에서 확인한다. GitHub 공식 [Pages workflow](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages)를 기준으로 Actions SHA를 공식 refs API에서 확인했다.

사이트 검증은 `python3 scripts/test_build_site.py`와 headless-only `scripts/test_site_browser.cjs`다. 후자는 SITE, 새 OUTPUT, 로컬 playwright-core 경로, Chromium 경로를 인자로 받으며 127.0.0.1의 임시 서버와 브라우저를 종료한다. Playwright는 개발 검수 도구이며 제품·공개 페이지 의존성이 아니다.

Windows hosted 후보는 `-C target-feature=+crt-static`으로 C runtime을 정적 링크하도록 구성했다. 기존 로컬 Windows 후보의 검증된 방식을 따르며 별도 Visual C++ 재배포 도구 설치를 사용자의 필수 선행 조건으로 만들지 않는다. 앱 크기는 정적 runtime만큼 늘 수 있으므로 실제 산출물 크기를 기록한다. packager는 실제 PE import를 검사해 공급하지 않은 vcruntime/msvcp 등 비시스템 DLL이 남으면 실패하며, build flag만으로 깨끗한 PC 인수를 완료 처리하지 않는다. 다른 OS에는 이 플래그를 적용하지 않는다.


## 2026-10-18 소개·후보 ZIP·업데이트 연결

source32607f7의 [site39057841523](https://github.com/joongi007/dlpad/actions/runs/39057841523)와 [CI39057841587](https://github.com/joongi007/dlpad/actions/runs/39057841587)는 모두success다. 실제 Windows 원notice 수집과 원artifact 보관까지 확인했으며 그 source에는 ZIP step이 없었다. 후속 main/PR CI는 GPU 검증 뒤 같은 원binary/source/raw lifecycle/notice를 재검증해 RUNNER_TEMP 새폴더에 installer ZIP/portable ZIP·SHA256SUMS·binding·notes를 만들고 windows-preview-candidates-SHA로14일보관한다. 재빌드는 없다. Pages 공개는 repo variable DLPAD_PAGES_ENABLED=true가 필요하며 아직 설정/게시하지 않았다.

release workflow의 candidate-only는 main에서만 허용하고 tag에서의 후보 전용 요청은 명시거절한다. 실제tag와Cargo버전이 정확히 맞을 때만 미공개preview draft로 이어진다. 취소한 실행은draft를 만들지 않는다. 선택적 Linux signer job은 repo/org-level DLPAD_UPDATE_SIGNING_ENABLED=true와 release-signing environment의 기존 secret DLPAD_UPDATE_PRIVATE_KEY_PEM 및 variable DLPAD_UPDATE_KEY_ID가 준비된 경우에만 활성화된다. 환경 수준 변수만으로 job-level enable조건을 대체하지 않는다. signer17회귀·versiongate5회귀를 CI에 포함했다. 키가없으면 sign은skip하고 기존 unsigned development표시를 유지하며, 활성화된sign이실패하면 unsigned draft로fallback하지않는다. 실제signer workflow·운영키외부전송·첫draft/publish는미실행이다.

앱은 공개버전 조회 UI와 정확한 Ed25519 bytes 검증/Windowsowned저장을 구현했지만 운영trustroot/교체helper/rollback은미완료다. release public-key aid를 제품trustroot로 자동수용하지 않는다. [통합 및 검증](work-items/S11-app-updater.md), [preview packaging](work-items/S11-auto-release-20261018.md), [서명 후보 경계](adr/0012-signed-app-update-candidates.md).


source49c3812의 [CI39060610796](https://github.com/joongi007/dlpad/actions/runs/39060610796)는 ZIP 생성/보관까지 전체success다. Root가 원artifact SHA·installer7/portable6·동일payload를 대조하고 실제GUID임시Windows설치·재검증·원파일/기록hash·자연종료를 확인했다. 공개Pages는 repo Pages source를 GitHub Actions로 구성하고 DLPAD_PAGES_ENABLED=true를 설정하는 공개 단계가 남으며 현재는 build/artifact만제공한다. 태그/미공개draft/publicRelease/운영signing도 아직미실행이다. [실제산출물·원검수](work-items/S11-auto-release-20261018.md).


## 2026-10-18 최신 Windows 후보 인수

앱 소스c5963b5의 [전체 CI39066254029](https://github.com/joongi007/dlpad/actions/runs/39066254029)는 docs/Windows CPU feature 포함 strict lint·workspace test/release·대체 렌더러·CPU/installer/companion·GPU compile·원 installer/portable ZIP 생성·artifact 보존까지 success다. Windows의 별도 CPU lint/tests label skip은 중복 방지이며 실제 capture 명령은 ci_whole_receipt.py의 --features dlpad/cpu를 포함한다. GPU 단계는 컴파일이며 hosted GPU runtime/시각시험을 뜻하지 않는다.

Root는 artifact12113907821의 원 SHA를 대조하고 exact installer7/portable6 member와 manifest5·CRC/SHA/원 binaries/source/run identity를 직접 검증했다. 원 ZIP의 가상 Windows 설치·재검증과 설치 앱의 Light/Dark 재시작·100/200% 설정/키보드 focus-scroll·정상 종료도 확인했다. 별도 ZIP→staging→새 고정 bundle 시험은 test-only 키를 사용했으며 운영 서명/자동 실행 인수가 아니다. 자세한 경로와 제한은 [통합 기록](work-items/S11-site-release-integration-20261018.md)과 [설치 준비](work-items/S11-update-bundle-preparation-20261018.md)에 있다.

검증한 c5963b5는 origin/main으로 normal fast-forward push했다. 먼저 repository 변수의 실제 읽기 접근과 Pages 공개 비활성을 확인했으며 태그·공개 Release·Pages 설정·운영키는 변경하지 않았다. 이 뒤 문서 전용 결과 갱신은 앱 소스를 변경하지 않으며 원격 자동 후속 실행과 위 완료 run을 구분한다. 공개 배포는 사용자 초기 별도 승인 범위에 따라 대기 중이다.


## Release 이벤트와 Pages 배포 ref — 2026-10-18 교정 완료

v0.1.0의 실제 published run39102954680은 사이트 build 성공 뒤 github-pages 환경이 tag ref를 거절했다. main 전용 환경 정책은 유지한다. pages-release.yml은 공개 이벤트에서 pages.yml의 main dispatch만 요청하며, 사이트 workflow는 main만 배포한다. 운영 서명키·환경 보호 규칙을 변경하지 않는다. 실제 relay hosted 회귀와 공개 release.json 확인은 [교정 작업](work-items/S11-pages-release-main-routing-20261018.md)에 기록한다.


## Windows updater 지원 v0.1.2 (2026-10-19)

고정 운영 공개키·부모/자식 수명·DPAPI 선택 기록·실패 복귀를 구현했다. 실제 상태/명령은 [통합 작업](work-items/S11-update-activation-20261018.md)에 기록한다. 과거 단락의 미구현 표시는 당시 단계이며 공개v0.1.0의 기능 설명이다. 기존 fixed7 원본 capture graph는 유지하고 기본OFF `app-update-fixture` 별도 bin/독립 CI capture로 signed A→B/cold B·시작 실패·crash·hash 오류·취소·busy·stale를 검증한다. 합성 키와 시험 bin은 제품 ZIP에 포함하지 않는다.

선택적 release-signing job은 생성한 공개키가 저장소 `crates/engine/src/app_update/release-public-key.json`과 정확히 같아야 metadata를 올린다. 운영 비공개키의 GitHub 공급은 아직 하지 않았다. 로컬 DPAPI 복호화·서명 후 공개 metadata 세 파일만 해당 draft에 추가하는 공급 경로를 제공한다. 앱의 Ed25519 신뢰와 Authenticode는 다르다. 첫 지원 버전은 수동 설치하며 기존 v0.1.0 태그/파일을 덮어쓰지 않는다.

프로필 epoch1에서는 후보의 읽기 전용 사전 검사와 역방향 호환성으로 복귀하고, 실행 후 사용자 변경을 과거 DB snapshot으로 복원하지 않는다. 위 일반 릴리스 시험의 backup 목표가 자동 DB overwrite 정책을 뜻하지 않는다.

# CI/CD와 배포 계획

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
| main | 위 검사 + 개발용 archive | 비서명 dev build, 7일 보존, 정식 installer 아님 |
| nightly | migration·장시간 soak·프로세스 cleanup·memory sample | 회귀 보고; 실제 사이트는 별도 선택 job |
| v* tag | 버전 일치·전체 검사·target별 build·SBOM·digest·패키징 | draft release 후보 |
| release 환경 승인 | 서명·notarization·설치/업데이트/제거 시험 | 서명 검증된 artifact만 공개 |
| publish | checksum·라이선스·변경 내역·지원 OS 표 게시 | 동일 검증 artifact 승격, 재빌드하지 않음 |

tag/release 자동화는 이 문서의 계획이며 아직 workflow로 구현하지 않았다. 인증서·저장소·패키지 형식과 앱 entrypoint가 확정되기 전에 비어 있는 릴리스를 생성하지 않는다. 최초 공개 전에는 maintainer가 release 환경 권한·승인 정책을 설정한다.

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

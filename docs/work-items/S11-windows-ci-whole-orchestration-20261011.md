# S11: Windows CI raw capture와 whole 기록 연결

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R11, R13
- 담당/수정 파일: root .github/workflows/ci.yml·공유 CI 인터페이스·문서·통합 검증; diagnostic_inputs는 /tmp의 ci_whole_orchestrate.py·test_ci_whole_orchestrate.py 후보만 작성한다. 동일 프로젝트 파일을 병렬 수정하지 않는다.
- 의존성: S11-windows-ci-capture-producer-20261011의 7개 원 argv capture·Job lifecycle·strict file ID 바인딩

## 범위

Windows CI의 Formatting 전 tracked source snapshot과 companion release 후 snapshot을 실제 7개 capture/lifecycle 및 target/release 4개 exe에 연결한다. 기존 Cargo 실행을 중복하지 않는다. root가 workflow 연결·pure/native 검증을 수행한다. 원격 저장소/계정/공개 게시·서명·updater runtime은 이번 작업에 포함하지 않는다.

## 인수 조건

변경된 source/lock/tracked 목록, missing/failed/forced/reader 미확인 lifecycle, raw byte 불일치, 잘못된 argv/identity, alias/reparse/duplicate JSON과 기존 출력 덮어쓰기를 거절한다. 정확한 합성 경로만 unsigned format receipt를 생성하며 remote_execution_authenticated/release_ready/legal_coverage_complete는 false다. 실패 raw 자료는 보존한다. 실제 remote Actions가 없으면 완료로 보고하지 않는다.

## 근거와 진행

읽기 감사에서 현재 7capture는 workflow에 연결됐으나 before/after source와 whole constructor 연결이 빠진 것을 확인했다. root가 최소 수직 단위를 기록했으며 후보 작성·실제 통합/시험은 진행 중이다. 사용자 순서인 기능 완성 → Git/CI → updater를 유지한다.

## root 통합 및 검증 시작

root는 후보 2개 script와 workflow를 통합했다. before는 fetch 후 formatting 전에 실행하며 컴파일 전 기존4 exe를 삭제 없이 거절한다. after는 companion release 뒤 원7개 argv/lifecycle/raw bytes와 final actualGit/source를 재검사하고 whole.json을 마지막 create-only로 생성한다. Job close 실패도 원 worker kill/wait와 모든 reader join을 끝까지 시도한 뒤 실패한다. 하위 담당의 pure18개·분리된 /tmp 실제 Git clean/dirty 시험 보고는 root 검증 전이며 Windows/native/remote를 수행했다는 뜻이 아니다. graph/registry 실제 원파일 결합·원격 CI 인증·서명/배포/updater는 완료로 표시하지 않는다.

## root 실행 결과

Linux `python3 -m unittest discover -s scripts -p test_ci_*.py`: 66 passed. root가 현재 통합 모듈로 별도 /tmp/dlpad-ci-git-owned-fzribko9의 실제 Git clean before/after 및 dirty 거절을 재실행했다. Cargo·Windows Job·원격 인증은 이 합성 capture/binary 실험에 포함하지 않았다. 실제 Windows66 첫 collector는 suite count assertion에서 실패하여 테스트를 실행하지 못했다. 원 로그를 보존하고 모듈 구성 교정 후 재실행한다.

PyYAML 실제 workflow parse와 exact Windows-only before/after 조건·순서 및 원7capture 연결을 확인했다. Windows66 v2는 실제 module import 상세에서 LLVM Python zlib 부재로 registry module 로딩 실패(전체54로딩/0실행)를 확인했다. 프로젝트 전용 공식 CPython으로 재검증 준비 중이다. 시스템 Python이나 보안 설정을 변경하지 않는다.

## 프로젝트 전용 검증 Python

공식 CPython3.14.8 Windows embeddable manifest exact URL/SHA로 새 .local-tools/windows/cpython-3.14.8-embed-amd64를 준비했다. 원 ZIP·manifest·license·runtime entries를 보존하고 root 전체 SHA/크기를 재검사했다. ZIP16MiB/flat256entry/entry16MiB/expanded64MiB·CRC·aliases/reparse/nooverwrite 검사를 적용했다. 시스템 registry/PATH/site/pip/기존 LLVM 도구는 변경하지 않았다. 앱 Cargo/runtime 의존성은 추가하지 않았다. 실제 native import·CI66는 다음 실행에서 검증하며 원격 인증/서명/전체법적인수로 표시하지 않는다.

개발 검증 도구 비용: 압축12,602,386B, 38 runtime files 총24,407,939B. [공식 manifest](https://www.python.org/ftp/python/3.14.8/windows-3.14.8.json)의 pythonembed-3.14-64 항목을 사용하며 일반 embed 파일과 SHA를 혼동하지 않는다. 공식 origin checksum 바인딩이며 별도 서명 인증 주장 없음.

## 실제 Windows66와 fixture 권한 한계

공식 Python의 version/64bit/zlib/gzip/ctypes/sqlite import·roundtrip preflight와66 loading은 통과했다. 실제60 passed,5 fixture creation ERROR Win32 1314,1 기존skip 결과를 보존했다. production validator를 바꾸지 않고 test-only symlink 생성 helper에 Windows1314만 명시skip하도록 통합했다. 다른 OSError는 재발생한다. 혼합 시험의 nonregular/nooverwrite/casealias 검사는skip전에 수행하도록 보존했다. 6 exact ID를 collector에서별도기록하고 symlink coverage_complete=false를 유지한다. Linux실제66/native 재검증을 이어간다.

root final local검토: Linux66PASS, 공식Python Windows60PASS/6권한skip; exact loader/원raw/SHA/source620/naturalJob0와version3.14.8/64bit/importroundtrip 검토 통과. 원격 Actions·7전체producer·native symlink fixture는후속 미완료다.

## 실제 CI7 로컬 검증 도구 준비

현재 GNUhost cross-MSVC 빌드는 actualRustMSVChost를 요구하는CIbefore게이트 대체물이 아니다. 게이트를완화하거나host를위조하지않고 공식1.94.0MSVChost를 별도projecttoolroot에 준비한다. 원GNU도구/lock/cache보존, freshownedGit checkout/target에서production7 argv 그대로 검증한다. officialMinGit2.56.0.2 archive39,806,486B SHA da35e72aa21c005a5a0d298cfbae110bc1609a815730ea0dde84b01a1b3cd3be 및376entries/expanded96,142,395B를새projecttoolroot에 준비했다. 원zip/모든license보존, create-only/CRC/aliases·traversal/special/sizebound를적용했다. 실제Gitversion/native검증을이어간다. 전역PATH/config/credentialmanager/Gitnetwork/원.git은 변경·실행하지않는다. 원격Actions 인증/업로드는아직없다.

## 격리 Windows Git 실행 확인

windows-mingit-native-version-20261011/root-review.json에서 실제 Git2.56.0.windows.2 및 원 archive/runtime SHA, 자연 종료·Job0·private desktop 종료를 root 검토했다. 공식 MSVC host 공급 후보의 tar 안전성 시험8개를 root 재실행하여 통과했고, 공식 manifest/checksum과 component SHA를 대조하는 프로젝트 전용 다운로드를 시작했다. 실제 CI7 및 remote Actions 성공을 뜻하지 않는다.

공식 MSVC host4버전 실제검증: windows-ci-msvc-host-version-native-20261011/root-review.json PASS_SCOPED. rustc1.94.0 actualhost x86_64-pc-windows-msvc, cargo1.94.0, rustfmt1.8.0-stable, clippy0.1.94를 원 로그에서 확인했다. root가 source620·모든 공급 원파일SHA·자연종료/Job0/private desktop을 독립 대조했다. 기존 GNUhost와 별개이고 실제CI7은 후속이다.

개발 전용 MSVC 도구 비용: 원 압축 합계108,643,560B, expanded615,552,874B/217files/265tarentries, cross-component overwrite/reuse0. 앱 패키지·Cargo 의존성에는 포함하지 않는다. 이번 Linux CI 회귀66개와 updater metadata 정책3개를 실제 재실행해 통과했다. 문서검사894개/원notice114hashmatched도 통과했으며 이후 문서 추가는 최종 재검사한다.

첫 실제CI7 로컬 실행은 새 WindowsGit init/add/commit과 actualMSVChost 확인 뒤 before에서 실패했다. 공식 embeddablePython의 isolated ._pth가 child script module경로를 포함하지 않아 ci_whole_receipt import실패였으며 Cargo7은0개 실행했다. 원Git/원도구/첫실패를보존하고 새프로젝트전용runtime복사에 exactownedscript경로만추가해재실행준비한다. 시스템Python/site/PATH는변경하지않는다. beforeworker/outerhost는 자연exit/Job0이며 성공receipt를생성하지않았다.

## 로컬 bootstrap 경로 재검증

두 번째 로컬 CI 준비는 1,807 source 파일을 새 폴더에 복사한 뒤 parent Python의 ci_whole_receipt import에서 실패했다. 이 실행에서는 Git/Cargo7을 실행하지 않았다. 새 interpreter에서 같은 실제 owned 경로의 모듈 import는 성공했다. 새 source 폴더를 생성하기 전 isolated ._pth 경로가 negative import cache에 들어가는 조건을 확인했고, local bootstrap만 복사 후 importlib.invalidate_caches를 호출하도록 교정한다. 실제 CI checkout/production gate를 완화하지 않는다. 기존 실패 output/runtime을 보존하고 fresh output 및 exact owned scripts 경로의 새 runtime 복사로 재검증한다. root가 통합 pure suite에서 시험 모듈 간 preload로 재현 assertion 실패를 발견하여 시험 격리를 교정 중이다.

Fresh import-cache 교정 local CI7 실행은 실제 Windows MSVC host에서 원7개 argv의 fmt/clippy/test/release/cpu-release/installer-release/companion-release와 before/after를 자연0로 완료했다. 기본workspace 시험603passed/0failed/22ignored/39harness는 별도CPU1063whole와 범위가 다르다. 원outerhost 자연0·Job0 및 immutable source1807/원도구/preservation pin guard를 완료했고 root 독립raw/lifecycle/Git/artifact 검토와 실제Win32 ID 재조회는 진행 중이다. 로컬 claimed GitHub identity만 사용했고 remote_execution_authenticated=false; 실제원격Actions/전체workflow GPU/graph/notice/서명 완료로 표현하지 않는다. 실패bootstrap 출력/기존original.git·원runtime/도구를 보존했다.

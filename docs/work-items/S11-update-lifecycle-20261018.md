# S11: 업데이트 실행 소유권과 전환 감사

- 상태: review
- 분류: feature
- 우선순위: P1
- 요구사항: R08, R10, R11
- 담당/수정 범위: release_audit; 별도 update-lifecycle worktree, 이 문서의 감사 결과만 우선 수정
- 선행 작업: 공개 v0.1.0 및 GitHub Pages, signed metadata/transport/staging/새 bundle 준비 인수

## 사용자 결과

기존 고정 A/B·Job·guardian·installer supervisor를 읽고 사용자 데스크톱과 독립된 실제 시험이 가능한 안전한 실행/복구 계약을 제시한다.

## 계약과 범위

사용자가 2026-10-18 업데이트 구현을 명시 요청했다. 현재 v0.1.0은 첫 updater 지원 버전의 1회 수동 설치가 필요하다. 첫 지원 릴리스는 업데이트 및 관련 UX에 집중하고 별도 대형 기능은 섞지 않는다. 주 담당이 공유 schema·Cargo/lock·인터페이스를 조정한다. 기존 .git/index는 보존하고 정상 worktree를 사용한다. 공개 태그/Release·운영키 외부 공급은 구현·합성 검증과 구분하며 기존 비밀 전송 승인 범위를 유지한다. 검증은 임시 폴더/합성 fixture/private desktop만 사용한다.

## 인수 조건

앱의 idle/작업 중 gate, 서명·크기·해시 오류, 중단·stale 상태, 준비/실행 실패, 기존 A와 사용자 미디어·도구·프로필 보존을 검증한다. 원 owner 종료·새 버전 readiness·복구 결과를 구분한다. UI/OS 동작은 단위 검사로 대체하지 않는다. 운영키 없음을 임의 test key 신뢰로 대체하지 않는다.

## 검증과 handoff

진행 중. D: 여유 약1.6GiB, /tmp 여유610GiB를 확인했다. 기본 PATH의 rustc는 없으며 기존 pinned Rust/Windows 도구 경로를 확인 후 사용한다. Git 정상 worktree/main은 fc6f559이며 root의 과거 index·불명 lock을 삭제하지 않는다. 낮은 D: 공간에 새 Cargo cache를 만들지 않는다.

## 실행 수명 1차 감사 (2026-10-18)

- `accessible_cpu_main::main`은 guardian 등록 → InstanceGuard 취득 → software_host::run → guard drop 순서다. 직접 실행 경로를 stable parent와 owned UI child로 분리하는 새 bootstrap이 필요하다. 기존 A를 rename하거나 A UI를 매 세대 supervisor로 남기지 않는다. 한 parent가 A/B UI를 순차 소유하므로 반복 업데이트에서도 owner chain이 늘지 않는다.
- ADR0009와 `installer_launch::run`/`active_launch::OwnedSelectedProcess::wait_cancelled`는 시작 ACK를 GUI 준비로 간주하지 않고 원 leader 종료와 Job 전체0을 모두 확인한다. installer가 stable parent를 소유하는 바깥 Job을 유지하고 parent가 UI를 nested Job로 소유하는 경로는 이 정책을 보존한다. Windows nested Job 실제 시험 전 검증 완료로 쓰지 않는다.
- `active_launch::launch_inner`는 argv0만, inherited HANDLE 없음 계약이다. 임의 argv/path를 받도록 넓히지 않고 별도 typed bootstrap을 둔다. 기존 process/windows.rs의 STARTUPINFOEX HANDLE_LIST 및 JOB_LIST 기법을 참고하여 fresh Job 배치를 CreateProcess와 원자화한다. 원 suspended child의 image identity 확인 뒤 ResumeThread한다.
- bootstrap 초기권한은 current_exe의 원 file/nativeID와 namespace pin에서만 만들고 임의 기록/path를 실행권한으로 승격하지 않는다. 새 B의 권한 bridge는 PreparedUpdateBundle과 서명 정책을 유지하는 별도 root 통합 계약이다. UI→parent에는 version/path만 보내어 준비cap을 재구성하지 않는다.
- child는 inherited control/event pipe와 readonly 원 parent process HANDLE만 받는다. protocol/nonce/original PID+creation/profile epoch의 bounded frame으로 ReadyForCommit→Commit→RuntimeReady를 구별한다. InstanceGuard 취득과 read-only profile preflight는 ReadyForCommit 전, Candidate::new/Store::open/DownloadRepository::open은 Commit 뒤다. Candidate::new는 activation/settings/recover를 시작하므로 GUI 생성 뒤 gate는 늦다.
- Store::from_connection은 migration table 기록과 ALTER를 수행한다. 첫 지원버전은 profile schema/serialization epoch 동결 + read-only 검증이 필요하며, B Commit 뒤 일반 사용자 설정쓰기까지 모두 무변경이라고 보고할 수는 없다. B가 쓰기 허용되기 전 실패하면 B Job0 확인 후 retained A authority로 복구하며, 이후 실패는 compatible-profile recovery로 별도 보고한다.
- Guardian registration은 각 실제 UI child의 current_exe에 다시 수행한다. stable parent의 A 경로를 B guardian으로 넘기지 않는다. restore IPC는 restore-only를 보존한다.

## 구현 분담 확장 (주 담당 지시)

본 담당은 신규 `crates/engine/src/app_update/bootstrap.rs`, `bootstrap/windows.rs`, 해당 하위 tests만 구현한다. root가 mod 등록/entrypoint/callers/shared epoch·journal·선택 정책을 담당한다. 먼저 API 스케치 합의 뒤 구현하며 Cargo/기존 shared 파일을 수정하지 않는다. 기존 감사 문서 외 문서는 root 소유다. native leaf는 current-image pin, original owned process/Job, exact inherited HANDLE allowlist, bounded protocol/frame와 취소·terminal proof까지만 담당한다.

## 필요한 실제 Windows 합성 인수

임시 self-image fixture를 private desktop에서 실행하여 직접 CPU와 installer 바깥Job 두 경로를 확인한다. A→B→C 세 번 순차전환에서 parent1/UI1 상한, 원 A 파일 hash/identity 보존, B guardian 자기 경로, cancel·owner crash·child crash에서 원Job0을 검증한다. epoch mismatch/nonce·PID·creation mismatch/시간초과/부분 frame/extra frame/pipe EOF를 거부하고 Commit 전 DB fixture bytes 불변을 확인한다. A guard 보유중 B preflight 실패, A 종료후 competing instance 취득시 B failclosed, B preflight 실패 뒤 원 A 재실행을 별도 시험한다. 기존 active selection의 trustedcap 없이 임의 path/변조된 저장기록만으로 B를 실행하지 않는다. 이 감사 단계에서는 native 시험을 실행하지 않았다.

## bootstrap leaf 구현 및 로컬 결과

합의한 신규 파일 `app_update/bootstrap.rs`, `bootstrap/windows.rs`, `bootstrap/tests.rs`만 추가했다. 초기 SelfImage는 current_exe의 original nativeID/전체 SHA와 ancestor pin을 보유한다. PreparedImage는 검증된 PreparedUpdateBundle에서만 만들며 CPU observation/registration ID·bytes·SHA를 대조한다. 임의 경로·저장 JSON·PID를 실행권한 constructor로 받지 않는다.

Windows CreateProcess는 fixed 내부 marker와 정확한 inherited pipe2/readonly parent-process HANDLE1의 allowlist, 원자 JOB_LIST, suspended image 검증/단일 resume를 사용한다. SpawnFailure는 생성된 원 child/Job cap을 보존하고 terminal 불확실성을 숨기지 않는다. runtime에는 강제 종료 deadline을 두지 않으며 poll_exit는 원 process signaled와 Job 전체0을 모두 확인한다.

Hello/ReadyForCommit/Commit/RuntimeReady는 각128B이며 원 parent/child PID·creation, random32B nonce, profile volume+opaque128bitID, epoch1, image role을 정확히 echo한다. SelfOriginal은 기존 crashed/legacy profile 복구를 보존하고 Prepared/Installed role은 caller의 read-only preflight가 필요하다. UiChild.requires_profile_preflight는 Hello 검증 직후 사용할 수 있다. UI의 Candidate::new/DB쓰기 허용과 first successful GUI frame의 RuntimeReady 연결은 주 담당이 구현한다.

Runtime 요청 Prepare(version≤32 ASCII, channel)/Cancel/RestartPrepared(generation)와 Progress/Prepared/Failed/RestartAccepted 이벤트도128B fixed frame이다. 경로·키·임의 JSON은 전송하지 않는다. RestartAccepted를 받은 뒤에만 UI가 기존 CloseWait/drain으로 종료하며 parent는 A Job0 전 B를 띄우지 않는다. try 수신은128B accumulator1개와 최초 부분frame20초 상한, 쓰기는 PIPE_NOWAIT readback 필수로 처리한다. progress coalescing/generation 정책/오류코드 의미는 주 담당이 담당한다.

실제 기존 fresh 설치 cap은 payload RW|DELETE writer와 디렉터리 DELETE cap/clone을 보유한다. strong source/shareREAD·namespace/denyDELETE pin을 곧장 열면 Windows sharing violation이 정상이다. 이를 느슨한 실행권한으로 우회하지 않았다. PreparedImage는 검증 직후 readonly weak original pins만 보유하고, root가 fresh bundle cap을 drop(삭제0)한 뒤 spawn이 original ReOpenFile로 source shareREAD/dirs shareREAD|WRITE로 강화하고 전체 nativeID/path/SHA를 다시 검사한다. 원Fresh cap이 남아 있으면 failclosed한다. 공유회귀2개를 native tests에 추가했으며 실제 native 결과는 root 통합 단계에서 확인해야 한다.

로컬 명령: pinned Rust1.94.0 rustfmt 세 파일 PASS, /tmp의 원파일 구조 복사+wire-shape Channel/NativeFileIdentity stub harness로 rustc --test 6/0, clippy-driver -D warnings PASS. 이것은 Linux wire 검사이며 Windows 실행/도메인 통합 검증이 아니다. 첫 absolute #[path] harness는 child module 상대경로 해석 오류로 컴파일 실패했고 원파일 구조 그대로 복사하여 고쳤다. Root의 최초 Windows 전체 strictClippy는 native type 오류 없이 manual_range_contains lint1건으로 실패했으며 본 소스에서 range.contains로 수정했다. 아직 이 문서 시점에 후속 native 성공을 주장하지 않는다.

Native 추가 회귀는 fullpipe writer timeout/cancel, empty reader timeout/partialframe20초, current-image cancel-before-create, weak writer/DELETE sharing, weak 변조 hash 거부다. 원A→B→A 실제 private GUI/installer nestedJob 시험은 root 통합 책임으로 남아 있다.

고정 parent의 실행코드는 최초A 버전에 남는다. 이번 계약은 같은 protocol/epoch/keyring과 호환되는 UI 버전 전환이며, parent 자체 교체나 비호환 키·protocol 변경을 완료했다고 주장하지 않는다. 이를 위해 owner를 detach하거나 세대별 parent를 누적하지 않는다.

추가 통합 bridge: typed private-field InstalledImageEvidence에서만 InstalledImage::pin_committed를 구성한다. journal의 서명/key/time/sequence 정책은 loader가 검증하고 bootstrap은 UserBundleInspection의 현재 원 read cap, active canonical SHA와 세 경로, 원 executable nativeID/size/SHA/finalpath/epoch를 다시 검사한다. spawn_installed_ui의 전후 bundle 검사가 실패하면 원child/Job cap을 SpawnFailure에 보존한다. Installed role도 candidate read-only profile preflight가 필요하다. source argv는 native UTF16을 그대로 유지하여 임의 UTF8 변환을 요구하지 않는다. 이 bridge는 native통합 검사 전이며 journal 구현 자체는 이 작업에서 수정하지 않았다.

## 실제 Windows 실패 분리 및 수정

주 담당 native v2: strictClippy PASS, app_update 89 PASS/4 FAIL. 이 작업의 실패2개는 weak sharing 테스트의 directory tighten에서 Winerror5였고, fullpipe write/read deadline·cancel·partialframe 시험은 실제 Windows PASS였다. 다른 실패 journal/profile는 각 소유자가 수정한다.

`diagnose` skill로 실제 실패회귀를 loop로 사용했다. 원인 가설은 directory ReOpenFile/root 특성, access/flags 문제, 미해제 DELETE handle 순서였다. owned private Job의 ctypes 최소 프로브에서 regularfile ReOpenFile는 성공했지만 모든 directory는 access/flags를 바꾸어도 error5였다. NtCreateFile의 empty-relative 시도는 해당 옵션 조합에서 INVALID_PARAMETER였다. 대신 공식 OpenFileById의 원directory HANDLE volumehint + 바로 그 HANDLE의 전체 opaque128 ID를 사용한 강화는 프로젝트 임시폴더와 D: drive-root까지7개 모두 성공하고 volume/128ID가 일치했다. `id-probe-v1/reopen.log`에 Job natural0/inputdesktop 불변을 보존했다. 최초sandbox실행은 Windows child 생성 전에 WSL interop socket권한 실패했고, 허용된 private native 시험으로 상승 재실행하여 통과했다.

수정은 source regularfile의 ReOpenFile를 유지하고 directory만 original volumehint+original128ID OpenFileById로 강화한다. 원weak HANDLE을 계속 보유하므로 ID 재사용 공백이 없고 결과 identity/path/attributes도 다시 검사한다. 임의 외부ID/path를 입력하지 않는다. 원leader가 이미 signaled인데 자손이 살아 있는 경우에는 원Job만 종료하고5초 내 Job0을 확인한다. live UI에는 runtime timeout을 적용하지 않는다. UiExit.descendants_terminated와 원leader exitcode를 함께 보존한다.

standalone native fixture는 .local-tools/update-lifecycle-fixture-20261018에 있으며 GUI/desktop 창을 생성하지 않는다. main에서 child_entry를 먼저 분기하고 actual handshake/원Job0, 반복3세대/동일parent, 초기실패/timeout/wrongprofile, owner process::exit에 의한 kernel Jobclose, child37 종료 뒤 같은 Job의 rawtail 정리를 시험한다. 명시된 project 소유 temporary base만 쓰며 실제 영상/계정/network는 사용하지 않는다. root native v3 rlib 생성 뒤 hash-pin하여 직접 rustc link하며 새Cargo cache/build를 병렬로 만들지 않는다. 이 기록 시점에서는 fixture 실행대기다.

## named namespace 최종 수정과 실제 수명 인수

위 OpenFileById 대안은 **채택 후 폐기**했다. root native v4의97 PASS/1 FAIL에서 sourcewriter는정상거부했지만directory DELETE handle이살아있어도OpenFileById 강화가성공했다. 추가실제프로브에서 by-ID handle만 있으면디렉터리rename도허용되는반면 named CreateFile denyDELETE는오류32로liveDELETE cap과충돌하고rename을차단함을확인했다. 따라서최종구현은원regularfile ReOpenFile + root→leaf named directory pin이다. 모든원weak HANDLE을계속보유하고, 각named새cap이원전체ID/최종경로/디렉터리·비reparse속성과같음을확인한뒤만채택한다. 부모namedpin을유지한상태로다음자식으로진행하여중간경로교체도차단한다. 임의path/기록을권한으로승격하지않는다. 열린하위파일없이directorynamedpin만으로rename차단/해제후성공을확인하는native회귀를추가했다. root가최신Windows C: target에서검증중이다.

`.local-tools/update-lifecycle-fixture-20261018/native-fixture-v3/fixture.log`에서실제Windows headless7case가모두통과했다: startup2단계/RuntimeReady/진행률/준비/RestartAccepted, 같은parent PID의3회순차UI실행, 초기실패1, startup timeout126, wrongprofile1, owner process::exit후nestedJob정리0, 원leader crash37뒤ownedrawtail정리와원37보존. 모든case에서원Job0을확인했고외부host도natural-zero/forced=False/private desktop닫힘/input desktop불변이었다. exe SHA는 `8afee482f1b58e4d79adfd62b0153d01931bbd30c2723abca35882633527fd41`, 원engine-v4 rlib SHA는 `3102400d0a3adf39e248c50bc56edb0e5c2feaa84d8599879414d52009a1865d`다. 이시험은SelfImage수명이며PreparedImage의최종named전환이나실제GUI검증과동일시하지않는다.

첫standalone링크는기본link.exe부재, 두번째는MSVC importlib경로누락으로실패했고각로그를보존했다. direct rustc에고정된lld-link, prepared-config의CARGO_ENCODED_RUSTFLAGS(staticCRT/um/ucrt/vclib -L), 기존windows_x86_64_msvc importlib -L을정확한argv로전달한뒤통과했다. rlib/source/exe는hash로전후검사했고실행중원rlib에readonly denywrite HANDLE을보유했다. 추가Cargo cache/target이나제품웹UI를만들지않았다. fixture임시데이터는명시project localtools/owned-temp만사용했다.

## 최종 독립 검증 결과와 인계

root native-v6의 원로그를 직접 읽어 workspace strictClippy, workspace tests, dev CPU/installer, engine lib 빌드가 모두 EXIT0임을 확인했다. 본 bootstrap 회귀12개는 모두 실제Windows PASS다. 특히 named directory만으로 rename 차단, fresh writer/DELETE cap 생존 중 강화 거부, weak 원본변조 거부, fullpipe/read/partialframe/cancel 회귀를 확인했다.

최신 native-v6 C: rlib SHA `667b68456aef95a503ed8ccd461b6eb65fb6203e9cc99aff13f5a36a9506d66f`로 같은 standalone7case를 다시 빌드·재실행하여 모두 통과했다. 최신 실행 로그는 `.local-tools/update-lifecycle-fixture-20261018/native-fixture-v4/fixture.log`, 새 exe는 `fixture-v2.exe` SHA `05b6539ee514156348e5bca9e4ae0028c6755a79cb77c3da757c2d72727a34dc`다. 같은parent PID의3회순차실행, 초기거부/timeout/wrongprofile, ownerexit, childcrash37 뒤 owned tail 정리를 모두 관측했다. 외부 Job 자연0/forced=False, 원입력 desktop 불변, source/rlib/exe hash 전후 일치다.

기존 dependency만 사용하며 Cargo/lock/feature를 추가하지 않았다. 고정frame128B, 수신partial accumulator128B1개, 파일hash64KiB buffer와 최대256 ancestor cap 상한이 있다. 추가 stableparent의 실제 제품 idle메모리/첫 GUI frame 지연은 이 headless 시험으로 측정했다고 보고하지 않는다.

소스3파일과 이 문서만 본 작업의 제품변경이다. .local-tools의 headless/ctypes probe와 로그는 명시개발검증 산출물로 남겼다. 새 cache/공개서버/운영key/태그/release/push/Git 이력변경은 없었다. 실제 signed B download→준비→실행→journal commit→coldstart/rollback 통합은 update_transport/주담당, 실제 GUI/제품설치/운영key정책·배포는 주담당의 후속 검증이다. 범위 밖 검증을 이 leaf 완료에 합치지 않는다.

## 별도 CI fixture capture 구현 범위

Root 추가 위임: 본 담당은 신규 scripts/ci_app_update_fixture.py와 해당 Python 실패회귀, 이 workitem만 수정한다. Cargo/defaultOFF required-feature bin은 Root/update_transport, workflow는 Root 담당이다. 기존 ci_capture_operation.py/ci_whole_receipt.py/ci_whole_orchestrate.py의 fixed7 계약은 수정하지 않는다.

목표는 source checkout에서 app-update-fixture feature bin의 고정 clippy/build 및 success/start-fail/runtime-crash/bad-hash/cancel/busy/stale 7case를 각 gated original Job 아래 실행하고 로그·원 Job0·정확한 CASE_PASS receipt를 별도 ci-update-captures에 보존하는 것이다. 출력/timeout/reader failure, 잘못된 case JSON/중복/변조, 다른 binary binding, 남은 Job 자손, 강제 cleanup이 성공으로 승격되지 않는 실패회귀가 인수 조건이다. native_gui_acceptance/release_ready/원격CI인증을 이 headless 도구로 true로 만들지 않는다. 사용자 홈을 고정 경로로 쓰거나 시험키/bin을 제품 ZIP에 넣지 않는다.

워크플로 배치는 기존 companion-release 뒤와 whole after 앞이며 별도 capture 폴더를 사용한다. fixture CLI의 --owned-parent는 기존 절대 임시부모를 명시하고 case마다 자기freshchild만 생성/정리한다. 새wrapper는 capture의 job_for/pump/job_active/wait_natural_empty/report와 receipt의 정규화·hash·create-only publish를 실제 재사용한다. source/runtime key 정책을 우회하지 않는다.

## 독립 CI capture 인계 결과

수정 파일은 scripts/ci_app_update_fixture.py, scripts/test_ci_app_update_fixture.py와 이 workitem이다. 기존 fixed7/capture/whole helper, Cargo, workflow는 수정하지 않았다. 새 Python 회귀11/0, 기존 capture9통과+native전용1skip, whole_receipt18/0, whole_orchestrate18/0 및 py_compile을 실행했다. 이 결과는 Linux helper 회귀이며 새 wrapper의 실제 Windows7case 실행을 했다고 주장하지 않는다. Root/transport의 portable bin 최종 준비 후 원 owned wrapper로 한 번 통합 실행해야 한다.

실제 고정 child argv는 다음과 같다.

```text
cargo clippy -p dlpad-engine --bin app-update-fixture --features app-update-fixture --locked --offline -- -D warnings
cargo build -p dlpad-engine --bin app-update-fixture --features app-update-fixture --locked --offline
<선택된 Cargo target>/debug/app-update-fixture.exe --case CASE --owned-parent EXISTING_ABSOLUTE_DIR
```

case는 success/start-fail/runtime-crash/bad-hash/cancel/busy/stale이다. build는 두 실제 lifecycle/log receipt와 exe hash를 build.json에 묶고, run은 그 원본과 raw log/binary hash를 다시 검증한 뒤7case 결과와 각 receipt hash를 별도 whole.json으로 발행한다. 원Job0/reader drain/exit0/forced_cleanup=False 및 CASE_PASS의정확한8필드·case별전이가 모두 필요하다. 기존receipt는덮어쓰지않고실패로그와부분산출물을보존한다. GUI검수/원격CI인증/release준비flag는false로남긴다.

Windows workflow 권고 호출(PowerShell, CI 환경 경로만 사용):

```text
python scripts/ci_app_update_fixture.py --root "$env:GITHUB_WORKSPACE" --directory ci-update-captures --phase build --seconds 180
python scripts/ci_app_update_fixture.py --root "$env:GITHUB_WORKSPACE" --directory ci-update-captures --phase run --owned-parent "$env:RUNNER_TEMP" --seconds 450
```

script가 build에서 fresh ci-update-captures를 만들므로 workflow가이를미리mkdir하지않는다. 기존companion-release 뒤와whole after앞에두고, always-upload하는별도진단artifact에는ci-update-captures만넣는다. 제품4exe artifact/설치ZIP에fixtureexe나시험키를추가하지않는다. --seconds는각phase전체가공유하는monotonic상한이며case별180초와합산되지않는다. 위180/450은Root가hosted예산에맞춰조정할workflow권고다.

추가 embedded Python 호환: Windows isolated _pth 환경은 scriptdir를 sys.path에 넣지 않아 entry/worker가 sibling capture helper import에 실패할 수 있다. 새 wrapper는 표준 import 뒤 own __file__의 resolve().parent만 sys.path 맨앞에 넣는다. 별도 cwd에서 Python -I --help 및 --worker가 실제 인자/platform guard까지 도달하는 회귀를 추가하여총11개가통과했다. 사용자홈/환경별sys.path를하드코딩하지않는다. 최종wrapper SHA는6ebefd077d7ce4de4b017e756999ba22ff31d56485b158a8fdb43227731862e8, test SHA는091527af0bc43c5124ab1c7c40f130515bd9d3752c5a62c5d8b09f4257e1a101이다.

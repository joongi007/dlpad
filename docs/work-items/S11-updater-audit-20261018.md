# S11: Windows 앱 업데이트 수직 경계 읽기 전용 감사

- 상태: implemented-scoped / Root Windows 통합·시각 검증 대기
- 분류: investigation
- 우선순위: P1
- 요구사항: R08, R11
- 담당/수정 범위: updater_audit; 본 문서, 신규 engine app_update verifier/staging/discovery, 신규 앱 업데이트 패널 및 catalog. 후속 Root 배정은 아래 범위 기록을 따른다. 공유 schema/Cargo/lockfile 수정 없음
- 선행 작업: 소개 페이지 → 설치기를 포함한 자동 릴리스 → 서명된 앱 업데이트; Root의 공개키·채널·metadata 계약

## 사용자 결과

기존 설치·사용자 미디어·외부 도구를 보존하는 업데이트 경계 감사와 서명 검증·원 capability 기반 패키지 export·명시 공개 릴리스 조회 UI를 제공한다. 자동 설치/기존 앱 교체/실행은 구현하지 않으며 Windows 전체 통합·시각 인수는 Root 후속이다.

## 계약과 범위

README → PRODUCT → CONTEXT → R08/R11과 관련 요구사항 → ADR0005/0009, ARCHITECTURE/INTEGRATIONS를 읽고 app_install, installer admission, worker/controller 경계를 감사한다. 최초 감사는 이 문서 한 개만 수정했다. 이후 Root 배정에 따른 신규 엔진/UI 구현 범위는 후속 단락에 별도 기록했다. 공개 배포, 운영 키 생성·비밀 전송, Cargo 수정과 실행 설치 시험은 제외한다.

## 인수 조건

- 코드의 기존 권한과 미구현 경계를 구분한다.
- 서명 신뢰·원 설치 identity·사용자 데이터 보존·취소/크래시 복구를 우회하지 않는 최소 수직 기능을 제안한다.
- 합성 폴더와 소유 프로세스만으로 실제 검증 가능한 acceptance 및 수정 파일 목록을 제시한다.
- ring 재사용의 직접 의존성 비용을 Cargo 수정 없이 확인한다.

## 검증과 handoff

초기 환경: 분리 worktree의 .git은 Windows absolute 경로를 보유하므로 Linux git status는128. .git 수정 없이 파일 읽기로 계속하며 Root에 보고했다. 완료 결과는 후속에 기록한다.


## 현재 코드의 확정 경계

1. `crates/engine/src/app_install.rs`의 public export에는 durable revision, cold writer, rollback이 없다. [fresh 실험](S07-fresh-revision-journal-20261016.md)과 [cold 실험](S07-cold-revision-journal-20261016.md)은 scratch/test-only 검증이고 제품에 통합하지 않았다. 이전17회귀 성공은 제품 자동 업데이트의 완료 근거가 아니다.
2. `app_install/local_prepare.rs::prepare_local_journal_bundle`은 `development_only=false` manifest를 명시 거부한다. `transaction.rs`는 `trusted_provenance=false`만 생성하고 consistency 검증도 이를 전제한다. 이 boolean을 바꾸거나 unsigned URL/SHA를 넘겨 정식 신뢰 권한을 만들면 안 된다.
3. `installer_main.rs::admission`은 Vacant/Existing 표시와 bounded32개 current journal 선택을 제공한다. Existing은 설치 overwrite 권한이 아니며, journal suffix는 제거 뒤 새 설치의 빈 namespace 선택이지 upgrade generation이 아니다.
4. `installer_launch.rs::Supervisor`는 원 bundle/journal/active를 worker에서 fresh 검증한 뒤 suspended child→원 Job→native image 검증→resume를 관리한다. Started는 GUI 준비가 아니다. HWND 종료 뒤 supervisor가 앱 종료까지 생존하며 원 owner가 사라지면 Job이 종료된다. 업데이트 helper를 현재 앱의 자식 Job에 단순 실행하고 앱을 닫으면 helper도 죽을 수 있으므로 독립 수명 handoff를 별도로 인수해야 한다.
5. `install_journal.rs`의 Prepared/Registered/Receipt/Active prefix는 fresh bundle 설치의 consistency 기록이다. 기존 active pointer 원자교체, 새 앱 health acknowledgement, DB schema downgrade, 자동 rollback은 제공하지 않는다.
6. `accessible.rs`가 현재 CPU/accessible 앱의 worker 수명과 CloseWait 통합 지점이다. 설정은 `accessible_settings.rs`; 도구 업데이트는 `accessible_tools.rs`의 bounded channel·request/tool generation·captured pin과 `tool_update_ui.rs` 표시를 사용한다. 도구 inventory나 앱 설정의 cached version을 앱 교체 권한으로 재사용하지 않는다.
7. `scripts/validate_update_candidate.py`는64KiB manifest·256MiB package·strict newer version·channel/target/host 문법을 검사하는 unsigned 정책이다. `release_ready`는 False만 허용하며 앱 설치 실행을 승인하지 않는다.

## 최소 구현 순서 제안 — Root 공유 계약 결정 전

소개 페이지와 설치기/포터블 Release 패키징을 먼저 끝낸다. 최초 앱 updater의 최소 사용자 결과는 설정에서 새 버전 확인 → 신뢰할 수 있는 후보의 명시 다운로드 → 원 설치를 보존한 새 sibling 버전 설치로 잡는다. 기존 설치 폴더 overwrite와 자동 rollback을 약속하지 않는다. 사용자 미디어·외부 도구·기존 프로필을 유지하고, 새 앱 실행은 기존 감독 실행 API의 fresh validation을 거친다.

단계 A는 서명된 metadata 검증과 bounded package staging이다. signed payload에는 domain/schema·key id·sequence·version·channel·target·정확한 asset digest/size/filename·호환성 정책·만료 정보를 결합한다. 서명은 원 payload bytes에 검증하고 JSON duplicate/unknown/future schema·wrong domain·stale sequence·wrong channel/target·키 미등록·expired 후보를 거절한다. GitHub Pages JSON이나 HTTPS+SHA가 서명 trust root가 아니다. key id는 컴파일된 제한된 keyring을 선택할 뿐 원격 키 다운로드 권한이 아니다. 공개키·endpoint·서명 domain과 rotation은 Root가 계약을 정한다. 메타데이터 서명과 Windows Authenticode는 별개다.

단계 B는 새 typed verified-candidate capability에서만 installer/새 sibling prepare로 넘어가는 adapter다. source/zip·staging·원 설치 및 프로필 경계와 native identity를 다시 검증한다. 정식 prepare는 기존 development-only API를 임의 확장하지 않고 별도 verified input entrypoint를 설계한다. 이 API는 signed digest와 실제 원 파일 pins를 결합해야 하며 저장 JSON에서 capability를 재생성하지 않는다. unsigned 개발 설치는 기존 동작을 유지한다.

단계 C는 앱 종료/작업 중단 ACK 및 helper 수명 handoff를 완료한 뒤 새 설치로 명시 전환한다. 설치기와 앱 업데이트 다운로드는 같은 release artifact 계약을 사용하되 updater의 실행 검증은 별도다. 기존 active v1를 교체하는 durable revision/단일 writer/실제 crash 복구까지 구현하기 전에는 “동일 폴더 자동 교체” 버튼을 표시하지 않는다. 초기에 다운로드 후 설치기 안내만 제공하면 “자동 업데이트 완료”로 보고하지 않는다.

## Root가 조정할 제안 파일

- 새 `crates/engine/src/app_update.rs`, `app_update/metadata.rs`, `app_update/download.rs`: 검증 타입·bounded verifier·staging·취소/timeout. 네트워크 fetch와 parser를 분리해 합성 입력으로 시험한다.
- `crates/engine/Cargo.toml`, `Cargo.lock`: Root만 ring 직접 의존성 조정.
- 새 `crates/app/src/app_update_ui.rs`, `app_update_worker.rs`: bounded1 worker/result·cached display·명시 action fresh 재검증·generation·CloseWait ACK.
- `crates/app/src/accessible.rs`, `accessible_cpu_main.rs`, `accessible_main.rs`, `catalog.rs`: 설정 navigation·모듈·수명·중앙 문자열. `accessible_settings.rs`의 appearance writer에 네트워크 다운로드를 얹지 않는다.
- 설치 단계에서만 `app_install.rs`, `app_install/local_prepare.rs` 및 새 verified installation adapter와 `installer_main.rs`/`installer_catalog.rs`를 Root가 별도 소유한다. 기존 개발 manifest의 provenance bool을 trusted로 덮어쓰지 않는다.
- Release workflow·metadata signing script·공개 keyring·CONTEXT·ADR0005 및 새 updater ADR은 Root 공유 계약이다.

## 실제로 재현 가능한 합성 인수 목록

- 고정 공개 test vector의 정상 Ed25519와 signature 한 비트·payload 한 비트·다른 키·짧은/긴 서명·unknown key 거절. 테스트 전용 키와 운영 공개키는 별도이며 운영 private key를 로컬 생성/커밋하지 않는다.
- duplicate/unknown JSON field, size64KiB 경계, wrong domain/schema/target/channel, version downgrade/equal, expired/replayed sequence, overflow 및 clock 문제에서 install mutation/child spawn0.
- 로컬 loopback fixture 서버의 chunked/overlong/truncated body·잘못된 content length·redirect host 변경·timeout·중간 취소·잘못된 digest에서 원 설치/프로필/media/외부 tool sentinel의 bytes/native identity 불변. fixture 서버는 제품 웹 UI가 아니다.
- 새 sibling prepare 중 단계별 취소/공개 직전 경합·unknown file·reparse·동일 bytes/다른 native ID·경로 overlap에서 원자료 보존과 실패 ACK; stale UI cache가 설치를 시작하지 못함.
- 합성 TEMP 설치 A→B의 새 프로필 호환성 확인, 원 A 재실행 가능성, 실행 중 다운로드/스캔/export 때문에 update 준비가 막히는 이유와 안전 종료 안내. 실제 사용자 Documents/실영상/계정 사용0.
- 별도 소유 프로세스를 단계별 종료하여 cold recovery를 실행하고 last confirmed head만 유지. unknown commit/부분 journal은 자료 보존·명시 재검사 요구이며 성공으로 변환하지 않음. power loss는 process kill로 검증했다고 주장하지 않음.
- private Windows desktop에서 설정 업데이트 check/download/cancel/확인·키보드 focus·100/200%·Light/Dark·상태 라벨·짧은 실패·재시도·close ACK. helper handoff 후 앱 parent 종료에도 정확한 소유 새 helper가 생존하고, 취소/실패 때 무소유 process0 확인.

## 의존성과 비용 감사

현재 Cargo.lock에는 `ring 0.17.14`가 존재하며 rustls0.23.45→ring, rustls-webpki→ring 경로로 이미 TLS에 포함된다. `ring::signature::UnparsedPublicKey::new(&ED25519, public_key).verify(payload, signature)`는 검증 API이며 new만으로 키를 검증하지 않고 verify가 검증한다. [고정 버전 API](https://docs.rs/ring/0.17.14/ring/signature/struct.UnparsedPublicKey.html).

직접 `ring = "=0.17.14"`를 같은 feature 범위로 사용하면 추가 crate version 없이 기존 resolver node를 공유할 수 있다. 실제 Cargo feature unification과 lock diff는 Root 확인 대상이며 이 감사는 Cargo를 수정하지 않았다. 검증 API에 고정32byte 공개키와64byte 서명을 쓰면 별도 base64/Ed25519 crate를 추가하지 않아도 된다. signature verifier 코드의 최종 PE 크기 증분과 검증 CPU 비용은 release build 전후 실측해야 하므로0byte/무료라고 보고하지 않는다. metadata는64KiB 상한으로 메모리 검증, package는 streaming SHA와256MiB disk 상한을 유지하는 제안이다.

## 실행 결과와 미검증

파일·API 정적 감사만 수행했다. 제품 Rust/Cargo/lock 변경0, 설치·native GUI·실제 updater 실행0, 키 생성·release·tag·push0. native/서명 lifecycle 인수는 위 계획이며 성공 결과가 아니다. Root의 keyring·endpoint·signed schema 결정 후 구현 작업을 별도 수직 항목으로 열어야 한다.


최종 문서 검사: `python3 scripts/check_docs.py --source-only` exit0;1,119문서 local-link/필수파일 PASS, local-only 증거 링크1,505개 생략(runtime 증거 미검증). Windows Git readonly status는 현재 WSL interop `UtilBindVsockAnyPort:317 socket failed1`로 실행되지 않았으며 .git/worktree metadata를 변경하지 않았다. Root는 문서 파일만 통합하고 본문 코드 위치와 계약을 독립 검토한다.


## 후속 구현 범위 — 2026-10-18

Root 배정에 따라 이 분리 branch에서 신규 `crates/engine/src/app_update.rs`와 하위 파일만 구현한다. Root가 lib.rs export·직접 ring 의존성·Cargo.lock·공유 schema/CONTEXT/ADR·UI를 조정한다. 최초 수직 경계는 pinned caller keyring으로 원 bytes Ed25519 서명 검증 → strict metadata/채널/순서/target/repository/URL 정책 → streaming package byte 검증이다. 설치·활성화·실행·기존 파일 쓰기·운영 키 생성0. 안전한 원 HANDLE 기반 staging publication은 기존 capsule 권한 검토 후 별도 단계이며 서명 검증만으로 설치 완료를 주장하지 않는다. signature 위조/unknown key/중복 필드/경계/만료·다운그레이드/body 변조/취소/timeout/부분 입력 실패를 fixture로 검사한다.


### Signed verifier 실제 구현/검증

신규3파일(`crates/engine/src/app_update.rs`, `app_update/metadata.rs`, `app_update/metadata/tests.rs`)을 추가했다. Root가 채택한 asset명은 `dlpad-{version}-windows-x86_64-installer.zip`이다. 공개 API는 caller-pinned 최대8키 keyring과 out-of-band key id hint/고정128자리 lowercase hex Ed25519 서명에서 원 payload bytes를 먼저 검증한다. 그 뒤 strict serde(unknown/duplicate 거절), domain/schema/key id·증가 sequence·strict numeric SemVer·stable/preview·Windows MSVC target·canonical repository/exact release URL·size·유효기간을 검사한다. metadata64KiB·package256MiB 경계와 최대31일 validity 제안을 사용하며 Root 공유계약에서 수정할 수 있다. HTTPS만으로 trust를 만들지 않는다.

`VerifiedCandidate::verify_package`는64KiB buffer로 signed byte count+SHA256를 검사하고 취소/기한을 read 전후에 확인한다. declared size+1byte까지만 소비한다. 반환 `VerifiedPackage`는 내용 관측이며 native identity pin·설치/실행/삭제권한이 아니다. 네트워크 HTTP·파일 staging/publication·원 설치 교체·UI는 구현하지 않았고 Root가 별도 연결한다. blocking Read를 강제 중단하지 않으므로 HTTP transport가 각 blocking call의 deadline을 설정해야 한다. action 실행 시 fresh 정책 재검증도 필요하다.

Linux Rust1.94.0 직접 toolchain으로 `/tmp/dlpad-updater-verifier-20261018` 독립 Cargo harness를 만들고 원3파일과 byte-identical 복사본을 확인했다. 기존 offline crate cache를 사용했으며 프로젝트 Cargo/lock/lib export는 변경하지 않았다. 임시 harness 첫 #[path] module 해석은 E0583 실패했고 원 코드의 정상 module tree를 복사한 harness로 교정했다. 실제 `cargo test --offline --manifest-path ...` 최종8 PASS/0 FAIL/0 ignored, `cargo clippy --offline --manifest-path ... --all-targets -- -D warnings` exit0, rustfmt1.94 exit0. 이 harness는 프로젝트 lock/전체 Windows 통합 검증을 대체하지 않는다. Root는 shared export와 pinned ring direct dependency 통합 후 실제 workspace/Windows 검증을 수행한다. 별도 운영키/서명/실제 다운로드/설치 시험0이다.


### 후속 owned staging 구현 범위

Root 배정에 따라 `app_update/staging.rs` 및 그 회귀 파일, `app_update.rs` export만 추가 수정한다. metadata 파일은 Root 통합본과 독립성을 유지하기 위해 편집하지 않는다. 기존 `CreatedRelativeFile` create-only capability를 재사용해 명시 선택한 기존 부모의 native pin과 canonical 경로 아래 새 stage를 만들고 signed count+1 상한 복사→flush/sync→같은 원 reader hash 재검증→current identity 확인→non-replacing publication을 제공한다. 실패 cleanup은 원 capability의 cancel_cleanup만 허용한다. 공개 뒤 불명확 상태는 path/ID 사실만 제공하고 cleanup authority를 복원하지 않는다. 자동 HTTP·설치·기존출력 overwrite·실행0. 비Windows는 Unsupported를 반환한다. Windows 합성 성공/충돌/변조/취소/부모 변경 시험을 준비하고 Root가 실제 Windows 통합 검증한다.


### Owned staging 구현 결과와 실제 검증 범위

`stage_verified_package`는 명시 caller-selected directory File와 **canonical native parent path**, nonce1~64 ASCII alnum/hyphen을 받는다. 원 `CreatedRelativeFile`로 `.dlpad-update-{nonce}.partial`을 create-only 생성하고 signed size+1 상한으로 source를 복사한다. read/write 전후 취소·기한을 확인하고 flush/sync→원 File seek0→signed hash 검증→원identity/current 확인→signed asset_name에 publish_no_replace→published readonly 원reader 재검증을 수행한다. 최대256MiB package의 디스크 재읽기2회와 고정64KiB 버퍼 비용이다. 성공 wrapper는 원 readonly reader/부모 pin/원identity와 버전·sequence·digest 관측을 유지하며 drop은 완성파일을 삭제하지 않는다.

실패는 creation NotAttempted/AttemptedUnconfirmed/OwnedCreated, publication NotAttempted/Unconfirmed/PublishedUnverified, cleanup NotNeeded/OriginalRemoved/OriginalRetained/Unconfirmed를 구분한다. factory post-create 오류 가능성을 숨기지 않는다. 원 created cap만 cancel_cleanup에 전달하고 publish 호출 이후에는 경로 삭제0이다. 공개 뒤 취소는 PublishedUnverified/OriginalRetained이며 이미 공개한 파일을 “취소로 없앴다”고 보고하지 않는다. path·ID 사실로 재개/삭제권한을 복원하지 않는다. metadata 파일2개 SHA는 이전 verifier 통합 시점 그대로이고 공유 capability·Cargo·lock·lib export는 수정0이다.

신규 Windows6회귀는 성공/원readonly pin+Drop보존, final/stage 충돌 bytes+ID보존, hash/short/long/readerror 실패, 사전/읽기중 취소·deadline, invalid nonce/불일치 parent, 실제 선택 directory 교체 거부를 준비했다. 실제Windows 실행은 Root 통합 단계에 남는다. Linux Unsupported는 source cursor를 소비하거나 출력파일을 생성하지 않는 실제 회귀를 추가했다.

독립 `/tmp/dlpad-updater-staging-20261018` harness는 원수정3파일 byte-identical임을 확인했다. Windows cache에서 Linux errno0.3.14가 없어 offline첫시도 실패했고 Linux용 `CARGO_HOME=/mnt/d/project/dlpad/.local-tools/cargo`로 바꾸어 실제11 PASS/0 FAIL/0 ignored(verifier8+새Unsupported1+기존caps/file identity2)를 확인했다. Windows 성공을 Linux 결과로 대체하지 않는다. Rustfmt1.94는 본인파일만 skip_children으로 처리했다.


### 설정 업데이트 패널 구현 범위 — 2026-10-18

Root 배정에 따라 새 `crates/app/src/accessible_app_update.rs`, 독점 `catalog.rs`, 새 engine `app_update/discovery.rs`와 본 문서를 수정한다. shared accessible.rs/entry/module export/Cargo/lock/metadata/staging/transport는 수정하지 않는다. 공식 public GitHub release 조회는 명시 클릭에서만 20초 worker로 수행한다. bounded1 결과 channel·중복 요청 금지·cancel/stop ACK·panic ACK·ctx wake를 제공하고 idle polling thread를 만들지 않는다. 운영 trust key0 및 자동 교체 미구현 상태는 UI에 정확히 표시하고 canonical release hyperlink의 사용자 클릭 외 외부 실행0이다. 색상·폰트·32px controls는 기존 공통 컴포넌트/System 테마를 유지한다. UI 문자열은 catalog에 중앙 등록한다. impeccable product 지침과 실제 DESIGN.md를 읽었으며 native private 시각 인수는 Root 후속으로 남긴다.


### 공식 릴리스 관측과 앱 설정 UI 구현 결과

`app_update/discovery.rs`는 고정 public GitHub API만 credential/cookie/proxy 없이 조회한다. stable은 latest, preview는 first20 공개 릴리스 중 numeric SemVer/preview flag 후보를 선택한다. official metadata 자체는 서명/설치 권한이 아니다. exact canonical GitHub repo/tag URL만 opaque observation에 남기며 UI 외부 링크도 이 값과 고정 releases URL만 사용한다. response256KiB·20행·header64KiB·redirect0·전체20초/각 resolve/connect/response/body5초 상한, 취소 checkpoint·unknown/malformed/잘못된 repoURL·중복필드/oversize·정제된 enum 오류를 적용했다. preview20행에서 안 보이는 과거 전체를 “없음”으로 단정하지 않고 “최근 공개 릴리스”로 설명한다. [GitHub 공식 Release API](https://docs.github.com/en/rest/releases/releases).

`accessible_app_update.rs::AppUpdatePanel`은 new/ui/poll/request_stop/stopped/busy를 제공한다. new는 network0/worker0이다. 명시 조회에서만 thread1개+resultchannel1개를 만들고 완료/panic/cancel ACK와 실제 JoinHandle finished 뒤만 join한다. duplicate/stale generation 결과를 막으며 CloseWait의 request_stop은 영구 새 dispatch를 차단한다. Root는 다른 화면/CloseWait에서도 global poll(ctx)를 호출해야 한다. 100ms repaint timer는 active 조회 때만 존재한다. cached observation은 실패/재조회 중 보존하되 “이전 확인 결과”로 표시하고 설치 권한으로 사용하지 않는다.

현재 Cargo 버전·Windows x64 지원·stable/preview·조회/중단·새 버전/공개 없음/오프라인/조회 제한/timeout/실패와 수동 release 다음 행동을 기존 controls/theme로 그린다. 키가 없는 상태에서 자동 설치/교체 버튼을 제공하지 않는다. “앱 내 자동 설치는 아직 제공하지 않습니다”를 본문에 명시한다. 새로운 카드/색상/애니메이션·idle thread·silent startup network는 없다. 중앙 catalog 문자열만 추가하고 기존 문구는 변경하지 않았다.

실제 독립 `/tmp/dlpad-app-update-panel-20261018` harness는 현재 vendor egui/기존 production ui_components·theme/tokens/catalog와 신규 파일을 사용했다. ui_components의 기존 unrelated cfg(test) 회귀는 임시 harness에서만 비활성화했고 source의 production 함수는 그대로 사용했다. 첫 임시 harness가 파일 첫 cfg(test) 뒤를 잘라 help 함수가 빠져 E0425였으나 production 전체를 보존하는 test-attribute 처리로 수정했다. 제품 원 컴포넌트는 수정0이다.

Linux1.94 실제최종22회귀 PASS/0 FAIL(새 UI6 + discovery4 + 기존verifier/stub/theme12), 명시원namespace 취소/중복/panic/stale generation·paint의 current/manual 안내·100/200% egui Context·URL/preview/순서/파싱경계·사전취소/기한/invalidversion network차단을 검사했다. strict Clippy의 최초20회귀는0; 추가 paint 회귀의 needless_borrow 한개는 실제실패 후 해당 테스트만 교정했다. 최종 strict검증은 후속 보고한다. 실제 external API/network·native raster·OS keyboard/reader·Windows whole 통합·운영 trust/자동교체는 이 결과로 인수하지 않는다.


최종 추가 paint회귀 needless_borrow 수정 뒤 `cargo clippy --offline --manifest-path /tmp/dlpad-app-update-panel-20261018/Cargo.toml --all-targets -- -D warnings` exit0을 실제 확인했다. 신규 UI/discovery/catalog3파일과 임시 harness 입력의 byte identity를 확인했다. Root shared wiring과 native 시각 인수는 별도다.

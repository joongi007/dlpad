# S11: 앱 업데이트 화면과 지원 버전 진입 UX

- 상태: implemented; read-only UI Windows 통합 검증 대기
- 분류: feature
- 우선순위: P1
- 요구사항: R08, R10, R11
- 담당/수정 범위: updater_audit; 별도 update-presentation worktree, discovery.rs, accessible_app_update.rs, 신규 app_update_client.rs, Root가 이관한 accessible.rs 및 동파일 tests, 본 문서
- 선행 작업: 공개 v0.1.0 및 GitHub Pages, signed metadata/transport/staging/새 bundle 준비 인수

## 사용자 결과

현재 앱 업데이트 화면과 busy/CloseWait/설정 저장·중앙 catalog를 감사하고 버전·변경내역·진행률·설치/재시작·오류/지원조건을 표시할 수 있는 최소 계약을 제시한다.

## 계약과 범위

사용자가 2026-10-18 업데이트 구현을 명시 요청했다. 현재 v0.1.0은 첫 updater 지원 버전의 1회 수동 설치가 필요하다. 첫 지원 릴리스는 업데이트 및 관련 UX에 집중하고 별도 대형 기능은 섞지 않는다. 주 담당이 공유 schema·Cargo/lock·인터페이스를 조정한다. 기존 .git/index는 보존하고 정상 worktree를 사용한다. 공개 태그/Release·운영키 외부 공급은 구현·합성 검증과 구분하며 기존 비밀 전송 승인 범위를 유지한다. 검증은 임시 폴더/합성 fixture/private desktop만 사용한다.

## 인수 조건

앱의 idle/작업 중 gate, 서명·크기·해시 오류, 중단·stale 상태, 준비/실행 실패, 기존 A와 사용자 미디어·도구·프로필 보존을 검증한다. 원 owner 종료·새 버전 readiness·복구 결과를 구분한다. UI/OS 동작은 단위 검사로 대체하지 않는다. 운영키 없음을 임의 test key 신뢰로 대체하지 않는다.

## 검증과 handoff

진행 중. D: 여유 약1.6GiB, /tmp 여유610GiB를 확인했다. 기본 PATH의 rustc는 없으며 기존 pinned Rust/Windows 도구 경로를 확인 후 사용한다. Git 정상 worktree/main은 fc6f559이며 root의 과거 index·불명 lock을 삭제하지 않는다. 낮은 D: 공간에 새 Cargo cache를 만들지 않는다.


## 2026-10-18 updater_audit 읽기 전용 감사

제품 코드 변경 전 감사 단계다. README→PRODUCT→CONTEXT→R08/R10/R11→ADR0012, DESIGN·ARCHITECTURE 및 기존 panel/host/engine 경로를 읽었다. impeccable context loader의 PRODUCT/DESIGN 전체 JSON과 product/audit reference를 읽었다. 제품 장면은 낮에는 다른 작업 옆, 밤에는 진행 확인이므로 기존 System/Light/Dark·회색/파란 역할 토큰·32px 공통 controls를 유지한다. 디자인 가이드의 원본 색상·native 크기 계약은 일반 skill의 웹 색상/모바일 hit area 조언보다 우선한다. 새 Cargo cache/제품 코드/Git/공유 profile/schema 변경은 없다.

### 구현 경계와 최소 API 제안

기존 `AppUpdatePanel::new/busy/stopped/request_stop/poll/ui`는 유지한다(현재 실제 draw 메서드 이름은 ui다). 추가 환경 입력은 `set_environment(UpdateEnvironment)`이고 확인/준비/재시작 각각의 허용 여부와 사용자용 사유를 받는다. 다운로드 중 panel 내부의 중단은 close-only stop과 구분한다. `request_stop`은 영구 stopping=true를 유지하므로 일반 중단 버튼에서 이를 재사용하면 다음 재시도를 막는다.

- `Check`: 사용자 클릭 → bounded release discovery → 서명 metadata 확인. 현재 버전/채널과 새 버전을 구별한다. 운영 고정 키가 없으면 discovery까지만 허용하고 구체적 수동 설치 링크를 제공한다. test key를 제품 기본 신뢰로 넣지 않는다.
- `Download`: 검증된 candidate와 앱 소유 새 저장 공간만 받는다. 공식 bounded transport→원 capability staging→bundle inspection→별도 새 bundle preparation을 worker에서 수행한다. 기존 앱/프로필/미디어/외부 도구는 이 단계에서 수정하지 않는다.
- worker terminal channel은 기존 capacity1이며 completion에 generation/channel과 실제 소유 결과를 넣는다. 진행률은 최신 phase/bytes/total 한 개만 담는 mailbox로 별도 전달하고 UI는 try_lock한다. 매 chunk 무제한 queue/repaint 금지, 완료/오류/ACK는 반드시 repaint한다. idle worker/timer는 없다.
- `Prepared`: 원 VerifiedCandidate/StagedPackage/PreparedUpdateBundle을 typed owner가 유지해야 한다. path/version 문자열만 보관해 다시 열지 않는다. 준비완료는 설치완료/실행권한이 아니다. 취소/오류의 보존·불명 상태는 사실대로 표시하고 경로만으로 삭제하지 않는다.
- `Restart`: `take_restart_request()` 같은 단발 intent를 Root coordinator에 반환한다. 앱 idle/startup-ready/원 설치 receipt identity/서명 정책/지원 host/원 supervisor handoff를 재검증하고 CloseWait에 들어간다. 키·handoff·rollback 지원이 없으면 버튼을 활성화하지 않는다. 첫 지원 버전까지 기존 v0.1.0 사용자에게 1회 수동 설치가 필요함을 설명한다.

ReleaseObservation은 기존 version/url/is_newer/channel 계약을 유지하고 `notes()->&str`(plain text UTF-8 최대16KiB), `signed_metadata_available()->bool` 관측 accessor만 추가할 수 있다. 기존 API body256KiB/preview20개 상한을 유지한다. assets는 최대64개, metadata/signature는 version별 exactname·canonical GitHub release URL·중복없음·크기상한을 확인한다. 이 존재 관측은 서명/다운로드/실행 권한이 아니다. version/channel을 바꾸거나 최신 관측에 실패하면 기존 자료는 캐시임을 표시하며 이전 signed candidate의 실행을 허용하지 않는다. 변경내역은 실행/Markdown HTML·외부 이미지 없이 egui plain text+접힌 세로 영역으로 표시한다. 새 profile field는 추가하지 않는다.

### 연결 위치와 확인한 gap

- `accessible.rs:2053`은 모든 화면에서 panel poll을 호출하므로 다른 탭에서도 ACK/진행률 수신이 가능하다. 유지한다.
- `accessible.rs:2612`는 현재 enabled가 `!close_wait`뿐이다. 읽기 조회와 다운로드/준비/재시작 권한을 분리하고 Root가 ready/worker/local-export/tools/library/diagnostic/appearance/tool-pref/picker/media gate를 계산해야 한다. 관측 조회 자체는 복구 실패에서도 허용할 수 있으나 새 설치 작업은 실패 상태에서 금지한다.
- `accessible.rs:2140` close guard와 `2196` stopped 조건에는 updater worker가 이미 포함된다. 그러나 `2156` 근처 close prompt의 실제 busy 목록에는 updater가 빠져 있다. 실제 다운로드 도입 시 사용자에게 중단·종료 결과를 알려야 한다. 일반 확인과 다운로드/준비를 분류해 prompt 여부를 결정한다.
- `accessible.rs:3004` 명시 stop 및 Candidate Drop의 stop은 이미 updater에 전달한다. Drop은 ACK를 기다리는 정상 종료 수단이 아니므로 정상 업데이트 전환은 반드시 기존 CloseWait에서 모든 worker/폴더 picker가 종료된 뒤 처리한다. request_stop 후 prepared ownership을 버리는 경우에도 기존 파일을 삭제하지 않는다.
- `software_host.rs:639`는 CancelClose/Close 처리 뒤 event_loop.exit한다. `run` 종료 결과와 기존 entry의 instance guard/supervisor owner 수명을 감싸는 Root callback/출구 mailbox가 필요하다. draw/poll 안에서 helper spawn은 금지하며 fatal host exit/Destroyed 경로는 성공 handoff가 아니다.
- eframe은 `accessible.rs:213` run_native 반환과 Candidate::update만 있다. 별도 callback/outcome owner를 run 외부에 두어 정상 CloseWait ACK 뒤 요청과 host 정상 반환을 함께 확인해야 한다. on_exit만으로 프로세스 종료·원 supervisor Job 인계가 완료됐다고 판단하지 않는다.
- 기존 startup worker는 download journal/설정 복구다. updater 전환 journal/실패 rollback recovery는 별도 coordinator 책임이며 앱 첫 실행 gate에 합쳐야 한다. path 존재/이전 sequence만으로 prepared 복원·새 앱 성공을 표시하지 않는다.

### 첫 지원 버전 UX와 필요한 중앙 문자열

기본 화면은 현재버전/채널, 한 줄 상태, 새버전과 변경내역, 한 개 primary 행동 순서다. 큰 카드 중첩/상시 기술 설명/물음표 버튼 증식은 피한다. 일반 오류·지원 불가 이유는 hover에 숨기지 않는다. 다운로드만 실제 bytes/total progress를 표시하고 검증/준비는 phase 문구로 구분한다. 100% 다운로드를 설치 완료로 표시하지 않는다. 취소 요청 후 ACK까지 중단중을 유지하고 재시작은 명시 사용자 클릭에만 진행한다.

Root catalog 소유자에게 필요한 의미별 새 문자열: 서명 확인중, 다운로드중, 다운로드된/전체 용량, 패키지 검사중, 새 버전 준비중, 준비완료(아직 적용 전), 다운로드 및 준비, 지금 다시 시작하여 적용, 작업 종료 후 재시작 가능, 시작 복구 확인중/실패, 설치형만 지원 또는 포터블 수동 설치, 운영 서명키 미설정, 이 버전은 수동 설치 필요, 지원 metadata 없음, 서명/버전·sequence/크기·해시 오류, 저장 공간 부족, 준비 실패·기존 앱 보존, 중단 요청/중단 완료·보존 결과, 변경내역/변경내역 없음, 재확인/릴리스에서 수동 설치, 복구 진행/복구 실패·기존 버전 유지. 실제 지원조건이 확정되기 전 설치형/포터블 허용 문구를 사실처럼 표시하지 않는다.

### 회귀 및 실제 인수 제안

release notes UTF-8 경계·과대본문/asset 중복·가짜 URL·누락 서명, 키0이면 다운로드 호출0, forged signature/채널변경/stale generation, progress latest-only/total0/과대 카운트, 다운로드 취소와 ACK전 CloseWait, 작업/복구/외부도구 busy에서 재시작0, partial preparation 실패의 기존 앱·프로필·미디어 보존, ACK전 종료0, 정상host출구와fatal출구 구분, 같은 restart intent 재사용0을 synthetic 시험한다. 실제 Windows private desktop에서 Light/Dark·800/960·150/200%·Tab/Enter/Escape·hover/pressed/disabled/긴 변경내역 scroll·진행중 탭전환·닫기·중단·재시도를 확인한다. 이번 감사는 코드 실행/화면 캡처 인수 완료가 아니다.

현재 정적 감사 점수는 접근성2/성능3/반응형3/테마3/불필요 장식4 =15/20다. 기존 공통 controls/상태+문자/단일scroll/worker ACK는 유지할 장점이다. 새 기능 기준 핵심 P1 gap은 독립 download/restart gate, 실제 단계 진행률, close prompt, 정상host 인계·startup rollback recovery다. 시각 반패턴 판정은 새 native 화면이 없으므로 미검증이며 정적 코드의 중첩카드/장식gradient 부재만 확인했다. 최종 개선 후 impeccable polish 관점의 실제 native 검수를 수행한다.


### 승인된 후속 구현 범위

Root가 discovery release notes·metadata 두 asset 관측과 read-only UI 표시를 승인했다. 기존 공개 panel caller 계약은 유지하며 Environment/RestartRequest/프로필 schema 변경은 후속 합의까지 구현하지 않는다. ReleaseObservation의 새 notes/notes_truncated/signed_metadata_available는 단순 관측이다. asset은 exact name·canonical URL·uploaded·nonzero/bounded size를 확인하고 key/signature/설치 권한으로 쓰지 않는다. catalog는 Root 담당으로 필요한 상수를 요청했고 전달 전 임시 literal 제품 문구를 추가하지 않는다.


### read-only 구현과 국소 실행 결과

승인된 두 소스만 구현했다. discovery는 bounded notes/UTF-8 경계와 잘림 관측, canonical uploaded metadata 두 asset의 중복·크기·URL 검사를 추가했다. UI는 inert plain-text 변경내역과 최대180px 내부 scroll, 잘림 안내, missing signed metadata의 수동 설치 이유와 stable/preview별 NoRelease를 표시한다. 기존 new/busy/stopped/request_stop/poll/ui 호출 계약과 조회 cancel/deadline/ACK는 유지했다. Root가 제공한 catalog를 이 worktree에 빌드용으로 복사했지만 소유 변경·통합 patch에 포함하지 않는다.

Linux Rust1.94, 기존 CARGO_HOME=`/mnt/d/project/dlpad/.local-tools/cargo`, CARGO_TARGET_DIR=`/tmp/dlpad-update-presentation-target-20261018`, DEV/TEST DEBUG=0, INCREMENTAL=0, --locked --offline로 실행했다. D:에 새 Cargo cache는 만들지 않았다.

- `cargo test -p dlpad-engine --lib --locked --offline app_update::discovery::tests:: -- --nocapture`: 8 PASS/0 FAIL, compile28.02초/실행0초. 기존 media_preview.rs:591 unused_mut 경고는 변경하지 않았다.
- 소유 두 파일 `rustfmt --edition 2024 --config skip_children=true --check`: exit0.
- `python3 scripts/check_docs.py --source-only`: 1131 documents PASS. local-only runtime evidence는 검사하지 않았다.
- `cargo test -p dlpad --bin dlpad-accessible --features alternative --locked --offline accessible_app_update::tests:: -- --nocapture`: baseline 비Windows test compile 오류로 exit101. accessible.rs에서 Instant import16건 및 PathPicker::completed_for_test 미지원1건이며 소유 panel의 컴파일 진단은 없었다. 담당 밖 코드는 수정하지 않았다. 따라서 신규 panel2개 포함 전체 panel8개의 실행 통과를 주장하지 않는다.

Root Windows 재검증 필터는 `app_update::discovery::tests::`와 `accessible_app_update::tests::`이다. 제품 UI의 물리 화면/200%/상호작용 및 실제 updater bootstrap 연결은 후속이며, 이번 관측·변경내역 작업을 자동 교체 완료라고 표현하지 않는다.

동결 소스 SHA256: discovery.rs `c51ea18c45826f3b7a71ef2b840d85ba4c42b181f9a62d5d2a256e9e25679716`; accessible_app_update.rs `40c63c785ce1a5d3c161c0a82fff1f6bf83312f39e124ca260fa6c778b078264`. Root에 경로·SHA와 최소 integration API를 전달했다. 새 bootstrap parent/client 계약이 확정되기 전 Environment/RestartRequest·소유권 전환 코드·profile 변경은 추가하지 않았다.

- `cargo check -p dlpad --bin dlpad-accessible --features alternative --locked --offline`: exit0, 47.55초. 기존 Linux engine unused_mut 및 외부 열기 adapter의 cfg별 dead_code5개 경고가 있으며 소유 discovery/panel 진단은 없었다. 제품 코드 컴파일 통과와 test 실행 미완료를 구별한다.


### 부모 소유 updater UI 연결 단계 (승인됨)

Root가 신규 app_update_client.rs와 기존 panel/discovery 담당 범위를 승인했다. parent가 signature/download/prepared capability를 소유하고 UI는 bounded typed intent만 보낸다. 기존 read-only 두 소스는 SHA를 통보하고 /tmp의 독립 checkpoint로 보존했다. Root는 accessible/host/main/catalog/shared bootstrap API를 소유한다. client는 background actor1, command1, latest state1로 긴 I/O를 UI 밖에서 수행하며 prepare cancellation ACK 뒤 종료한다. phase와 고정 오류1~12는 Root 확정 계약을 따르고 새 프로필/Cargo/schema 수정은 없다.


### accessible.rs 통합 담당 이관

Root가 accessible.rs의 독점 수정 책임을 이관했다. Root와 본 worktree 원본 SHA가 동일함을 확인한 뒤 작업한다. panel lazy None은 유지하며 global client의 busy/shutdown/stopped를 normal CloseWait와 fatal host drain에 포함한다. startup-ready 및 worker/tools/library/history/diagnostics/appearance/toolpreferences/media/export/picker idle 환경을 전달하고 Parent RestartAccepted만 정상 drain을 요청한다. Root는 host/main/catalog·공유 bootstrap를 계속 담당한다.


### 부모 소유 업데이트 연결 구현과 실행 결과

ClientHandle은 clone/snapshot(try_lock Option)/bind_context/prepare/cancel/request_restart/notify_runtime_ready/begin_shutdown/stopped/busy를 제공한다. Windows install은 UiChild를 background actor 한 개로 옮기고 중복 install을 거절한다. UI에는 version≤32byte, generation, 고정 phase/error code, signed size상한 내 bytes/total만 보인다. 원 metadata/키/패키지/설치 capability는 parent에 남는다. RuntimeReady는 첫 성공 프레임 local intent를 받은 뒤 actor에서 최대10초 RPC로 알린다. 그 전에 try_event/prepare를 호출하지 않는다.

command capacity1과 최신 state1을 사용한다. 취소·shutdown은 atomic 우선이며 UIpaint는 pipe/network/join을 호출하지 않는다. 준비 중 shutdown은 CancelPrepare를 보낸 뒤 parent의 terminal 응답을 기다려 peer를 닫고 stopped ACK를 공개한다. UI만의 별도 ACK timeout/프로세스 kill은 추가하지 않았다. RestartRefused는 prepared version/generation을 유지하고, 일치 generation의 RestartAccepted만 한 번의 host exit intent가 된다. 원 app/profile/media/tool 삭제·교체 권한은 client에 없다.

공유 부모 계약에 따라 generation은 새 Prepare마다 증가하고 시작 전 실패도 새 >0 generation을 사용한다. 준비 중 이전/다른 세대 이벤트, 잘못된 version·phase·received/total는 성공으로 게시하지 않는다. 256MiB 초과 progress를 거절한다. terminal protocol 실패 뒤 버튼은 비활성화하고 앱 재실행/수동 릴리스 행동을 안내한다.

Runtime 비용은 새 의존성0, background thread1, command1, snapshot1, 버전 문자열32byte 상한이다. parent event를 확인하는100ms recv_timeout 때문에 idle actor가 약10회/초 깨어날 수 있다. 같은 snapshot에서는 UI repaint를 요청하지 않는다. 이 비용은 설계 상한이며 실제 Windows CPU/RSS/idle 측정은 Root 인수로 남긴다. thread stack의 가상 예약량을 실제 메모리로 주장하지 않는다.

accessible 통합은 global client를 별도 조회해 settings panel이 None인 경우도 close/fatal-drain/Drop에 stop과 stopped gate를 적용한다. read-only discovery와 prepare/restart 요청을 busy로 보되 prepared 상태 자체는 idle이다. Parent 승인 후에는 기존 download·media·tool/history/library/diagnostic·appearance/preferences·picker·export를 모두 정상 중단하고 CloseWait의 원 ACK 조건을 통과해야 Close를 보낸다. 새 다운로드/복구/도구 작업·폴더 설정·미디어 drop/새 작업 행동은 상호 gate한다. 기존 lazy panel None은 유지했다.

Root는 main/software_host의 실제 bootstrap install/첫 성공 paint callback과 parent coordinator·중앙 catalog를 담당한다. nonCPU alternative host에서 사용하지 않는 bootstrap entry의 dead_code 허용은 Root module선언에 한정하고 CPU 제품 lint는 유지하도록 권고했다.

- /tmp 독립 harness는 원 panel/client 파일과 원 ui_tokens/ui_theme/catalog/controls를 사용했다. controls의 private(crate) 함수만 얇은 forwarding wrapper로 연결하고 변경된 동작을 재구현하지 않았다. product 테스트 오류를 피하려고 unrelated accessible 테스트를 통과했다고 표시하지 않는다.
- `cargo test --manifest-path /tmp/dlpad-update-presentation-harness-20261018/Cargo.toml --test presentation --locked --offline -- --nocapture`: 최종16 PASS/0 FAIL, compile22.21초/실행0.43초. panel10+actor6이며 취소 ACK, generation/stale, restart거절/승인, nonblocking snapshot, 실제 egui literal/오류 표시를 검사했다. fixture panic 문구는 catch_unwind 실패 경로의 의도된 출력이며 테스트 결과는 성공이다.
- 전체 Candidate의 원 production modules를 /tmp entry에서 등록해 `cargo check --manifest-path /tmp/dlpad-update-presentation-harness-20261018/Cargo.toml --bin candidate-check --offline`: exit0,53.30초. 최초 harness는 iced 직접 의존성이 빠져 exit101이었고 /tmp manifest만 실제 app과 맞춰 다시 통과했다. repo Cargo/Cargo.lock 변경0. Windows install이 없는 Linux stub 경로와 기존 adapter에서 dead_code18건/engine unused_mut 경고를 구분했다.
- 소유 accessible/panel/client/discovery `rustfmt --check`: exit0. Windows IPC/실제 renderer/전체 workspace strict lint와 새 accessible CloseWait 회귀는 Root가 통합 실행한다. 이번 Linux harness를 Windows 전체 성공으로 표시하지 않는다.

동결 최종 SHA256: accessible.rs `c2b23f52111157e770f65c9fafed378b79facec0840202d8884eaac977489f27`, panel `8df34d3b8ddaa43dc5f803691d0630a91f1a9e67e7a100036f75aefd7460e445`, client `3238c4eb9c58d18961ab09bcfcb7463e5126c8089b364c3aeef824505b6abd04`, discovery `c51ea18c45826f3b7a71ef2b840d85ba4c42b181f9a62d5d2a256e9e25679716`. Root에 복사/Windows 통합 시작을 알렸고 그 뒤 소스 추가 수정은 하지 않았다. 운영키 기본 신뢰/공개 릴리스/사용자 데이터 전송은 수행하지 않았다.


### private UI 검수 도구 준비 범위

Root가 `.local-tools/update-ui-review-20261018/`의 observer/runner 준비를 승인했다. 기존 windows_ui_rework_private_probe.cs의 private desktop·UIA·원본 파일 pin·create-only 증거·Job cleanup 코드를 기반으로 새 headless parent+window child 구조를 관측한다. binary 준비 전 앱 실행은 하지 않는다. 창은 단순 PID가 아니라 원 parent HANDLE/creation, child HANDLE/creation, 직접 parent tree, 같은 pinned image, 원 Job membership, private desktop·원 HWND/class/thread/property token으로 검증한다. current/preview/notes/실제 관측된 progress/지원 사유와 Light/Dark/800·960/앱200% raster를 준비하며 존재하지 않는 다운로드 progress를 synthetic 화면으로 가장하지 않는다. 원 input desktop 유지·visible fallback0·최종 Job0을 확인한다. Root main/host/제품 소스는 이 도구 작업에서 수정하지 않는다.


### 취소 경합과 시작 복구 알림 후속 인수

Root 감사로 발견한 두 경로를 client/panel에 수정했다. cancel_sent 이후 Prepared는 cancellation ACK가 아니므로 Cancelling/busy를 유지한다. 이후 Failed1(Cancelled) 또는 Failed4 등 실제 operation terminal은 정상 종료로 수락하여 무한 대기를 막고, 뒤따른 취소 ACK는 idle에서 무시한다. cold rollback의 Recovered13/gen0은 runtimeReady+Idle+local command pending 없음에서만 알림으로 수락하고 수행 중 요청을 덮지 않는다. Root가 control/catalog/host wire 의미와 Windows install reexport wrapper를 통합했다.

`cancel-recovery-v1.log` 및 `.result.json`을 /tmp 독립 harness에 저장한 실제 재실행은 18 PASS/0 FAIL,0.81초,exit0이다. client SHA는99ec56f8451a70b123dd039f30b75dc53dd2cc065e98ddcdadcf358e2e1d76ee, panel SHA는87360d2107ec1f668e4d488f2b469c9688245f94750e42aa9c706d3044107bc8로 Root에 전달했다. accessible/discovery는 앞서 동결한 SHA를 유지했다. Windows 통합본의 install wrapper 한 줄은 Root 변경이며 이 worktree 원본으로 덮어쓰지 않는다.

### observer 준비 결과 (제품 미실행)

`.local-tools/update-ui-review-20261018/`에 기존 private GUI 수집기의 bounded/UIA/증거 코드를 바탕으로 parent+child용 observer와 Python runner를 준비했다. `run_review.py --compile-only`는 Windows 내장 .NET csc로 winexe를 빌드해 exit0이며 앱을 실행하지 않았다. observer 자체 console 창도 만들지 않는다. C# 소스 SHA aaebdca8ca66bd0abc4de2db344281902dacc614544fd40639b335d2c37a1b4e, 실행파일 SHA c5c699ff63bcc9268d209263528da214643014612fffde82d7a25aa90aa1038f를 build receipt에 기록했다. 이전 수집기의 사용하지 않는 S10 문자열 필드 두 개에 CS0649 경고가 있으나 컴파일은 성공했다.

Root가 최신0.1.1 binary/SHA를 확정하기 전 GUI 실행은 대기한다. private desktop은 no visible fallback이고 source exe read pin, original parent live HANDLE/creation, child live HANDLE/creation/direct-parent tree, same pinned image, outer Job membership, thread desktop, original HWND/thread/class/property token을 매 행동 전 검증한다. main PID로 창을 추측하지 않는다. 최종 parent+child exit0 및 Job0·input desktop unchanged를 요구하며 강제 정리가 필요하면 FAIL이다. 실제 signed download progress가 없는 공개 릴리스에서는 query checking/current/preview/notes/manual 제약만 관측하고 다운로드·설치·재시작 성공을 주장하지 않는다.

상대경로가 물리 D:/Project 대소문자로 resolve된 observer 후속 쓰기에서 EROFS를 한 번 관측했다. Root가 알려 준 sandbox 경로 규칙에 따라 소문자 절대 /mnt/d/project/dlpad 경로로 수정해 정상 저장·컴파일했다. 마운트·권한·시스템 설정 변경은 하지 않았다.


### 전역 업데이트 복귀 안내 후속 (2026-10-18)

- Root 감사의 사용자 오인 위험을 반영해 기존 `accessible.rs` 담당을 다시 인수했다. 범위는 업데이트 실패 후 이전 버전 복귀를 기본 다운로드 탭을 포함한 모든 화면에서 알리는 안내이며, 기존 시작 복구 상태와 분리한다.
- Candidate에 Unseen/Visible/Dismissed 상태 하나를 두고 runtime-ready의 Failed13/generation0만 최초 관측한다. 공통 상단 패널의 중앙 catalog 문구와 표준 secondary 확인 버튼을 사용하며 확인 후 같은 상태 관측으로 다시 열지 않는다.
- 실제 Candidate 4개 탭 렌더링, 문구의 painter clip 포함, AccessKit Focus 후 Enter 확인, 재관측 시 미재표시, startup status 보존, 앱 업데이트 lazy panel None 회귀를 추가했다. Windows 실제 실행은 Root가 통합한다.
- rustfmt 실행 exit0. 소스 SHA256: `59c6cd777596912db2bf1ff6eeef0a19b3fe0ee8cdbbf85ab55e6dd280a89507`. 로컬 catalog는 Root 원본을 컴파일 목적으로만 동기화했고 제출 범위에 포함하지 않는다.
- private GUI 관측기 컴파일은 성공했지만 제품 실행은 아직 0회다. Root가 최종 배너 포함 빌드를 전달할 때까지 baseline 실행을 보류한다. fixture env `DLPAD_LIBRARY_ROOT`가 비어 있지 않으면 Candidate startup recovery가 initial/default Documents 위치를 생성하지 않는 분기이며 bootstrap 자식은 null environment로 부모의 fixture env를 상속한다. 따라서 관측용 임시 media/data만 사용한다.

- 전역 배너 후속 실제 production-module Linux check: `cargo check --manifest-path /tmp/dlpad-update-presentation-harness-20261018/Cargo.toml --bin candidate-check --offline`, exit0, 15.52s. 기존 platform/dead-code 경고 18개가 있는 비제품 harness이며 Windows strict lint 결과로 대체하지 않는다. 로그 `/tmp/dlpad-update-presentation-harness-20261018/global-recovery-check.log`. 실제 Windows render/키보드 회귀와 최종 PNG 검수는 Root 후속 빌드 대기 중이다.


### Windows private UI functional 검수 및 후속 링크 교정 (2026-10-18)

- Root v7 native strict workspace0, app_update102PASS 및 전역복귀배너 실제 렌더 회귀 각2bin1PASS 확인 후 승인한 debug CPU binary SHA256 `41933d3876d501fa7a8e68b0f3f42e9da28fff6ec973a21c48f0914b14bf5a2f`를 실행했다. Release 성능 판정이 아니다.
- 관측기 초기 실패는 private desktop 빈 enum, 외부thread GetThreadDesktop 질의, 지원되지 않는 UIA action pattern, 일반탭 탐색 누락이었다. 제품 성공으로 보고하지 않았으며 gui-v1/v2/v4/v5 및 observer-v3 컴파일 실패 증거를 보존했다. 실패마다 소유 Job만 종료해 active0, 부모/자식 종료, private desktop 닫힘, 사용자 input desktop unchanged를 확인했다.
- observer-v6는 original parent creation+Job, 원자식 direct parent/creation/image/Job, 원 private desktop handle의 매회 HWND/thread enumeration, 원 window class/property token을 검증한다. UIA 및 원 HWND 경계 안에서 검증한 창 메시지만 사용하고 SwitchDesktop/SendInput/foreground/clipboard/visible fallback을 사용하지 않는다.
- `python3 .local-tools/update-ui-review-20261018/run_review.py --observer-name observer-v6 --binary <approved debug exe> --sha256 41933d... --output .local-tools/update-ui-review-20261018/gui-v6`: native exit0, 101.769s. 첫 parent29416/child28340, 재실행 parent26460/child29092 모두 자연 exit0, 두 Job active0 자연 종료, private desktop closed, MTA 복귀 및 input Default unchanged.
- PNG26장 중 Light stable/preview, Light800/200%, Dark800/200%, 재실행 Dark preview 실제 이미지를 직접 열어 한글 폰트, 선택/focus, 줄바꿈과 주요 레이아웃을 확인했다. 임시 theme.sqlite를 read-only 조회해 theme Dark와 library_entries 빈 상태를 확인했다. 증거 `.local-tools/update-ui-review-20261018/gui-v6/independent-review.json`; native REVIEW_REQUIRED 원본은 덮어쓰지 않았다.
- 실물 UI에서 현재0.1.1보다 오래된 preview0.1.0에도 설치를 유도하는 링크와 수동설치 안내가 보였다. Root 승인 후 panel만 수정해 신규버전에만 설치 fallback, 같거나 오래된 릴리스에는 중앙 `APP_UPDATE_RELEASE_VIEW` 문구를 사용하고 수동설치 안내를 숨겼다. Root 소유 catalog는 컴파일용 동기화만 하며 제출 제외한다.
- 작은 창의 notes/body 중첩 스크롤은 주요 기능 차단 없이 후속 UX 작업으로 기록했다. 실제 signed package 다운로드 진행률/설치활성화·rollback, 실물 OS DPI/physical keyboard 및 Release 성능은 이번 관측으로 검증하지 않았다. 복귀 안내는 Root의 실제 렌더 회귀로 검증했으며 실제 운영 업데이트 rollback을 관측한 것으로 주장하지 않는다.
- panel 후속 rustfmt0, frozen SHA256 `37337605a48ca20deea1278b6fa04f609a315bccd96110fb65c9f74088c6a180`. 최종 Release 바이너리의 링크 교정 화면 확인은 Root 후속 통합 검증 대상이다.

- 후속 링크 교정 로컬 검증: 최초 `--lib`는 normal-library 전용 harness에서 native controls의 cfg(test)를 켜서 accessible 모듈이 없는14 compile errors로 exit101했다. 이 잘못된 target 실행을 성공으로 쓰지 않는다. 정확한 기존 통합 target `cargo test --manifest-path /tmp/dlpad-update-presentation-harness-20261018/Cargo.toml --test presentation --offline`는 exit0, **18PASS/0FAIL/0.82s**. 로그 `/tmp/dlpad-update-presentation-harness-20261018/release-link-v2.log`; 실제 제품 Windows 통합/최종 Release 시각검증은 Root 담당이다.


### 최종 Release 실물 UI 재검수 (2026-10-18)

- Root 최종 workspace fmt/clippy/test/release 모든 명령0 및1837PASS/0FAIL/44ignored 보고 후 CPU Release 원본을 독립 읽기해 11082240B, SHA256 `65c9bd4d54c5f26b503817bea6ddceaa6a2804b29949c3cd97711e6afcff695f` 일치를 확인했다. Root 승인 후 observer-v7로 실행했다. 제품 소스는 계속 동결했다.
- 명령은 기존 run_review.py에 `--observer-name observer-v7 --binary <approved target/release/dlpad-cpu.exe> --sha256 65c9bd... --output .local-tools/update-ui-review-20261018/gui-release-v1`. native exit0, 109.390s. 첫 parent29496/child27176, 재실행parent26104/child25784 모두 자연exit0, 각 Job active0 자연종료, 강제종료0, private desktop 닫힘/MTA복귀/input Default unchanged.
- Release 실제 PNG26장 중7장을 직접 열어 Light stable/preview, Light800/200%, Dark800/200%, 재실행Dark preview를 검수했다. 더 오래된 공개 preview0.1.0에 대해 현재0.1.1에서는 `릴리스 보기` 링크와 불필요한 수동설치 안내 제거가 실제로 반영되었다. 한글 깨짐/주요 요소 겹침은 관측하지 않았다. 임시 theme.sqlite read-only조회로 Dark 유지와 library_entries0을 확인했다. 원 native REVIEW_REQUIRED JSON은 보존하고 `gui-release-v1/independent-review.json`에 독립 판정을 별도로 기록했다.
- Root 요청에 따라 원본 parent/child HANDLE CPU와 메모리 카운터를 quiet10.0076582초 동안 정확1회 참고 관측했다. 그 구간 관측기 UIA/캡처/입력0이며 앱 기본metrics관측은 켜져 있었다. parent CPUdelta0s(Windows accounting 해상도 내), WorkingSet12046336B/private commit2039808B. child CPUdelta0.046875s, 한 코어 기준0.468391%, WorkingSet83365888B/private commit52924416B(after). **WorkingSet은 전체RSS, PrivateUsage는 private commit이며 privateRSS가 아니다.** 한 번의 참고 관측으로 updater만의 순수 추가 비용/성능 인수 통과를 주장하지 않는다.
- 새 observer-v7 source SHA256 `89ee039725bfebcb955c38833c2aa22f4747be615d57cb051b0e86c59d6a2af4`, exe SHA256 `d03aa554be8b0965849300fcd6c825a3595796c938301376219ecf2ae3f1f3d3`. 구버전v6 소스와 모든 실패 증거도 보존했다.
- 여전히 미검증: 실제 운영 서명 package 진행률/설치 활성화·rollback, physical keyboard, OS DPI 변경, first-frame timing 및 독립 성능 인수. 작은 창 notes/body 중첩 스크롤은 후속 개선이다. 일반/앱업데이트 UI 범위를 넘어 설치기 전체를 다시 검수한 것으로 주장하지 않는다.

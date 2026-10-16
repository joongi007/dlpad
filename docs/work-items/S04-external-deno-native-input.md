# S04: 외부 Deno 네이티브 입력 연결

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R07, R11
- 담당/수정 범위: root — crates/app/src/accessible.rs, main.rs, catalog.rs 및 이 문서; diagnostic_cache — 읽기 감사만
- 선행 작업: 선택 Deno typed options·worker 버전 검사·Attempt 진단 연결

## 사용자 결과

기존 외부 Deno의 절대 경로를 네이티브 화면에서 입력하고 실행 계획과 다운로드에 적용한다. 빈 입력은 기존 runtime 정책을 유지하며 외부 파일을 관리 설치로 바꾸거나 제거하지 않는다.

## 계약과 범위

CPU/접근성 공유 화면과 기본 후보의 FFmpeg 입력 패턴을 따른다. 문자열은 중앙 catalog, 입력32768자 상한, 실행 중 변경 금지, core plan으로 절대경로/parent/control 검증. DLPAD_DENO는 합성 시험용 초기 입력이며 영구 전역 환경 변경은 없다. 저장 옵션과 이력 재개는 기존 snapshot을 유지한다. 관리 package UI 연결은 별도 작업이며 새 Cargo/schema는 없다.

## 인수 조건

한글·공백 절대 경로가 option/argv에 literal로 전달되고 잘못된 경로가 거절된다. 실행 중 입력이 고정된다. CPU/기본 후보 컴파일·회귀와 격리 GUI 입력/계획을 확인하며 다른 OS/실제IME는 별도 인수다.

## 검증과 handoff

구현 시작. 실제 GUI 검증 전 완료로 표시하지 않는다.

첫 CPU 전체타깃 Clippy는 exit0. 새 회귀의 첫 컴파일은 OsString/String 비교 타입 불일치로 exit101이며 assertion을 OsString 동종 비교로 교정했다. 독립 읽기 감사는 CPU 공유 Candidate 경로와 저장 snapshot 재개 정책을 확인했다. UI options 경계에서 Deno 설정 후 core plan을 다시 검증하며 worker의 버전 I/O를 UI에 추가하지 않았다. 후속 시험과 실제 GUI 검증은 진행 중이다.

교정 후 `cargo test -p dlpad --locked --features cpu deno` exit0. 기본·접근성·CPU 후보에서 신규 회귀 각각1개, 총3실행이 통과했다. 한글공백 runtime literal, 실행중변경거절, 상대경로 시작차단 및 DB미생성을 확인했다. 실제 GUI와 최신 Windows 검증 전 in-progress를 유지한다.

후속 fmt check와 CPU all-target Clippy 및 `cargo build -p dlpad --bin dlpad-cpu --release --locked --features cpu` 모두 exit0. 새 Linux CPU 바이너리 SHA256은 3636206921ce2979096c59fcf25ca43d17e84ac0a4d3476673d853aac1a74c87이다. 이 고정 바이너리의 격리 GUI 수집기와 Windows GNU 전체 collector를 실제 시작했다. terminal 전에는 GUI/Windows 통과로 집계하지 않는다. 수집기의 상대 evidence 입력은 절대경로 계약으로 거절되어 실행되지 않았고 자체 ROOT를 사용해 경로 별칭 문제를 교정했다.

Windows GNU CPU collector의 capability/fmt/clippy/test/release가 terminal exit0이며 passed/cargo_checks_complete=true이다. root는 result.json source_baseline을 현재 snapshot과 대조하여 SHA 불일치0을 확인했다. 증거: docs/validation/evidence/deno-native-input-windows-workspace-live/result.json. Windows GUI/공식 Deno 실행과 macOS 증거가 아니다.

Linux 첫 격리 GUI는 선택 경로의 실제 Text exact값과 metadata가 아닌 plan argv(--no-js-runtimes/--js-runtimes/deno:literal) 정확 일치를 확인했으나 후속 관측 timeout으로 exit1이다. 원 result.json/PNG를 보존했고 owned_cleanup=true 및 바이너리 SHA 전후 일치를 확인했다. 전체 GUI PASS로 표시하지 않는다. 수집기 preview toggle 경계를 독립 검토 중이며 Linux 전체 workspace collector도 실제 시작했다.

첫 후속 GUI의 빈값 삭제 후 검사는 actualText=''를 확인했지만 화면 밖의 빈본문을JSON으로파싱하여실패했다. 다음 재시험은같은이름의라벨Text를본문으로읽은것을확인했고, 역할기반교정의첫시험에서는수집기의추측LABEL28과실제29불일치로실패했다. 모든원시결과/소유정리/SHA를보존하고실제GetRole29만skip하여본문TEXT/ENTRY를읽는재시험을진행한다. 이수집기실패들을제품GUI전체PASS로집계하지않는다.

독립 FFmpeg 후속 공급 감사는 확보된Linux tar.xz 및ffmpeg/ffprobe/LICENSE hash일치를확인했으나선택세트271.65MiB/73nestedentries가현재ZIP-only/256MiB계약에맞지않음을기록했다. [후속 감사](S04-ffmpeg-package-followup-audit.md)의압축형식·용량·공급trust/notice조건을충족한뒤관리공급을연결한다.

최신 Linux 전체 collector는 state=completed/workspace_checks_complete=true로 terminal exit0이다. fmt·기본clippy/test/release·CPUclippy/test/release와toolchainversion2개가모두exit0/source_unchanged=true이며root가source_final현재SHA불일치0을독립확인했다. 증거: docs/validation/evidence/deno-native-input-linux-workspace-live/result.json. WindowsGNU전체5operationPASS와같은제품Rustsnapshot검증이며실제GUI/IME/공식runtime/3OS릴리스의완료를뜻하지않는다.

## 실제 Linux GUI 인수 완료 범위

`cpu_deno_plan_probe.py`의 고정SHA observed-role 재시험은 terminal exit0/PASS이다. root가 result.json의 실제Text·selected/empty argv·relative고정오류·원URL생략·DB/실행sentinel미생성·closeexit0/ownedcleanuptrue/SHA전후일치를직접대조하고 relative.png의오류가시성을검토했다. [원시 결과](../validation/evidence/deno-native-input-linux-gui-observed-role/result.json). 사용자인증/영상/clipboard/visiblefallback은없다. 신규화면입력은공백·달러·작은따옴표의literal이며한글path는별도core/UI단위회귀다. 실제한글IME·WindowsGUI Deno/공식runtime·macOS·관리공급전체는후속게이트다. 첫네실패원시는보존한다. 최신Linux기본/CPU전체와WindowsCPU전체도terminalPASS/source현재SHA일치이며전체P0완료를뜻하지않는다.

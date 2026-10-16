# S00: 사용자 평가에 따른 Windows UI/UX 재작업

- 상태: in-progress (1차 UI·아이콘 후보 구현/Windows whole·private GUI·성능·ZIP 검증 완료; 후속 UX·최종 사용자 인수 남음)
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — accessible.rs, software_host.rs, catalog.rs, 디자인·검증 문서; 아이콘 모듈/Windows executable resource는 인터페이스 확정 후 기록
- 선행 작업: Windows 개발 미리보기 및 사용자 직접 평가

## 사용자 결과

gtmux 기준의 차분하고 일관된 앱 화면과 실제 앱 아이콘을 제공한다. 다운로드 시작까지의 주요 동작을 빠르게 찾으며 기술 옵션은 단계적으로 펼친다.

## 계약과 범위

2026-10-07 사용자 평가는 가이드 미준수, 앱 아이콘 부재, 현대적이지 않은 UX를 지적했다. UI/UX 인수는 통과가 아니다. Rust 네이티브 제약은 유지한다. PRODUCT.md와 DESIGN.md 및 실제 gtmux tokens.css를 기준으로 스타일·정보 구조를 대조한다. 작업별 수정 전 담당 파일을 확정하며 Cargo/lock/shared interface는 root가 조정한다. 기존 기능 검증을 사용자 경험 인수로 승격하지 않는다.

소스 대조: accessible::configure_context는 버튼 높이32만 지정하고 기본 egui 스타일의 색상·간격·타이포그래피를 대부분 사용한다. 현재 navigation은 상단 제목과 가로 탭으로, DESIGN의 좌측 탐색/목록·상세 구조와 다르다. software_host의 Window::default_attributes에는 앱 아이콘 지정이 없다. 가이드가 존재하지만 제품 화면으로 충분히 이전되지 않았다. Rust 때문에 불가능하다는 증거는 없다.

## 인수 조건

Light/Dark/System 모두 gtmux 역할 토큰·간격·타이포그래피·상태 표현을 대조한다. URL/품질/저장위치/시작의 위계를 명확히 하고 고급 제어의 점진 공개를 유지한다. 작업표시줄/창과 Explorer 실행파일 아이콘은 각각 실제 Windows에서 확인한다. 정상·빈 목록·실패·진행중·설정·파일목록, 800x600과 배율100/150/200%, 키보드 포커스·한글 입력을 검증한다. 자동 화면 검증은 private desktop만 사용하며 visible fallback은 하지 않는다. 변경 binary에서 전체 검증과 적절한 GUI/성능 회귀를 수행하고 새 ZIP을 만든 뒤 사용자에게 재평가를 요청한다.

## 검증과 handoff

실행: impeccable context loader, 원본 gtmux tokens.css 읽기, accessible.rs/software_host.rs/accessible_settings.rs 소스 대조. 실제 수정 구현·새 빌드·시각 인수는 아직 미완료다. shell의 rustc는 PATH에 없고 Git은 exit128 not-a-repository이며 .git을 보존한다. 기존 프로젝트 전용 Windows toolchain을 사용할 수 있다. 종전 ZIP의 기능 시험 성공은 유지하되 사용자 UI/UX 승인으로 표현하지 않는다.

### 수정 담당 확정

root는 accessible.rs·accessible_options.rs·catalog.rs·software_host.rs와 엔트리 모듈 연결·Cargo/build resource·문서·최종 검증을 담당한다. metrics_recovery는 /tmp theme 초안, diagnostic_cache는 /tmp icon/resource 초안, soak_recovery_audit는 /tmp private GUI 검증 초안을 담당한다. 프로젝트 동시 수정·동시 native/Cargo 실행은 없다. 공유 DB/schema/lock는 변경하지 않는다.

### 구현 기록

2026-10-07 root 구현: Light/Dark 두 Style에 실제 gtmux 토큰을 이전하고13/12/16/20 글꼴·8/12/16 간격·32 높이·6 radius를 설정했다. OS System 선택과 기존 한글 fallback은 유지한다. 900logical px 이상에서는216좌측 탐색, 좁은 창에서는 가로 탐색을 사용한다. URL/저장위치/품질을 기본 화면에, 구간·자막과 도구·인증을 접힌 영역에 배치한다. 설정의 입력 검증도 접힌 영역으로 옮겼다. 기존 입력 모델/worker/취소/복구는 변경하지 않는다.

새 파일 ui_theme.rs, app_icon.rs, build.rs와 assets/dlpad.ico·dlpad.res·dlpad-icon.o를 추가했다. 원 기하 도형을16/32/48/256 크기로 생성하고 기존 Windows llvm-rc/llvm-windres를 소유 private Job에서 실행해 리소스를 만들었다. build.rs는 사전 생성 리소스만 연결하며 외부 프로세스나 사용자 홈 경로에 의존하지 않는다. 새 crate/Cargo.lock/schema 변경은 없다. icon32 RGBA는4096B이고 실행파일 크기 증가는 실제 빌드 뒤 기록한다. GUI 창에32아이콘과 PE아이콘 리소스를 별도로 연결하며 실제 taskbar 표시와 코드 연결을 구별한다. 전체 fingerprint에 assets를 포함했다.

첫 whole은 새 탐색 helper의 context 참조 오류로 Clippy가 실패했다. 두 번째는 기존 CPU dead-code attribute 이동과 아이콘 manual_range_contains 경고로 실패했다. 실패 evidence를 보존하고 수정한 뒤 최종 whole을 실행한다. 현재 Clippy와 테스트는 통과했으며 release·실제 화면 검증은 진행 중이다. 아이콘 algorithm/ICO/RES/AMD64 COFF는 독립 읽기 검토했고 root는256아이콘 이미지를 직접 확인했다. 작은 크기/PE추출/현재 화면은 아직 검증 전이다.

### 실제 private GUI에서 발견한 문제

수집기 C#의 short-circuit out 변수 확정 대입 오류를 초기화로 수정했다. URL UIA setter는 실제 입력에 반영되지 않아 원 HWND만의 기존 Fill 메시지+readback으로 시험했다. 최초 대입/강제 소유 Job 정리 실패 evidence는 보존했다. 아이콘 추출은 큰것과 작은것 각 하나를 받아 실제 반환2와 양쪽 HICON을 관측했는데 observer가1로 가정하여 실패했다. 최소 native fixture는 실제RT_GROUP_ICON1/큰·작은2/LoadImage 성공을 확인했으며 [Microsoft API 계약](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-extracticonexw)과 대조해 수정했다. winit의 with_window_icon은 Windows Small만 설정하여 Big 검사에 실패했으므로 with_taskbar_icon을 별도로 연결했다. 제품 변경 후 새 whole과 actual GUI를 다시 수행한다.

root는 중간 밝은 settings960 및 download960/800 PNG를 직접 봤다. 이는 실패 실행의 관측이며 최종 인수로 쓰지 않는다. 새 sidebar는 stock selectable button/접근성 label을 유지하면서 전체 행 크기와 좌측 정렬을 적용한다. 품질의 긴 설명은 도움말에, 첫 사용 도구 준비 안내는 기본 화면에 배치한다. 옵션 오류문구는 theme foreground로 읽도록 수정했다.

### 최종 소스 통합 검사

`windows-msvc-ui-rework-polished-workspace-20261007`의 실제 fmt/Clippy/test/release 네 명령이 exit0이며852 passed/0 failed/13 ignored(41 summaries)다. root는 원 로그/자연 Job 종료/599개 정확 source map과 현재 세 실행파일 SHA·크기를 직접 대조했고 독립 read-only 검토도 보관했다. CPU는9,040,896B(이전 사용자 ZIP 대비330,240B 증가), 설치 화면은1,103,360B(286,208B 증가), CLI는974,848B로 동일하다. 신규 crate·lock/schema 변경은 없고 아이콘 리소스가 증가분에 포함된다.

후속 polish는 좁은 창 여백16, 넓은 창24, footer 상태·디스크 지표 한 줄 배치, 입력 포커스의 중앙 scroll, 품질 설명의 실제 정책인 오디오 우선 표현을 반영했다. private observer는 원 HWND Unicode PostMessageW와 매 문자150ms readback pacing을 사용하며 실제 OS IME 시험으로 표현하지 않는다. 최종 90초 관측 worker 제한도 전체 화면 행렬에 부족하여180초로 늘렸으며 외부 Job 전체 기한은 유지한다. 이전 실패 증거는 보존했다.

빈 파일 목록의 안내 강화와 도구 화면의 중복 제목·기술 설명 정리는 후속 UX 항목이다. 이번 수정으로 모든 gtmux 세부 가이드·모든 상태·물리 키보드/IME·reader·OS DPI 인수를 완료했다고 표현하지 않는다.

polished-layout 최초 실행은 실제 한글 readback·Light100/150/200·창 Small/Big·PE 양쪽 추출까지 확인했으나90초 worker 기한으로 FAIL이다. 자연 종료 인수로 사용하지 않는다. observer만180초로 조정하고 fresh 출력에서 재시험한다.

### 현재 소스 실제 GUI 인수 범위

`windows-msvc-ui-rework-polished-layout-final-real-20261007`에서 Light 초기→Dark 선택→새 PID Dark 재시작의51 PNG와 실제 한글 합성 URL readback을 수집했다. 960x640/800x600 다섯 화면, 앱 배율150/200의 URL·시작 버튼 가시성/접근 flags,100복원, Small/Big 창 아이콘 및 실행파일 아이콘 양쪽 추출을 확인했다. 두 원 process exit0·Job0·private/input 복원은 root와 독립 담당이 원 로그/receipt/pins/현재599 source로 대조했다. theme.sqlite의 Dark 저장과 theme/library DB 무결성도 읽기 전용 검토했다. root는 최종8PNG를 직접 봤으며 열린 UX 항목을 root-pixel-review.json에 기록했다. 물리 작업표시줄 pixels·OS DPI·IME/reader·전체 controls 키보드·실제 다운로드·busy/오류/CloseAwait는 이 GUI 실행의 완료 범위가 아니다.

[빈 상태와 확대 포커스 후속](S00-windows-empty-state-focus-polish.md)은 구현 전 proposed다. 전체 가이드·최종 사용자 UI 승인 및 전체 P0를 완료로 표시하지 않는다.

### 새 binary 비용

`windows-msvc-ui-rework-gui-cost-real-20261007`은 다른 dlpad native 시험 없이 합성1만 목록을 가진 private GUI에서3×60초를 측정했다.18개의 고정10초 구간이 모두 단일 코어 CPU<1% 및 Job 합산 WS/private<80MiB를 통과했다. root가 원 Job kernel/user counter·구간 길이로 다시 계산한 최대 CPU는0.7797142358553474%, WS76,963,840B(약73.4MiB), private43,737,088B다. 전체 호스트가 조용했다는 주장이나 실제8시간 GUI/실제 영상/물리first-paint 증명은 아니다. 첫 loaded목록 PNG도 root가 직접 확인했다.

### 실제 실행 명령

```sh
python3 scripts/windows_msvc_workspace_probe.py --configuration /tmp/dlpad-msvc-workspace-stdout-plan.json --reuse-owned-target --output /mnt/d/project/dlpad/docs/validation/evidence/windows-msvc-ui-rework-polished-workspace-20261007
python3 scripts/windows_msvc_ui_rework_collect.py --proof /mnt/d/project/dlpad/docs/validation/evidence/windows-msvc-ui-rework-polished-workspace-20261007/result.json --output /mnt/d/project/dlpad/docs/validation/evidence/windows-msvc-ui-rework-polished-layout-final-real-20261007 --observer-source /mnt/d/project/dlpad/scripts/windows_ui_rework_private_probe.cs --mode layout
python3 scripts/windows_msvc_whole_gui_cost_probe.py /mnt/d/project/dlpad/docs/validation/evidence/windows-msvc-ui-rework-polished-workspace-20261007/result.json /mnt/d/project/dlpad/docs/validation/evidence/windows-msvc-ui-rework-gui-cost-real-20261007 --no-concurrent-dlpad-probes
python3 scripts/windows_msvc_notice_graph_probe.py --proof /mnt/d/project/dlpad/docs/validation/evidence/windows-msvc-ui-rework-polished-workspace-20261007/result.json --configuration /mnt/d/project/dlpad/docs/validation/evidence/windows-msvc-ui-rework-polished-workspace-20261007/effective-configuration.json --output /mnt/d/project/dlpad/docs/validation/evidence/windows-msvc-ui-rework-graph-20261007
```

위 최종 네 수집 명령은 exit0이다. MSVC whole의 내부 명령은 `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked --offline --features cpu --target x86_64-pc-windows-msvc -- -D warnings`, `cargo test --workspace --locked --offline --features cpu --target x86_64-pc-windows-msvc`, `cargo build --workspace --release --locked --offline --features cpu --target x86_64-pc-windows-msvc`이며 Rust1.94.0이다. 기존 실패 실행들을 이 최종 결과로 덮어쓰지 않는다.

### 새 사용자 후보 및 남은 범위

[새 UI ZIP](../validation/evidence/windows-msvc-ui-rework-graph-20261007/dlpad-user-preview.zip)은13,677,880B, SHA256 `882b1b7628c47e1397870773b85901724e34a7494e9b07452234d0b437c8ca82`다. root와 독립 담당은466 members 전체 SHA/CRC/크기/권한/timestamp, Windows GUI앱·Console CLI·GUI설치 화면 세 PE 역할, 실제 current graph258/whole599 map/binary pins 및455 notices 원자료를 확인했다. 해당 target selected notice 누락0이나 whole-lock43 누락과 CRT/std/linked/legal/서명/cleanOS/정식 설치 미완료는 유지한다. 원3만 합성 파일010203+EOF·정확파일집합과3DB integrity/1만 present행·size3/Windows ID형식/DB SHA전후 동일도 root와 독립 검토했다. NativeID 전수 재조회나8시간GUI 보존으로 표현하지 않는다.

패키지 명령: `python3 scripts/windows_msvc_candidate_package.py --whole-proof /mnt/d/project/dlpad/docs/validation/evidence/windows-msvc-ui-rework-polished-workspace-20261007/result.json --graph-proof /mnt/d/project/dlpad/docs/validation/evidence/windows-msvc-ui-rework-graph-20261007/result.json --native-installer-exe /mnt/d/project/dlpad/.local-tools/windows/msvc-supply-20261006/target-msvc/x86_64-pc-windows-msvc/release/dlpad-install.exe --native-installer-sha256 999342dfb75dea434aeef62905c181fd435036abff3bbdf8adf6bb9e4f7292a9 --output /mnt/d/project/dlpad/docs/validation/evidence/windows-msvc-ui-rework-graph-20261007/dlpad-user-preview.zip` — exit0.

수정 파일 최종 목록: accessible.rs/accessible_options.rs/catalog.rs/software_host.rs, accessible_cpu_main.rs/accessible_main.rs/installer_main.rs, 새 ui_theme.rs/app_icon.rs/build.rs/assets3, windows_current_workspace_probe.py, 새 windows_ui_rework_private_probe.cs/windows_msvc_ui_rework_collect.py, DESIGN.md/README.md/P0-status.md/windows-user-preview.md/작업 문서·병렬 board. UI skill은 impeccable의 product 기준을 적용했다. Git exit128 not-a-repository는 유지했고 기존.git을 건드리지 않았으며 remote CI/공개 배포/사용자 데스크톱 실행은 하지 않았다.

빈 상태·포커스·도구 설명 후속은 별도 작업의 인수 조건에 따라 이어야 한다. 이번1차 후보를 사용자가 다시 평가해야 최종 UI/UX 승인을 판단할 수 있다. 전체P0·정식설치·물리IME/reader/OS DPI·실환경 장기·updater는 완료가 아니다.

최종 `PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_docs.py`: exit0,852 작성 문서 검증·정확SHA 일치 원 notice19개 보존. 이전 추출 ZIP 원문 두 개의 개행 검사 실패는 [notice 문서 검사 구분](S00-packaged-notice-doc-check.md)으로 해결했다. ZIP·원 라이선스 bytes는 변경하지 않았다.

## 사용자 재평가 이후

1차 사용자 후보는 가이드와 공간 구조가 다르다는 이유로 재거절됐다. 위 source599 검사와 ZIP은 당시 이력이며 최종 UI 승인본이 아니다. [실제 원 gtmux 구조 대조·네이티브 폴더 선택·두 번째 레이아웃](S00-native-folder-picker-layout.md)이 현재 작업이다. OS 제목의 앱 이름을 유지하고 내부 브랜드바/고정 sidebar를 제거한다.

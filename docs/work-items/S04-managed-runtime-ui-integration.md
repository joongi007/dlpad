# S04: 관리 FFmpeg·Deno 네이티브 UI 수직 연결

- 상태: proposed
- 분류: feature
- 우선순위: P0
- 요구사항: R03, R05, R08, R11
- 담당/수정 범위: metrics_recovery — 이 문서만; 후속 root 파일·공유 인터페이스 조정
- 선행 작업: 정확 package/version/HTTPS/native삭제·runtime argv 인수, 공급trust/license/notice 정책

## 사용자 결과

도구 화면에서 yt-dlp·Deno·FFmpeg 세트를 선택해 고정 공급 정보와 지원 상태를 확인하고 설치·검증 선택·비활성 제거한다. 선택한 runtime은 다운로드 명시 옵션으로 연결되고 외부 도구는 자동삭제하지 않는다. 임의 manifest 파일 업로드를 설치 UX로 사용하지 않는다.

## 실제 현 위치와 최소 인터페이스

accessible_tools.rs의 Action Refresh/Install/Select(InstallManifest)/Remove/Use와 sync_channel1 worker를 재사용한다. 현재 perform/inventory/Use는 yt-dlp가 하드코딩되고 Outcome Selected(PathBuf)는 accessible.rs497~503에서 self.tool에만 반영된다. accessible.rs에는 deno String과 DownloadOptions.deno가 이미 연결되어 있지만 관리 UI 결과 연결은 없다. managed.rs는 고정 yt-dlp candidate와 다른 native 화면의 별도 Action worker를 보유한다. 중복 UX에 서로 다른 공급authority를 만들지 말고 candidate catalog를 공통화한다.

공유 typed ToolId {YtDlp,Deno,Ffmpeg}와 Candidate {Direct(manifest,bytes),Package(package)}는 app managed.rs 안의 고정 검증 공급으로만 생성한다. accessible_tools Command에 tool ID를 담고 각 refresh 결과는 선택tool의 metadata inventory/active evidence를 포함한다. 설치package는 download_archive_and_install로 inactive만 만든 뒤 UI에서 정확 version 검증 선택을 명시한다. SelectDeno는 select_verified_deno, SelectFfmpeg는 select_verified_ffmpeg를 쓰고 일반 select_package를 사용자 verified버튼으로 노출하지 않는다. 제거는 기존 ToolManager.remove의 package routing과 ActiveVersion·exact fullset preflight를 그대로 이용한다.

Outcome::Selected {tool,path}로 기존 yt-dlp assignment를 유지하고 Deno는 self.deno, FFmpeg는 self.ffmpeg에 반영한다. ffprobe는 같은 package aux authority이고 별도 unrelated 외부경로로 조합하지 않는다. ToolManager active_installed는 actualfullhash 후 path만 반환하므로 tool별 Use에 사용한다. metadata 목록에서 package_installed fullhash를 호출하여128×큰archive 읽기를 만들지 않는다. 필요한 summary는 existing installed/recorded_package metadata 경로에서 component count/license/version/target/bytes를 bounded 전달한다.

## manifest authority와 실행 중 삭제 보호

설치후 archive-package.json은 source archive hash와 component hash를 구분한 원authority이고 manifest.json은 maincomponent compatible record다. 공급 후보 JSON을 사용자 문자열이나 설치root 이름에서 재구성하지 않는다. 실제 선택 worker는 동일 installlock과 originaldirectory/main/aux pins 및 postfullhash 뒤 active DB를 갱신한다. remove_package도 동일root lock·원DELETE capability preflight를 보유한다.

현재 install lock은 설치/선택/제거 mutation 동안의 보호이며 전체 download execution lifetime 보호로 확대하면 안 된다. UI는 기존 enabled 조건의 worker/tools/history/library/diagnostics busy 및 CloseWait 상호배제를 유지해 같은 앱 실행 중 관리 mutation을 차단한다. engine SelectedDeno는 원 native pins 및 반복verify를 제공하지만 Unix unlink가 열려있는파일을 허용하므로 원pins만으로 다른manager의 제거를 완전히 거절한다고 주장하지 않는다. 필요하면 root가 managedtool execution lease 또는 crossprocess shared/exclusive use lock을 전체metadata/download수명에 도입한다. 활성package 제거거절은 기본 보호지만 작업이 선택한 이전version의 active가 바뀌는 경계는 별도 실제 인수가 필요하다. 최종 fullset hash 실패에서는 실패 표시와 기존download 옵션 경로를 보존하고 새 경로로 조용히fallback하지 않는다.

## 공급 미채택 UI 제한

Deno와FFmpeg는 공식 content hash/version 실증과 publishertrust/source/thirdparty notice/target별 OS지원 채택을 구분한다. 현재 후보를 공급Accepted로 표시하지 않는다. 설치버튼은 고정candidate/trust/license/target선행이 충족된tool에만 활성화하고 미충족항목은 고정catalog 안내로 표시한다. 외부 executable 선택은 기존대로 제공하며 외부tool 삭제버튼은 없다. FFmpeg LGPL3/원source·buildrecipe·dependency notice와 Windows/macOS미지원은 이름만 LGPL이라서 생략하지 않는다. 비용에는 compressed/installedset/runtime추가RSS를 구분한다.

## 파일 담당과 실제 인수

root managed.rs/candidate authority 및 catalog.rs 중앙문구·공급상태; UI담당 accessible_tools.rs typedcommands/outcome/metadata summary; root accessible.rs 선택field wiring·busy/CloseWait; engine담당 선택/version/removal 및 execution lifetime gate를 분리한다. sharedschema/Cargo는root만 조정한다. samefile 병렬수정은 하지 않는다.

사설 Linux/WindowsGUI에서 tool switching/unsupported 공급/installbusy/cancel/failedinactive/verifyselect/Use→정확literalpath/활성제거거절/비활성exactset삭제/외부선택보존을 UIA·PNG·DB·원source/foreign로 확인한다. source/hash/version/license/metadata/DELETEmode 실패는 prioractive와원files를 보존한다. 실행중tool선택·제거disabled와CloseWaitworkerack, Tab/Enter/Space/한글path/960×640 및800×600 scroll/focus가시성을 검사한다. 실제runtime metadata/download argv 및 Attemptdiagnostic은engine evidence와연결한다.

## 검증과 handoff

읽기전용 조사와문서만작성했다. Rust/Cargo/sourcefreeze/GUI를변경·실행하지 않았다. 기존backend통과를GUI완료로확대하지 않는다. root공급gate와tool별outcome/metadata요약API를확정한뒤구현한다.

## 관리 tool UI 구현 인계

accessible_tools.rs에 root ToolId3개 selector와 command의 tool identity, typed Outcome Selected{tool,path}/take_selected_path tuple을 연결했다. tool 변경은 busy때 금지하고 stale inventory/selection을 비운 뒤 Refresh를 보낸다. installed/active 조회는 tool.id 기준이며 runtime Install은 공급 보류 문구 및 disabled다. runtime Select/Use는 actual package_installed fullproof 뒤 select_verified_deno/ffmpeg를 수행한다. tool과 manifest 불일치를 거절하고 remove의 engine exactset 경로를 유지했다. 기존단일worker/CloseWait/cancel queue를 보존했다. root가 accessible tuple 소비와 공급/catalog를 통합한다. 세 회귀는 tool별empty/noUse, 공급install 및mismatch/외부거절, typed selection/disabled inert를 포함한다. Cargo는 수행하지 않았고 담당파일포맷 후 root actualGUI/native 검증을 기다린다.

## 주 담당 통합 보강

root는 managed.rs ToolId 공통 식별, catalog 중앙label/pending문구, accessible.rs typed선택의각필드연결·Unicode검사·CloseWait변경거절을 통합했다. package_inventory.rs recorded_package_for_manifest는bounded metadata만반환하며 version검증선택은 별도callbackfullhash API를 사용한다. runtime UI의 Select/Use에package_installed 선행payloadhash를 제거하여10초/cancel검사 밖의큰read를 피했고runtime목록/active표시는metadata에서구성한다. actualUse는두version 및fullhash를재검증한뒤만경로전달한다. yt-dlp 기존경로유지. 첫CPUentry80시험 PASS 뒤metadatalookup보강을추가했으므로현재소스는재시험중이다. GUIcollectorempty/select-use/wrong-version을별도agent가작성했고actual실행은release빌드이후수행한다.

### 최신 scoped 검증

metadata lookup 보강 이후 `cargo test -p dlpad --bin dlpad-cpu --features cpu --locked` 80PASS, `cargo clippy --workspace --all-targets --features cpu --locked -- -D warnings` exit0, CPU release build exit0. `runtime-tools-linux-empty-first` 격리 GUI collector 실제 실행 중이며 종료 결과를 확인하기 전 GUI 완료로 표시하지 않는다. source 변경 이후 전체 기본/CPU/Windows workspace는 후속이다.

### Windows 현재 통합 및 설명 보강

runtime탭을yt-dlp전용설명과분리한중앙catalog안내로바꾸고release/CPUworkspaceClippyexit0를확인했다. WindowsGNUCPUworkspacefmt/clippy/test/release4명령terminal0, runtime-tools-windows-workspace-first/root-review.json에서root현재380sourceSHA대조불일치0. Windows실제runtime관리GUI는미검증이다. Linuxempty-enabled-retry GUI는새binary의4tool전환및runtimeinstalldisabled/pendingPASS,정상선택과version거절은별도후속이다.

# S06: Windows private 테마·저장 재실행 인수 준비

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R06, R13, R01
- 담당/수정 범위: diagnostic_cache; 이 문서 및 `/tmp/windows_settings_private_probe.cs` 초안만. 프로젝트 Rust/Cargo/기존 collector는 읽기 전용.
- 선행 작업: CPU settings 저장·종료 acknowledgement; root 최신 Windows release 확정; CloseWait/S10 native collector 종료 후 직렬 실행.

## 사용자 결과

Windows에서 선택한 System/Light/Dark가 정상 종료 후 재실행 화면에도 적용되는지 실제 private desktop GUI로 확인한다. 현재는 준비 단계이며 Windows GUI 테마 인수 완료가 아니다.

## 계약과 범위

README→PRODUCT→CONTEXT→R06/R13와 기존 설정/디자인/ADR 기준을 따른다. `accessible_settings.rs`의 `load_theme`는 이벤트 루프 이전에 readonly SQLite를 읽고 legacy `theme.json`보다 우선한다. `AppearanceSettings::apply`는 저장값을 ctx에 적용하며 선택은 UI에 즉시 반영하고 직렬 worker에 bounded coalescing한다. 실제 종료 시 worker acknowledgement는 root 종료 계약이다.

`software_host.rs:335`의 숨긴 window 생성→`window.theme()`를 egui-winit에 전달→AccessKit 초기화→show 순서를 확인했다. `window_event`는 egui-winit에 이벤트를 먼저 전달한다. pinned egui-winit 0.33.3의 ThemeChanged는 system_theme를 갱신하고 repaint를 요청한다. vendor winit Windows `WM_SETTINGCHANGE`는 preferred native theme 없음 및 실제 값 변경 시 ThemeChanged를 보낸다.

System 기대값을 `AppsUseLightTheme` registry 하나로 단정하지 않는다. pinned `dark_mode.rs::try_theme`는 Windows build 지원 여부, ShouldAppsUseDarkMode ordinal132, high-contrast, SetWindowTheme 및 native dark-mode 적용 성공을 고려하며 실패는 Light이다. 현재 OS 설정 읽기와 실제 System 픽셀 관측은 가능하지만 실제 OS 테마 변경 인수는 이번 범위 밖이다. 합성 WM_SETTINGCHANGE나 egui RawInput 시험을 실제 OS 변경으로 표현하지 않는다.

초안은 root 소유 standalone S10 C#의 private desktop/원 PID+FILETIME+HWND/owned Job/90초 worker/원 input desktop readonly 비교/정리 코드를 재사용한다. 실행 경로는 report·cookie·download를 호출하지 않는다. 원 공유 helper가 남아 있어 보안·compile 독립 리뷰가 필요하며 아직 native 실행하지 않았다. `CreateDesktop` 실패 시 fallback 없이 종료한다. clipboard/Paste/시스템 설정 변경/전역 키 입력을 호출하지 않는다.

## 인수 조건

1. argv는 `binary SHA freshEvidenceDir seed(System|Light|Dark) select(System|Light|Dark)` 다섯 개. fresh create-only private ACL evidence/data/media, 원 binary readonly pin+SHA 전후 일치. 원 사용자 설정 사용하지 않는다.
2. bounded synthetic legacy seed로 첫 실행→실제 Settings UIA labelled theme ComboBox/button 확인→선택→WM_CLOSE 실제 exit acknowledgement→같은 owned data 재실행. 실제 저장 theme.sqlite 존재 및 root가 SQLite 값/선택 tree/PNG를 독립 대조한다. 단순 파일 존재는 persistence PASS가 아니다.
3. Light/Dark/System 세 fresh 시나리오 각각 seed를 반대로 선택한다. 첫/restart PNG의 실제 화면 및 새 UIA tree에서 선택값을 확인한다. 이름/role/패턴 unsupported는 실패로 보존하고 추측 좌표로 통과시키지 않는다.
4. 원 90초 누적 budget/Job 종료/empty/input desktop 불변/worker restoration을 유지한다. 첫 실행 cleanup가 incomplete이면 두 번째 실행하지 않는다. 실패 desktop close false를 성공으로 바꾸지 않는다.
5. 현재 초안의 `before-uia.png`는 **처음 관측한 캡처**이며 first presentation도 첫 painted frame도 증명하지 않는다. 후속 초기 관측은 프로세스 Resume 시각 기준, UIA 및 강제 WM_PAINT 이전 PrintWindow에서 bounded glyph/nonblank 픽셀 검출을 구현해야 한다. 50ms poll 양자화와 PrintWindow 자체 관측 한계, OS/해상도/font/binary SHA를 기록하고 'first observed painted frame'만 주장한다. target 1초를 완화하지 않는다.
6. 실제 System OS-event는 미검증으로 남긴다. 원 사용자 시스템 테마를 변경해서 시험하지 않는다. busy SQLite save-in-flight drain은 별도 owned lock collector가 필요하다.

## 검증과 handoff

읽기 감사 및 `/tmp` 코드 생성만 수행했다. Cargo/네이티브/사용자 화면 시험 없음. 원 초안 SHA256 `108d6f262e03678715f3b83e6bfcb773b7856e2c498ee36c18f786b080471e87`을 보존하고 root 종료 리뷰를 반영한 현재 C# SHA는 `86be673acf902ce2186a68fc1d86131bbefc929eb2ae79b31bad598db4033805`이다. 양 Run status가 REVIEW_REQUIRED이고 원 process HANDLE에서 GetExitCodeProcess로 자연 종료 코드 0을 확인하며 양쪽 Job empty/process exit/desktop close/input unchanged/MTA restore가 모두 true이어야 한다. ComboBox는 원 labelled control의 ExpandCollapse/Invoke만 사용하고 unsupported는 실패한다. 초기 픽셀 observer 구현은 남았다. 독립 read-only root 검토 후 root가 게시/실행한다.

CLI wrapper `/tmp/windows_settings_private_native_probe.py` SHA `e62f69c26117893cdbecd3047d3b3eeeabc175b25b1f957763a594da5f16a16b`는 `output --collector <root-published-project.cs> --seed System --select Light` 계약이다. 기존 private CLI host/native csc를 그대로 사용하며 원 binary와 collector/runner SHA를 전후 대조한다. 두 Run의 각 worker deadline 90초는 동일하다. 외부 host 230초는 두 순차 실행 및 bounded 정리 합계 상한이며 한 UI worker budget 증가는 아니다. `python3 -m py_compile /tmp/windows_settings_private_native_probe.py`는 통과했다. native compile/runtime, UIA selected-choice 실제값과 SQLite/pixels 독립 판정은 아직 실행하지 않았다.

### 실행 계약 독립 검토 후 최종 staging

최종 C# SHA `58338d7fdc061ac0175e23009a29fed0a27e988028e00baa3eddd18df9314d17`, wrapper SHA `5c2ef4d42fe4c1e349681a840061f931a08a63b3afdd03e21766bd580c8f0dea`. 위 SHA들은 이전 handoff 기록으로 유지한다. Wrapper는 원 collector의 absolute/no-parent/no-symlink·1MiB 상한을 확인하고 evidence 안에 create-only bounded 사본을 만들며 실제 csc 입력도 그 사본이다. 원 source/사본 SHA를 컴파일 전후 확인한다. 바이너리 복사도 create-only 256MiB 실제 streaming 상한이며 전체 메모리 적재하지 않는다. py_compile 재통과.

`n.configure`는 host 제어 stdin 및 프로젝트 전용 도구/TEMP만 지정하며 사용자 시스템 설정을 바꾸지 않는다. `n.run`은 로그 8MiB/관측 tail64KiB/outer deadline을 유지하고 실패 시 원 소유 native host stdin C/EOF로 Job 정리를 요청한다. 추측 Windows PID 종료는 없다. 최종 passed는 host exit0·source snapshot 동일·private desktop 및 input unchanged 증거를 포함한다. 취소 후 host가 종료되지 않을 경우 증거는 실패로 남으며 완전한 정리를 추정하지 않는다.

### 첫 native 실패와 toolbar collector 교정

실제 `windows-settings-system-light-20261004-resume`에서 compile exit0(3.43초), GUI exit1(13.13초)을 확인했다. 설정 toolbar target clientY45에 공유 S10 진단 body anchor107/EnsureContent를 잘못 적용해 scroll32회 뒤 실패했다. 이는 실제 System/Light 선택 실패 증거가 아니라 설정 진입 collector의 잘못된 body-region 가정이며 원 failure를 보존한다.

수정 staging C# SHA `8b504335ff1b311c67564a54b685dc7ef8fd85918b0cf224f5cbd078020569a3`는 `SettingsAction`을 별도로 둔다. 이름/실제 Button·TabItem·RadioButton·CheckBox·ListItem/enabled 유일성을 확인하고 원 PID/FILETIME/HWND의 실제 client bounds 안에서 focus 반영을 검사한 뒤 owned Space down/up만 보낸다. toolbar와 선택항목에 진단 body anchor/scroll 규칙을 적용하지 않는다. clipboard/전역입력/fallback/timeout 변경은 없다. `first/result.json`와 실행된 경우 `restart/result.json`를 각 Run 반환 즉시 create-new로 저장하므로 상위 `First run failed`가 원 오류 및 cleanup 결과를 가리지 않는다. 교정본 native 실행은 root 담당이며 아직 재검증하지 않았다.

제품 UI 대응은 `accessible_settings.rs`의 `appearance-theme` ComboBox→`labelled_by(화면 테마)` 및 선택값 System/Light/Dark이다. 초안은 실제 labelled control 하나만 허용하고 원 ExpandCollapse/Invoke capability를 사용한다. 결과의 `requested_choice`는 선택 요청이지 실제 저장값이 아니다. 초기 UIA name 및 지원 시 실제 ValuePattern 값을 별도로 기록하고 root는 tree/SQLite/PNG를 대조한다. 원래 native provider가 ComboBox를 Button 또는 다른 pattern으로 노출하면 unsupported를 보존하며 compile만으로 GUI PASS 처리하지 않는다. 실제 OS theme event 및 첫 presentation/first observed painted frame 미검증은 유지한다.

## 2026-10-04 실제 native 재검증

root가 settings 전용 owned focus/Space 수집기를 통합하고 `python3 scripts/windows_settings_private_native_probe.py <freshEvidence> --collector <projectCollector> --seed System --select Light`와 Light→Dark, Dark→System을 실행했다. 세 case 모두 compile/native PASS. root는 실제 SQLite settings.theme의 Light/Dark/System 값, 재실행 UIA 밝게/어둡게/시스템 값, 세 실제 restart PNG의 밝은/어두운/시스템 관측 화면, 양 Run 자연 exit0·Job 비움·private desktop 닫힘·원 input desktop 불변과 current source snapshot을 확인하고 각 root-review.json을 기록했다. 근거 폴더는 windows-settings-toolbar-corrected-20261004-resume, windows-settings-light-dark-20261004-resume, windows-settings-dark-system-20261004-resume이다. 첫 잘못된 toolbar scroll 실패는 그대로 보존한다. 최초 painted frame, 실제 OS theme change, busy SQLite drain은 미검증이므로 전체 R06 완료는 아니다.

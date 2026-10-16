# S00: 실제 gtmux 구조 대조와 네이티브 폴더 선택

- 상태: completed — 구현·Windows scoped 통합 검증; 최종 사용자 UX 인수는 별도
- 분류: feature
- 우선순위: P0
- 요구사항: R01, R08, R11
- 담당/수정 범위: root — accessible.rs, accessible_library.rs, catalog.rs, native picker adapter/host, DESIGN.md, 검증·사용자 안내; 하위 담당은 /tmp 초안과 독립 검토만 수행
- 선행 작업: 1차 UI 후보 및 사용자 재평가(미통과)

## 사용자 결과

OS 창 제목 외 앱 이름을 반복하지 않는다. 실제 gtmux 컴포넌트·레이아웃을 대조해 compact한 화면을 구현한다. 저장 위치는 운영체제 폴더 선택 창으로 고르며 파일 목록의 경로와 동작이 같은 레이아웃에 맞는다.

## 계약과 범위

1차 사용자 ZIP은 UI 승인본이 아니다. root가 공유 UI·native HWND/owner/modal lifecycle 인터페이스와 전체 검증을 조정한다. diagnostic_cache는 원 gtmux와 기존 PNG 대조, metrics_recovery는 기존 Windows COM 기반 folder picker 초안, soak_recovery_audit는 실제 private native-dialog GUI 검증 초안을 /tmp에 작성한다. 동일 프로젝트 파일 동시 수정·동시 native/Cargo 실행은 없다. invalid Git은 보존한다. 기존 stored location validation/scan/취소/worker는 유지하며 picker 선택 자체를 경로 승인·삭제 권한으로 표현하지 않는다. 취소/실패에는 기존 위치를 보존한다. OS별 미지원은 명시하고 WebView/서버/새 UI stack은 도입하지 않는다.

## 인수 조건

원 gtmux 소스 컴포넌트와 새 실제 앱 PNG를 직접 비교한다. 앱 내부 중복 dlpad와 장식 브랜드바를 제거하고 화면 제목·compact toolbar·경로 배치를 확인한다. 파일 목록과 다운로드 저장 위치에 Windows native 폴더 선택을 연결한다. 실제 private desktop에서 선택/취소/ownerclose/새 경로 반영과 기존 위치 실패 보존, 좁은800x600/넓은960x640/앱 배율100·150·200을 확인한다. 긴 경로가 버튼을 밀어내지 않고 label/input/browse 높이와 폭이 일치한다. source고정 Windows 전체4검사·새 GUI/비용/ZIP 및 문서검사를 수행한다. 물리IME/reader/OS DPI는 별도다.

## 검증과 handoff

2026-10-07 사용자가1차 UI를 재거절했다. 아직 새 구현·인수는 진행 중이다. 변경마다 실제 private 캡처를 root가 직접 확인한다. visible desktop fallback을 하지 않는다.

## 중간 실행 기록

첫 `windows-msvc-folder-ui-workspace-20261007`은 fmt 통과 후 Clippy의 entrypoint별 미사용 host API와 adapter lint에서 실패했다. 이를 수정한 `windows-msvc-folder-ui-workspace-retry-20261007`은 Windows fmt/Clippy/test/release 모두 exit0, 862 passed/0 failed/13 ignored, source602를 확인했다. 실제 private layout 캡처에서 root는 settings-initial, initial-library-960, selected-download-960 PNG를 직접 열었다. 그 결과 TextEdit inner폭에 margin이 추가되어 browse행 오른쪽 여백이 부족한 점, 선택 탐색 표시가 희미한 점, 설정이 여전히 세로 입력 나열인 점을 확인했다. 이 snapshot은 중간 증거이며 최종 인수본이 아니다.

독립 읽기 검토에서 fatal host drain 중 이미 완료된 선택을 적용할 수 있는 경합을 발견했다. coordinator에 취소 요청을 기록해 이후 선택 결과를 폐기하는 정책과 실패 경로 회귀를 추가한다. native COM은 transient STA worker에 두고 hidden proxy→원 app owner를 연결한다. UI는 cancel 메시지와 bounded completion만 처리하고 owner HWND는 cleanup ACK까지 유지한다. OS cleanup 오류는 정상 종료 증거로 취급하지 않는다.

최종 수정 파일: `crates/app/src/accessible.rs`, `accessible_library.rs`, `accessible_settings.rs`, `accessible_tools.rs`, `accessible_cpu_main.rs`, `accessible_main.rs`, `catalog.rs`, `software_host.rs`, 새 `path_picker.rs`, `folder_picker.rs`, `folder_picker_worker.rs`; `scripts/windows_folder_picker_private_probe.cs`, `windows_msvc_folder_picker_collect.py`, `windows_folder_ui_layout_private_probe.cs`; 관련 디자인·도메인·결정·사용 안내·현재 상태 문서. Cargo workspace/lock/schema/새 dependency 변경은 없다.

캡처에서 발견한 입력 outer여백을 계산에 포함했고 탐색 선택 상태를 tinted surface로 표시했다. 설정은 설명/조작을 나란히 배치하고 큰 앱 배율에서는 쌓는다. 네트워크 관측 고급 설정은 disclosure로 옮겼다. 추가로 빈 경로는 native picker의 명시적 기본 시작 위치로 열도록 바꾸었다. 저장 위치 복원은 기존 recovery worker에서 saved setting이 있을 때만 validation을 거친다. 저장값 부재/읽기 경합에는 Documents fallback을 호출하지 않는다. 실패는 수정 가능한 UI 상태로 알리고 사용자 새 입력·CloseWait에는 늦은 복원값을 반영하지 않는다. 기존 Restore 정책상 저장된 누락 폴더의 생성은 가능하며 외부 폴더 삭제·비밀·원 영상 시험은 없다. native UTF16 경계와 제품 UTF8 32KiB 입력 상한은 별도다.

## 실제 native trial 진단

전체 검사와 별개로 새 observer의 C# 초기화/선언 오류를 발견해 원 실패를 보존했다. Down/Up만 보내는 시험 click에서 dialog가 관측되지 않았고, 원 UI/소유 window만 계측했다. Move/Focus를 보완한 가상 마우스 모사도 인수되지 않아 물리 pointer 지원 증거로 사용하지 않는다. 실제 원 버튼 UIA Invoke에서는 private desktop의 dialog→proxy→원 app 관계를 확인하고 PNG를 직접 열었다. Windows folder mode의 native Edit는1152였고 기존 file mode1148 가정을 수정했다. native Button은 실제 HWND/class/원 PID/TID/owner/enable을 검사하며 BM_CLICK 프로토콜이므로 UIA Invoke pattern을 native Button 인수 조건으로 삼지 않는다. native 폴더 입력 readback과 결과 directory identity를 정확히 확인한다.

Dialog close 전환에서 사라진 UIA element는 원 app HWND/PID/creation으로 다시 조회하는 bounded specific retry로 처리한다. ownership/unique control/type/결과 readback 실패는 여전히 실패다. 실제 파일 행은 이름만이 아니라 `selected-only.mp4 · 3 bytes · 로컬 파일` 전체 문자열이다. `windows-msvc-folder-picker-transition-fixed-20261007`의 두 process는 normal exit0/Job0이며 UI 선택·자동 Apply·새 process 복원을 통과했다. root는 actual SQLite두 개를 mode=ro로 열어 integrity=ok·setting/root·행1·original native volume/fileID와3bytes를 직접 대조했고 읽기 전후 SHA 불변을 확인했다. 이 시험은 물리 mouse/keyboard 인수가 아니다. binder 초안은 /tmp 검토 자료이며 실제 다른 desktop 원인이 입증되지 않아 Cargo feature 변경으로 통합하지 않았다.

최종 layout 수집에서도 설정 control의 수평 기준선이90px 어긋난 것을 독립 PNG 검토로 재확인했다. allocate_ui의 폭은 최소 사용 폭을 강제하지 않으므로 description child에 set_min_width를 명시하는 수정이 필요하다. 같은 frame의 viewport Close가 이미 입력된 경우 poll이 선택을 먼저 적용해 새 scan을 시작할 수 있어, Close 입력을 먼저 cancel에 반영하고 실제 queued completion/RawInput 회귀를 추가한다. 기존 aligned 전체876/0/13과 native5사례는 해당 수정 전 이력이며 최종 source 검증으로 소급하지 않는다.

`windows-msvc-folder-ui-close-safe-workspace-20261007`은 새 회귀의 test-only 초기화에서 Drop 타입의 struct update(E0509)로 Clippy compile 실패했다. 필드를 명시 초기화한 fresh retry로 교정한다. 원 실패를 유지하며 최종 source와 구분한다.

기존 위치 실패 보존을 현재 바이너리의 실제 GUI에서도 확인하기 위해 별도 observer variant `scripts/windows_folder_picker_invalid_private_probe.cs`를 작성했다. 선택한 합성 폴더의3bytes 파일 자체를 저장 위치로 입력하고 기존 Apply를 눌러 오류·기존행·재시작 복원과 original DB/bytes를 확인한다. 기존5개 observer의 원 source/pins는 수정하지 않는다.

첫 추가 invalid 위치 GUI 시험은 native 선택까지 정상 관측했으나 기존 F24/WM_CHAR 합성 입력의 surrogate pair가 누락되어 readback124/122에서 실패했다. Apply는 실행하지 않았으며 위치 검증 실패를 제품 실패로 표현하지 않는다. 원 실패/입력 계측을 보존했다. 시험 대상 invalid 파일을 같은 소유 root의 ASCII 이름3bytes 파일로 분리해 위치 실패 보존을 검증하고, emoji 입력/실제IME 인수는 별도로 남긴다. 선택·복원 대상은 여전히 Unicode/emoji 폴더다.

## 최종 scoped 완료

실제 명령은 `scripts/windows_msvc_workspace_probe.py`(MSVC fmt/Clippy/test/release), `windows_msvc_ui_rework_collect.py`(layout), `windows_msvc_folder_picker_collect.py`(choose/cancel/owner-close/download-empty/download-cancel), `windows_msvc_whole_gui_cost_probe.py`(3×60초), `windows_msvc_notice_graph_probe.py`, `windows_msvc_candidate_package.py`다. 최종 source602/lock 고정 전체878 passed/0 failed/13 ignored. root는 원 로그·SHA/source maps·native receipt와 SQLite setting/행/Windows identity·010203 원자료·PNG를 직접 검토했다. 하위 read-only 독립 검토를 별도 보관한다.

[whole 원검토](../validation/evidence/windows-msvc-folder-ui-close-safe-workspace-retry-20261007/root-review.json), [실제 layout 원검토](../validation/evidence/windows-msvc-folder-ui-close-safe-layout-real-20261007/root-raw-review.json), [개인 PNG 검토](../validation/evidence/windows-msvc-folder-ui-close-safe-layout-real-20261007/root-pixel-review.json), [native5/10 원검토](../validation/evidence/windows-msvc-folder-ui-close-safe-workspace-retry-20261007/native-five-root-review.json), [잘못된 위치 보존](../validation/evidence/windows-msvc-folder-ui-close-safe-workspace-retry-20261007/native-invalid-root-review.json), [비용18구간·원자료·실제PNG](../validation/evidence/windows-msvc-folder-ui-close-safe-cost-real-20261007/root-final-review.json), [ZIP 원검토](../validation/evidence/windows-msvc-folder-ui-close-safe-graph-20261007/package-root-review.json). root가 확인한 캡처는 source고정 실제 UI이며 mockup이 아니다.

최대CPU0.623811%·WS66.59MiB. CPU9,081,344B(+40,448B/39.5KiB 대1차UI), CLI974,848B/installer1,103,360B는 동일하다. 새 crate/feature/lock/schema 변경0. ZIP13,718,328B/466members SHA`5b4d76392959b2f029c1ac0bb4f3730d911c1b3b30b09627ae424498365c0e14`. resolved graph258/86/258, selected notice258 누락0; 전체lock 누락43·법적 완비/서명/정식 installer/전체P0는 미완료다.

남은 인수: 최종 사용자의 UI/UX 평가, 물리 pointer·한글IME·reader·OS DPI, 모든controls keyboard접근, 실제 사이트 및 정식 설치/upgrade/cleanOS. 기존 위치 실패 보존은 현재 whole의 engine/App 실패 경로 회귀와 추가 실제 GUI의 잘못된 저장 위치 거절·기존 행/설정·재시작 복원으로 확인했다. 추가 시험의 ASCII 경로 입력과 native Unicode/emoji 폴더 선택을 구분하며 합성 emoji 메시지 입력 실패 이력은 유지한다. 프로젝트 밖 데이터 삭제·사용자 Documents/영상 시험·visible fallback·공개배포·Git 재구성은 수행하지 않았다.

독립 검토는 최종 whole/settings, layout/geometry, native5 원자료/DB, cost18/30k/3DB, ZIP15checks 실패0으로 나눠 원 자료와 root 판단을 대조했다. [ZIP 독립검토](../validation/evidence/windows-msvc-folder-ui-close-safe-graph-20261007/package-independent-review.json). 마지막 추가 invalid 위치 시험의 source/원 guard/DB·pixel은 root가 통합 검증했다.

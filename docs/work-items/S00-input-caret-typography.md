# S00: 입력 커서·여백과 Windows 타이포그래피 정돈

- 상태: implemented (scoped)
- 분류: bugfix
- 우선순위: P0
- 요구사항: R06, R08, R11
- 담당/수정 범위: root app/ui_components.rs, ui_theme.rs, accessible.rs, path_picker.rs, accessible_options.rs, media_timeline.rs, catalog.rs와 문서·검증; 하위 담당은 /tmp 읽기 전용 UI/입력 감사. 공유 schema/Cargo/lock은 root, 변경 예정 없음.
- 선행 작업: UI 정렬과 도구 설치 934 후보

## 사용자 결과

입력 텍스트·커서·선택 영역을 32px 상자의 중앙에 맞추고 GTMUX의 12px 내부 여백을 적용한다. URL·경로·숫자·옵션 전반을 점검한다. 글꼴과 표면 대비·중복 안내를 함께 정돈한다. 도구 설치 지원은 기존 검증 범위를 유지한다.

## 계약과 범위

Rust native, UI thread 긴 I/O 금지. 폰트는 시작 전 bounded 로딩, 시스템 설치 폰트만 사용하고 네트워크/패키지 폰트 추가 없음. 실제 GUI는 private desktop만 사용한다. 사용자 Documents와 영상 대신 소유 TEMP/합성 fixture 사용. Git 저장소가 정상 아님을 확인했고 기존 .git 보존.

## 인수 조건

한글/영문/긴 경로/빈 입력, 클릭·Home/End·Shift 선택·가로 스크롤·Tab·disabled·IME 취소의 실제 좌표/상태를 확인한다. 100/150/200%, Light/Dark, 800/960 창에서 caret/텍스트/선택 범위가 맞고 browse 행이 넘치지 않는다. 회귀와 전체 fmt/Clippy/test/release, 가상 화면 PNG 직접 검토, 현재 artifact ZIP 및 문서 갱신. 물리 IME·실환경 설치·사용자 디자인 만족은 자동 시험으로 대신하지 않는다.

## 검증과 handoff

원인 관측: text_input은 min_size32만 지정하고 egui 기본 상단 정렬·수평4/수직2 margin을 유지해 커서/글자가 상단에 붙는다. 경로 행 폭은 기존 margin 합8을 하드코딩했다. Windows는 Malgun을 모든 proportional 글자의 첫 폰트로 사용해 ASCII 경로의 backslash도 원화 모양으로 보인다. GTMUX Input은 높이32·좌우12와 system-sans다. 현재 수정·종단 검증 진행 중.

2026-10-08 구현 통합: 공통 text_input margin(12,2)/Center/32px,1.5px accent focus, URL/경로 폭24px 보정, IME 공통 입력, Segoe→Malgun fallback, Dark 저장 설정 표면과 짧은 도구 안내를 적용했다. 폰트는 시작 전 각32MiB/총64MiB 상한이며 실제959,752B+13,459,196B다. Cargo/dependency/lock 변경 없음.

실행: `python3 scripts/windows_msvc_workspace_probe.py --configuration docs/validation/evidence/windows-msvc-folder-ui-close-safe-workspace-retry-20261007/configuration.json --output docs/validation/evidence/windows-msvc-input-caret-20261008 --reuse-owned-target`. 원fmt/Clippy/test/release 모두exit0,41 harness944passed/0failed/16ignored. 새 실제 공통 helper 회귀5개가 두 앱 harness에서 실행됐다. source610 원 map/lock 보존을 주 담당·독립 담당이 확인했다. GUI·새 비용·패키지는 후속 검증 중이며 이전934 후보 증거를 새 binary에 소급하지 않는다.

공개영상 60초 탐색과 시간 필드 focus 관측에서 이미 보이는 제어까지 중앙으로 이동시키는 불필요한 scroll이 확인됐다. 공통 입력은 gained focus 때 scroll_to_me(None)으로 가려진 대상만 노출하고, header/advanced/browse/timeline도 동일 정책으로 수정한다. 좁은 저장 경로는 hover에서 전체 경로를 표시한다. source comment의 video-only 설명도 실제 계약에 맞춘다. 새 통합 전체 검사와 실제 GUI를 다시 수행한다.

focus-final 최초 회귀는 clipped offset assertion에서 FAIL했다. 시험이 ctx.run 밖에서 focus를 미리 지정해 다음 begin_pass에 이미 이전 focus로 취급되어 gained_focus가 발생하지 않았던 fixture 오류다. 실제 입력과 같이 pass 안에서 요청하도록 고쳤으며 원 FAIL 증거를 보존하고 새 전체 검사로 확인한다. 통과 전에 인수 완료로 쓰지 않는다.

추가 좌표 계측은 offset165인데 같은 frame rect264..296/clip5..139로 이전 offset의 geometry를 반환함을 보였다. egui ScrollArea는 child UI 생성 후 animated offset을 적용한다. 다음 settled frame에서 검사하도록 수정한 단일 실제 Windows 시험은1passed/0failed, natural0/Job0/input불변이었다. 테스트 수치를 완화하거나 실패를 삭제하지 않았다. 최종 전체 검사는 public-ui-focus-settled 후보로 수행한다.

신규 실제 first-char 결함을 단위 회귀로도 재현했다: 조건부 상위 validation weak가 추가되자 TextEdit ID가60F5→DE34로 바뀌고 문자열 입력이 끊겼다(input-identity-red FAIL 보존). 범위/옵션·URL·환경 경로·IME·폴더 입력에 논리 ID를 명시해 안내 상태와 레이아웃 변화가 caret/selection/focus 상태를 버리지 않도록 고친다. UIA freshFind 보강과 별개인 제품 결함이며 Cargo/schema 변경 없음.

input-identity-green 단일 Windows actualhelper 회귀1passed/0failed로 확인했다. 첫 문자 후 안내 삽입과 12문자 전체 입력·논리 ID·focus 보존을 검증한다. 전체 검사는 public-ui-input-identity-final 후보로 다시 수행한다. 기존 전체947은 이전 source 범위의 완료 증거다.

전체 identity-final의Clippy는production호출이모두ID지정형으로이동하여이전무ID시험helper두개가unused인것을검출했다. 해당함수는실제공통builder를호출하는회귀adapter로test빌드에만남기고, warning허용없이수정한다. 원FAIL증거보존.

동일후보의 두 실제 URL 시험은1–2초 입력/readback·품질오디오선택·고급펼침/접힘·preview취소·4페이지·natural0/Job0/input불변을관측했다. 다만 품질 popup은UIA/키보드commit후열린채남아 advanced UI를덮었다. vendor의자동CloseOnClick은pointer.any_click에만반응하며appearanceCombo에는이미명시ui.close가있고qualityCombo만빠졌다. actual DownloadForm AccessKit 선택/현재값재선택 회귀와 명시close로 고친다.

품질 popup RED는 실제 DownloadForm AccessKit 선택 후 popup-open assertion에서 실패했다. `.clicked()`에서 `ui.close()`하도록 수정한 GREEN은 다른 품질/현재값 재선택 모두 통과했다. 최종 `windows-msvc-public-ui-popup-final-20261008` whole은 fmt/Clippy/test/release exit0, 41 harness 합계951passed/0failed/16ignored, source610 map/원GNU/lock 보존이다. root와 독립 담당이 원 로그·실제 세 exe SHA를 대조했다. CPU bacaa08bf950381c3e3d560cea5aba55fbca8e122ffe13674eb950ee42c77b7b,9,415,168B. 실제 두 URL 새 advanced-open PNG에서 품질 선택 뒤 popup이 닫힌 상태를 직접 확인했다. 새 전체 배율·비용·패키지는 계속 별도 검증한다.

최종current실제폴더선택추가회귀의첫실행은oldobserver가탐색Button의TogglePattern을지원하지않아dialog전단계에서FAIL했다. 원FAIL/소유Job정리기록을보존했고,현재layout/publicobserver와같은typedToggleOn분기를보강하여재실행한다. root추가담당파일 scripts/windows_folder_picker_private_probe.cs이며제품browse미지원으로표현하지않는다.

## 최종 통합 검증 (2026-10-08)

현재 source610의 Windows MSVC 전체 검사 `windows-msvc-public-ui-playback-clippy-fixed-20261008`에서 fmt/Clippy/test/release 모두 exit0, 41 harness 합계 **953 passed / 0 failed / 17 ignored**다. ignored 공개망 시험은 별도 명시 실행에서 실제 production engine의 0/2/4초 연속 batch 각각12frame을 검증해1passed/0failed였다. 이전 FAIL·RED와 수정 후 GREEN 로그를 보존했다. 의존성·Cargo.lock·공유 schema 변경 없음.

두 실제 공개 URL의 최종 private GUI `windows-msvc-public-youtube-playback-final-20261008`, `windows-msvc-public-chzzk-playback-final-20261008`에서 준비·짧은 무음 미리보기·재생/일시정지 Invoke·1/2초 시간 입력 readback·오디오 품질 선택·고급 펼침/접힘·preview 취소 동작·4페이지 탐색·자연 종료/Job0를 확인했다. root가 원 PNG를 직접 검토했고 원 collector REVIEW_REQUIRED와 별도로 root-review.json에 scoped 판정을 기록했다. YouTube 길이는01:35:22.000, 치지직은04:36:40.000이므로 짧은 영상만 시험한 것이 아니다. 전체 길이 재생·전체 다운로드·실제 cut·취소 terminal ACK·물리 pointer/IME를 검증한 것은 아니다.

최종 layout63PNG(800/960,Light/Dark/restart,앱150/200%), RawInput19PNG·실제 FFmpeg 합성 색 변화, 실제 Windows 폴더 선택 두 흐름을 재검증했다. 라이브러리 선택은 재실행 후 root/1행3B 복원과 DB integrity를 확인했고 다운로드 폴더 선택은 draft 변경 범위다. 비용은 합성1만 파일 목록 idle 3×60초/18창에서 최대 single-core CPU0.786260%·sampled whole-Job WS72,925,184B(69.55MiB), 각 DB1만행·원합성3만파일 보존이다. 재생/다운로드 비용·quiet host·물리 첫 화면 지연은 측정하지 않았다.

남은 인수: 최종 사용자 디자인 평가, 긴 영상 정밀 제어 가시성,200% 밀도/스크롤 맥락, canonical 경로 표시 정돈, library 미디어 연결/로컬 export, 물리 IME/reader/OS DPI, preview managed lease/cache, 서명·clean OS·정식 설치/updater·전체 법적 공지 및 전체P0. 현재 공개 두 URL의 scoped 성공을 모든 사이트 호환으로 일반화하지 않는다.

최종 패키지: `python3 scripts/windows_msvc_candidate_package.py --whole-proof docs/validation/evidence/windows-msvc-public-ui-playback-clippy-fixed-20261008/result.json --graph-proof docs/validation/evidence/windows-msvc-public-ui-final-graph-20261008/result.json --output docs/validation/evidence/windows-msvc-public-ui-final-graph-20261008/dlpad-user-preview.zip --native-installer-exe .local-tools/windows/msvc-supply-20261006/target-msvc/x86_64-pc-windows-msvc/release/dlpad-install.exe --native-installer-sha256 edb7c3daea935e2ea103867e89a1a2df534d7f0d78be476d77ecc0dc40ad5529`(실행 argv는 absolute path) exit0. ZIP17,244,580B/466members/SHA256 `e639daf6bd387bb463082a4d5cd2162cea9949854f1b89be0de787aa8d128745`. root CRC/3exeSHA/PE·고정timestamp/permission·원 notice258/missing0를 검토했고 whole appendix43 및 legal/release false를 유지한다. 앱 CPU9,420,288B·CLI974,848B·installer4,290,560B. 구현/검증은 이번 수직 기능의 scoped 완료이며 전체P0와 최종사용자UI 인수를 완료로 표시하지 않는다.

최종 공통 문서 검사: `python3 scripts/check_docs.py` exit0, PASS866documents·required/local file links, 보존95hash-matched upstream notices. 외부 URL/heading anchors는 checker 범위 밖이다.

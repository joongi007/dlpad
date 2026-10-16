# 개발·검증 이력

README 정리 전 본문을 아래에 그대로 보존했습니다. 각 날짜의 “현재/최신”과 검사 결과는 당시 source/binary snapshot을 뜻하며 현재 배포물이나 릴리스 완료를 주장하지 않습니다. 현재 설치와 상태는 [README](README.md)를 먼저 확인합니다. 이 문서는 저장소 루트의 `DEVELOPMENT_HISTORY.md`로 배치하여 기존 상대 링크를 유지합니다.

---

2026-10-11 현재 Windows 후보: 실행 계획의 접근성 스크롤과 진단 입력의 잘림·끝 커서 스크롤을 수정하고, 로컬 영상 연결·정밀 구간 저장·선택 도구 설치를 통합했다. MSVC fmt/clippy/test/release는 **1,015 passed / 0 failed / 28 ignored**(43 harness)이며 실제 HTTP/파일 쓰기 복구 시험 2개도 별도로 통과했다. 고급 옵션의 오류·복원·도움말·실행 계획을 실제 가상 화면에서 확인했다.

[최신 설치 ZIP](docs/validation/evidence/dlpad-install-diagnostic-caret-20261011.zip)을 모두 압축 해제한 뒤 `dlpad-install.exe`를 실행한다. 새 폴더에 설치하며 기존 폴더를 덮어쓰지 않는다. 현재 5개 파일의 실제 설치·재검증과 원본 보존을 확인했다. 새 가상 화면에서 진단 입력9건·선택 표시·마지막 줄 표시를 확인했다. 합성 파일 10,000개 idle18구간(이전 바이너리)의 최대 CPU는 0.627%, working set은 68.84MiB다.

[현재 작업·검증](docs/work-items/P0-quality-completion-integration.md), [옵션 입력 인수](docs/work-items/S00-windows-option-help-plan-20261011.md), [설치 기록](docs/work-items/S11-install-zip-20261011.md), [Git 준비](docs/work-items/S11-isolated-source-git-bundle-20261011.md), [남은 작업](docs/validation/P0-remaining-20261011.md). 사용자 UI 인수, 실제 원격 Git/CI, 자동 updater, 서명·clean OS 검증은 남아 있다. 전체 P0나 정식 릴리스 완료로 표시하지 않는다. 아래는 이전 후보의 검증 이력이다.

2026-10-08 최신 네트워크 입력 교정: 앞선 실제 URL 점검에서 빠졌던 네트워크 관측 입력을 공통32px/12px/중앙 정렬·안정ID·focus 처리로 변경했다. 실제 Windows private GUI에서 첫/재실행 클릭·Home/End/Backspace·Tab·유효/무효/빈값 적용·Light/Dark·앱150/200를 확인했다. whole fmt/Clippy/test/release957/0/17(41harness), actualpanel2회귀 RED/GREEN과원자연종료/Job0를 root/독립 검토했다. [최신 개발 ZIP](docs/validation/evidence/windows-msvc-network-input-graph-20261008/dlpad-user-preview.zip), [실행 안내](docs/development/windows-user-preview.md), [현재 교정 범위](docs/work-items/S00-network-observation-input.md), [전 입력 coverage](docs/development/input-coverage-audit.md). 진단singleline3곳은후속이며전수입력/물리입력·Windows망측정·관측선택영속성·전체P0 완료로보고하지않는다. 이전953의공개영상/비용전체시험은아래당시후보근거로보존한다.

2026-10-08 이전 입력·실제 영상 후보: 입력의 커서/여백/글꼴과 논리 ID를 정돈하고, 품질 메뉴 닫힘·재생/정지 동작·YouTube video-only 미리보기 형식 선택·프레임 로딩/오류 안내를 수정했다. 사용자 제공 YouTube/치지직 URL의 실제 private GUI에서 준비·짧은 무음 미리보기·재생/정지·시간 입력·품질/고급 설정·탐색을 확인했다. Windows fmt/Clippy/test/release **953 passed / 0 failed / 17 ignored**(41 harness 합계), 공개망 engine 회귀는 별도1passed/0failed. 최종 layout63PNG/RawInput19PNG와 native 폴더 선택·라이브러리 재실행 복원, 합성1만 파일 idle18창(최대CPU0.786260%/WS69.55MiB)을 검토했다. [최신 개발 ZIP](docs/validation/evidence/windows-msvc-public-ui-final-graph-20261008/dlpad-user-preview.zip), [실행 안내](docs/development/windows-user-preview.md), [작업·검증·후속](docs/work-items/S00-public-video-ui-walkthrough.md). 전체 영상 다운로드/재생·물리IME/reader/OS DPI·최종 사용자 디자인 인수·정식 설치/서명/clean OS·법적 공지 완결·전체P0는 미완료다. 아래는 이전 후보 이력이다.

2026-10-08 이전 UI 정렬 후보: UI 기준선·32px 제어·저장 설정 열·품질 선택을 정리하고, 앱에서 yt-dlp·Deno·FFmpeg/ffprobe 설치와 활성 경로 재실행 복원을 지원한다. 개발 설치 화면에 기본 해제된 선택 설치 3개를 추가했다. Windows 전체 검사934/0/16, 실제 세 도구 설치·Use·재실행, Deno 단독 선택 설치, 새layout63PNG/RawInput19PNG/URL·realFFmpeg 및 비용18구간을 검증했다. 최대CPU0.78527%·WS70.45MiB(합성 파일 목록 idle). [최신 개발 ZIP](docs/validation/evidence/windows-msvc-ui-polish-graph-final-20261008/dlpad-user-preview.zip), [실행·도구 준비](docs/development/windows-user-preview.md), [도구 작업](docs/work-items/S03-windows-in-app-tool-install.md), [UI 인수·후속](docs/work-items/S00-ui-visual-consistency.md). 사용자 최종 UI 인수·물리입력·실제사이트·정식 설치/서명/cleanOS·공지완결·전체P0는 미완료다. 아래는 이전 후보 이력이다.

2026-10-07 미디어 중심 UI 후보: GTMUX 실제 컴포넌트의 상태·작업면·입력 위계를 감사하고 URL 자동 확인, 영상 미리보기, 시각적 저장 구간/시간 입력, 로컬 영상 drop 미리보기와 선택/focus/hover/disabled를 구현했다. Windows fmt/Clippy/test/release906/0/16, 현재 binary의 private URL GUI·real FFmpeg 색 변화·자연 종료, 비용18창(최대CPU0.93667%/WS69.97MiB)을 root와 독립 검토했다. 같은 UI source의63PNG·RawInput19PNG와 새binary의 연결 근거도 기록했다. [최신 개발 ZIP](docs/validation/evidence/windows-msvc-media-workspace-graph-event-idle-20261007/dlpad-user-preview.zip), [실행 안내](docs/development/windows-user-preview.md), [GTMUX 비교와 현대 UI 판단](docs/development/modern-ui-audit.md), [인수와 후속 작업](docs/work-items/S00-media-workspace-interaction.md). 긴 영상 정밀 탐색·200% 하단 정보·라이브러리 연결·물리 입력·실제 사이트 미리보기·정식 설치·최종 사용자 UI 인수와 전체P0는 미완료다. 아래는 이전 후보 이력이다.

2026-10-07 이전 두 번째 UI 후보:  실제 gtmux toolbar/input/settings 소스와 캡처를 대조해 내부 중복 앱 이름·브랜드바·고정 sidebar를 제거하고 기능 탐색·저장 폴더 행·설정 정렬을 수정했다. 다운로드/파일 목록에 Windows native 폴더 선택을 연결했다. 최종 Windows fmt/Clippy/test/release878/0/13(source602), private layout63PNG·native6사례12process의 선택/취소/ownerclose/빈 경로/새 process 복원과18개 비용 구간을 검증했다. 최대CPU0.623811%·WS66.59MiB. [새 UI 테스트 ZIP](docs/validation/evidence/windows-msvc-folder-ui-close-safe-graph-20261007/dlpad-user-preview.zip), [실행 안내](docs/development/windows-user-preview.md), [범위·근거](docs/work-items/S00-native-folder-picker-layout.md). 최종 사용자 UI/UX·물리IME/reader/OS DPI·정식 설치 및 전체P0는 미완료다. 아래는 이전 snapshot 이력이다.

2026-10-07 현재: 사용자 UI 피드백을 반영해 gtmux Light/Dark 스타일·넓은 창 좌측 탐색·고급 설정 점진 공개·시작 버튼 위계·앱/실행파일 아이콘을 구현했다. 새 Windows 전체 검사852/0/13, 실제 private GUI 두 process/세 theme phase의51PNG·한글 합성 입력·앱 배율150/200·정상 종료, 새 CPU18구간 최대0.779714%/WS73.4MiB를 검증했다. [새 UI 테스트 ZIP](docs/validation/evidence/windows-msvc-ui-rework-graph-20261007/dlpad-user-preview.zip), [실행 안내](docs/development/windows-user-preview.md), [구현·검증 범위](docs/work-items/S00-windows-user-ui-rework.md). 빈 상태·좁은 창 포커스 등 UX 후속과 물리 IME/reader/OS DPI·최종 사용자 인수·정식 설치 및 전체P0는 남는다. 아래는 이전 snapshot 이력이다.

2026-10-06 현재: 기존 설치를 새 창에서 두 경로만으로 검증·명시 제거하는 기능과 앱·CLI·설치 화면을 포함한 개발 ZIP을 scoped 완료했다. Windows 전체4검사846/0/13, GUI12건·원 자료 보존·최신 CPU18구간(최대0.9363%/메모리68.31MiB)을 주 담당·독립 검토했다. 실제 ZIP두 개의13,043,842B·466 members·SHA/CRC/권한/timestamp 및 notice455 원자료가 일치했다. [기존 설치 검증](docs/work-items/S07-windows-native-existing-bundle-reverify.md), [개발 ZIP](docs/work-items/S07-windows-native-installer-companion-package.md), [전체 상태](docs/validation/P0-status.md). 정식 Windows 설치·업그레이드·실환경 인수·공지 완비와 전체 P0는 미완료다. 아래는 이전 snapshot 기록이다.

2026-10-06 최종 파일 선택 통합: Windows fmt/Clippy/test/release 모두 exit0, 850 passed/0 failed/13 ignored, source593 전체 map 일치. 설치 명세 선택·취소·owner 종료와 재시작 확인취소·제거의 실제 private GUI 5사례/10수집 실행 및 root 추가 native42명령을 검증했다. root는 PNG7장을 직접 확인했다. CPU/CLI는 이전 인수본과 바이트 동일하며 installer는817,152B로 이전799,744B보다17,408B 증가했다. 전체P0·물리IME/reader·정식 설치/upgrade/cleanOS는 미완료다. [사용자 시험 안내](docs/development/windows-user-preview.md), [파일 선택 완료 범위](docs/work-items/S07-windows-native-manifest-picker.md).

2026-10-06 최신: 개발 설치 화면의 명시 제거를 scoped완료했다. Windows 전체4검사844/0/13(source592)·실제GUI11case22실행·root추가native73명령/직접PNG18장에서 확인취소/정상제거/foreign·경로수정거절/원worker취소·CloseACK/기존설치회귀와원자료ID·SHA보존을root/독립검토했다. CPU0bf43b…8,710,656B·CLId0e4ae…974,848B·installer2e91bb…798,720B(+38.5KiB), lock불변. 최신CPU비용은새3×60/18구간에서측정중이며이전비용을소급하지않는다. 새창에서기존bundle의검증·제거를잇는typed2root후속은/tmp에서병렬준비한다. 정식installer/default·browse·upgrade·cleanOS/physical·전체P0는남는다. [완료범위](docs/work-items/S07-windows-native-development-removal-ui.md), [다음작업](docs/work-items/S07-windows-native-existing-bundle-reverify.md). 아래는이전snapshot기록이다.

2026-10-06 최신: 명시 개발 bundle 제거 엔진·CLI16 범위 인수를 완료했다. Windows 전체4검사841/0/13 및 실제CLI36행/45명령·추가native20명령에서 실제7파일/4root 제거·outer부재와 원source/journal ID·SHA 보존, foreign/torn/mixed 거절을 root/독립 검토했다. 제거 확인·취소·완료 표시 UI를 병렬 준비 후 통합한다. 현재CPU11bb9b…/CLI65e0fe…/installer65c80d…이며 이전a7e CPU 비용 측정을 최신binary에 소급하지 않는다. 정식installer·upgrade·cleanOS/physical·전체P0는 남는다. [제거 CLI](docs/work-items/S07-windows-explicit-user-bundle-removal-cli.md), [현재 UI](docs/work-items/S07-windows-native-development-removal-ui.md). 아래는 이전 snapshot 기록이다.

2026-10-06 최신: 별도 Rust native 개발 설치 화면을 scoped완료했다. Windows전체4검사832/0/13·실제 격리GUI5case/10명령·추가native24명령/PNG8에서 설치·재검증·기존폴더거절·복사취소·CloseWait·wrongSHA 및원자료를root/독립검토했다. installer4c2542…759,296B·CPUa7e60e…8,710,656B·CLI7e695e…934,400B, lock/GNU보존. 현재CPU18budget은 최대CPU0.630450%/WS71,680,000B·3PNG/합성30k/3DB 보존을 root/독립 검토했다. 명시bundle제거의 원cap 이전·실패시 보존·재시도 엔진과 회귀를 통합하고 Windows 검증을 준비 중이다. System표준폼의개발범위이며 browse/defaults/packaging/upgrade/physical·cleanOS/정식installer·전체P0는남는다. [완료 범위](docs/work-items/S07-windows-native-development-bootstrap.md), [현재 작업](docs/work-items/S07-windows-explicit-user-bundle-removal.md), [개발 설치 화면 안내](docs/development/windows-native-installer.md). 아래는이전snapshot기록이다.

2026-10-06 최신: journal-aware 개발 CLI15를 완료했다. Windows전체4검사827/0/13·실제15명령/성공3/예상실패12+native7명령, 실제 복사 취소/30초기한3명령/2case+native9명령을 root/독립 검토했다. CPU5bcc9f…8,710,656B·CLIc4cbc7…935,424B, lock/기존GNU 보존. 최신 앱 GUI 비용은 측정 중이며 별도 native bootstrap 화면의 service/UI/observer 초안은 /tmp에서 병렬 구현한다. 정식 installer·upgrade·cleanOS/physical와 전체P0는 미완료다. [완료 범위](docs/work-items/S07-windows-journal-aware-bundle-cli.md), [현재 작업](docs/work-items/S07-windows-native-development-bootstrap.md). 아래는 이전snapshot기록이다.

2026-10-06 최신: 실제 검증 단계마다 기록하는 journal-aware coordinator API를 Windows 전체4검사826/0/13 및 실제 public factory·root native7명령/19관측·독립 검토로 인수했다. 실제4stage SHA/chain/manifest와 원 nested ID·자료 보존을 확인했다. outer ID는 현재 post-Drop 관측이며 crash/history 권한은 미증명이다. 이제 15번째 개발 CLI 연결을 /tmp 구현·인수 검토와 병렬 진행한다. 정식 installer/최신 GUI 비용/physical·cleanOS와 전체P0는 미완료다. [완료 범위](docs/work-items/S07-windows-journal-aware-fresh-bundle.md), [현재 작업](docs/work-items/S07-windows-journal-aware-bundle-cli.md). 아래는 이전 snapshot 기록이다.

2026-10-06 진행: 설치 journal API와 bounded native fixture는 Windows 전체 4검사(818통과/0실패/13무시), 실제 fixture 및 root 추가 native 9명령·독립 검토로 현재 일관성/원 ID·SHA 보존 범위를 인수했다. 14번째 `inspect-install-journal` CLI도 실제12명령·원24파일·native32ID 검토로 완료했다. 현재 explicit process exit 보존 시험을 통합·검증하고, journal-aware coordinator 계약을 /tmp에서 병렬 조사 중이다. 이전 성능 18구간 PASS는 당시 고정 앱의 결과이며 최신 CLI와 전체 P0 인수로 소급하지 않는다. 정식 installer·crash recovery·physical/clean OS 게이트는 남는다. [journal 작업](docs/work-items/S07-windows-install-journal-next-slice.md), [CLI 작업](docs/work-items/S07-windows-install-journal-readonly-cli.md). 아래는 이전 snapshot 기록이다.

2026-10-06 현재: 명시묶음readonly재검사/13번째CLI를완료했다. 새Windows전체4검사813/0/13·실제17명령/7행·원11nativeID와canonical/digest/7files보존·typed4/namespace negative2를root/독립인수했다. CPU75d05…8,710,656B·CLI0d1b24…847,872B(+6,144B), GNU/lock보존. 같은최종CPU의18구간18PASS(최대CPU0.935892%, WS72,212,480B)와합성30k/DB3보존도완료했다. 최초개행collectorFAIL과이전CPUbudgetFAIL은보존한다. 앞서worker/지표복귀·숨김복귀/목록·scan CloseWait release/timeout도각binary범위완료했다. 현재설치journal 공유API를통합하고Windows전체검사·nativefixture초안·독립검토를병렬진행중이다. [재검사완료](docs/work-items/S07-windows-user-bundle-readonly-reopen.md), [다음journal](docs/work-items/S07-windows-install-journal-next-slice.md). 전체P0·최종스택·정식installer/physical/cleanOS게이트는미완료다. 아래는이전snapshot기록이다.

2026-10-06 현재: 새 개발 설치 묶음·12번째 CLI의 source579 Windows 전체4검사(774/0/13)와 실제 생성/cold검사/격리GUI3case·14명령/11행·NativeID10, 별도 복사중취소/기한3명령/2case를 root/독립 인수했다. CPUa6d9223a…8,709,632B·CLI97217440…841,728B, GNU/lock 보존. 이후 GUI wake 계측4파일을 통합해 기본/별도진단 빌드를 검증 중이며 같은 source579 항목수도 다른 정확map이다. 이전 CPU1.247473% 실패·성능 원인 조사·정식 journal/upgrade/UI설치·physical/cleanOS 게이트와 전체P0 미완료를 유지한다. 아래는 각 이전 snapshot 기록이다.

2026-10-06 현재: [프레임 비용 계측](docs/work-items/S00-windows-private-frame-phase-diagnostic.md)을 통합한 source578의 기본 Windows 전체4검사·760passed/13ignored가 root/독립 PASS다. CPU16b4d0e…8,709,632B·CLIa0bb2ab8…795,136B이며 원GNU/lock은 보존했다. 실제 observer C# 컴파일은 통과했고 별도 defaultOFF 계측 feature 전체빌드를 진행 중이다. 이전 source577의 CPU1.247473% 실패는 보존하며 새 바이너리 성능 인수·원인 해결·전체P0 완료는 아직 미완료다. 아래는 이전 snapshot 기록이다.

2026-10-06 현재: 원 Job readonly 관측과 부모 종료 후 자식 수명 fixture의 source577 전체 MSVC4검사·760passed/13ignored가 완료됐다. CPU594676ba…·CLIa0bb2ab8…의 실제 자식 대기/취소/기한 인수도 root/독립 PASS다(12명령/3case·nativeID30·원파일21). 현재 source577의 loaded GUI 비용은 18구간 중 한 구간 CPU1.247473%로 FAIL이며 모든 메모리 구간과 자연 종료·격리 화면 복원은 통과했다. 같은 구간 main78.125ms/library46.875ms를 관측했고 원인은 미확정이다. [프레임 단계 계측](docs/work-items/S00-windows-private-frame-phase-diagnostic.md)의 Rust 구현·관측 수집기·독립 증거 검토를 병렬 진행한다. 이전 GUI·ZIP·비용은 각 고정 binary의 기록이며 전체 P0는 미완료다.

2026-10-06 최신: 시작 검사 기한과 실행 수명을 분리한 `run-selected`를 구현했다. 새 MSVC 전체 fmt/clippy/test/release와 760passed/13ignored·설치89 회귀는 root/독립 검토 PASS다. CPUc9f09dc0…·CLIab8a6cde…의 실제 격리 GUI3개와 추가 CLI·원nativeID10 보존(17명령/14사례)을 root/독립 검토했다. 기존 bounded 실행의 새 바이너리 회귀도 실제15명령/13사례·GUI4까지 root/독립 PASS다. 현재 부모 종료 후 자식 수명 fixture와 원 Job readonly 관측을 새 source에 통합해 검증 중이며 아래760/GUI 인수는 직전 snapshot의 기록이다. 이전 GUI/ZIP/비용 결과는 이전 binary의 기록이다. [현재 작업](docs/work-items/S07-windows-selected-supervisor-lifetime.md). 전체 P0는 미완료다.

2026-10-06 현재: MSVC 전체 fmt/clippy/test/release·758passed/13ignored, [설치 선택본의 격리 GUI·기본 CLI 실행/재시작/취소/기한](docs/work-items/S00-windows-msvc-selected-private-gui.md), [대상 graph258의 notice와 동일 개발 ZIP 두개](docs/work-items/S07-windows-msvc-development-package.md)를 root/독립 검토로 완료했다. CPU121f02cd…/8,709,632B·CLI a3001bd2…/786,944B·source576·GNU 원 앱/CLI·lock 보존. [새 MSVC GUI 비용](docs/work-items/S00-windows-msvc-current-gui-cost.md)은 18window 중 CPU1.09% 한 구간의 FAIL을 보존하고 원인을 조사 중이며 전체P0는 미완료다. 정식 bootstrap·upgrade/rollback, cleanOS/최소OS/서명, physical IME·reader·clipboard도 남는다. 아래는 이전 snapshot 기록이다.

2026-10-06 최신: [명시 선택 개발 CLI 실행](docs/work-items/S07-windows-explicit-selected-launch-cli.md) 범위 완료. source576·758passed/13ignored·Windows 전체4검사 및 실제 격리GUI4case/추가CLI9/native3query를 주 담당·독립 담당이 확인했다. CPU54c33d95… 불변, CLI5ae864b8…/754,688B. 원파일34/ID10·설정Light 재시작 보존. 현재 MSVC 프로젝트 도구 공급을 이어서 진행한다. 정식 bootstrap/upgrade·cleanOS/서명·physical 게이트와 전체P0는 남는다. 아래는 이전 snapshot 기록이다.

2026-10-06 최신 GUI 인수: [명시 선택 설치본 실행·재시작](docs/work-items/S07-windows-selected-copy-private-launch.md) scoped DONE. source576/CPU54c33d95…의 실제 두 private GUI·시스템→밝게 선택·원세경로 재시작 밝게 표시·readonly SQLite integrity/theme·원파일/nativeID10·자연Job0/input불변을 주 담당/독립 검토했다. 원 parent Job 단조회 FAIL은 보존했고 수정 재시험은 강제 정리 없이 PASS다. [MSVC 프로젝트 전용 공급](docs/work-items/S07-windows-msvc-user-tool-supply.md)과 정식 bootstrap/upgrade·cleanOS/서명·physical 게이트는 남는다. 전체 P0는 미완료다. 아래는 이전 snapshot 기록이다.

2026-10-06 최신 통합: [명시 선택 CLI](docs/work-items/S07-windows-active-selection-local-cli.md)와 [준비 신호 취소·기한](docs/work-items/S07-windows-active-cli-control-fixture.md)은 범위 인수를 완료했다. 선택 앱의 원 소유 런처는 실제 정상 종료·실행 후 취소·기한 종료를 검증했다. 후속 GUI fixture의 Windows 전체 fmt/clippy/test/release 및 독립 검토는 현재 source576(항목 수)·755passed/13ignored·설치89회귀로 통과했다. CPU54c33d95…/7,917,568B·기본9명령 CLI f97300cc…/723,456B, lock 불변. [명시 선택 GUI 재시작](docs/work-items/S07-windows-selected-copy-private-launch.md)의 실제 격리 화면·DB 인수와 [MSVC 도구 공급](docs/work-items/S07-windows-msvc-user-tool-supply.md)을 이어서 진행한다. 이전 snapshot의 GUI 성능을 최신 binary에 소급하지 않는다. 전체 P0·정식 installer/upgrade·cleanOS/서명·physical 인수는 미완료다. 아래 문단은 이전 snapshot 기록이다.

2026-10-06 최신: [명시 설치 선택 엔진](docs/work-items/S07-windows-active-install-coordinator.md) scoped DONE. Windows전체4검사·설치87회귀·실제6case와별도process readonly canonicalSHA·정상원nativeID10·원두등록쌍/receipt/notice/sentinel보존을root·독립검토했다. CPU53d12740…/7,917,568B(+0B), lock불변. [CLI두명령연결](docs/work-items/S07-windows-active-selection-local-cli.md)은통합후전체검증중이다. 정식installer/active실행·upgrade복구·cleanOS/MSVC/서명·physical인수와전체P0는미완료다. 아래는당시snapshot이력이다.

2026-10-06 최신: [현재 설치 후보 격리 GUI 인수](docs/work-items/S00-windows-current-install-artifact-ui.md) scoped DONE. 기본CPU f3bae801…/7,917,568B의3×60초18창 CPU<1%/WS<80MiB·원합성파일30000/3DB보존·Unicode47단계 및같은source 별도defaultOFF ownerIMM취소/확정이root/독립PASS다. 원 검증wrapperFAIL은계약오류로보존하고native실제PASS와구별했다. 물리IME/reader·기본binaryOScomposition·cleanOS/서명·전체P0/스택채택은미완료다. 다음 active선택/재시작readonly대조의공유계약·schema·원caps 검토를병렬진행한다. 아래문단은당시snapshot이력이다.

2026-10-06 최신: 검증된 GUI+일곱 명령 개발 CLI의 [동일 입력 ZIP 두개 인수](docs/work-items/S07-windows-cli-companion-package.md) scoped DONE. 실제 두 ZIP SHA1ed2a3ef…/13,364,856B·864entries 및 root/독립 전체member·PE·현재 whole-source/proof·내장font 대조 PASS. 전체lock notice 누락43과 release_ready=false를 유지한다. 현재 병렬 담당은 새CPU 격리 GUI 비용·IME 감사/collector·독립 패키지 검토로 분리했으며 root가 native 실행을 직렬 조정한다. P0 전체는 미완료다. 아래 문단은 당시 snapshot 이력이다.

2026-10-06 최신: 외부 receipt 개발 CLI 일곱 명령 인수 scoped DONE. Windows 전체4검사·설치75/parser4·실제28명령·현재 복사중취소/기한2case·별도원18nativeID와 root/독립검토 PASS. CPU f3bae801…/7,917,568B 불변, CLI efcef0b0…/651,776B(+74,752B), dependency/lock 변경 없음. source570은 항목 수로 이전 제거 snapshot과 CLI hash map이 다르다. 전체 P0는 미완료다. 다음 Windows현재배포물 notice/실제active설치·upgrade복구·cleanOS/MSVC/서명 및physical인수를 진행한다. 아래 이전문단은 당시 snapshot 이력이다.

[현재 CLI 인수](docs/work-items/S07-windows-registration-receipt-local-cli.md).

2026-10-06 최신: 명시적 등록 묶음 제거 engine source570 scoped DONE. Windows GNU 전체 fmt/clippy/test/release·설치75회귀·실제4묶음/7행·취소/기한/foreign accepted0과 같은 원caps 재시도·원 receipt4 nativeID 및 root/독립검토 PASS. CPU f3bae801…/7,917,568B(+0B), Cargo.lock 불변. 전체 P0는 미완료이며 현재 개발 CLI receipt 생성·대조·명시 제거를 분리 초안으로 병렬 구현한다. 이전 source 수는 snapshot 항목 수이며 동일 수의 실패 hash map을 완료 근거로 쓰지 않는다.

[현재 제거 인수](docs/work-items/S07-windows-registered-cold-removal.md).

2026-10-06 최신: source567 등록묶음+외부receipt readonly대조 scopedDONE. WindowsGNU전체4검사/설치68회귀/원source563묶음 별도process 원ID·SHA·canonical/5+1namespace·취소/기한4실패/root독립검토PASS. CPU3658c316…/7,917,568B·Cargo.lock불변. P0전체는미완료다. [완료scope](docs/work-items/S07-windows-registration-receipt-pair.md). 다음 명시cold제거·active설치/upgrade복구·공급/cleanOS/MSVC/서명·physical/remoteCI 인수를진행해야한다. 정상Git없음은보존하고공개배포하지않았다.

2026-10-06 최신 설치 기반: source563 외부 등록 receipt도 전체4검사·설치61회귀·실제5cases·원native2IDs/root독립대조를 통과했다. 새CPUdea3c8e0…/7,917,568B이며이전GUI비용을소급하지않는다. 현재 receipt 재시작readonly 검사후속을통합검증한다. [작업](docs/work-items/S07-windows-registration-receipt.md). P0전체는미완료다.

# dlpad

> 사용자 범위 변경: 최초 릴리스는 **Windows 우선**이다. Linux·macOS 및 3OS 출시 인수는 후속으로 보류한다. 기존 문서의 3OS 게이트는 후속 목표이며 Windows P0 완료를 차단하지 않는다. 공통 Rust 구현과 기존 검증 근거는 유지한다. [범위 계약](docs/work-items/P0-windows-first-release.md).

가볍고 관리하기 쉬운 Rust 네이티브 yt-dlp GUI. Windows, macOS, Linux에서 긴 영상 다운로드, 구간 저장, 파일 목록과 작업 상태를 일관되게 관리한다.

현재는 **Windows 네이티브 CPU 후보의 P0 구현·인수 단계**이며 MSVC 개발 빌드와 실제 격리 Windows 시험을 진행한다. 기존 GNU 검증 기록도 보존한다. 기존 Linux 개발 구현과 검증 기록도 보존한다. URL·구간·외부 yt-dlp 경로 입력, 단일 다운로드 worker·취소, 임시 파일 목록, 실패 보고서 미리보기·복사·새 파일 저장을 연결했다. Linux에서는 작업 유지 숨김과 같은 앱 재실행 복귀, FFmpeg 처리 배속 표시를 개발 기능으로 연결했다. 복귀 준비가 실패하거나 미지원 OS이면 숨김을 비활성화한다. Windows GNU native의 격리 GUI에서도 합성 작업 중 숨김·동일 worker 유지·재실행 복귀·중단 후 종료를 검증했다. 8시간 headless 합성 engine·SQLite·guardian 시험은 고정 바이너리에서 완료했다. 두 공개 URL의 짧은 미리보기는 별도로 검증했으며 실제 사이트 전체 다운로드·GUI 장시간 비용·3 OS UI·정식 설치·서명 검증은 끝나지 않았다. **P0 또는 첫 릴리스 완료 상태가 아니다.** S10 P0는 첫 릴리스 게이트에 포함한다.

2026-10-06 watch500 CPU snapshot은 Windows 전체 fmt/Clippy/test/release, 실제 10k 스캔 완료145.160초·Next2.031초·Stop ACK, 자동 파일 변경·누락 폴더 회복, GUI 비용3회/18개10초구간 CPU<1%·80MiB를 통과했다. 같은 snapshot의 S10 대형 보고서3case, 테마3종 재시작, 200% 합성 키보드47단계와 Unicode 한글 텍스트 readback, 동일 개발 ZIP2개·내장폰트 결합도 검증했다. 실제 IME/reader·설치·공급·서명 등 인수는 남는다. 후속 설치 preparation source550의 Windows 전체4검사·22개 실제 tests와 copied 앱의 두 private 시작/Light DB 재시작은 통과했다. 읽기 전용 prepared 폴더 재검증도 source552 전체4검사·설치 관련31회귀와 별도 실제 복사본 inspection을 통과했다. 개발용 로컬 prepare/inspect CLI도 source553 전체4검사·파싱2회귀·실제11명령·원nativeID·격리 windowless 취소/기한·foreign/partial 검사까지 통과했다. source555의 명시적 fresh discard도 전체4검사·설치40회귀·실제4bundles 정리/원caps 재시도·기존prepare/CLIcontrol 호환을 통과했다. source558 개발용 fresh 등록도 전체4검사·49회귀·실제CPU4묶음/원record-seal/nativeID대조를 통과했다. source560 등록 재시작 읽기 검사도 전체4검사·53회귀·별도 실제복사본 원ID/SHA·네가지 취소/기한 실패·cursor37 보존을 통과했다. 개발 CLI 등록 준비·검사도 전체4검사·parser3·실제16명령·원nativeID 및현재legacy취소/기한보존을 통과했다. 이전 snapshot 비용 결과를 새 코드에 소급하지 않는다. [최신 상태](docs/validation/P0-status.md), [크기 변경 검증](docs/work-items/S00-frame-reuse-resize-pixel-native.md), [숨김·복귀 검증](docs/work-items/S03-windows-current-cpu-background.md).

[S00 측정과 남은 게이트](docs/validation/S00.md), [요구사항별 구현 상태](docs/validation/P0-status.md), [작업 기록](docs/work-items/S00-implementation.md)을 읽는다. 정상 Git 저장소가 없어 기존 빈 `.git`을 보존했으며 commit/remote CI는 수행하지 않았다.

2026-10-04 [남은 P0 인수 감사](docs/validation/P0-remaining-audit-20261004.md)와 [WSL 중단 관찰·복구](docs/validation/wsl-interruption-recovery.md)를 추가했다. Linux 출력 상한 종료 수정은 engine library201개와 실제 service terminal22개 회귀를 통과했다. Windows 격리 GUI 합성 인증 다운로드 성공·metadata 실패는 검증했고, download 실패·취소도 후속 격리 GUI 시험을 통과했다. 제품 쿠키 gate의 실제 합성 다운로드4case와 same-boot receipt 복구2case는 통과했다. 이전 boot pending 회수 권한과 부모 직접 Stop을 피하는 의존성 정리를 구현하고 합성 시험 및 최신 기본 workspace/CPU feature의 Clippy/test/release를 통과했다. 이후 EOF keeper로 초기 identity 미확보 잔여물을 보강하고 실제 시작 실패 세 단계 및 정상 대조를 검증했다. 현재 keeper·옵션 도움말 변경과 Windows Clippy 배치 수정은 Linux 기본/CPU 전체7명령 및 Windows GNU CPU 전체4명령을 통과했다. 제품 foreign live/queued sibling 보존2case도 검증했다. 실제 재부팅과 다른 OS 후속 인수는 남는다. 부분 검증 통과를 P0 완료로 표시하지 않는다.

2026-10-03 사용자 재개 지시로 개발을 다시 진행한다. 현재 상태는 [진행 지점과 재개 순서](docs/validation/P0-status.md)를 먼저 확인한다. 이미 시작한 격리 headless 장시간 시험은 자동 종료까지 유지하며 실제 terminal 결과를 계속 확인한다.

2026-10-04 후속 [실제 8시간 종료 검증](docs/validation/evidence/soak-terminal-independent-20261004/root-review.json)과 [Windows 정상 크기 보고서 가시화](docs/work-items/S10-report-focus-scroll.md)를 완료했다. 장시간 결과는 당시 고정 headless 바이너리의 증거이며 최신 제품 전체·GUI·실제 영상 검증이 아니다. 쿠키 복구의 잠금 전 소유 기록 검증과 bounded capability 교환을 통합했고 Linux CPU workspace 시험·clippy·release를 통과했다. Same-boot launcher receipt 연결과 전체 P0 인수는 남는다.

2026-10-04 Windows 인증 실행에 독립 native helper·durable reader gate·원 receipt를 연결하고 제품 startup 회수를 구현했다. 실제 앱 유사 프로세스 종료 후 정리와 제품 인증 다운로드 성공/단계별 실패/취소4case를 native 시험했다. 최신 변경 전체workspace 및 GUI 인수는 진행 중이며 전체 P0/릴리스 완료가 아니다. [현재 범위와 남은 인수](docs/validation/P0-status.md)를 기준으로 이전 snapshot 기록을 구분한다.

2026-10-04 Windows 후속 인수에서 진단11case 중 원5case와 입력 pacing 교정 후6case를 각각 검증했고 원실패도 보존했다. 테마3종 저장·재시작, 합성10k 파일 목록의128개 페이지·수동변경·자동변경·잘못된 저장 위치 보존, 원native file ID와 DB identity 일치, Unicode 저장경로와 실제 owned ACL 오류5/원ACL복원/설정보존을 확인했다. 후속 Windows GNU CPU fmt/clippy/test/release 네명령도 통과했다. 실제 IME/reader·clipboard·대형보고서·loaded 비용·설치/site 등 남은 게이트는 [현재 인수 상태](docs/validation/P0-status.md)를 따른다. 전체 P0 및 최종스택 선정은 아직 미완료다.

2026-10-05 Windows native 목록 스캔·정확 동일 목록의 불필요 DB 갱신 생략·identity 할당 최적화를 통합했다. 실제 목록27회귀, 합성1만파일 전체native ID/크기/시간 대조 및 최신540소스의 Windows CPU 전체fmt/clippy/test/release가 통과했다. 앞선537소스 검증은 이력이다. 10k 목록의 주기 CPU 예산은 아직 해결 중이다. SQLite FULL 저널·게시 복구와 외부 HTTPS notice 동일 설치 트랜잭션을 구현·실측했으며, S10 대형 JSON/Markdown/미리보기의 Job committed peak를 측정했다. 실제 reader/IME·S00 최종 선정·공급/설치 인수는 남는다. [현재 증거와 남은 게이트](docs/validation/P0-status.md)를 따른다.

## 문서 읽는 순서

1. [제품 방향](PRODUCT.md), [요구사항과 인수 조건](docs/REQUIREMENTS.md)
2. [도메인 용어](CONTEXT.md), [아키텍처](docs/ARCHITECTURE.md), [기술 스택](docs/STACK.md)
3. [디자인 가이드](DESIGN.md), [사용 흐름](docs/UX.md)
4. [도구·옵션·인증 관리](docs/INTEGRATIONS.md), [실패 진단과 AI 전달](docs/DIAGNOSTICS.md), [성능 검증](docs/PERFORMANCE.md)
5. [구현 순서](docs/ROADMAP.md), [CI/CD와 배포](docs/CICD.md)

모든 코딩 에이전트와 기여자는 [AGENTS.md](AGENTS.md)와 [개발 안내](CONTRIBUTING.md)를 따른다. 결정 변경은 [ADR](docs/adr/README.md)에 기록한다. 외부 자료는 [출처](docs/SOURCES.md)에 모았다.

## 지금 실행 가능한 검증

```sh
python3 scripts/check_docs.py
```

추가 Python 패키지는 필요 없다. Windows에서는 `python scripts/check_docs.py`를 사용해도 된다. Rust workspace와 정확한 toolchain/Cargo.lock을 추가했다. CI는 Windows를 기본으로 검사하며 Linux/macOS는 수동 실행의 선택 항목으로 보존한다. remote가 없어 Actions를 실제 실행하지 않았다.

## 개발 화면 실행

Rustup과 Linux의 X11/Wayland/xkbcommon 런타임, 한글 시스템 폰트가 필요하다. Rust 1.94.0 및 rustfmt/clippy는 rust-toolchain.toml로 고정한다. 다른 OS는 아래 명령의 컴파일/실행을 아직 검증하지 않았다.

```sh
cargo run -p dlpad --bin dlpad -- --fixture
cargo run -p dlpad --bin dlpad -- --fixture --diagnostics
cargo run -p dlpad --bin dlpad-egui-spike --features alternative -- --fixture
# 접근성 다운로드 후보: 임시 app data/media 경로를 지정해 시험한다.
DLPAD_TEST_DATA=/tmp/dlpad-accessible-data DLPAD_LIBRARY_ROOT=/tmp/dlpad-accessible-media cargo run -p dlpad --bin dlpad-accessible --features alternative
# CPU 접근성 후보: 전체 P0 기능 이식과 OS별 검증을 진행 중이다.
DLPAD_TEST_DATA=/tmp/dlpad-cpu-data DLPAD_LIBRARY_ROOT=/tmp/dlpad-cpu-media cargo run -p dlpad --bin dlpad-cpu --features cpu
```

소프트웨어 renderer가 개발 기본값이며 최종 UI 스택 채택을 뜻하지 않는다. dlpad-accessible은 기존 엔진의 다운로드·일시 중단/재개·정제 보고서 복사를 연결한 별도 native 후보다. [실제 GUI](docs/validation/accessible-download-validation.md)와 [한글·무음 reader](docs/validation/accessible-native-validation.md)를 검증했지만 전체 화면 기능과 다른 OS는 미검증이다. [대기 RSS](docs/validation/accessible-performance-validation.md)가 제안 예산을 초과해 최종 스택으로 채택하지 않았다. fixture는 합성 10,000행과 정제된 실패 자료이며 미디어나 Documents를 생성하지 않는다. Linux에 한글 폰트가 없으면 검증용 `DLPAD_TEST_FONT`에 기존 로컬 폰트 파일을 지정한다. 폰트를 배포물에 묶거나 네트워크에서 자동 로드하지 않는다.

```sh
DLPAD_TEST_DATA=/tmp/dlpad-dev-data DLPAD_LIBRARY_ROOT=/tmp/dlpad-dev-media cargo run -p dlpad --bin dlpad
```

파일 목록 시험은 먼저 임시 media 폴더와 합성 파일을 만들고 진행한다. DLPAD_TEST_DATA는 앱 전용 임시 데이터 폴더여야 한다. 다운로드는 UI에서 지정한 외부 yt-dlp 또는 명시적으로 활성화한 앱 관리 도구 경로를 사용한다. 앱 시작 시 이전 작업 journal을 확인하며 확인 중/실패에는 새 다운로드를 막는다. 상태 복구 뒤 이전 다운로드를 자동 재시작하지 않는다. 작업 이력에서 검증된 원래 옵션으로 같은 ID를 재개할 수 있으며, 일시 중단과 취소를 구분한다. 전체/구간·품질 입력 후 시작하고, 실패 시 보고서로 이동한다. 구간 저장은 빠른/정확 자르기를 선택하며 고급 옵션에서 속도 제한·재시도·조각 동시성을 설명과 함께 조절한다. 선택한 외부 FFmpeg의 절대 경로는 실행 계획에 고정하고 확인한 버전을 Attempt와 실패 보고서에 남긴다. 외부 Deno의 절대 경로도 네이티브 입력에서 선택할 수 있고 metadata·다운로드의 동일 argv와 Attempt 버전 증거에 연결한다. 빈 Deno 입력은 기존 yt-dlp 기본 탐색을 유지하며 실제 사이트 호환성과 관리 공급 완료를 뜻하지 않는다. 빈 FFmpeg 입력은 yt-dlp의 PATH 탐색을 유지한다. 일반 실행은 OS app data에 설정을 보관하고 저장한 위치 또는 Documents/dlpad를 준비한다. 위 환경 변수는 임시 폴더 시험용이다. 저장 위치 적용·기본 폴더 복귀와 다운로드 속도/ETA, Linux 선택 인터페이스 관측을 연결했다. 실제 yt-dlp/FFmpeg의 합성 전체·구간 다운로드와 두 공개 URL의 짧은 미리보기는 통과했지만 실제 사이트 전체 다운로드와 다른 OS는 미검증이므로 개발 화면을 일반 사용 릴리스로 사용하지 않는다. 도구 화면에서 공식 고정 yt-dlp의 선택 다운로드·검증·관리 버전 전환을 개발 기능으로 연결했다. 관리 경로 사용은 명시적으로 적용하며 외부 설치는 제거하지 않는다. Windows 개발 후보는 FFmpeg/ffprobe와 Deno의 관리 다운로드·검증·활성화를 지원한다. 쿠키 로그인과 업로드는 제공하지 않는다.

작업 중 창을 닫으면 **돌아가기 / 중단 후 종료 / 백그라운드 계속**을 선택한다. 백그라운드 계속은 창을 숨기고 같은 작업을 유지한다. 숨긴 작업이 완료되어도 창을 자동으로 띄우지 않는다. 같은 실행 파일을 같은 앱 데이터 위치로 다시 실행하면 기존 창이 복귀한다. 복귀 기능 준비에 실패하면 숨김을 허용하지 않는다. Linux Xvfb와 Windows private desktop의 합성 시험 결과는 [숨김·복귀 작업 기록](docs/work-items/S03-background-integration-refresh.md)에 있다. macOS, 로그아웃·절전 동작은 미검증이다.

## Rust 통합 검증

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --release --locked
```

GUI 재현 측정은 사용자 데스크톱과 분리된 Linux Xvfb에서 `python3 scripts/native_probe.py target/release/dlpad /tmp/dlpad-probe/iced.json`으로 한다. Xvfb가 없으면 사용자 화면에 창을 띄우지 않고 실패한다. 실제 데스크톱 시험은 명시적 `--desktop` 선택이 필요하며 포커스를 가져갈 수 있다. 자신의 테스트 GUI와 가상 X 서버만 실행·종료하며 임시 데이터와 합성 fixture를 사용한다. 스크린샷/CPU/RSS 측정은 IME·접근성·장시간 시험을 대체하지 않는다.

Linux 프로세스 크래시·시작 복구는 [guardian 검증](docs/validation/guardian-validation.md)에 기록했다. 앱은 GUI 이전 내부 headless guardian으로 소유 도구와 자손을 감시한다. 독립 engine 개발 시험은 `cargo test --workspace --locked`로 helper까지 빌드한다.

## CPU 후보의 현재 검증 범위

CPU/accessible 후보에 세션 한정 Netscape 쿠키 입력과 시작 복구를 연결했다. 인증 raw 로그·자유 metadata는 생략하고 DB/실행 계획에는 쿠키를 저장하지 않는다. Linux는 private 복사본의 소유 기록과 이전 boot 복구를 합성 fixture로 검증했으며, 같은 boot의 미확인 reader는 복사본을 보존한다. [구현·실제 시험 범위](docs/work-items/S02-session-cookie-download-integration.md)를 따른다. 이전5cb52의 격리 GUI3flow와 최신terminal/recovery소스검증은 별도snapshot이다. Windows 쿠키 ACL·Job 종료 확인·중단 가능한 pipe의 합성 native 시험을 통과했고 후보 worker 입력/보존 안내를 연결했다. [Windows 후보 연결 범위](docs/work-items/S02-windows-cookie-ui-integration.md)는 Rust 시험 근거이며 실제 인증 다운로드/GUI·Windows 회수·최종 UI 선정·전체 P0는 미완료다.

Linux [사용자 서비스 기본 보호 통합](docs/validation/user-service-final-validation.md)의ea7595 snapshot은기본210/CPU349시험과7개fmt/clippy/release명령을통과했다. [단독앱기본GUI3flow](docs/validation/user-service-app-default.md)에서숨김·복귀·중단후종료와같은앱guardian/cgroup정리를확인했다. 이후 [시작실패정리](docs/validation/user-service-startup-fault.md)는actualdebug3fault+manager누락과strict13회귀를통과했으며최신Linux전체회귀/release는별도진행중이다. Windows GNU native의최신소스255PASS/1ignored·clippy/release와입력261파일SHA일치도확인했다. WindowsGUI/MSVC/macOS검증을뜻하지않는다.

[접근성 펼침 상태 통합](docs/validation/cpu-expanded-state-validation.md)의608eeeb snapshot은 기본200/CPU339/Windows255시험과fmt·clippy·release를 통과했다. 실행 계획·진단·MPD 헤더의 실제 Linux AT-SPI 상태를 확인했다. 아래1669 비용·큰 보고서·한글 근거는 각 당시 snapshot이며 후속 guardian source 변경과 구분한다.

`dlpad-cpu`는 기존 Rust 엔진과 egui/AccessKit를 유지한 optional software renderer 후보다. Linux 격리 화면에서 [파일 목록·128개 페이지](docs/validation/cpu-library-gui-validation.md), [스캔 종료 대기](docs/validation/cpu-library-closewait-validation.md), [설정·테마·배율](docs/work-items/S02-cpu-settings-integration.md), [품질·구간·자막·고급 옵션](docs/work-items/S02-cpu-options-integration.md)을 검증했다. [이력·원래 옵션 재개](docs/validation/cpu-history-validation.md)는522e snapshot, [숨김·같은 앱 복귀](docs/validation/cpu-background-validation.md)는초기/callback-only b8bb snapshot의 실제 합성 GUI 근거다. 최신 binary의 모든 화면을 재실행했다는 뜻은 아니다.

최신 Linux1669 snapshot에서는 [정제된 실행 계획](docs/validation/cpu-plan-validation.md), [실제 볼륨·인터페이스 지표](docs/validation/cpu-metrics.md), [공식 고정 관리 도구 설치·실제 전송 취소](docs/validation/cpu-tools-official-install-validation.md), [최대 유효 진단 보고서의 페이지·전체 복사·새 파일 저장](docs/validation/cpu-report-stress-validation.md)을 확인했다. 지표 GUI는앞선3a1 snapshot 근거이며 sharedsource의 이후 CLI 회귀와 구분한다. 보고서의 실제 JSON225페이지·약1.84MB자료 인수는 방어2MiB 상한을 정확히 채운 fixture 인수가 아니다. [계획 헤더 native 토글](docs/validation/cpu-plan-toggle-diagnosis.md)은 pointer/AT-SPI/Enter/Space 두 번 확인했지만 expanded-state 게시와 전체 reader 사용성은 남는다.

[최신 대기 표본](docs/validation/cpu-paged-cost.md)은10.013초 CPU0.499%/RSS52.8MiB, Linux executable12402040bytes다. 대형 유효보고서 paged-4 peak HWM70.8MiB는 별도 좁은 stress 결과이며 모든 기능·OS·8시간 peak의 증명이 아니다. [Windows GNU native CLI](docs/validation/windows-cpu-paged-preview.md)는250passed/0failed/1ignored와clippy/release를 확인했으며 executable7244288bytes다. 새 Windows GUI·clipboard·IME/reader, MSVC/macOS, 실제 사이트·8시간 soak의 terminal 결과·절전·패키지/서명·전체설치 인수는 남는다. 기본 `dlpad`와 최종 UI/renderer 선정의 Proposed 상태는 변경하지 않았으며 P0/첫 릴리스 완료 상태가 아니다.

프로젝트의 Cargo 선언과 동일한 MIT 원문은 [LICENSE](LICENSE)에 있다. 제3자 notice와 미완료 공급 검증은 개발 후보 inventory에서 별도로 기록한다.

2026-10-05 동일 배경 목록의 UI 전송 생략·128행 공유를 통합하고 Windows 후보 각14회귀 및 최신540소스 전체fmt/clippy/test/release를 통과했다. 격리GUI3회 평균CPU0.73/0.65/0.81%와메모리는목표를충족했으나 strict10초구간은실패했다. 동일CPU3f9d 기준 개발ZIP2개(각12,682,630bytes/863entries)의전체SHA와모든entry를root·독립담당이대조했다. 실제wholelock notice누락43개이며 unsigned/development-only/release_ready=false를유지한다. [개발후보기록](docs/work-items/S07-current-windows-candidate-refresh.md). S10최신대형진단회귀는진행중이고전체P0는미완료다.


2026-10-05 Windows hard-link alias 수정 뒤 목록12B가남는결함을 실제 반복·단일파일로재현하고최신metadata/full128ID 대조로교정했다. 내부/외부alias·held/noheld·크기/동일크기mtime8조합을포함한목록30·watcher9·두UI각14회귀와최신541소스 전체fmt/clippy/test/release가PASS다. CPU0478aa51…/7,892,992B의격리GUI3회평균1.61/1.59/1.61%·strict10초구간은목표미달이라성능을이어교정한다. 이전3f9개발ZIP/S10자료는과거snapshot근거이며새CPU패키지/P0전체완료가아니다. [결함 교정과 비용](docs/work-items/S01-windows-hardlink-metadata-freshness.md), [현재 상태](docs/validation/P0-status.md).

2026-10-05 후속 목록 페이지 개선: 다음128개는 마지막commit DB 목록을조회하도록전체scan에서분리했다. 파일감시·수동Refresh·시작/위치변경의전체freshscan은유지한다. missing/legacy/corrupt DB를새로만들거나migration하지않는readonly조회와실패보존을검증중이다. [페이지작업](docs/work-items/S01-library-page-read-only.md). 제품CPU예산과전체P0는미완료다.

Windows 개발용 새 폴더 복사·읽기 전용 재검증 명령은 [로컬 설치 명령](docs/development/windows-local-install.md)을 따른다. source553 전체4검사·설치31회귀·parser2회귀 및 실제11명령은 통과했고, 실제 console callback 취소·기한 만료 후 일부 파일 보존은 후속 시험 중이다.

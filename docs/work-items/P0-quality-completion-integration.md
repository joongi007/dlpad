# P0: UI 품질·미디어 흐름·설치 및 배포 준비 통합

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R01,R04,R05,R06,R07,R08,R10,R11,R13
- 담당/수정 범위: root 공유catalog/schema/Cargo/통합/전체검증; diagnostic_inputs accessible_diagnostics.rs /tmp 후보, timeline_precision media_timeline.rs /tmp 후보, release_preparation CI/export/updater /tmp 후보. root accessible.rs,accessible_library.rs,path_picker.rs,engine media_preview/새local export·worker 및문서/검증 담당. 같은파일 동시수정 없음.
- 선행 작업: network 입력957 후보, 각기존 S00/S02/S10 작업

## 사용자 결과

사용자가 지시한 남은 입력·버튼 점검, 진단 입력 교정, 긴 영상 정밀 구간 선택,200%레이아웃,긴 경로 표시,미리보기 도구 수명,파일 목록→미리보기와 로컬 구간 export,설치파일·Git/CI/업데이트 준비를 구현/검증한다. 현재전체 P0 완료로 주장하지 않는다.

## 계약과 범위

Rust native/private GUI/no visible fallback/임시fixture. Git 정상아님을2026-10-11 재확인,원.git 보존;branch/worktree 불가능하므로하위에이전트는/tmp 후보만쓰고root통합한다. remote주소질문pending이며외부게시/서명·공개릴리스는별도구체적승인/외부조건을따른다. 사용자프로필/도구/원본을outputcache처럼삭제하지않는다. snapshot/lease/ACK/invariants 유지. 이전/tmp 초안은현재없어재위임했다.

## 인수 조건

각작업문서계약을실제현재source와Windows전체fmt/Clippy/test/release,actualContext/RawInput·private nativeUI/PNG·합성ffmpeg/output내용·취소/실패/원자료보존으로검증한다. installer와ZIP에현재proof/notice/toolchain을연결한다. CI/updater 준비와실제Git게시/배포/물리입력인수를구분한다.

## 검증과 handoff

2026-10-11 이어서 실제상태 재확인. 기존957/0/17/source610은원후보증거로보존한다. 준비계획만으로완료처리하지않으며현재작업진행중.

2026-10-11 재개: root가 installer_main.rs/catalog의 대상 폴더 중심 packaged UI와 accessible_library.rs의 긴 행 truncate/실제 geometry 회귀 후보를 통합했다. 하위 3작업은 installer collector, S10 collector 진단, local export 원cap 검토로 파일 담당을 분리했다. 통합 후보는 아직 신규 Windows 검증 전이며 이전972 proof로 완료하지 않는다.

실제 추가 검증: `python3 /tmp/dlpad-one-test.py real_long_unicode_library_rows_fit_virtual_height_at_narrow_width_and_all_scales .../windows-msvc-library-long-rows-20261011` 1passed/0failed, original natural exit0/Job0/private desktop closed/input desktop unchanged. `python3 scripts/test_generate_install_payload.py` 10tests PASS 및 `python3 scripts/test_update_candidate.py` 3tests PASS. `python3 scripts/check_docs.py` 873documents/114preservednotices PASS(후속 문서로 개수 변동 가능). 신규 전체 Windows 검사는 실행 중이며 테스트 raw aggregate982/0/17를 관측했으나 release 완료 전 전체 PASS로 선언하지 않는다.

2026-10-11 다음 통합: 표시 문자열과 원 output PathBuf를 분리하고 미편집 Apply는 원 root를 보존한다. native 설치 OS messagefont/통일배경/idle결과숨김과 local export engine 후보를 통합했다. 미리보기 busy/fatal의 재개 gate 및 bypass checkbox stableID를 보강했다. 신규 compile/실제 Windows 검사 전이다.

새 font/path/localexport 전체 source 후보는 Windows 41harness 1009/0/19와 fmt/clippy/release 모두 통과했다. 이후 실제 후반 timestamp와 local-layout/installer v6/preview begin 실패 경로 보강으로 source가 추가 변경됐으므로 이 whole은 당시에 대한 증거다. 후속 전체 검사를 실행한다. 실제 8h 후반·짧은영상·A/V 네범위·managed pending 실패 검증은 별도1건씩 통과했다. 설치 ZIP 생성·검증 Python18회귀, 리소스2, update3, sourceexport5 root 독립 통과. latest generated payload5files 및 SHA/CRC ZIP은 중간후보이고 새source최종설치/native검증 전이다.

2026-10-11 최신 통합: windows-msvc-local-av-ui-installer-v6-formatted-whole-20261011에서 fmt/Clippy/workspace test/release 모두 원자연종료0·Job0·private desktop/Input desktop guards 통과. root 원 raw log·616 source SHA map·GNU 원본/lock 보존 대조 PASS_SCOPED. 41 harness1009passed/0failed/24ignored. CPU9537024B, installer4310016B. 새 GUI/현재 패키지 검증은 별도로 진행 중이며 전체P0 완료를 뜻하지 않는다.

최신 cost 수집 첫 시험은 정상 ready 상태에서 사라진 이전 작업 다시 확인 버튼을 요구하여 관측 전 FAIL했다. 제품은 ready=false일때만 retry를 제공하므로 observer를 현재 계약에 맞춰 수정: 완료footer+빈URL+disabled primary+pending/fatal 부재 확인은 유지한다. 원 실패는 보존하고 새3x60 수집을 실행한다.

최신 currentCPU607de7 비용 ready-contract-idle-cost: 실제3x60/18windows scopedPASS. maxsinglecoreCPU0.621048393%,WS72,933,376B(69.55MiB). root rawterminal/원pins 대조; 전체hostquiet/물리IME/OSDPI/reader는 미검증. 앞missingretry관측실패를 성공에 합치지 않는다.


### 2026-10-11 진단 viewport 수정 후 새 전체 검증

root가 실제S10 PNG의 하단sliver를 발견하여 중복 controls ScrollArea를 제거했다. 새 실제 Candidate regression RED→GREEN과 기존DiagnosticPanel14회귀를 통과한 뒤 source616을 고정했다. `windows-msvc-diagnostic-viewport-final-whole-20261011`의 fmt/Clippy/test/release는 모두실제WindowsPASS. root원rawguard/원GNU/lock/source616/artifact재hash 대조 결과41harness 합계**1011passed/0failed/24ignored**다. CPU9,536,512B/SHAe241510c83fb35a506b52282b044afb32c71c5560d98d724774f7746486c403e; installer4,310,016B/SHAb47c85cee65321557c8cc0f5f0571452448eef3747a2620a4742bdb643e5f7f4 및CLI974,848B는이전과byte동일하다. GUI/private저장/idle/새payload검증은 이새CPU범위로후속한다. installer-only3folder경로의이전byte증거를 새CPU검증으로 소급하지않는다.


새CPU의 `windows-msvc-diagnostic-viewport-final-idle-20261011`은private3×60초/18fixed10초구간 모두PASS. root는원rawJob0/자연종료/입력desktop불변·전체pinSHA·실제3DB각10,000rows·3idlePNG를직접대조했다. 최대singlecoreCPU0.6227566003057565%/WS72,273,920B(~68.93MiB). quietHost/physicalIME/DPI/reader는미검증이다. 최신S10edit는세field32px fullvisibility와JSON/DB보존범위PASS_SCOPED다.

### 실행 계획·최종 설치 후보 인수

현재 `windows-msvc-plan-scroll-whole-20261011`: 실제 Windows fmt/clippy/test/release PASS,42harness1013/0/26. `windows-msvc-option-plan-native-scroll-20261011`은 numeric6/help/argv exactonce와readonly ScrollItemPattern·실제glyph를 검증했다. `windows-msvc-plan-scroll-idle-20261011`은3회/18구간 최대CPU0.6260397581322937%/WS72183808B, root DB3각10000 entries/PNG3/raw/pins를 대조했다. `windows-msvc-plan-scroll-installed-payload-20261011` 실제 5파일 설치와재검증·원본보존을통과했다. [최신 ZIP](../validation/evidence/dlpad-install-plan-scroll-20261011.zip)16334636B SHA0f1e7e18d1e0ea0f228734ea7904ddf564948ce278feab880cac11fb15e4941a. CLI/installer는이전whole과byte동일하지만CPU는b8434e…9537024B로바뀌었다. 이전 로컬저장decode/GUI인수는이전CPU후보의기록으로유지하며새binary에서전부재실행했다고주장하지않는다. 전체P0/원격CI/자동updater/cleanOS/서명은미완료다.

## 최신 설치 후보 입력·도구 설정 통합 인수

whole-fifth 1,055/0/28 및 최신설치ZIP에 추가제품수정 없이, diagnostic-caret-tool-preferences-fifth 실제9입력/선택/ROW35와 managed-conflict-seventh 실제4GUI/4closedDB를 root검토했다. 수동3path save/restart/empty/Auto5phase도통과했다. S04수동설정기능은 scoped complete이며 전체P0는미완료다. Windows 공식MSVChost4version·MinGitnativeversion·CI66Linux/60Windows+6권한skip 검증완료; 실제7CIproducer는 freshimmutableexport/새Git/cache/target로진행준비중이다. 고정8시간actual전송은별도owner로계속되고 최신idlecost는concurrentprobe없는조건에서검증한다. 원격Git/Actions/updater/runtime·물리OS·cleanOS/서명게이트는완료로표시하지않는다.

최신 첫 진입 조회 후보 whole1,063/0/28 및 실제 noManualRefresh managed GUI4단계·SQLite4, 새 설치 ZIP d1047262…와 실제5파일 설치/재검증·exact7member/원본보존/root PNG 검토를 완료했다. README의 현재 설치 링크는 dlpad-install-tools-first-open-20261011.zip이다. 기존1055 및 이전ZIP 결과는 해당snapshot에만 적용된다. 수동 tool executable browser를 추가 준비하며 최신idle비용은 실제8h 종료 뒤 검증한다. CI local bootstrap 재검증/원격 hosted Actions/자동 updater/사용자 최종 UI 및 물리 OS 인수는 남아있다.

현재806bf 후보의 실제 Windows private 진단 입력 회귀를 root 검토했다. 9입력readback/LINE-C noncollapsed선택/40줄의ROW35 실제PNG/invalid999999 보고서byte보존/closedSQLite integrity 및 단일Failed job/원source620·원SHA·natural0Job0를 확인했다. 증거 windows-diagnostic-current806bf-20261011/root-review.json. 물리IME·reader/providercaret geometry는 미검증이며 이전 binary 결과와 별도snapshot이다.

새 picker 통합 whole-second fmt/clippy/test/release 모두 통과: 1,085passed/0failed/28ignored/43harness, root source623 및 원raw/Job0/원SHA 확인. CPU9,652,736B SHA dcd3e53ef08238fd593f5841e1b3b2bbb4564533a17c34746b31595d8f964db5는 직전9,625,600B보다27,136B(26.5KiB) 증가했다. 신규 Cargo/lock/의존성/DBschema 없음, worker/channel1은 명시 선택 동안만 실행되며 실제idle비용은 후속이다. C# observer 실제compile 및 root원pins 검토도 통과했다. 실제private browser 선택/cancel/Save/restart/owner-close 및 새 설치snapshot 인수는 실행중으로 아직 완료가 아니다. 현재README 설치ZIP은 이전검증806bf 후보를 유지한다.

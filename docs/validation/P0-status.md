# 현재 Windows P0 상태 — 2026-10-17

## 최신 v37 설치기 재검사 복구 — 2026-10-17

초기 위치/기록 검사를 중단한 뒤 다시 검증하면 기본 기록 경로로 직접 검사해 설치·실행이 막히던 버그를 수정했다. 실제 private nativeState RED→GREEN과 원35 payload/원36→새37 설치기의 동일 cold 중단/재검사 실험에서 현재 suffix-1 재발견·실행 활성으로 복귀했다. 원 파일/기록·각 설치기2개 자연0/Job0·private/input 복구를 확인하고 Root가 원 로그·pins·PNG3장을 직접 검토했다. 전체37 네 Cargo 검사0·1685/0/43ignored·43harness, 새 ignored회귀 별도명시PASS다. 최신 설치/portable ZIP의 전체CRC/SHA·graph258/notice456 검토를 완료했다. CPU/CLI는36동일, 설치기+512B, schema/lock/의존성 변경0. [검수](ui-review-v37-20261017.md), [실행](../development/windows-user-preview.md), [native검토](evidence/installer-cancel-retry-root-20261017/root-review.json).

새37bundle 전체runtime 인수·실제 사용자 Explorer 선택·전체P0 완료로 확대하지 않는다. 아래 원36/35/33 근거는 각 snapshot으로 유지한다.


## 잔여 작업 목록 갱신 — 2026-10-17

최신 잔여 항목은 [v37 기준 현재 표](P0-current-20261016.md#현재-잔여-작업--v37-기준-갱신)를 따른다. 당시 추가한 설치기 ‘중단 요청’ 문구 개선과 새 중단 후 재검사 버그는 **v37에서 수정·검증 완료**했다. v36 실제 Explorer 선택·UI 최종 검수, Windows 실환경/진단 복사/치지직 저장, 공개 운영을 분리했으며 기존 8시간·대용량 시험을 다시 대기열에 넣지 않았다. 최신 패키지는 v37이다. 아래 v36 목록 갱신과 당시 구현 상태는 역사 기록으로 구분한다. 아래 기록은 원 snapshot 근거로 보존한다.

## v36 탐색기 파일 선택 오류 수정 — 2026-10-17

최신 후보는 **v36**이다. v35 사용자 실사용에서 보고된 탐색기 선택 오류의 canonical 경로 파싱 실패를 실제 Windows 합성 실험으로 재현하고, 원 pin/파일·부모 identity와 같은 Shell 표기만 사용하도록 수정했다. 실제 parser의 한글/공백/기호·260자 초과 roundtrip과 다른 항목 alias 거절을 회귀했다. Windows whole36 실제1685PASS/0FAIL/42ignored/43harness 및 네 Cargo 검사, 새 설치/portable ZIP의 전체CRC/SHA·graph258/notice456을 Root가 검토했다. [수정 검토](ui-review-v36-20261017.md), [실행](../development/windows-user-preview.md), [native 원인](evidence/reveal-native-fix-root-20261017/root-review.json), [whole36](evidence/windows-msvc-ui-feedback-v36-whole-20261017/root-review.json).

실제 사용자 Explorer 선택 완료는 남는다. UI 레이아웃과 설치기·CLI PE는 바꾸지 않았으며 아래35의 실제 설치/GUI와33 성능·긴 경로·CI·8h는 해당 snapshot 범위로 보존한다. 새 자동 앱 교체/업데이트나 전체 P0 완료가 아니다.

## 이전 v35 사용자 검수 기록

## v35 사용자 피드백 반영 — 2026-10-17

최신 검수 후보는 **v35**다. 기존 설치 첫 검증 결과 재사용, 앱 준비 중 비활성 이유·실패 재시도, 로컬 상세의 탐색기 파일 선택을 구현했다. Windows whole35 실제1679PASS/0FAIL/42ignored/43harness와 네 Cargo 검사, Root CPU PNG4개/104프레임, 실제 private 설치·제거·재설치·cold 실행 준비/설치 앱 자연종료를 확인했다. [검수](ui-review-v35-20261017.md), [최신 실행](../development/windows-user-preview.md), [whole35](evidence/windows-msvc-ui-feedback-v35-whole-20261017/root-review.json), [native](evidence/windows-msvc-ui-feedback-v35-install-reinstall-launch-private-v1-20261017/root-review.json).

실제 Explorer 선택과 사용자 UI 최종 승인은 남는다. SemVer0.1.0과 검수 번호는 별개이며 자동 업데이트/기존 앱 교체는 제공하지 않는다. 최신 UI 비교에는 새 폴더 설치 또는 portable을 사용한다. 아래33 성능·CI·긴 경로·8h와 외부 환경 인수는 각 원 snapshot 범위로 유지한다. 전체 P0 완료 선언이 아니다.

## v33 마감 당시 기록

**v33의 현재 환경 자동 구현·검증 작업은 마감했고 전체 P0·정식 출시는 미완료**다. 최신 완료/잔여 판단은 [현재 표](P0-current-20261016.md)를 기준으로 한다. 아래 v30 이하 고정 기록은 원문 그대로 보존한다.

whole33 Windows fmt·strict workspace/all-targets clippy·test·release 실제0, **1,670 PASS / 0 FAIL / 42 ignored / 43 harness / source675**다. [전체 검토](evidence/windows-msvc-ui-feedback-v33-whole-20261016/root-review.json). 현재v33 로컬CI도 **15단계·8Cargo·7capture·1,670/0/42·43harness**를 완료했다. [CI 검토](evidence/current-ui-feedback-v32-local-ci-lowstorage-v4-20261016/root-review.json). 경로의 current32/lowstorage-v4는 역사적 namespace이고 실제 admission은 whole33이다. hosted CI와 구분하며 원CI2 OS112 실패는 보존한다.

v33 기본3×60초 평균CPU는 **0.572377745%/0.494153664%/0.494159952%**, sampled WS peak **77.125MiB**, strict CPU17PASS/1FAIL(**1.092044216%**)·memory18PASS다. Root는 원 counter·WinNative30,000파일·readonly3DB·6PNG와 세앱 자연0/Job0을 대조했다. **원collectorFAIL/Python1/nativeidle2는 그대로 보존**한다. 평균 제안목표와 추가strict진단은 서로 다르며 quiet host/인과는 미확정이다. [성능 원검토](evidence/current33-default18-native-root-review-20261016/root-review.json).

v33 설치/제거/동일위치재설치/cold검증/설치앱 실행과 한글긴경로577/598UTF16은 실제완료했다. [설치](evidence/windows-msvc-ui-feedback-v33-install-reinstall-launch-private-v1-20261016/root-review.json), [긴경로46pins/Root3PNG·child14PNG](evidence/windows-msvc-ui-feedback-v33-long-install-private-v1-20261016/root-review.json). 두검수ZIP·선택258/notice456·폰트4개fullbyte·공식schema도 검토됐다. [공급](evidence/current33-supply-sidecars-root-20261016/root-review.json), [실행 안내](../development/windows-user-preview.md).

S10 현재100/200% preview/JSON971B저장·충돌거절·DB보존은 [현재 원검토](evidence/current33-s10-preview-composite-root-20261016/root-review.json) 범위다. 편집/제외/첨부/상한/CloseWait는 **역사Windows native11case+현재24매핑ok행(중복포함)+별도feature2행**으로 구체계약을 대조했다. [S10 매핑](evidence/current33-p0-closeout-root-20261016/S10-MAP.md). Windows8h actualwall은 이미28800.0003048초/474samples로 완료돼 **새8h재시험 불필요**다. [감사](evidence/current33-p0-closeout-root-20261016/WINDOWS-8H.md). 실제5GiB/u64/fullSHA/publication/DB와8개fault경로도 확인했다. [byte](evidence/r04-large-byte-axis-v6-20261016/root-review.json), [fault](evidence/current33-p0-closeout-root-20261016/FAULT-MAP.md). 이전snapshot근거를 새33native실행으로 바꾸지 않으며 광범위한 추가전수시험을 만들지 않는다.

v32 Help/History/도구복귀/영상상세·v32성능과 A31→B32→A31 수동합성프로필은 [고정UI기록](ui-review-v32-20261016.md)과 [수동시험](evidence/s07-manual-profile-private-v2-20261016/root-review.json)으로 보존한다. ADR0005 전체durable revision/coldlauncher/일반DBdowngrade는 Proposed이며 확정P0제품전체의무로 확장하지 않는다.

남은 인수는 **실제IME/reader/OS테마·DPI, 격리환경 Windowsclipboard, 실제 Documents/OneDrive리디렉션·sleep/resume, cleanOS·지원baseline, 사용자제공 유효치지직 저장, 공급/서명·공개/Gitremote·hostedCI 운영결정**이다. Windows P0·S10 최초릴리스 범위는 유지하며 전체완료로 표시하지 않는다. P1/P2·networkupdater에는 착수하지 않았다. [마감 작업](../work-items/P0-current33-closeout-20261016.md).

## 이전 v30 고정 기록

## 최신 v30 개발 검수 후보

[실행 안내](../development/windows-user-preview.md), [v30 UI/검증 기록](ui-review-v30-20261015.md). Windows whole30 실제1,619 PASS/0 FAIL/42 ignored/43 harness 및 fmt·strict clippy·release0을 Root가 원로그/source674/3PE/lock과 대조했다. portable·installer ZIP 생성/전체CRC/SHA/notice 검토와 private 설치→제거→동일위치 재설치→fresh검증→installedCPU실행·자연0/Job0·원33pins/3PNG를 확인했다. 도움말100/200% 3도구 Escape6관측은 모두닫힘이다.

남은 실제 UI 실패는200% 작은창 FFmpeg/Deno설명 입력가림과 작업이력 Failed 검색 안내문구 하단5px 잘림이다. 도구 확인중 진행 문구 구분·영상 상세 외부 앱 열기는 새 사용자 요청으로 scratch 구현 중이며 v30 ZIP에는 없다. 현재성능 feature 진단은 test의 StorageFull112로 실패해 원실패와 default3정확복구를 보존했다. 소유빌드캐시를 정리했고 shipping 기본예산해결은 미증명이다. 물리IME·reader·DPI/System·sleep/resume·OneDrive·cleanOS·긴경로현버전·치지직실제저장·공급trust/legal/서명·hostedCI/최종스택·전체P0는미완료다. 실제YouTube3.008초/144p/decode0는 [원근거](public-cut-review-20261015.md) 범위에서 통과했고 Chzzk HTTP404관측은 원인확정/다운로드성공이아니다.

## 이전 v29/v28 고정 기록

2026-10-15 추가 진전: [실제 YouTube 구간 저장](public-cut-review-20261015.md)은 원 엔진 완료 기록과 기존 파일3.008초/144p/decode0를 함께 검증했다. 원 fixture 경로 실패는 보존했다. History/Help 후속5파일은 Live에 통합했고 [새 whole29](ui-review-v29-20261015.md)가1,613 PASS/0 FAIL로 통과했다. 실제 화면/패키지 검수 진행 중이며 사용자 검수 ZIP은28이다.

현재 사용 가능한 검수 후보는 **v28/source674**입니다. Windows 전체 fmt·strict workspace/all-targets clippy·test·release가 실제 0이며 **1,607 PASS / 0 FAIL / 42 ignored / 43 harness**를 Root가 원로그·소스·3실행 파일·lock과 대조했습니다. 정보 아이콘·라벨 정렬·진행률·SI 속도·도구 상태·목록 검색을 포함합니다. [전체 검사](evidence/windows-msvc-ui-feedback-v28-whole-20261015/root-review.json), [현재 화면·한계](ui-review-v28-20261015.md).

v28 설치·삭제·동일 경로 재설치·새 설치기로 검증·설치된 CPU 앱 실행을 private desktop에서 확인했습니다. ZIP 전체 CRC/SHA·기존/새 journal 각5개·payload5개·원33pins·native image/Job/자연 종료0과 화면3장을 Root가 대조했습니다. [재설치](evidence/windows-msvc-ui-feedback-v28-install-reinstall-launch-private-v1-20261015/root-review.json), [직접 화면 검토](evidence/windows-msvc-ui-feedback-v28-install-reinstall-launch-private-v1-20261015/root-pixel-review.json).

v28은 검색 결과 한 번 공개·History 단일 본문 scroll·Escape 설명 억제·도구 경로 focus 문맥 개선을 포함합니다. 실제 Search7PNG/Help8PNG의 scoped 검토를 마쳤으나 긴 History 상태 문장·tooltip 입력 가림·Help UIA 간헐 실패는 남았습니다. YouTube metadata는 인증 없이 준비됐고 치지직은 ToolFailed/보조Exit1이며 원인은 미확정입니다. [메타데이터 검토](evidence/p0-real-site-metadata-v3-20261015/root-review.json). 성능 strict 실패, 실제 YouTube/치지직의 최종 다운로드·구간/중단 인수, 물리 IME·reader·DPI·sleep/resume·OneDrive·clean OS, 공급 notice·서명·hosted CI 등은 미완료입니다. S10 P0 로컬 진단은 최초 릴리스 범위에 포함합니다. Linux/macOS 인수는 사용자 지시로 후속입니다.

v25 frame-phase 진단은 이미 실행됐습니다. 평균 CPU0.598319%와 phase 수치를 확인했으나 strict CPU w6 및 memory w5/w6 실패를 보존했고 원인/기본 빌드 예산 해결은 아직 증명하지 못했습니다. [진단](evidence/current-main-thread-frame-phase-v25-v2-20261014/root-review.json). 8시간 합성 시험과 예전 긴 경로 시험도 그 고정 snapshot의 근거이며 현재 실제 사이트·GUI 인수로 대체하지 않습니다.

## 이전 v25 이하 고정 기록

당시 v25/source671 기록이다. 삭제후동일위치재설치의journal선택을수정하고Windows 전체4검사1,542 PASS/0 FAIL/42ignored·43harness를Root가독립대조했다. private설치→제거→같은위치새journal재설치→cold재검증과기존/새journal5·payload5·두installer자연0/Job0·원33pins·직접3PNG를확인했다. CPU/CLI는v24와바이트동일하고설치기만6,656B증가했다. [전체검사](evidence/windows-msvc-ui-feedback-v25-whole-20261014/root-review.json), [재설치](evidence/windows-msvc-ui-feedback-v25-reinstall-private-v3-20261014/root-review.json), [작업](../work-items/S11-reinstall-location-20261014.md).

전체P0/물리IME·reader/cleanOS/Git·hostedCI/updater는미완료다. 현재strictCPU3실패의main/present·worker원인분리후속을준비하며사용자검수대기만으로독립개발을멈추지않는다. journal후보32상한/여러기록비용/실제업그레이드는이번완료범위가아니다.

## 고정된 v24 및 이전 검증 기록


당시 v24/source670 기록이다. Windows fmt·strict clippy·test·release는 **1,538 PASS / 0 FAIL / 42 ignored**이며 Root가 원로그43 harness·현재 소스·실행 파일 세 개를 독립 대조했다. 선택 정보줄의 왼쪽 정렬과 파일명 우선 폭을 수정했다. 최신 선택24원화면·설치ZIP 두개·실제 정상/긴경로/변조거부 시험은 Root 독립 검토를 통과했으며 이전 v22/v23 증거를 현재 성공으로 소급하지 않는다. 최신 설치88원화면·로컬CI15단계 독립 검토를 완료했다. 최신 세평균CPU<1%·메모리18구간은 충족했으며 strict10초CPU3구간 실패를 보존한다. 사용자 수동 검수는 나중에 진행한다. [전체 검사](evidence/windows-msvc-ui-feedback-v24-whole-20261014/root-review.json), [최신 통합](../work-items/S00-windows-v24-integration-20261014.md), [잔여 작업](P0-remaining-20261011.md). 아래는 각 이전 snapshot의 검증 기록이다.

검증된 최신 배포 후보는 v18이다: 네actual명령1,327/0/33·43harness·source657·3실행 파일/lock, 목록 명령8/tool17회귀·63화면 팀검수·설치8화면·direct설치/실행outer0·변조거부·27ID·Dark200하단scroll을 확인했다. 키보드 도움말·진단 번호 오류 연결·SQL binding을 통합했다. 실제 Settings 선택 한글 확정 시험의 semantic FAIL을 보존하고 수정 중이다. 전체P0/물리UI/정식배포는 미완료다. [현재 검수표](UI-UX-AUDIT-CURRENT-20261013.md).

최종 v13 소스 검사1,297/0/30 및 source656·3artifact 대조, 최신 ZIP archive 대조를 완료했다. v13 완료 안내 전이·설치 실행/변조 거부·일반/긴 경로 독립27ID·앱63화면 직접 검수·로컬 CI15단계·7captures·8Cargo/export 독립 대조를 scoped 인수했다. v13 기본 비용은reported3평균 목표·메모리 통과/strict18 CPU1실패를 독립 확인했으며 footer 줄바꿈 뒤 배율 조작 일시 가림을 추적한다. 전체 P0·정식 릴리스는 미완료다. [현재 검수표](UI-UX-AUDIT-CURRENT-20261013.md), [현재 잔여 작업](P0-remaining-20261011.md).

## 아래는 각 snapshot의 검증 이력

현재 최신 compact 오류 수정본: Windows fmt/clippy/test/release1284/0/30 및root raw/source655/artifacts 독립 대조PASS. CPU0093b3b7…9,836,544B. Dark200800·Light100960 입력title/field/error 전체 공개·258UTF16 거절·clear·Off/Auto 및PNG 직접검토PASS, 원f15 잘림FAIL은보존했다. v10 ZIP과 실제일반·577/598UTF16 긴설치/각독립27ID/root검토PASS이다. [현재 검수 현황](UI-UX-AUDIT-CURRENT-20261013.md). 설치 후 앱 실행 수명candidate는개발중이며v10에미포함, 물리입력/reader/OSDPI/cleanOS/upgrade·rollback/서명·공지/최신비용·로컬CI/원격Git/updater 및전체P0는미완료이다. 아래는snapshot별이전기록이다.

현재2026-10-13 최신 입력 문맥·초기 관측·창 테마 통합: Windows whole-v2 fmt/clippy/test/release **1284/0/30**, raw43 harness·655항목 source map에 묶인 actual PASS다. CPU `f15a830dfcba928bdfff4494cbc834fab8ccaa4876a056a1fe19610b315e1a6b`/9,836,544B, installer `b803c103…`/4,350,464B와 CLI `bb5976d8…`/978,432B는 직전과 같다. dependency/Cargo.lock 변경은 없다. [최신 whole 원결과](evidence/windows-msvc-ui-label-context-startup-whole-v2-20261013/result.json). 원 whole-v1 footer 기대값 FAIL을 보존하며 v2 성공으로 덮어쓰지 않는다.

동일 CPU의 private titlebar 단일 앱 Light→Dark→System actual UIA 선택·창/HWND/PID/UI TID/title/icon/client geometry 보존·natural0/Job0 및5PNG를 확인했다. pre-UIA 두 PNG에 `수신·송신 관측 준비 중`이 보인다. Get20 성공값0/1/0, Get35/36 여섯 HRESULT는 모두−2147024809(E_INVALIDARG)이며 out값0은 의미 없는 값이다. `all_get_values_verified=false`, Set HRESULT/수동 캡션·텍스트 적용/System 기본색 복귀/물리 DWM 색상은 미검증이다. private OS chrome는 클래식 표시로 남았다. [원 Get·capture 결과](evidence/current-default-titlebar-theme-private-20261013/result.json). 이 scoped 결과를 전체 GUI/OS/사용자 인수로 확대하지 않는다.

현재f15 800Dark200 label3 native 시험은 실제FAIL이다. 라벨35+간격32+입력64+오류105=236px가body219px를넘어문맥전체공개가실패했으며원20 geometry표본/raw로그를보존한다. short-error copy 수정은nextbatch pending이고 source/whole1284 PASS는독립이다. [원 실패](evidence/current-network-label-input-error-dark200-800-v2-20261013/result.json). 다음 문구 후보는 “연결 이름의 길이와 문자를 확인하세요.”이며 현재동작/검증완료로표현하지않는다.

README의 v9 ZIP·단독 실행 파일은 f15/1284 snapshot이다. v9 패키지/실제 일반·긴 경로 설치/각 독립27ID는 PASS이며 최신 전체UI/UX·기본 비용/로컬CI·물리IME/reader/OSDPI·upgrade/rollback·cleanOS/서명/공지·원격Git/updater 및 P0는 미완료다. 직전 기본 strict18 CPU17PASS/1FAIL과 메모리18PASS, 이전 CPU 실패·gallery wrapperFAIL은 보존한다. 평균<1% 목표와 추가 구간 진단, 계측 빌드와 기본 성능을 구분하며 새f15 기본 비용 성공은 아직 주장하지 않는다. 아래 기록의 “현재·최신”은 각각 기록 당시 snapshot의 범위다.

현재2026-10-13 후속 통합: 파일 목록 선택 행 hover/press·네트워크 입력 근처 오류 공개·첫 관측 준비 안내를 구현했다. Windows whole fmt/clippy/test/release **1278/0/30**, root 원로그/655 source/artifact/lock 대조PASS다. 이 snapshot의 native 화면·v8 설치·비용/CI는 진행 중이다. 이전 source654/F42의 로컬 CI7+guardian 총8 Cargo 호출과 v7 긴 경로/독립27ID 및세checkbox settled blue는 actual PASS이며 새 source655에 소급하지 않는다. 이전F42 CPU strict18구간16PASS/2FAIL과memory18PASS를 유지한다. 평균 목표와추가strict진단을구별한다. 전체UI/UX/물리IME·reader/OSDPI/upgrade·rollback/배포/updater와전체P0는미완료다. [현재 whole](evidence/windows-msvc-ui-feedback-library-worker-whole-20261013/root-review.json), [CI 고정 이전 snapshot](evidence/current-ui-semantics-repaint-local-ci7-20261013/root-review.json). 아래는 snapshot별 이전 기록이다.

현재2026-10-13 후속 통합: 속도 단위 목적·구간별 오류 연결, 네트워크 문법 오류 설명과 CPU repaint 세대 선택을 구현했다. Windows whole-v3 fmt/clippy/test/release **1272/0/30**, root 원로그·현재소스·artifact 검토PASS. 최신v7 실제5파일설치·독립27ID·54화면/8드롭다운 선택복원PASS. Light100 속도단위/구간 입력·수정·복귀 actual PASS_SCOPED이며 reader 속성은 공급자 미지원으로 미검증이다. 합성 pointer9상태는 OS/물리 마우스 인수가 아니다. 설치 체크 색상 차이와 최신 기본18구간 비용·로컬CI·전체UI/UX·upgrade/rollback·배포 준비는 진행 중이다. 직전v6 긴 경로 증거와 기존CPU2FAIL을 보존한다. [최신 전체 검사](evidence/windows-msvc-ui-semantics-repaint-whole-v3-20261013/root-review.json), [최신 설치](evidence/windows-msvc-ui-cross-surface-v7-20261013-popup-audit-v4/root-review.json). 아래는 고정 snapshot별 이전 기록이다.

현재2026-10-13: 공통UI·설치수정whole-v8 Windows1253/0/30·fmt/clippy/release/root검토PASS, 최신v6ZIP의577/598UTF16 실제5파일설치와독립27canonical/원20object·6parent ID대조PASS. 앱5화면·설치54capture국소검토와전체UI/UX인수를구별한다. 기본WMpointer pressed/설치popup collector실패를보존하고재검증하며, 진단6구간CPU/메모리PASS는이전기본18구간CPU2FAIL을대체하지않는다. 최신localCI·upgrade/rollback·물리IME/reader/OSDPI·cleanOS/서명/공지·원격Git/updater와전체P0는미완료다. [현재잔여](P0-remaining-20261011.md), [최신통합](evidence/windows-msvc-ui-cross-surface-whole-v8-20261013/root-review.json), [최신설치검토](evidence/windows-msvc-ui-cross-surface-v6-20261013-long-install/root-independent-review.json). 아래는각이전snapshot기록이다.

## 이전 검증본 — 2026-10-13

현재 사용자 평가: v3 디자인 미인수. 앱·설치 UI 통일을 구현 중이며 직전 비용 시험의 CPU 2/18 실패는 보존한다. 기존 1,235 테스트 및 GUI 결과는 당시 snapshot의 기능 증거이며 새 소스의 통합/시각 검증은 별도다. [전수 작업](../work-items/S00-ui-cross-surface-audit-20261013.md).

최신whole-v2는1,235/0/30 및fmt/clippy/release, root 원로그·652source·artifacts/GNU원본 대조PASS다. [원검토](evidence/windows-msvc-ui-focus-plan-whole-v2-20261013/root-review.json). core/옵션 단위 성공을 전체OS/UI인수로확대하지않는다.

속도단위/preset/Resetfocus, pure미리보기, 네트워크compact/focus/Manual실패안내의 국소work-item을완료했다. 최종CPUactualrate3phase21state/15plan 및 network2scope가PASS이며root대표PNG를직접확인했다. 새testexe/설치ZIP은README에연결했다. 일반설치5파일·재검증/현재27ID와긴경로577UTF16의원20ID/6parent 보존scope를확인했다. 긴설치canonicalID독립대조는준비중이다.

전체UI/UX·physicalIME/reader/OSDPI·cleanOS/릴리스·upgrade/rollback·최신비용/로컬CI·원격Git/hosted/updater는미완료다. invalid.git와원실패/이전ZIP은보존했다. [진행task표](P0-remaining-20261011.md).

아래는 고정 snapshot별 과거 이력이다.

## 최신 완료 범위 — 2026-10-12


추가 진행: 좁은 상태줄 간격과 품질 선택의 숨은 포커스를 실제 Candidate 회귀로 재현·수정했습니다. 두 변경의 새 전체 검사와 native 화면 검증은 진행 중입니다. 기존 설치 ZIP은 이 추가 변경을 아직 포함하지 않습니다.


진행 중: 자막 선택·체크박스의 공통 컴포넌트 적용을 소스에 통합했습니다. 이 후속 수정과 실제 자막 조작 회귀를 포함한 Windows fmt·clippy·test·release 및 독립 대조를 완료했습니다(1147/0/30). Light/Dark200의960폭 격리 자막 조작·재시작·독립 raw/DB 검토와 대표14PNG 직접 검수를 완료했습니다.800폭 및 전체 UI/UX 인수는 이어서 진행합니다. 현재 설치 ZIP과 아래1145 기록은 수정 전 snapshot입니다. [작업 기록](../work-items/S00-subtitle-control-consistency-20261012.md).

현재 Windows CPU/MSVC 전체 fmt·clippy·test·release 검사는 **1145/0/30**이다. [현재 whole 검토](evidence/windows-msvc-bootstrap-closewait-clarity-whole-20261012/root-review.json)와 동일 snapshot의 [설치·재검증](evidence/windows-msvc-bootstrap-closewait-final-20261012-install/root-independent-installed-review.json), 현재27 native ID 대조를 확인했다. README 설치 ZIP에는 빈 파일 목록 안내, 설치 화면 초기 크기·가독성, 종료 대기 안내 개선이 포함된다.

ordinary149 및 Dark200 focused11은 각각 고정한 이전 snapshot의 제한된 증거다. Library35 캡처의 도달 geometry·provider 검토, 현재 일반 release의 Metadata 실패 보고서 단계·버전·종료 코드와 별도 DEBUG CloseWait 해제/타임아웃 검토를 추가했다. 기능·provider 검증은 전체 픽셀·물리 입력·사용자 UI/UX 인수가 아니다. [UI 우선 검토](../work-items/S00-ui-current-priority-review-20261012.md)를 진행 중이며 전체 디자인 일관성과 상호작용 인수는 남아 있다. 전체 P0·clean OS·서명·remote CI·updater·정식 릴리스는 미완료다. [현재 잔여](P0-remaining-20261011.md).

## 역사 snapshot — 아래 “최신·현재·진행 중”은 기록 당시 범위

2026-10-12 UI 추가 통합: 배율 선택 후 문맥 유지, 상태 지표 단위 줄바꿈, 파일 목록 전체 페이지 스크롤을 통합했다. Windows 실제 전체4검사 **1,111 passed/0 failed/28 ignored** 및 current source/artifact를 독립 확인했다. 교정된 동일 회귀 시험에서 기존 footer/library는 RED, 수정본은 GREEN이다. 실제 GUI 캡처는 진행 중이며 최신 설치 ZIP은 아직 이전 History 복구 후보다. 합성128파일 GUI 관측기는 C# 호출 인자 오류로 앱 실행 전 실패하여 보완 중이다. 최신 UI/UX·설치본·CI·비용·8시간 최종 인수는 미완료다. [현재 범위](P0-remaining-20261011.md).

2026-10-12 최신: History 시작 복구 오류에서 읽기 조회를 허용하고 Resume 실행 gate는 유지했다. 좁은 폴더 설정·도구 그룹 및 활성 도구 표시를 통합했다. Windows 전체 fmt/clippy/test/release의 **1,101 passed/0 failed/28 ignored**를 원본 로그·해시와 독립 검토했다. 최신 전체 화면 69 캡처의 geometry/semantic inventory와 실제 History 빈/오류 화면을 제한된 범위로 인수했다. 최신 설치 ZIP의 실제 5파일 설치·재검증, 현재 27레코드 native ID 대조와 installer PNG 5개를 확인하고 README 링크를 갱신했다. UI/UX 최종 인수·실제 저장 종료 대기 안내·최신 CI/비용 검증·8시간 terminal은 남아 있다. [현재 UI 작업](../work-items/S00-current-five-screen-ui-polish-20261011.md), [현재 잔여 작업](P0-remaining-20261011.md). 전체 P0·정식 릴리스 완료는 아니다. 아래는 당시 snapshot별 이력이다.

2026-10-11 최신 소스: Windows whole-fifth **1,089 passed / 0 failed / 28 ignored**, 43 harness, source623의 fmt/clippy/test/release와 원본 로그·해시를 root가 검토했다. 최신 설치 후보의 실제5파일 설치·재검증 및 파일·디렉터리27개의 현재 native ID 조회를 확인했다. 도구 파일 선택의 실제GUI5단계·DB5·모달10 인수를 완료하고 최신 설치 링크를 갱신했다. 최종소스 CI7 검증은 이어서 진행한다. [현재 범위와 남은 작업](P0-remaining-20261011.md), [도구 선택 기록](../work-items/S04-native-tool-executable-picker-20261011.md). 실제8시간 전송은 진행중이며 전체P0·정식릴리스·hosted Actions·updater완료가 아니다. 아래 수치는 각각 당시snapshot의 이력이다.

2026-10-11 현재: Windows whole **1013/0/26**, 42 harness, 618 source를 root가 원본 로그·해시와 대조했다. 고급 숫자 입력 6사례와 도움말·readonly 실행 계획 ScrollItemPattern/실제 glyph 인수, 새로운 5파일 설치·재검증, idle18구간(최대 CPU0.627% / WS68.84MiB)을 통과했다. [설치 ZIP](evidence/dlpad-install-plan-scroll-20261011.zip), [옵션 UI 기록](../work-items/S00-windows-option-help-plan-20261011.md), [잔여 작업](P0-remaining-20261011.md). 물리 IME/reader/OS환경·최종 사용자 디자인 인수·원격CI·서명/cleanOS·자동 updater와 전체 P0는 완료로 주장하지 않는다. 아래 기록은 당시 후보의 결과다.

2026-10-08 최신 네트워크 입력 수직 교정: 실제MetricsPanel glyph/높이/12px/중앙회귀RED→GREEN, disabled/64char/invalidIDfocus회귀와Windows전체957/0/17·610source PASS. privateGUI 첫/재실행 click/Home/End/Backspace/Tab/readback/유효·무효·빈값적용·Light/Dark·앱150/200/자연0/Job0 PASS_SCOPED. [새 후보 ZIP](evidence/windows-msvc-network-input-graph-20261008/dlpad-user-preview.zip), [실제 수행 범위와 미검증 입력](../development/input-coverage-audit.md), [교정 작업](../work-items/S00-network-observation-input.md). 진단단일행3곳교정/전수입력/물리입력/Windows망측정/관측선택영속성은완료하지않았다. 전체P0·사용자UI/정식설치/서명/cleanOS/공지완결과기존미디어후속은여전히남는다. 이전953의원공개영상/비용근거를새957에서재실행한것으로소급하지않는다. 아래는이전snapshot이다.

2026-10-08 이전 공개영상 scoped 검증: 입력/품질 popup/재생 action 논리 ID·실제 URL selector/프레임 상태를 수정했다. Windows whole953/0/17(41 harness)·source610와 별도 실제 공개망 engine1/0, 사용자 지정 YouTube·치지직 private GUI의 준비/짧은 미리보기/재생·정지/시간·옵션·탐색/자연 종료 PASS. 최종 layout63/RawInput19, 실제 native 폴더 선택·library 재실행/DB integrity, idle 비용18창 최대CPU0.786260%/WS69.55MiB PASS. [개발 ZIP](evidence/windows-msvc-public-ui-final-graph-20261008/dlpad-user-preview.zip) 17,244,580B·466members·SHA256 `e639daf6bd387bb463082a4d5cd2162cea9949854f1b89be0de787aa8d128745`: root/독립 검토, selected notices258/missing0, whole-lockmissing43, legal/release_ready=false. [현재 범위·실패 이력·후속](../work-items/S00-public-video-ui-walkthrough.md), [사용 안내](../development/windows-user-preview.md). 전체 사이트/전체 영상 인수·긴 영상 정밀 제어·200% 정보 밀도·library 미디어 연결/로컬export·preview managed lease/cache·사용자 UI 승인·물리IME/reader/OSDPI·정식설치/서명/cleanOS/전체공지와 **전체P0는 미완료**다. Linux/macOS는 후속이다. 아래는 이전 snapshot이다.

2026-10-08 이전 UI 정렬 snapshot: 앱3도구 실제 설치·선택/Use·재실행복원과 개발설치화면 Deno선택설치 검증PASS. UI정렬/밀도수정 후 whole934/0/16·610source, 새layout63PNG/RawInput19PNG/실제URLsyntheticFFmpeg/자연종료 및 비용18구간PASS(CPU최대0.78527%/WS70.45MiB). [새 개발ZIP](evidence/windows-msvc-ui-polish-graph-final-20261008/dlpad-user-preview.zip), [도구 인수](../work-items/S03-windows-in-app-tool-install.md), [UI 인수와남은비평](../work-items/S00-ui-visual-consistency.md). ZIP17,228,708B SHA384c9108656d1cc1397563b554c6d4310f4fd309e2eb32fa451263fda86eaf80. 최종사용자UI·실제사이트·물리IME/reader/OSDPI·longvideo정밀/로컬export·정식설치/서명/cleanOS·전체공지완결·전체P0는미완료, Linux/macOS는후속. 아래는이전snapshot이다.

2026-10-07 미디어 중심 UI 통합 검증: Windows whole906/0/16와 새 비용18창 모두 통과(CPU최대0.93667%/WS69.97MiB). 현재binary의 private URL/realFFmpeg 재생, 합성extractor·Range2·원본보존·자연종료를 검토했다. UI 동일 source의63PNG/RawInput19PNG와608 source 대조 근거를 보존했다. [최신 개발 ZIP](evidence/windows-msvc-media-workspace-graph-event-idle-20261007/dlpad-user-preview.zip), [현재 작업과 한계](../work-items/S00-media-workspace-interaction.md). 긴 영상 정밀 선택·200% footer·library연결·physical/cleanOS·정식installer·법적공지완비·최종사용자UI승인과 전체P0는 미완료다. Linux/macOS는 후속이다. 아래는 이전snapshot 이력이다.

2026-10-07 최신 두 번째 UI 후보: 실제 gtmux toolbar/input/settings 소스와 캡처를 대조해 내부 중복 앱 이름·브랜드바·고정 sidebar를 제거하고 기능 탐색·저장 폴더 행·설정 정렬을 수정했다. 다운로드/파일 목록에 Windows native 폴더 선택을 연결했다. 최종 Windows fmt/Clippy/test/release878/0/13(source602), private layout63PNG·native6사례12process의 선택/취소/ownerclose/빈 경로/새 process 복원과18개 비용 구간을 검증했다. 최대CPU0.623811%·WS66.59MiB. [새 UI 테스트 ZIP](../../docs/validation/evidence/windows-msvc-folder-ui-close-safe-graph-20261007/dlpad-user-preview.zip), [실행 안내](../../docs/development/windows-user-preview.md), [범위·근거](../../docs/work-items/S00-native-folder-picker-layout.md). 최종 사용자 UI/UX·물리IME/reader/OS DPI·정식 설치 및 전체P0는 미완료다. 아래는 이전 snapshot 이력이다.

2026-10-07 현재 UI 후보: [사용자 피드백 재작업](../work-items/S00-windows-user-ui-rework.md)으로 gtmux 테마·탐색·입력 위계·아이콘을 구현했다. Windows whole852/0/13·599 exactsource, private GUI51PNG/두 원 process natural0·Light→Dark→restartDark/앱 배율150·200/합성 한글, 같은 새 CPU18비용창(<1%/80MiB)을 root와 독립 검토했다. CPU53a211…9,040,896B/CLIbbcf66…974,848B/installer999342…1,103,360B이며 Cargo.lock은 그대로다. [새 UI 테스트 ZIP](evidence/windows-msvc-ui-rework-graph-20261007/dlpad-user-preview.zip)은13,677,880B·SHA882b1b7628c47e1397870773b85901724e34a7494e9b07452234d0b437c8ca82다. [남은 빈 상태/포커스 UX](../work-items/S00-windows-empty-state-focus-polish.md), 사용자 UI 승인, 실제 물리 IME/reader/OS DPI·taskbarpixels·실환경/정식installer/법적공지 완비·전체P0는 미완료다. Linux/macOS 출시 인수는 Windows 우선 지시에 따라 후속이다. 아래는 이전 snapshot 기록이다.

2026-10-06 현재: [기존 설치 재검증](../work-items/S07-windows-native-existing-bundle-reverify.md)과 [세 binary 개발 ZIP](../work-items/S07-windows-native-installer-companion-package.md)을 scoped 완료했다. Windows 전체4검사846/0/13·실제GUI12case/24 collector 명령·root추가native80명령/PNG18장, CPU3×60/18구간 최대0.936280887%/WS71,630,848B·PNG3장·원합성30k/DB3 보존을 root/독립 검토했다. CPU80b657…8,710,656B·CLIbbcf66…974,848B·installerae4b1d…799,744B/lock불변. 실제 currentgraph258와ZIP2/13,043,842B/466members/SHA45c4d500… 및 notice455 원문도 root/독립 인수했다. selectednotice258/missing0이지만 wholelockmissing43·CRT/std/linked/legal/서명/clean OS/정식installer는 미완료다. 다음 개발 범위는 native manifest 선택·기본경로·설치/upgrade/복구이며 physical IME/reader·실제 장기/절전 인수 및 전체P0도 남는다. 아래는 각 이전 snapshot 기록이다.

2026-10-06 최종 파일 선택 통합: Windows fmt/Clippy/test/release 모두 exit0, 850 passed/0 failed/13 ignored, source593 전체 map 일치. 설치 명세 선택·취소·owner 종료와 재시작 확인취소·제거의 실제 private GUI 5사례/10수집 실행 및 root 추가 native42명령을 검증했다. root는 PNG7장을 직접 확인했다. CPU/CLI는 이전 인수본과 바이트 동일하며 installer는817,152B로 이전799,744B보다17,408B 증가했다. 전체P0·물리IME/reader·정식 설치/upgrade/cleanOS는 미완료다. [사용자 시험 안내](../development/windows-user-preview.md), [파일 선택 완료 범위](../work-items/S07-windows-native-manifest-picker.md).

2026-10-06 최신: 개발 설치 화면의 명시 제거를 scoped완료했다. Windows 전체4검사844/0/13(source592)·실제GUI11case22실행·root추가native73명령/직접PNG18장에서 확인취소/정상제거/foreign·경로수정거절/원worker취소·CloseACK/기존설치회귀와원자료ID·SHA보존을root/독립검토했다. CPU0bf43b…8,710,656B·CLId0e4ae…974,848B·installer2e91bb…798,720B(+38.5KiB), lock불변. 최신CPU비용은새3×60/18구간에서측정중이며이전비용을소급하지않는다. 새창에서기존bundle의검증·제거를잇는typed2root후속은/tmp에서병렬준비한다. 정식installer/default·browse·upgrade·cleanOS/physical·전체P0는남는다. [완료범위](../work-items/S07-windows-native-development-removal-ui.md), [다음작업](../work-items/S07-windows-native-existing-bundle-reverify.md). 아래는이전snapshot기록이다.

2026-10-06 최신: 명시 개발 bundle 제거 엔진·CLI16 범위 인수를 완료했다. Windows 전체4검사841/0/13 및 실제CLI36행/45명령·추가native20명령에서 실제7파일/4root 제거·outer부재와 원source/journal ID·SHA 보존, foreign/torn/mixed 거절을 root/독립 검토했다. 제거 확인·취소·완료 표시 UI를 병렬 준비 후 통합한다. 현재CPU11bb9b…/CLI65e0fe…/installer65c80d…이며 이전a7e CPU 비용 측정을 최신binary에 소급하지 않는다. 정식installer·upgrade·cleanOS/physical·전체P0는 남는다. [제거 CLI](../work-items/S07-windows-explicit-user-bundle-removal-cli.md), [현재 UI](../work-items/S07-windows-native-development-removal-ui.md). 아래는 이전 snapshot 기록이다.

2026-10-06 최신: 별도 Rust native 개발 설치 화면을 scoped완료했다. Windows전체4검사832/0/13·실제 격리GUI5case/10명령·추가native24명령/PNG8에서 설치·재검증·기존폴더거절·복사취소·CloseWait·wrongSHA 및원자료를root/독립검토했다. installer4c2542…759,296B·CPUa7e60e…8,710,656B·CLI7e695e…934,400B, lock/GNU보존. 현재CPU18budget은 최대CPU0.630450%/WS71,680,000B·3PNG/합성30k/3DB 보존을 root/독립 검토했다. 명시bundle제거 원cap 이전·실패 보존·부분 재시도 엔진/회귀를 통합하고 Windows 전체 검증 중이다. System표준폼의개발범위이며 browse/defaults/packaging/upgrade/physical·cleanOS/정식installer·전체P0는남는다. [완료 범위](../work-items/S07-windows-native-development-bootstrap.md), [현재 작업](../work-items/S07-windows-explicit-user-bundle-removal.md). 아래는이전snapshot기록이다.

2026-10-06 최신: journal-aware 개발 CLI15를 완료했다. Windows전체4검사827/0/13·실제15명령/성공3/예상실패12+native7명령, 실제 복사 취소/30초기한3명령/2case+native9명령을 root/독립 검토했다. CPU5bcc9f…8,710,656B·CLIc4cbc7…935,424B, lock/기존GNU 보존. 최신 앱 GUI 비용은 측정 중이며 별도 native bootstrap 화면의 service/UI/observer 초안은 /tmp에서 병렬 구현한다. 정식 installer·upgrade·cleanOS/physical와 전체P0는 미완료다. [완료 범위](../work-items/S07-windows-journal-aware-bundle-cli.md), [현재 작업](../work-items/S07-windows-native-development-bootstrap.md). 아래는 이전snapshot기록이다.

2026-10-06 최신: 실제 검증 단계마다 기록하는 journal-aware coordinator API를 Windows 전체4검사826/0/13 및 실제 public factory·root native7명령/19관측·독립 검토로 인수했다. 실제4stage SHA/chain/manifest와 원 nested ID·자료 보존을 확인했다. outer ID는 현재 post-Drop 관측이며 crash/history 권한은 미증명이다. 이제 15번째 개발 CLI 연결을 /tmp 구현·인수 검토와 병렬 진행한다. 정식 installer/최신 GUI 비용/physical·cleanOS와 전체P0는 미완료다. [완료 범위](../work-items/S07-windows-journal-aware-fresh-bundle.md), [현재 작업](../work-items/S07-windows-journal-aware-bundle-cli.md). 아래는 이전 snapshot 기록이다.

2026-10-06 진행: 설치 journal API와 bounded native fixture는 Windows 전체 4검사(818통과/0실패/13무시), 실제 fixture 및 root 추가 native 9명령·독립 검토로 현재 일관성/원 ID·SHA 보존 범위를 인수했다. 14번째 `inspect-install-journal` CLI도 실제12명령·원24파일·native32ID 검토로 완료했다. 현재 explicit process exit 보존 시험을 통합·검증하고, journal-aware coordinator 계약을 /tmp에서 병렬 조사 중이다. 이전 성능 18구간 PASS는 당시 고정 앱의 결과이며 최신 CLI와 전체 P0 인수로 소급하지 않는다. 정식 installer·crash recovery·physical/clean OS 게이트는 남는다. [journal 작업](../work-items/S07-windows-install-journal-next-slice.md), [CLI 작업](../work-items/S07-windows-install-journal-readonly-cli.md). 아래는 이전 snapshot 기록이다.

2026-10-06 현재: 명시묶음readonly재검사/13번째CLI를완료했다. 새Windows전체4검사813/0/13·실제17명령/7행·원11nativeID와canonical/digest/7files보존·typed4/namespace negative2를root/독립인수했다. CPU75d05…8,710,656B·CLI0d1b24…847,872B(+6,144B), GNU/lock보존. 같은최종CPU의18구간18PASS(최대CPU0.935892%, WS72,212,480B)와합성30k/DB3보존도완료했다. 최초개행collectorFAIL과이전CPUbudgetFAIL은보존한다. 앞서worker/지표복귀·숨김복귀/목록·scan CloseWait release/timeout도각binary범위완료했다. 현재설치journal 공유API를통합하고Windows전체검사·nativefixture초안·독립검토를병렬진행중이다. [재검사완료](../work-items/S07-windows-user-bundle-readonly-reopen.md), [다음journal](../work-items/S07-windows-install-journal-next-slice.md). 전체P0·최종스택·정식installer/physical/cleanOS게이트는미완료다. 아래는이전snapshot기록이다.

2026-10-06 현재: 새 개발 설치 묶음·12번째 CLI의 source579 Windows 전체4검사(774/0/13)와 실제 생성/cold검사/격리GUI3case·14명령/11행·NativeID10, 별도 복사중취소/기한3명령/2case를 root/독립 인수했다. CPUa6d9223a…8,709,632B·CLI97217440…841,728B, GNU/lock 보존. 이후 GUI wake 계측4파일을 통합해 기본/별도진단 빌드를 검증 중이며 같은 source579 항목수도 다른 정확map이다. 이전 CPU1.247473% 실패·성능 원인 조사·정식 journal/upgrade/UI설치·physical/cleanOS 게이트와 전체P0 미완료를 유지한다. 아래는 각 이전 snapshot 기록이다.

2026-10-06 현재: [프레임 비용 계측](../work-items/S00-windows-private-frame-phase-diagnostic.md)을 통합한 source578의 기본 Windows 전체4검사·760passed/13ignored가 root/독립 PASS다. CPU16b4d0e…8,709,632B·CLIa0bb2ab8…795,136B이며 원GNU/lock은 보존했다. 실제 observer C# 컴파일은 통과했고 별도 defaultOFF 계측 feature 전체빌드를 진행 중이다. 이전 source577의 CPU1.247473% 실패는 보존하며 새 바이너리 성능 인수·원인 해결·전체P0 완료는 아직 미완료다. 아래는 이전 snapshot 기록이다.

2026-10-06 현재: 원 Job readonly 관측과 부모 종료 후 자식 수명 fixture의 source577 전체 MSVC4검사·760passed/13ignored가 완료됐다. CPU594676ba…·CLIa0bb2ab8…의 실제 자식 대기/취소/기한 인수도 root/독립 PASS다(12명령/3case·nativeID30·원파일21). 현재 source577의 loaded GUI 비용은 18구간 중 한 구간 CPU1.247473%로 FAIL이며 모든 메모리 구간과 자연 종료·격리 화면 복원은 통과했다. 같은 구간 main78.125ms/library46.875ms를 관측했고 원인은 미확정이다. [프레임 단계 계측](../work-items/S00-windows-private-frame-phase-diagnostic.md)의 Rust 구현·관측 수집기·독립 증거 검토를 병렬 진행한다. 이전 GUI·ZIP·비용은 각 고정 binary의 기록이며 전체 P0는 미완료다.

2026-10-06 최신: 시작 검사 기한과 실행 수명을 분리한 `run-selected`를 구현했다. 새 MSVC 전체 fmt/clippy/test/release와 760passed/13ignored·설치89 회귀는 root/독립 검토 PASS다. CPUc9f09dc0…·CLIab8a6cde…의 실제 격리 GUI3개와 추가 CLI·원nativeID10 보존(17명령/14사례)을 root/독립 검토했다. 기존 bounded 실행의 새 바이너리 회귀도 실제15명령/13사례·GUI4까지 root/독립 PASS다. 현재 부모 종료 후 자식 수명 fixture와 원 Job readonly 관측을 새 source에 통합해 검증 중이며 아래760/GUI 인수는 직전 snapshot의 기록이다. 이전 GUI/ZIP/비용 결과는 이전 binary의 기록이다. [현재 작업](../work-items/S07-windows-selected-supervisor-lifetime.md). 전체 P0는 미완료다.

2026-10-06 현재: MSVC 전체 fmt/clippy/test/release·758passed/13ignored, [설치 선택본의 격리 GUI·기본 CLI 실행/재시작/취소/기한](../work-items/S00-windows-msvc-selected-private-gui.md), [대상 graph258의 notice와 동일 개발 ZIP 두개](../work-items/S07-windows-msvc-development-package.md)를 root/독립 검토로 완료했다. CPU121f02cd…/8,709,632B·CLI a3001bd2…/786,944B·source576·GNU 원 앱/CLI·lock 보존. [새 MSVC GUI 비용](../work-items/S00-windows-msvc-current-gui-cost.md)은 18window 중 CPU1.09% 한 구간의 FAIL을 보존하고 원인을 조사 중이며 전체P0는 미완료다. 정식 bootstrap·upgrade/rollback, cleanOS/최소OS/서명, physical IME·reader·clipboard도 남는다. 아래는 이전 snapshot 기록이다.

2026-10-06 최신: [명시 선택 개발 CLI 실행](../work-items/S07-windows-explicit-selected-launch-cli.md) 범위 완료. source576·758passed/13ignored·Windows 전체4검사 및 실제 격리GUI4case/추가CLI9/native3query를 주 담당·독립 담당이 확인했다. CPU54c33d95… 불변, CLI5ae864b8…/754,688B. 원파일34/ID10·설정Light 재시작 보존. 현재 MSVC 프로젝트 도구 공급을 이어서 진행한다. 정식 bootstrap/upgrade·cleanOS/서명·physical 게이트와 전체P0는 남는다. 아래는 이전 snapshot 기록이다.

2026-10-06 최신 GUI 인수: [명시 선택 설치본 실행·재시작](../work-items/S07-windows-selected-copy-private-launch.md) scoped DONE. source576/CPU54c33d95…의 실제 두 private GUI·시스템→밝게 선택·원세경로 재시작 밝게 표시·readonly SQLite integrity/theme·원파일/nativeID10·자연Job0/input불변을 주 담당/독립 검토했다. 원 parent Job 단조회 FAIL은 보존했고 수정 재시험은 강제 정리 없이 PASS다. [MSVC 프로젝트 전용 공급](../work-items/S07-windows-msvc-user-tool-supply.md)과 정식 bootstrap/upgrade·cleanOS/서명·physical 게이트는 남는다. 전체 P0는 미완료다. 아래는 이전 snapshot 기록이다.

2026-10-06 최신 통합: [명시 선택 CLI](../work-items/S07-windows-active-selection-local-cli.md)와 [준비 신호 취소·기한](../work-items/S07-windows-active-cli-control-fixture.md)은 범위 인수를 완료했다. 선택 앱의 원 소유 런처는 실제 정상 종료·실행 후 취소·기한 종료를 검증했다. 후속 GUI fixture의 Windows 전체 fmt/clippy/test/release 및 독립 검토는 현재 source576(항목 수)·755passed/13ignored·설치89회귀로 통과했다. CPU54c33d95…/7,917,568B·기본9명령 CLI f97300cc…/723,456B, lock 불변. [명시 선택 GUI 재시작](../work-items/S07-windows-selected-copy-private-launch.md)의 실제 격리 화면·DB 인수와 [MSVC 도구 공급](../work-items/S07-windows-msvc-user-tool-supply.md)을 이어서 진행한다. 이전 snapshot의 GUI 성능을 최신 binary에 소급하지 않는다. 전체 P0·정식 installer/upgrade·cleanOS/서명·physical 인수는 미완료다. 아래 문단은 이전 snapshot 기록이다.

2026-10-06 최신: [명시 설치 선택 엔진](../work-items/S07-windows-active-install-coordinator.md) scoped DONE. Windows전체4검사·설치87회귀·실제6case와별도process readonly canonicalSHA·정상원nativeID10·원두등록쌍/receipt/notice/sentinel보존을root·독립검토했다. CPU53d12740…/7,917,568B(+0B), lock불변. [CLI두명령연결](../work-items/S07-windows-active-selection-local-cli.md)은통합후전체검증중이다. 정식installer/active실행·upgrade복구·cleanOS/MSVC/서명·physical인수와전체P0는미완료다. 아래는당시snapshot이력이다.

2026-10-06 최신: [현재 설치 후보 격리 GUI 인수](../work-items/S00-windows-current-install-artifact-ui.md) scoped DONE. 기본CPU f3bae801…/7,917,568B의3×60초18창 CPU<1%/WS<80MiB·원합성파일30000/3DB보존·Unicode47단계 및같은source 별도defaultOFF ownerIMM취소/확정이root/독립PASS다. 원 검증wrapperFAIL은계약오류로보존하고native실제PASS와구별했다. 물리IME/reader·기본binaryOScomposition·cleanOS/서명·전체P0/스택채택은미완료다. 다음 active선택/재시작readonly대조의공유계약·schema·원caps 검토를병렬진행한다. 아래문단은당시snapshot이력이다.

2026-10-06 최신: 검증된 GUI+일곱 명령 개발 CLI의 [동일 입력 ZIP 두개 인수](../work-items/S07-windows-cli-companion-package.md) scoped DONE. 실제 두 ZIP SHA1ed2a3ef…/13,364,856B·864entries 및 root/독립 전체member·PE·현재 whole-source/proof·내장font 대조 PASS. 전체lock notice 누락43과 release_ready=false를 유지한다. 현재 병렬 담당은 새CPU 격리 GUI 비용·IME 감사/collector·독립 패키지 검토로 분리했으며 root가 native 실행을 직렬 조정한다. P0 전체는 미완료다. 아래 문단은 당시 snapshot 이력이다.

2026-10-06 최신: 외부 receipt 개발 CLI 일곱 명령 인수 scoped DONE. Windows 전체4검사·설치75/parser4·실제28명령·현재 복사중취소/기한2case·별도원18nativeID와 root/독립검토 PASS. CPU f3bae801…/7,917,568B 불변, CLI efcef0b0…/651,776B(+74,752B), dependency/lock 변경 없음. source570은 항목 수로 이전 제거 snapshot과 CLI hash map이 다르다. 전체 P0는 미완료다. 다음 Windows현재배포물 notice/실제active설치·upgrade복구·cleanOS/MSVC/서명 및physical인수를 진행한다. 아래 이전문단은 당시 snapshot 이력이다.

[현재 CLI 인수](../work-items/S07-windows-registration-receipt-local-cli.md).

2026-10-06 최신: 명시적 등록 묶음 제거 engine source570 scoped DONE. Windows GNU 전체 fmt/clippy/test/release·설치75회귀·실제4묶음/7행·취소/기한/foreign accepted0과 같은 원caps 재시도·원 receipt4 nativeID 및 root/독립검토 PASS. CPU f3bae801…/7,917,568B(+0B), Cargo.lock 불변. 전체 P0는 미완료이며 현재 개발 CLI receipt 생성·대조·명시 제거를 분리 초안으로 병렬 구현한다. 이전 source 수는 snapshot 항목 수이며 동일 수의 실패 hash map을 완료 근거로 쓰지 않는다.

[현재 제거 인수](../work-items/S07-windows-registered-cold-removal.md).

2026-10-06 최신: source567 등록묶음+외부receipt readonly대조 scopedDONE. WindowsGNU전체4검사/설치68회귀/원source563묶음 별도process 원ID·SHA·canonical/5+1namespace·취소/기한4실패/root독립검토PASS. CPU3658c316…/7,917,568B·Cargo.lock불변. P0전체는미완료다. [완료scope](../work-items/S07-windows-registration-receipt-pair.md). 다음 명시cold제거·active설치/upgrade복구·공급/cleanOS/MSVC/서명·physical/remoteCI 인수를진행해야한다. 정상Git없음은보존하고공개배포하지않았다.

2026-10-06 최신: source563 외부fresh receipt scopedDONE(전체4검사/61회귀/실제5cases/canonical2305B/native2IDs/root독립). 현재 readonlyreceipt후속 통합검증중. 원receipt는삭제authority/active설치/역사소유권이아니다. [완료](../work-items/S07-windows-registration-receipt.md), [후속](../work-items/S07-windows-receipt-reopen.md).

2026-10-06 최신: 개발 등록 CLI scopedDONE. 현재 source560 전체4검사·설치53/parser3회귀·실제16명령·별도원nativeID·현재legacy취소/controlled expiry를root와독립검토했다. 병렬담당은독립증거검토/제거권한설계/안내초안으로분리했다. [작업](../work-items/S07-windows-registered-local-cli.md). 전체P0 및정식installer는미완료다.

2026-10-06 최신: source560 RegisteredInspection scopedDONE(Windows전체4검사/app_install53회귀/별도 prior등록CPU 원ID/SHA·typed실패4회/cursor37·allbytes보존). 현재 CLI prepare-register/inspect-registered 연결통합 중이다. cold삭제/active설치·cleanOS/MSVC/서명·공급/physical 인수는미완료. [readonly인수](../work-items/S07-windows-registered-reopen.md), [CLI후속](../work-items/S07-windows-registered-local-cli.md).

2026-10-06 최신: source558 fresh 등록 scoped DONE(Windows전체4검사/app_install49회귀/실제CPU4bundles/canonicalseal·종료뒤6개nativeID대조). CPU4888f534…7,917,568B·크기0증가. 현재 등록된묶음의 제품readonly재시작검사를 병렬초안에서통합한다. P0전체·cold삭제·서명/cleanOS·공급/physical인수는미완료다. [등록인수](../work-items/S07-windows-fresh-app-registration.md), [후속](../work-items/S07-windows-registered-reopen.md).

2026-10-06 최신: source555 fresh discard scoped DONE(전체4검사/40회귀/실제4bundles/기존prepare·CLI제어호환). 현재 개발용 fresh 설치 등록 record/seal 결합을 병렬 구현 중이며 cold remove/설치등록의 전체수명은 미완료다. [정리 인수](../work-items/S07-windows-fresh-prepared-discard.md), [다음 등록](../work-items/S07-windows-fresh-app-registration.md).

2026-10-06 최신 진행: source553 Windows 전체4검사와 실제 로컬 prepare/inspect CLI·원nativeID·windowless 취소/기한·unknown/partial 보존 검증 PASS. 현재 원 fresh PreparedBundle의 명시적 discard를 세 파일 담당으로 병렬 구현하고 주 담당이 Windows 통합 인수를 진행한다. P0 전체 미완료이며 아래 수치는 각각 당시 고정 snapshot 범위다. [완료 CLI](../work-items/S07-windows-local-install-cli.md), [진행 discard](../work-items/S07-windows-fresh-prepared-discard.md).

2026-10-06 후속 설치 기반: source548 Windows fmt/Clippy/test/release·신규15actual회귀와 실제CPU PE/SHA/nativeID/cursor 보존·한글freshroot/privateACL/sentinel 시험이 통과했다. 현재 CPU4f7a…/7,917,568B이며 기존 watch500 source543 GUI/개발ZIP과 구분한다. 다음 새 폴더 준비 트랜잭션은 세 하위 파일 담당으로 병렬 구현 중이다. signed installer/clean-machine/실제IME·reader·공급/legal/remoteCI 인수는 남으며 P0 전체 완료가 아니다. [기반 인수](../work-items/S07-windows-app-install-foundation.md), [다음 구현](../work-items/S07-windows-app-prepare-transaction.md).

2026-10-06 후속 통합: prepared 재열기 source552 최종 Windows fmt/Clippy/test/release 및 app_install 실제31회귀 PASS. CPU f274db5d…/7,917,568B(크기0B 증가), 최종 별도 프로세스에서 이전 source550 실제 copied exe의 ID/SHA/marker/정확 namespace 및 네 typed 실패·모든 bytes 보존 PASS. S07-windows-prepared-reopen scoped DONE. 다음 개발용 로컬 prepare/inspect CLI는 root Cargo console feature, 3개 분리 담당의 binary/fixture/보안검토 초안을 통합 중이다. CLI 아직 미컴파일·미인수이며 실제 installer 등록/정확 제거/공급·서명 게이트는 남는다.


2026-10-06 최신 통합: source550 Windows 전체 4검사와 app_install 실제 22 tests PASS. 검증된 새 복사와 같은 copied exe 두 private 시작/테마 DB 재시작 PASS(7,917,568B, SHA f1a9c1ad…); root 및 독립 evidence 완료. 설치 lifecycle·version upgrade·정확 제거·공급 신뢰 전체 완료는 아니다. 다음 S07-windows-prepared-reopen은 root 공유 API와 3개 disjoint 담당(ReadRoot/ReadFile, reopen coordinator, native fixture 초안)으로 병렬 구현 중이며 후속 소스는 아직 통합 검증 전이다.


2026-10-06 최신: scan watch500 제품 소스의 Windows GNU 전체 fmt/Clippy/test/release와 fixture 회귀가 통과했다. 실제 completion은 첫 INSERT 34.075초·commit 145.160초, Next 응답은 2.031초로 통과했다. 각 전후 DB 10,000개·전체 합성 payload/CAP/sentinel 보존과 자연 종료를 확인했다. Stop은 ACK 5초·commit 금지·전후 보존을 통과했다. 실제 watch-only GUI, startup missing-root 자동 회복 2.010초, current GUI CPU3회/18개 구간 strict<1%·80MiB도 통과했다. 같은 snapshot의 S10 대형 진단3case와 최신 font-package 결합, 테마3종 재시작, 200% 키보드47단계·Unicode 합성 한글 readback도 통과했다. 실제 IME/reader 인수는 남는다. 후속 설치 기반 소스는 별도 전체 검증 중이며 위 결과를 새 소스 전체 인수로 소급하지 않는다. [작업 계약](../work-items/S01-scan-watch-poll-budget.md). 전체 P0 완료는 아니다.

2026-10-06 독립 화면 인수: 기본 CPU b278ef…의 resize/expose(720×520 → 1100×800 → 960×640)는 실제 이미지 6개·전체 합성 10,000개 payload/DB·자연 Job0 검토를 통과했다. 같은 기본 앱의 active hide/secondary restore/Stop도 원 worker heartbeat 12→18·원 PID/생성 시각 유지·secondary exit 0·복귀 이미지·최종 단일 Job/Attempt Cancelled·자연 Job0를 확인했다. 숨김 전 DB 사본은 없으므로 전체 DB 전후 보존은 미검증이다. [크기 변경](evidence/windows-frame-resize-pixel-fixed-20261006-continue/root-pixel-review.json), [숨김·복귀](evidence/windows-current-cpu-background-prefill-20261006-continue/root-review.json). 스캔 완료 180초 실패는 아래 기록대로 남아 있고 원 스레드 CPU 진단을 이어간다. 전체 P0 완료가 아니다.

2026-10-06 최신 실제 결과: CPU 대기 순서 수정의 Windows 전체 4검증·fixture Clippy/앱 회귀는 통과했지만 실제 완료 시험은 첫 INSERT 71.047초 관측 후 180초 commit 실패다. 기본 실행파일 b278ef…/7,917,568B와 fixture 44012f…를 구분한다. 수정으로 완료 시간이 개선되었다고 주장하지 않는다. 실제 원 스캔 스레드 CPU 계측과 독립적인 resize/숨김·복귀 검증을 이어간다. [최신 작업](../work-items/S01-composed-periodic-budget-order.md), [실패 기록](evidence/windows-composed-budget-completion-20261006-continue/result.json). 전체 P0와 릴리스는 미완료다.

2026-10-06 현재 작업: CPU 대기가 끝난 뒤 2ms 작업 구간을 시작하도록 공유 예산과 앱 호출 순서를 수정했다. 전체 Windows 검증을 실행 중이며 새 소스의 완료 시간·CPU·화면 동작은 아직 인수 전이다. 직전 500ms 대기 후보는 GUI CPU 3회 및 30,000개 합성 payload/DB 보존을 통과했지만, 실제 스캔 완료 시험은 180초 제한에 실패했다. 해당 실패의 최초/최종 DB 10,000행 전체 일치·무결성·cap·sentinel 보존을 독립 확인했다. 아래 실행파일별 결과는 당시 소스의 이력이며 현재 수정에 소급하지 않는다. [현재 작업](../work-items/S01-composed-periodic-budget-order.md), [실패 DB 감사](evidence/windows-library-idle-wake-completion-20261006-continue/independent-failed-completion-db-review.json). 전체 P0는 미완료다.

2026-10-06 최신 후보: quiet500ms·기존cap1 StopWake·bounded 동일frame재사용을통합했다. Windowswhole4 및rootsource543/currentdefault3ca7dfdaa62a36170e0866b4a524e4b554ce91f817f7b70834f94a32f8a918c0/7917568B 대조PASS. GUI60초3회는avg0.364343/0.260065/0.389964%, max10초0.625598/0.623536/0.779698%, memory3PASS다. root18창原수식/원source·compiledpins/自然Job0·private/input을독립재대조했고원3만payload·DB 보존을이어간다. 현재fixture feature 검사와새소스 완료/Next/Stop·pixel/resize 인수전이며P0전체/스택/릴리스완료가아니다. quietwatch전달추가지연은최대500ms+OS로변경되며Stop은채널wake, scan중100ms/180초/30·1·2초내부gate는유지한다. [현재whole](evidence/windows-library-idle-wake-workspace-20261006-continue/root-review.json), [原GUI3회](evidence/windows-library-idle-wake-thread-window-gui-60sec-20261006-continue/gui/result.json), [root수식](evidence/windows-library-idle-wake-thread-window-gui-60sec-20261006-continue/root-metadata-review.json). 아래ac0/71ec소스는각당시snapshot이다.

2026-10-06 최신 소스: 동일 프레임의 software raster를 최대2MiB의 정확primitive/clip/scale/size와원surface age1에서만 재사용하는 후보를통합했다. 플랫폼·접근성·전체present·hidden/resize/texture/failedpresent 무효화를유지한다. Windowswhole fmt/clippy/test/release4명령과새순수3회귀 및root543source/原Job검증PASS, defaultCPU ac0ad5abb7f28c3da59fce391d3189afea409f485a678ec7c062a79718a612e6/7917056B(앞선71ec대비2048B증가). 현재새GUI60초3회가실행중이며CPU·pixel·80MiB 인수전이다. 직전2ms/0.4% 기능인수는실제완료123825ms/Next2039ms/StopACK·원10000행과payload보존PASS이고원45초Stop선행조건FAIL도보존했다. 후속75초관측에서는33.739초에신호를관측하여원45초실패원인은여전히미확정이다. [현재전체검증](evidence/windows-identical-frame-workspace-20261006-continue/root-review.json), [실험계약](../work-items/S00-bounded-identical-frame-raster-reuse.md). 아래71ec및이전후보는각당시소스이력이며최신GUI인수로합산하지않는다. P0/스택/릴리스완료가아니다.

2026-10-06 재개 현재: actual threadCPU0.4% 누적 예산과 wall2ms/500ms 후보의 Windows CPU 전체fmt/clippy/test/release4명령 및 fixture featureClippy/앱회귀가PASS다. 기본CPU SHA71ec7e489a1e0d7f0ae4301c179353a1ccdf4a0c37d63cd35e1c345121c16c94/source543. 새10초별 원thread collector의GUI60초3회는평균0.598209/0.727867/0.364378%, round1 첫10초1.090614%로전체FAIL이다. 해당창 main781250/library312500/restore0 CPUticks가관측됐으며phase 원인을확정하지않는다. 최신2ms 실제완료/Next/Stop을재검증중이고기준·180초기한완화0, P0/릴리스완료가아니다. [현재전체검증](evidence/windows-periodic-thread-cpu-2ms-workspace-20261006-continue/root-review.json), [구간별원결과](evidence/windows-periodic-thread-window-gui-60sec-20261006-continue/gui/result.json), [병렬작업계약](../work-items/S00-gui-idle-window-attribution.md). 아래b2와wall-only 후보는이력이다.

현재 후속 인수에서 threadCPU 예산 후보는 실제periodic첫INSERT/4cap까지도달했으나180초commit실패였다. 원결과와inner강제소유cleanup을보존하며현재Next/Stop·단독phase/10초별thread계측으로원인을좁힌다. [작업/실패 근거](../work-items/S01-periodic-thread-cpu-budget.md). 완료와CPU를함께만족하기전P0/스택/릴리스완료가아니다.

2026-10-06 현재 후보: 주기 worker의 원 thread CPU counter/0.4% 누적 부채 대기를1500µs/500ms wall 정책과 통합했다. Windows CPU 전체 fmt/clippy/test/release4PASS, 현재 source543개 및 CPU b2bd9d584810eb0f4d614ad27a414dd76857867fa64cfb059c09882a2a3853fe/7915008B를 root 대조했다. [원 전체 검증](evidence/windows-periodic-thread-cpu-workspace-20261006-continue/result.json), [root 검토](evidence/windows-periodic-thread-cpu-workspace-20261006-continue/root-review.json). GUI60초3회 avg0.5463/0.3381/0.6768%·memory3PASS지만 round3첫10초1.2487%로전체FAIL이다. root가원합성파일3만개와각DB1만행/무결성을확인했으며원18창계산·cleanup독립감사도완료했다. 최신완료/Next/Stop 및10초별thread계측인수를이어가며P0완료가아니다. 앞선2ms/1500µs wall-only GUI strict2FAIL과2ms의118.773초 완료/Next/Stop은 당시 snapshot 근거다. 신규 dependency/schema/lock/추가thread0.

2026-10-06 후속 최신 후보: 주기예산2ms/500ms·typedWatchCause·Next 선점/명령보관은 Windows CPU 전체4검증(source542/currentCPU c744b913f06b3dd77911dd5aa6d4482edceabb046059b5a443e072fcf5ebced6/7,912,960B)을 통과했다. [전체 원증거](evidence/windows-periodic-budget-workspace-20261006-continue/result.json). GUI60초3회 평균은 모두1%미만이지만 strict10초 두회미달로 성능 인수는 미완료다. 원3회/natural cleanup을 보존하고 단계계측·실제 periodic 완료/Next/CloseWait 인수를 이어간다. 새 basic-OFF fixture 후속 source는 별도 검증 중이며 위 proof를 새 source proof로 표시하지 않는다. 새dependency/schema/lock 변경0; 당시defaultCPU 증가17,920B, runtime추가thread/mutex0.

<!-- 최신 scoped 결과; 누적 이력은 아래 원 snapshot 범위를 따른다. -->

2026-10-06 재개: fresh 경로 버퍼 재사용·lazy TEMP identity 인덱스를 포함한 최신 소스541개의 Windows CPU fmt/clippy/test/release가 통과했고 root와 독립 검토의 현재 hash 차이는0이다. 현재 CPU SHA bb60a5d5e3a5aec90940061077fbf3155c723ebb7689b27e8af98f5af5964927/7,895,040B. [전체 원증거](evidence/windows-library-observation-cost-workspace-20261006-continue/result.json). 같은 quietworker 조건의 평균CPU1.2222%는 미달이며 최신 제품 GUI 예산은 아직 미검증이다. periodic 예산/우선명령 통합은 [작업](../work-items/S01-periodic-scan-priority-budget.md)으로 진행한다. 이전8f21 개발ZIP/font 결과는 당시 binary에만 적용되며 현재 binary 공급 완료를 뜻하지 않는다.

# P0 구현 상태와 출시 차단 사항

## 현재 재개 지점 (2026-10-06 연속 작업)

최신 후속은 library 임시 identity index를 변경 publication 때만 만드는 최적화다. fmt·fixture Clippy·Windows library17회귀(재사용/rename/취소 포함)는 PASS이고 동일quietworker비용재측정중이다. 아래ce7efd전체/GUI는이library.rs변경전snapshot이다. P0는미완료다.

WatchSession 단일 root 관측을 통합했다. 실제 Windows red는 2조회 대 기대1로 실패했고, 수정 후 watch10회귀는 PASS다. 8f21 전체/GUI/개발ZIP 증거는 이 watch.rs 변경 이전 snapshot이다. 동일60초 scan없는 감시비용은0.4142%→0.12954%였다. 후속줄바꿈교정후 최신Windows 전체fmt/clippy/test/release4PASS와root source현재일치검증을완료했다. 새CPUce7efd97…/7895040B의격리GUI60초×3은평균1.69/1.46/1.98%,10초max4.54/4.21/4.83%로CPU FAIL,메모리PASS다. root는원3만payload·sentinel·readonlyDB각1만행/integrity를확인했다. P0 CPU 기준은 미완료다.

다음128개에서불필요fullscan을제거하고기존DB를생성/migration하지않는readonlypageAPI를통합했다. missing의원red→green·두앱각19/engine2 실패보존회귀·fixture포함Clippy와페이지 작업 당시541소스Windows CPU 전체fmt/clippy/test/release4PASS를root검증했다. 페이지 작업 실행파일8f21c516…/7895040B는격리GUI 실제Next/수동추가·삭제·rename/invalidroot/자연종료도PASS다. root는원page2PNG/UIA·정확128행cursor/ID/크기/DB무결성을대조했다. [완료된페이지작업](../work-items/S01-library-page-read-only.md). CPU1%예산은이binary에서새로통과했다고주장하지않는다. scan없는WatchSession100ms반복관측은 평균0.4142%였다. 단일관측 수정 후 동일sleepcontrol/관측 비용을 재측정한다.

notice receipt 당시 소스고정WindowsCPU normal/buildtree259개에기존notice누락43의교집합0을actualnative/lockSHA/12실패정상회귀·독립검토로확인했다. wholelock43과embedded/link/legal/releasefalse는그대로보존한다. 아래0478·3f9전체/GUI/패키지는각당시snapshot의이력이다.

추가 default-off pacing fixture의 실제 전체scan·Stop/supersession 회귀는 통과했다. scan138.417초/CPU0.75초이며 한10초구간1.24995%로 실패했고 동일30초workload 예산은 증명하지 않았다. 제품pacing정책은 미채택이다. fixture 추가 뒤 최신전체workspace를 다시 검증하며 아래0478전체PASS는 그 이전snapshot이다. OS thread description 3회 GUI의 평균1.899/1.612/1.638%도FAIL이고 원3만파일bytes·DB1만행/무결성은 root가 확인했다.

Windows hard-link alias 수정 뒤 DB 크기가12B로남는결함을원burst반복과watcher없는최소시험으로재현했다. 최신파일metadata/full128ID대조 및구형API미지원의원HANDLE조회로교정했고alias안/밖·held/noheld·크기/동일크기mtime8조합과fallback/rename rollback이통과했다. 중복preflight조회축소후watcher9·library30·두UI각14·fixture포함Clippy와최신541소스 전체fmt/clippy/test/release가PASS다. 새CPU는0478aa51…/7892992B이며default계측marker는없다. 원1만burst도첫교정engine에서green이나그원CPU비용0.61~0.64초는목표미달이다. 중복축소후1만scan3회CPU0.328초를확인했고새CPU격리GUI60초×3회평균1.61/1.59/1.61%·strict10초max4.21/4.06/4.22%는FAIL이고메모리는PASS다. root는원3만bytes·각DB1만행/무결성·source/Job/private/input을검증했다. relative원directoryHANDLE 조회와실제worker스레드identity 비용실험을이어간다. **GUI성능·최종P0/릴리스는미완료다.**

바로 이전 CPU3f9d snapshot은 전체fmt/clippy/test/release와 S10 대형 JSON/Markdown/236쪽 미리보기3case를 통과했다. GUI 평균0.73/0.65/0.81%와 메모리는PASS, strict10초구간은FAIL이다. 원CPU 개발ZIP2개는 동일12682630B/863entry/SHA83ca7b..이며 actual wholelock notice누락43개/release_ready=false다. 정확 내장폰트4payload와 원공지문 연결 및15실패회귀는 별도 검증했다. [메타데이터 결함 교정](../work-items/S01-windows-hardlink-metadata-freshness.md), [현재 합치기 작업](../work-items/S01-watch-hint-coalescing.md), [폰트 증거](../work-items/S06-current-linked-font-audit.md), [S10 회귀](../work-items/S10-current-ui-large-report-regression.md).

최종 S00/physical IME·reader·clipboard·설치/공급/실제 site 및 remoteCI는 독립 게이트다. 정상 Git 없음과 기존 .git 보존을 유지한다. **아래 기록은 각 당시 snapshot의 이력이며 최신 소스 전체 인수로 합산하지 않는다.**

2026-10-05 현재 계속 진행: 정확동일 목록의 불필요publication 생략은 Windows27회귀와 최신 전체fmt/clippy/test/release PASS다. 후속 identity 임시할당 제거·동등집합 query 보강의27회귀·1만원ID전체대조 및 최신 전체4검증도PASS다. defaultCPU에는 계측feature가 포함되지 않음을 원binary에서 확인했다. GUI60초 예산은 별도 재측정하며 P0는 미완료다. [작업](../work-items/S01-library-exact-unchanged-publication.md).

2026-10-05 이전 native 통합: Windows native 목록 scanner 후보를통합했다. 전체1만한글file ID/size/mtime/bytes와DB재열기검증, rename후교체경합 red→green 및 Windows목록22회귀가PASS다. 첫전체Clippy의표현경고2건을교정한현재540소스 전체 fmt/clippy/test/release가 모두 PASS이며 CPU release SHA는630e63e38444…다. 원로그의 자연 Job0·가상 데스크톱 종료·입력 데스크톱 불변과 현재 소스 일치를 root가 확인했다. 최신GUI60초×3회 평균CPU는0.728/1.118/0.910%이며 메모리는 모두PASS지만 strict10초 구간CPU는 모두FAIL이다. 성능작업은 미완료이며 단계별계측을 이어간다. [작업계약](../work-items/S01-windows-native-library-scanner.md). 아래snapshot은이력이다.

2026-10-05 계속 진행: Windows 관리 runtime 정상 GUI 인수에서 합성 Deno/FFmpeg 2.0 활성화·active DB·정확 다운로드 필드 전달·원파일/외부 sentinel·Job자연0를 검증했다([작업](../work-items/S04-windows-runtime-positive-gui.md)). 목록의 native directory ID 실험 첫3회는API+parse wall약2.4~2.9ms와128원file교차확인 포함CPU15.625ms였다. 제품scanner 대체는 미채택이고, 중복·누락 bitset/경로 재검증을 보강해 시험 중이다. 보호소스538개가 되어 이전537전체workspace 결과는 이력이며 새전체검증은 후속이다.

2026-10-05 현재 작업: 새 identity 비용 example 포함537소스의 Windows CPU 전체 fmt/clippy/test/release와 root SHA 검토가 PASS다. [최신 전체 증거](evidence/windows-library-identity-workspace-20261005-resume/result.json). 바로 이전536소스 통합에서는 목록 prepared INSERT 통합 뒤 Windows CPU feature workspace의 fmt/clippy/test/release 네 명령이 실제 PASS했다. 536소스 SHA와 자연 Job0·private desktop 종료·input desktop 불변을 주 담당이 검토했다. 현재 CPU는 `0ed4251630d1…`, 7,872,512B이며 [전체 증거](evidence/windows-library-prepared-workspace-20261005-resume/result.json)를 따른다. 취소·rollback·원파일 ID·closewait 목록 회귀12개 PASS. public10k reconcile CPU 평균1.177083→1.15625초 변화는 예산 해결 증거가 아니며, 최신 GUI60초×3회는 모두 CPU예산실패(평균5.04/4.74/4.82%), 메모리예산은 통과했다. Windows identity 단계 비용 실험은3회PASS(CPU약1.05~1.06초)이며 경로 정규화·metadata·openregular의 wall비용이 큰 것을 확인했다. 안전성을 유지할 다음 최적화는 후속이다. 개발용 패키지는 이전dfae 빌드의 notice누락44건 후보이므로 최신CPU 패키지/릴리스로 표시하지 않는다. P0는 미완료이며 아래 문단은 당시 snapshot 이력이다.

2026-10-05 최신 재개 통합: Windows 기본 CPU fmt/clippy/test/release 네 명령 PASS, 현재535소스 SHA 불일치0을 주 담당과 독립 검토가 확인했다. CPU `dfae315d…`, 7,871,488B. [최신 전체 증거](evidence/windows-published-full-current-default-20261005-resume/result.json). 실제 SQLite FULL에서 원 저널 보존과 게시된 파일의 recover1→0·재열기를 검증했다. 외부 HTTPS notice v3는 동일 설치 트랜잭션으로 구현했고 공식 Deno의 비활성 설치·전체 검증·변조 거절·복원·정확 세트 제거를 실제 Windows에서 확인했다. runtime 활성 채택과 공급 trust/GUI는 별도다. 라이선스는17 source/30 package exactmapping으로 늘었으며 신규 후보2ZIP을 측정 중이다. 첫 native queue 포커스 시험은 desktop 조회 FAIL을 보존하고 계측 중이며 기존 synthetic47단계 PASS를 실제 물리 입력/reader 검증으로 확대하지 않는다. S10 대형 보고서의 Job committed peak 실측을 진행하며 전체 RSS·reader·실제 IME·최종 S00 채택·공급/배포 인수 등 P0 게이트는 남아 있다. 아래 날짜가 같은 이전 통합 문단은 당시 snapshot 이력이다.

2026-10-05 현재 통합: 최신 Windows 기본 CPU 전체 fmt/clippy/test/release 네 명령과 현재534소스 SHA 검증이 완료됐다. CPU966c6b79…/7856640B, 원Job natural0·privateDesktop 종료·inputDesktop 불변을 확인했다. S10 방어2MiB/+1·write실패와 실제 큰 JSON1841855B/Markdown1933102B 저장·원본보존·DB 실패상태 검증을 완료했다. 200% 키보드47단계는 scoped PASS지만 실제 OS reader/물리IME/포커스 outline 전체 판정은 남는다. 라이선스는15 source/28 package를 매핑했고 신규 개발용2ZIP SHA d9462f25…/12580624B/846entries가 독립 검토 PASS이며 notice 누락55건·release_ready=false다. 관리 runtime 공식 ZIP 밖 notice를 동일 설치 트랜잭션에 연결하는 후속 P0 작업에 착수한다. 전체 P0 및 최종 스택은 미완료다.

2026-10-05 같은 세션 재개: 실제 Windows FFmpeg 시간색 source4구간과 fast/exact/off-GOP 정확cut3개·decode·DBreopen·실처리후 취소/timeout이 원 native PASS다. 원 파일/DB 독립 검토는 진행 중이다. 200% 키보드 시험에서 실제 화면 밖 포커스 세 곳(고급 옵션·IME field·순환 후 테마 ComboBox)을 발견하고 단계별 실패 증거를 보존하며 수정·재검증한다. first-paint Light/Dark/System 원owner3case는 PASS이며 최종 최신 defaultCPU 전체·패키징 복원은 아직 진행 전이다. 라이선스 notice는 고정 clipboard-win1338B를 추가해 source8/package23 매핑이 root검증됐으나 전체 notice/SBOM과 신규2ZIP 검증은 미완료다. [병렬 보드](../work-items/P0-parallel-board.md), [키보드 작업](../work-items/S00-windows-session-zoom-keyboard.md), [미디어 작업](../work-items/S05-windows-real-ffmpeg-cut.md). 전체 P0는 미완료다.

2026-10-05 이전 세션 마무리: [다음 세션 인수인계](SESSION-HANDOFF-20261005.md)를 먼저 확인한다. 현재 개발 패키지 독립 검증은 PASS이나 license 누락68건으로 release_ready=false이며 전체 P0는 미완료다. 사용자의 중지·현재 작업 마무리 요청에 따라 새 작업은 시작하지 않았다.

> 사용자 범위 변경: 최초 릴리스는 **Windows 우선**이다. Linux·macOS 및 3OS 출시 인수는 후속으로 보류한다. 기존 문서의 3OS 게이트는 후속 목표이며 Windows P0 완료를 차단하지 않는다. 공통 Rust 구현과 기존 검증 근거는 유지한다. [범위 계약](../work-items/P0-windows-first-release.md).

## TAR/XZ v2 관리 설치 통합 (2026-10-04)

명시v2layout/root/membermapping과선택합384MiB한도를구현하고기존v1ZIP/저장JSON/256MiB정책을유지했다. 관리installer는원File/nativepin·rootlock·동일기한·staging/전체검증/제거를재사용한다. Linux도구70PASS/1ignored·신규package3회귀/engineall-targetClippyexit0. 실제고정FFmpeg원archive의v2비활성설치·전체검증·제거·원SHA보존이PASS이며install36.7초다. [계약](../work-items/S04-tar-xz-package-integration-contract.md), [실제결과](evidence/ffmpeg-v2-managed-install-first/result.json). 실제ffmpeg/ffprobe버전확인선택·최신source전체workspace·GUI/Windows/macOS/공급trust/notice와전체P0는미완료다. 아래직전전체PASS는v2추가전snapshot으로구분한다.

## FFmpeg 원 TAR/XZ native 판독 실증 (2026-10-04)

고정공식LinuxFFmpegarchive137863452byte를Rust메모리제한XZ→boundedTAR로전체판독하고ffmpeg/ffprobe/LICENSE의정확size/SHA와원source전후SHA보존을확인했다. 예제486280byte,최대RSS10348KiB이며선택별전체3회순회했다. 시간관측별원시는작업문서에구분기록했다. [현재계약](../work-items/S04-bounded-tar-xz-gate.md), [실제결과](evidence/ffmpeg-native-tar-xz-first/result.json). 관리설치·활성선택·trust/license준비완료증거는아니다. 기존ZIP256MiB계약은유지하며FFmpeg형식/설치용량연결은후속이다.

TAR4개그룹·XZ강화5회귀와engineall-targetClippy가통과했다. 최신WindowsGNUCPU capability/fmt/clippy/test/release전체terminalexit0와root현재sourceSHA불일치0을확인했다. [Windows전체](evidence/tar-xz-windows-workspace-first/result.json). Linux최신전체workspace는실행중이다. Windows실제FFmpeg원archive·GUI/macOS/공급배포·전체P0는미완료다.

## 외부 Deno 네이티브 입력 최신 검증 (2026-10-04)

CPU/접근성 공유 화면과 기본 후보에 중앙 catalog의 외부 Deno 경로·도움말을 연결했다. 실행 중 선택은 고정하고 원 저장 snapshot 재개를 유지한다. 신규 회귀는 세 native 후보에서 각각1개 PASS이며 최신 fmt·CPU 앱 all-target Clippy 및 Linux CPU release가 exit0이다. [작업 계약](../work-items/S04-external-deno-native-input.md).

Windows GNU CPU 전체 capability/fmt/clippy/test/release가 terminal exit0이다. root가 source_baseline과 현재 모든 파일 SHA를 독립 대조하여 불일치0을 확인했다. [원시 증거](evidence/deno-native-input-windows-workspace-live/result.json). 실제 Windows GUI Deno/공식 runtime 실행·macOS는 별도다.

Linux GUI 첫 두 시험은 선택 경로 actualText와 literal argv를 확인했으나 수집기의 빈입력 삭제·화면밖 본문 관측 오류로 실패했다. 원시 두 결과를 보존하고 제한된 실제 입력/펼침 상태/본문 스크롤 관측을 교정하여 재시험 중이다. 최신 Linux 전체workspace는 후속 terminal에서 기본/CPU7명령·version2개 모두 exit0이며 root가 source_final의 현재 SHA 불일치0을 확인했다. [전체 증거](evidence/deno-native-input-linux-workspace-live/result.json). 후속 observed-role 실제 격리 GUI는 terminal exit0/PASS이며 root가 selected/empty argv·relative고정오류·소유정리·SHA와 오류 PNG를 직접 확인했다. [GUI 증거](evidence/deno-native-input-linux-gui-observed-role/result.json). 실제 Windows GUI/한글IME/macOS·관리공급과 전체 P0 및 릴리스 게이트는 미완료다.

## Deno 다운로드 옵션 통합 검증 (2026-10-04)

선택 Deno를 하위 호환 optional 저장 옵션으로 연결했다. metadata와 다운로드는 동일 canonical runtime을 literal argv로 전달하며 실제 버전을 Attempt와 실패 보고서에 남긴다. runtime 검사 중 확인된 사용자 취소는 Cancelled/Paused로 분류하고 정리 미확인은 보존한다.

Linux core24개·Deno관련12개 및 CPU all-target Clippy가 통과했다. 최신 Windows GNU CPU workspace Clippy와 test는 terminal exit0이다. root는 docs/validation/evidence/deno-download-options-native-cancel-classification/{clippy,test}.json의 source_before를 현재 파일과 독립 대조하여 불일치0, source_unchanged 및 input_desktop_unchanged를 확인했다. Windows 공식 runtime 실행·GUI runtime 선택·최신 전체 release 빌드·macOS와 전체 P0는 미완료다. 이전 snapshot의 typed argv/Attempt 미연결 기록은 현재 backend 상태와 구분한다.

## Deno 검증 선택 통합 (2026-10-04)

[제품 선택 API](../work-items/S04-verified-deno-selection.md)는 실제 owned --version/host target/원directory·file ID/전체sethash 재확인 후 같은rootlock안에서active를전환한다. root 실제Linux tools63PASS/1ignored, Windows42PASS 및 두OSCPU전체Clippy0을확인했다. Linux 실제합성process 성공·wrongversion/nonzero/selfmutation/취소/foreign 실패는prioractive를보존한다. Windows42개에는순수parser 및기존install/remove가포함되며 Windows공식Deno process 선택PASS가아니다. 공식Linux제품API실증은개발예제guardian등록누락의원실패를보존한뒤selfentry연결후actualexit0/PASS로검증했다. 실제select_verified_deno 활성전환·active삭제거절·원archive SHA보존과3.69초/peak4992KiB를확인했다. typeddownloadargv/Attempt진단/GUI/runtime공급notice/trust와새source전체workspace는미완료다. 아래이전전체PASS는선택API추가전snapshot이다.


## 최신 관리 package 통합 (2026-10-04)

관리 archive 비활성 설치·전체 component 검증 선택·정확 세트 제거를 engine에 연결했다. 공통 root lock·전체 설치128/탐색2048·원 native root/file pin과 metadata-only 목록을 통합했다. Windows bool/Clippy/readcap 오류를 실기로 확인·교정하고 원 directory/file namespace swap을 분리해 도구35PASS를 확인했다. 이후 [HTTPS archive worker](../work-items/S04-archive-https-worker.md)를 같은 streaming loop·한 lock·원 File·동일120초 budget으로 연결한 도구 회귀는 Linux56PASS/1ignored·Windows38PASS였다. actual midstream timeout 시험의 Windows read0 실패를 remaining-budget으로 교정한 뒤 Windows39PASS 및 Linux57PASS/1ignored를 확인했다. 후속 source에서 Windows GNU CPU 전체 capability/fmt/Clippy/test/release terminal0과 현재SHA370개 일치를 root가 확인했다. Linux 전체 collector는 release 단계이며 terminal 전에는 전체PASS로 집계하지 않는다. 이전 snapshot의 전체PASS를 후속 package/HTTPS 소스로 소급하지 않는다. [공식 archive owned 설치 실증](../work-items/S04-archive-owned-install-experiment.md)은 Linux 로컬/HTTPS 설치·검증·inactive 유지·exactset 제거 actualPASS이며 HTTPS6.92초/peak5792KiB이다. [공식 Deno 버전 출력](../work-items/S04-official-deno-version.md)은 원 SHA를 확인한 임시 도구 --version actualexit0/0.817초를 기록했다. 제품 runtime 활성화·typed argv·GUI 연결은 남는다.

[격리 복사 능력 시험](../work-items/S10-named-windowstation-clipboard-probe.md)은 실제 C# compile PASS 뒤 새 station 생성 권한 오류5로 중단했다. 사용자 clipboard 접근 없이 원 process station/input desktop 유지 및 cleanup을 확인했다. Windows 제품 복사 GUI는 미검증이다. [다음 네이티브 UI 연결](../work-items/S04-package-native-ui-integration.md)은 공급 신뢰·라이선스·HTTPS·실행 버전 및 tool별 선택 계약을 정리한 proposed 작업이며 구현 완료가 아니다. 전체 P0 및 릴리스 게이트는 계속 미완료다.

## 관리 ZIP 계약·스트림 추출 검증 완료 범위 (2026-10-04)

archive/component별 SHA·크기·소유권·flat allowlist 공유 계약과 rawzip/deflate 스트림 추출 API를 구현했다. 아래 cookie 전체검증은 archive 추가 전 snapshot이다. archive 변경의 별도 Windows GNU CPU fmt/clippy/test/release가 모두 terminal0이고363개 현재소스SHA 일치를 확인했다. Linux 공식 ZIP의 Rust 추출은 actual95830104bytes/SHA일치·원본보존·peak2388KiB였으며 Linux 기본/CPU 전체fmt·clippy·test·release도 terminal0/SHA일치로 통과했다. [현재 작업 범위](../work-items/S04-bounded-archive-contract.md). 설치/active 전환/GUI/관리 제거·공급 trust/license의 전체 수직 기능은 계속 미완료다.

## 최신 Windows 시작 복구 GUI와 Linux 전체 검증 (2026-10-04)

최신 Windows CPU release(6898fd11...)에서 independent C# private desktop runner로 receipt 완료 회수·pristine 보존·unsafe cache 오류 3case를 실제 실행했다. 세 result PASS, 원 source/foreign 보존, 사용자 input desktop 불변, 자연 종료 및 원 Job0를 확인했다. root가 원 PNG를 검토했으며 오류 안내와 재확인 버튼도 canonical 원본에서 가시성을 확인했다. 첫 .NET verbatim 경로 실패 원시도 유지한다. [실제 범위](../work-items/S02-windows-gui-cookie-seed.md).

`windows-cookie-late-seed-linux-workspace-first`는 최신 기본 fmt/clippy/test/release 및 CPU clippy/test/release terminal exit0, source_final 현재SHA 불일치0이다. Windows 동일 최신 source capability/fmt/clippy/test/release도 terminal exit0로 통과했다. windows-cookie-late-seed-native-workspace-first의360개 입력 SHA를 현재 파일과 대조해 불일치0, input desktop 불변, CPU executable7452672bytes/SHA6898fd11...를 확인했다. 이 시작 복구 3case는 실제 GUI crash·busy Bound·partial deletion·같은 앱 guardian 인증 다운로드·IME·전체OS/릴리스 게이트를 대신하지 않는다.

## 최신 Windows 인증 실행 연결 (2026-10-04)

Windows 제품 importer의 original journal/marker와 receipt 기반 public startup 회수를 연결했다. native UI startup worker도 회수를 호출한다. 제품 metadata/download는 독립 headless helper의 원 HANDLE 생성 시각→durable Bound→native journal lock handoff ACK→Begin을 사용한다. helper의 원 Job empty 확인·native exit/Terminal/EOF와 실제256-byte receipt resolve 뒤에만 pending을 해제한다.

[실제 앱 유사 프로세스 강제 종료](../work-items/S03-windows-helper-app-kill.md) 1case, [제품 import에서 helper receipt→publicrecover](../work-items/S02-windows-helper-receipt-public-recovery.md) 1case, [Windows DownloadRepository 인증 성공·metadata 실패·download 실패·취소](../work-items/S02-windows-cookie-download-gate.md) 4case가 native PASS다. 실제 source/foreign 보존·출력/DB 비밀정보 생략·원 slot receipt 및 사본 회수를 확인했다. 앱 CPU/engine native Clippy와 engine156PASS/1ignored는 integration4case 추가 전 snapshot의 근거다.

[최신 Windows GNU CPU 전체workspace](evidence/windows-cookie-product-native-workspace-first/result.json)는 capability/fmt/Clippy/test/release exit0이며 root가 현재 source SHA359개 일치를 대조했다. [같은 snapshot Linux 기본/CPU 전체workspace](evidence/windows-cookie-product-linux-workspace-first/result.json)는 nine operations terminal PASS/currentSHA357개를 root가 대조했다. [capture 독립 검토](../work-items/S03-windows-cookie-capture-independent-review.md)에서 Terminal 이후 늦은 취소 명령 경계를 실제 native 회귀로 재현하고 수정했다. green exact1/Clippy PASS이며 이 수정은 위 전체workspace snapshot 이후이므로 후속 전체 검증이 필요하다. 최신 Windows GUI startup 회수·sameapp guardian·GUI crash/IME/clipboard/성능, 실제 사이트·네트워크/디스크/재부팅·macOS/MSVC/설치·remoteCI 게이트와 전체P0는 미완료다. 아래 이전 snapshot의 'Windows public recovery 미연결'은 당시 상태이며 최신 연결을 부정하는 현재 결론으로 사용하지 않는다.

## 이전 소스 통합 검증 기록

[Windows guardian 실행 루프](../work-items/S03-windows-guardian-runtime.md)를 연결해 실제 Request5case와 Clippy를 통과했다. bounded config는 Linux5순수시험을 통과했다. 내부 headless entry는 컴파일을 확인했으며 독립 helperprocess/appcrash/원handle상속/publicstartup 회수는 미완료다. runtime snapshot 전체 통합 검증은 진행 중으로 terminal 전에는 완료로 인정하지 않는다.

[Windows journal 잠금 인계 전체4검증](evidence/windows-journal-handoff-native-workspace-first/result.json)은 모두 exit0이며 root가 당시 입력 SHA를 대조했다. 원 journal 잠금 인계6native시험과 guardian 파일 경계1native시험, Linux 고정프레임4시험 및 pending Bound1시험을 통합했다. [같은 snapshot Linux 전체 검증](evidence/windows-journal-handoff-linux-workspace-first/result.json)은 필수7명령과 version2개 모두 exit0이며 root가 terminal과 당시 source SHA 일치를 확인했다. 후속 guardian 실행 루프 변경은 이 snapshot에 포함되지 않는다. 독립 guardian 실행 루프·원 handle 전달·public startup 회수는 미완료다.

아래 default-tab 검증은 이전 고정 snapshot의 근거이며 이후 handoff 변경을 포함하지 않는다.


[Windows receipt journal 기반](../work-items/S02-windows-reader-journal.md)을 구현해 원FileId/ACL/비차단잠금/actualreceipt와 unusedpayload 보존4시험 및 순수record6시험을 실제Windows에서 확인했다. [현재 Windows전체4검증](evidence/windows-journal-default-tab-workspace-native-first/result.json)은 모두exit0/currentSHA일치. 제품Windowsguardian/publicrecover는 아직 연결하지 않았고 startup회수는 미완료다.

[기본Iced Tab 오류](../work-items/S05-default-tab-focus-fix.md)는 실제URL xy잔류를 재현하고 ignoredTab→focus task를 연결했다. 고정새binary 격리화면에서 URL→저장경로 이동/ShiftTab복귀를 확인했다. 전체controls/reader/다른OS키보드 인수는 별도다.


[소유 실행기 실제 통합](../work-items/S00-workspace-validation-owned-runner.md)은 [terminal result](evidence/workspace-owned-runner-native-first/result.json)의 기본/CPU7검증+2도구버전 모두exit0 및 현재입력SHA 일치를 root가 확인했다. WSL snapshot과 live runner PID/start 대조도 수행했다. 취소/timeout/실제WSL중단 인수는 남으며 자동 재개를 주장하지 않는다.


[Windows layout 수정](../work-items/S00-windows-current-test-layout.md) 후 [Linux 전체7명령](evidence/cookie-keeper-options-layout-linux/result.json)과 [Windows GNU CPU 전체4명령](evidence/windows-current-options-keeper-workspace-second/result.json)이 exit0이다. Root가 Linux before/after 및 현재 모든 입력 SHA 일치를 확인했다. Windows CLI 통과를 GUI/IME/clipboard/설치 인수로 확대하지 않는다.

[엄격 plan 키보드 인수](../work-items/S05-plan-keyboard-strict-validation.md)는 고정 CPU 바이너리의 Linux private 화면에서 GrabFocus/실제 FOCUSED/Return·Space 열기·닫기 및 preview JSON을 확인했다. [도움말5개 키보드](../work-items/S05-option-help-keyboard-native.md)도 동일 고정 바이너리에서 각Return/Space 열기·닫기와 본문 가시성을 실제 확인했다. [Tab/ShiftTab 실제16단계](../work-items/S05-cpu-tab-navigation-native.md)로 URL에서 도움말5개까지 도달하고 원URL로 복귀했다. 선택 상태별 모든필드·전체앱 탐색, focus표시의 최종판정, 전체 reader와 다른 OS는 남는다.

## 최신 EOF keeper 통합 검증

[EOF keeper 제품 연결](../work-items/S02-cookie-eof-keeper-integration.md)로 초기 identity 확보 전 active(exited) 잔여물을 보강했다. 실제 서비스3·인증 다운로드4·same-boot 회수2 시험과 [초기 실패 세 단계/정상 대조](../work-items/S02-cookie-keeper-startup-fault-native.md)의 native2 시험이 통과했다. [고정 second source 전체 검증](evidence/cookie-keeper-workspace-second/result.json)은 기본 및 CPU의 fmt/Clippy/test/release 전체7명령 exit0, Rust/Cargo 입력 해시 일치다. 뒤에 추가하는 debug barrier와 sibling fixture는 이 통과에 포함하지 않는다.

[단독 release keeper 비용](../work-items/S03-cookie-keeper-cost-probe.md)은5회 PASS, RSS2124~2144KiB, 각10초 idle100Hz CPU tick delta0, EOF→reap 약5.3~5.7ms였다. systemd·앱 전체 비용과8시간 연속성의 증거는 아니다. 이후 [제품 foreign live/queued sibling 실제2PASS](../work-items/S03-cookie-keeper-sibling-native.md)로 원 Invocation·PID/pidfd·pending Job 보존 및 자연 종료 뒤GC를 검증했다. [옵션 도움말 포함 최신 전체 검증](../work-items/S03-cookie-keeper-options-workspace.md)은 기본/CPU7명령과 서비스3·다운로드4·복구2·초기실패2 회귀가 통과했다. CPU 도움말5개는 고정 release 바이너리의 격리 GUI에서 펼침/마지막 문단 전체 가시성을 확인했다. Windows 후속/실제 재부팅/최종UI 및 전체P0는 미완료다. 아래 단락들은 각 과거 snapshot의 범위로 읽는다.


최신 후속 구현은 [이전 부팅 pending 회수](../work-items/S02-cookie-priorboot-pending-recovery.md)와 [service/parent 의존성 통합](../work-items/S02-cookie-service-dependency-integration.md)이다. 원 부팅/marker digest에 묶인 별도 삭제 권한을 구현했고 실제 lease 파일의 synthetic boot 네 조합 및 engine library251PASS/2ignored를 확인했다. 부모 직접 Stop을 없앤 제품 gate3PASS·다운로드4PASS·sameboot 회수2PASS도 확인했다. 첫 fixture 선행조건 실패, DBus SubState 인터페이스 실패 및 진단 출력의 닫힌 stderr 실패 원시는 보존했다. 최신 전체 workspace fmt/Clippy/test/release는 [일치하는 source 검증](evidence/cookie-priorboot-dependency-workspace-seventh/result.json)에서 모두 exit0이다. 같은 source의 [CPU feature Clippy/test/release](../work-items/S02-cookie-priorboot-cpu-integration.md)도 모두 exit0이다. Windows 후속 검증, 실제 재부팅, 초기 service identity 미확보 정리와 제품 foreign queued 경계는 미완료이며 P0 완료가 아니다. 기존 output-limit TerminalUnconfirmed는 고정 binary24회에서 재현하지 못했으므로 원인 해결로 표시하지 않는다.

현재 2026-10-04 후속 기준은 [제품 쿠키 gate 통합](../work-items/S02-cookie-process-gate-integration.md)과 [동일 boot receipt 복구](../work-items/S02-cookie-sameboot-product-recovery.md)다. 실제 DownloadRepository 합성 성공·metadata 실패·download 실패·취소4PASS, 앱 종료·실행중 강제종료 뒤 same-boot 회수2PASS, engine library247PASS/2ignored를 확인했다. 아래의 “sameboot 미구현/항상 보존” 표현은 이전 snapshot의 검증 범위다. 증거 없는 empty/Intent·미확인 reader는 현재도 보존한다. 최신 전체 workspace/CPU/Windows 검증, previousboot pending 회수와 parent release 경합 보강은 진행 중이며 P0/첫 릴리스 완료가 아니다.

## 2026-10-04 현재 통합 지점

최근 전체 Linux/Windows release 검증은 아래29ce8c snapshot이다. 이후 reader unwind 보호는 engine전체시험/Clippy와네이티브UI Rust시험(main16/accessible69/CPU74)을통과했다. direct시작실패를항상typed미확인으로분류한후 공개absenttool1/쿠키실행8시험과fmt/engineClippy/docs를재검증했다. [현재 엔진 보호 결과](evidence/cookie-reader-guard-engine/result.json), [direct 시작 실패 결과](evidence/direct-startup-cookie-ownership/result.json). 해당후속소스를전체release/실제GUI/Windows에검증한것은아니다.

29ce8c의성공·취소 private GUI는 [root 원시자료 대조](evidence/session-cookie-recovery-native-gui/root-review.json)에서SQLite정합성/정확job·options/1attempt,version쿠키제외,metadata·download동일privatecopy,실제artifacthash,PID/cgroup부재를확인했다. 원실행의unsafe-cache수집기실패는보존하고 [수정 수집기 결과](evidence/session-cookie-recovery-native-gui-unsafe-corrected/result.json)와구분한다. [sameboot 보존 원시자료](evidence/session-cookie-recovery-native-gui-same-boot-raw/root-review.json)는이전보류copy보존과새선택다운로드만증명하며sameboot안전회수는미구현이다.

현재 파일을 분리해 [정상 Child 수명](../work-items/S03-owned-child-lifetime.md)과 [OS 공통 쿠키 parser](../work-items/S02-cookie-shared-parser.md)를구현중이다. 전자는취소/Drop의무한대기와rawPIDwait를제거하며후자는WindowsACLbackend의선행작업이다. P0/최종UI선정/3OS입력·접근성/설치·서명/실제사이트인수는미완료다. Git은정상저장소가아니며기존.git을보존한다.

## 2026-10-03 개발 재개

최신startup대기/reaper 및일시admission포화수정snapshot `97cfbcecf2cc664326adfe37a3d97f92a952b4a3fb2b147ebf5b9b5e0729a7ce`는 [전체7명령](evidence/guardian-startup-wait-admission-retry/workspace-result.json)을통과했다(기본220/CPU359PASS·각7ignored). 최초parallel실패raw를보존하고RUST_TEST_THREADS미설정인같은조건으로재검증했다. 등록 앱 GUI3flow는 [root 독립 검토](evidence/guardian-startup-wait-app-20261004/root-review.json)로 실제DB·같은앱guardian·PID/cgroup정리를확인했다. [비용 재검증](evidence/user-service-cost/current-5/root-review.json)은idleRSS54.617MiB, active각역할peakRSS합71.574MiB이며동시PSS/숨김/8h/다른OS최대값을뜻하지않는다. 최신Windows와전체P0는별도미완료다.

세션 쿠키의 Unix private importer·실행 전 소유권 재검사·실행 전용 DownloadSession·native 후보 입력을 구현했다. 일반 DB/옵션 snapshot에 쿠키를 넣지 않으며 인증 실행 raw 로그와 자유 metadata를 생략한다. typed 미확인 정리 오류는 복사본을 보존하고 FD를 닫는다. [전체 통합 검증](../work-items/S02-cookie-workspace-integration.md)은전체7명령exit0/기본246PASS·CPU387PASS·각7ignored이며source시작끝SHA가일치했다. importer15/download6과CPU worker 입력 시험이통과했고malformed-duration회귀도포함한다. 5cb52a14쿠키GUI3flow는원시DB/artifact/hash/PID/cgroup을root가독립대조했다. 이후terminal·durable복구결합29ce8c는Linux전체7명령/기본262CPU405PASS·각8ignored/sourceSHA일치와WindowsGNU3명령/262PASS·1ignored를통과했다. [전체 최신 근거](../work-items/S02-cookie-workspace-integration.md). 29ce8c성공·취소GUI의root독립검토는위현재통합지점을따른다. [terminal 정리 확인](../work-items/S03-terminal-cleanup-ack-review.md), [보류 쿠키 복구](../work-items/S02-cookie-startup-recovery-review.md), 실제GUI와OS별권한은미완료다. 인증 기능의첫릴리스인수완료를뜻하지않는다.

사용자 재개 지시로 중단 명령 전까지 개발을 계속한다. 아래 중단 절은 과거 checkpoint이며 현재 중단 상태를 뜻하지 않는다. P0 전체·S00 스택·최초 릴리스는 미완료다. S10 P0를 최초 릴리스 게이트로 유지한다.

CPU 도구·이력·숨김 복귀·지표·실행 계획과 큰 진단 미리보기를 연결했다. 기본 UI와 최종 스택의 Proposed 상태는 유지한다. Linux 등록 앱의 기본 소유 프로세스 보호는 사용자 서비스 guardian으로 채택했다. [ea7595 통합 snapshot](user-service-final-validation.md)은 기본210/CPU349시험과7개fmt/clippy/release명령을통과했다. 각7ignored는별도실증이다. [단독앱defaultGUI3flow](user-service-app-default.md)는숨김중같은worker유지·완료후명시복귀·중단후종료와같은실행파일guardian/3unitGC를확인했고root가DB/PID/cgroup을독립대조했다. CPU12,438,904B다. 이후시작실패정리수정은debug3fault+manager누락과strict13회귀PASS이며 [후속전체검증](evidence/user-service-fault-final/workspace-result.json)은진행중이다. 이전608eeeb의접근성펼침상태증거와snapshot을구분한다.

- [큰 진단 보고서](cpu-report-stress-validation.md): 유효 fixture Markdown1,933,100B/JSON1,841,853B를 전체 복사·새 파일 저장하면서 미리보기는8KiB 페이지로 표시한다. 현재 Linux 실측 HWM70.8MiB/RSS66.8MiB, 이전 전체 표시 HWM511.8MiB였다. 원래 Failed job/Attempt 보존과 DB·페이지·저장파일을 주 담당이 독립 대조했다. 정확한2MiB 경계는 단위시험이며 모든 OS 최대 GUI 검증을 뜻하지 않는다.
- [실행 계획](cpu-plan-validation.md): 정제된 argv·옵션 변경·잘못된 입력 및 키보드/마우스 토글을 실제 GUI에서 확인했다. 기존 [펼침 상태 접근성 누락](cpu-plan-toggle-diagnosis.md)은 [새 snapshot](cpu-expanded-state-validation.md)의 실제 AT-SPI 상태와 이벤트 회귀로 보완했다. 전체 reader/다른 OS는 남는다.
- [공식 도구 설치·취소](cpu-tools-official-install-validation.md): 고정 yt-dlp 실제 다운로드·검증·명시 적용·보관 버전 제거와 전송 중 취소를 Linux 격리 GUI에서 확인했다. 기존 버전·외부 sentinel·active DB를 보존했다. 다른 OS 및 공급 패키지 전체 게이트는 남는다.
- [이전1669 대기 비용](cpu-paged-cost.md):10초 표본 idleCPU0.499%/RSS52.8MiB. 모든 기능 peak나 장시간 예산으로 확대하지 않는다.
- [Windows paged snapshot](windows-cpu-paged-preview.md): 당시 native250PASS/1ignored·clippy/release를 확인했다. 후속 순수 이력시험3개를 OS 공통으로 옮긴 결과는 [별도 증거](evidence/portable-history-policy/result.json)를 따른다. Windows GUI·IME·clipboard/MSVC/macOS는 별도 게이트다.

주 담당은 공유 제품·schema/Cargo·통합 검증·Windows CLI와 장시간 시험을 맡는다. metrics_recovery는 native 사용자 서비스 module/debug fault hook, stress_recovery는 격리 GUI 비용 표본, soak_recovery_audit는 기본앱 GUI/실제 startup fault·strict13 회귀를 담당한다. 같은 파일을 동시에 수정하지 않는다. Git 저장소가 없어 기존 빈 .git을 보존하며 local 검증을 계속한다.

### WSL 종료 후 재개 확인

이전 boot794988eb는12:14:28 System Power Off로 종료했고 현재 boot0bc0c937는20:48:56에 시작했다. 종료 요청 주체는 미확인이다. [실제 중단 감사](soak-interruption-audit-20261003.md)에 따라 이전8시간 시험은 약5시간59분만 보존되었고 terminal이 없어 미검증이다. 기존 PID와/tmp를 live로 주장하지 않는다.

[지속 workspace 결과](evidence/cpu-metrics-workspace-recovery/result.json)는 새 boot에서 fmt/defaultclippy/default200tests/CPU325tests/release 모두 exit0이며 각5ignored다. 이 결과는 paged-preview 후속 변경 전 snapshot이다. LinuxCPU3a1...의 [실제 지표 GUI](cpu-metrics.md)는 볼륨·valid/invalid/missing/blank·정상 종료를 확인했다. 최신 paged71개 UI unit 및 Windows 결과와 구분한다.

[실제 guardian 사용자 서비스 실험](systemd-product-guardian-prototype.md)의8case는 literal argv·EOF/T/K·정확 owned cleanup을 확인했다. 이후 [Rust native 통합](user-service-engine-integration.md)의 실제13case와 [등록앱 기본 GUI](user-service-app-default.md)를통과했다. 초기서비스실패정리의보강/OS·장시간게이트가남아R08전체완료가아니다.

새 [8시간 합성 시험](../work-items/S03-soak-recovery-run.md)은2026-10-03 UTC12:35:39부터 Linux/tmp의 immutable binary로 실행 중이다. archive의 boot/PID-start/heartbeat를 계속 확인한다. 이전 부분 시간과 합산하지 않으며8시간 terminal 및 정리·복구 증거 전에는 PASS로 표시하지 않는다. 실제 사이트·영상/sleep/disk-full 검증을 대신하지 않는다.

## 2026-10-03 사용자 요청에 따른 중단 지점

이 절은 이전 중단 당시 기록이며 아래의 진행 기록은 각 당시 snapshot에 대한 이력이다. 현재 착수한 구현·단기 검증과 문서 정리까지만 마쳤고 추가 개발은 중단한다. **전체 P0, S00 스택 확정, 최초 릴리스는 미완료**다. S10 P0 진단은 계속 최초 릴리스 필수 범위다.

CPU 후보에 테마의 bounded read-only 시작 읽기/직렬 저장/종료 대기, 배율, 입력 최소 높이, 품질·구간·자막·고급 옵션과 원래 snapshot 재개를 통합했다. [최종 통합 계약](../work-items/S02-cpu-options-integration.md), [설정 실제 GUI](cpu-settings-validation.md), [옵션 실제 GUI](cpu-options-validation.md), [Windows GNU snapshot](windows-cpu-options.md)을 따른다. 기본 dlpad의 기존 숨김·복귀 구현과 CPU 후보의 아직 미이식된 숨김·복귀는 구분한다.

최종 Linux 기본 시험 197 passed/5 ignored, CPU 포함 280 passed/5 ignored, 실패 0. fmt/default 및 CPU clippy/release 성공. Windows GNU 200 passed/1 ignored이며 주 담당이 239개 입력 해시를 대조했다. CPU Linux SHA256 `2459ba126c6bcc13470b470c8a384d9d8ecef779893fd832411db0a78932f559`. 주 담당이 최종 옵션 GUI 11건, SQLite 사본 8개와 argv/재개 snapshot을 독립 검토했다. 최신 옵션 binary의 성능은 미측정이며 기존 10초 설정 표본을 최신 비용으로 주장하지 않는다.

이미 시작한 두 **8시간 headless 합성 시험과 증거 archive만 유지**한다. 단기 GUI 시험은 모두 종료했으며 사용자 화면을 열지 않는다. 마지막 소유 PID/start 관측에서 장시간 시험은 실행 중이었고 종료 결과는 아직 없다. [시험 재개 기록](soak-restart.md)과 `evidence/soak-restart-20261002/pause-handoff-observation.json`을 따른다. 남은 시간은 당시 단조 시계 기준 약 3시간 9분이며 정확한 벽시계 종료 시각은 보장하지 않는다. WSL 종료/재시작 여부를 추정하지 않는다.

다음 세션 순서:

1. AGENTS→README→PRODUCT→CONTEXT→관련 요구사항/ADR을 읽고 Git/환경을 재확인한다. 정상 Git 저장소는 없으며 기존 .git을 보존했다. 초기화·삭제·이력 재구성을 하지 않는다.
2. 기존 장시간 시험의 실제 terminal JSON, archive 상태, boot ID와 PID/start identity를 확인한다. 관측 timeout이나 PID만으로 성공/실패를 단정하지 않고, 결과가 없으면 미검증으로 남긴다. 다른 소유자의 프로세스를 종료하지 않는다.
3. CPU 후보의 관리 도구 UI·전체 작업 이력·숨김/복귀 listener 및 지표 등 기존 기능 이식과 S00 OS별 IME/reader/테마/성능 게이트를 이어간다. 라이브러리 입력은 100% 32px만 실제 확인했으며 전 배율/테마/focus/루트 적용은 남았다.
4. guardian 자체 SIGKILL 시 detached 자손 생존 차단을 해결한다. [app subreaper 검토](../work-items/S03-app-subreaper-fallback-review.md)는 안전한 소유 spawn/reap 계약 없이는 단순 fallback을 채택할 수 없다는 검토이며 구현 완료가 아니다.
5. S10 P0의 Windows/macOS 진단 GUI·clipboard 및 2 MiB 보고서 스트레스, 실제 사이트/FFmpeg·8시간 실제 media·sleep/disk-full·redirected Documents·도구 공급/license·MSVC/macOS/설치 검증을 완료한다. Windows 격리 clipboard station 생성 실패는 해당 검증만 보류하며 사용자 station을 우회 사용하지 않는다.

P1/P2는 P0 선행 조건 충족 후에만 진행한다. 실제 계정/외부 전송/공개 배포/시스템 설정 변경은 기존 별도 승인 범위를 유지한다. 현재 에이전트는 모두 추가 작업을 멈추며 파일 담당 이력은 [병렬 보드](../work-items/P0-parallel-board.md)에 있다.

2026-10-02 현재. P0 전체와 S00 완료를 선언하지 않는다. S10 P0는 최초 릴리스 범위이며 P1/P2는 P0 선행 게이트가 충족되지 않아 착수하지 않았다. 독립적으로 안전한 P0 foundation을 구현/통합했다.

| 요구사항 | 구현된 개발 범위 | 남은 인수 조건 |
|---|---|---|
| R01 | 대상 볼륨 여유, 실제 다운로드 progress/ETA, Linux 선택 interface 수신·송신 및 FFmpeg 처리 feed/GUI | 다른 OS 전용 provider/검증 |
| R02 | OS Documents 기본 생성·선택 적용·SQLite 복원 UI, 임시 provider 실패 검증 | 실제 redirected Documents/OS 권한 실패 |
| R03 | transaction scan/paging, native dirty watcher, 시작·재진입·수동 갱신 및 watcher 재연결/mtime·주기 보완 UI | 출처 연결/OS별 실행 검수; Windows native identity는별도증거참조 |
| R04 | external yt-dlp worker, bounded metadata, SQLite Attempt/상태, final 검증/등록, 실제 합성 HTTP403·단절·동일Job 재개 및 writer 실패 회귀 | 실제 YouTube·치지직/FFmpeg, 8시간 soak, sleep·만료·디스크 부족 |
| R05 | 밀리초 구간·길이 검사, 빠른/정확 cut·외부 FFmpeg 선택과 Attempt 증거, 소유 도구 SHA256 import/rollback/removal, 공식 고정 yt-dlp HTTPS 설치·경로 적용 UI, Linux staging 회복 | 실제 사이트 구간 저장, FFmpeg/JS runtime 공급·archive·license·비Linux 회복 |
| R06 | System/Light/Dark/scale 개발 UI, 직렬화된 SQLite theme 저장/실패 및 실제 화면 복원 검증 | OS 변경·첫 프레임·IME·접근성·설정 failure-path 완성 |
| R07 | 타입 옵션/예약 차단/한국어 catalog/URL 생략 argv preview, 속도 제한·재시도·조각 동시성·자막 선택 UI, 영상+자막 다중 공개/Unix 복구, 실제 도구 합성 media+VTT·Xvfb 선택/충돌/freeze 검증 | 실제 사이트 자막·추가 품질·공식 pin 버전 fixture 남음; config 가져오기 P1 미착수 |
| R08 | worker UI, 취소/타임아웃, 그룹/Job Object 코드, single-instance guard, Linux guardian/시작 journal 복구, native 파일 identity, Linux 숨김·복귀/Windows 복귀 IPC CLI 및 privateDesktop 실제 작업 유지·복귀 | guardian 자체 고장·macOS crash, Windows IME·MSVC/설치, macOS 복귀·OS 종료/sleep |
| R09 | P2 범위 유지 | OAuth 프로젝트/실제 계정 전송 승인, 구현 미착수 |
| R10 | AGENTS/work item/문서/고정 workspace와 명령 | 정상 Git가 없어 commit/분리 worktree는 불가 |
| R11 | 3 OS fmt/clippy/test/release + 비교 backend CI 설정 | remote Actions·설치·서명·배포 검증 없음 |
| R12 | P1 미착수 | 선택 benchmark/환경 진단 |
| R13 | 정제 stage/version 보고서, MD/JSON 미리보기/복사/새 파일 저장, 로그 수정·제외/로컬 MPD 구조 첨부 | Windows/macOS clipboard 및 전체 UI 흐름,  실제 실패/모든 omission UI 검수; P1/P2 미착수 |

## 다음 gate

S00에서 최종 스택 선택을 위한 OS별 IME/접근성 증거를 확보해야 한다. 구현량으로 이 gate를 건너뛰지 않는다. 그 후 S01/S02 실제 흐름을 검수하고 S03의 guardian 자체 고장·Windows 프로세스 생성·재개 UI와 OS 복구를 완성한다. S04 공급처·license/hash를 확정하여 관리 설치를 연결하고 S05/S06 및 S10 P0를 검증한다.

메타데이터 stdout는 8 MiB, 일반 로그는 256 KiB tail/행 64 KiB로 제한한다. 큰 JSON과 stderr를 섞지 않고 필요한 작은 필드만 남긴다. media stream을 메모리에 적재하지 않는다. hardlink 미지원 filesystem은 실패하며 fallback은 미완료다. Linux private endpoint 준비 시 second instance는 기존 창을 복귀한다. 다른 OS endpoint와 안전한 복귀 준비가 없으면 거절한다.

외부 공개·업로드·실제 계정·서명/유료 리소스를 사용하지 않았다. 기존 .git과 사용자 데이터는 정리하지 않았다. FFmpeg 공급/최소 OS/배포 채널/서명은 남은 결정이다.

## 중단 후 재개 검증

[재개 결과](resume-validation.md)에 실행 명령과 측정을 기록했다. Rust 61 tests, 실제 yt-dlp/FFmpeg 합성 전체·구간/decode 시험 1 test, GUI 시험 안전 회귀 2 tests를 통과했다. 최종 release 빌드와 fmt/clippy 성공. 사용자 요청에 따라 자동 GUI 검증은 Xvfb로 분리하며 실제 desktop을 자동으로 열지 않는다. S00 최종 선택과 P0 전체 출시 게이트는 유지한다.

## 후속 P0 통합 검증

[후속 결과](followup-validation.md): 74 workspace tests, 실제 도구 합성 통합 시험과 가상 GUI 안전 검사. 기본 폴더와 저장 위치 복원·watcher 복구·실제 속도/ETA·Linux interface·Linux 진단 clipboard를 연결했다. 가상 GUI RSS 약 67.5 MiB, idle CPU 약 0.20%의 5초 표본이며 장시간/IME/다른 OS 검증을 대체하지 않는다.

[옵션·외부 FFmpeg 후속 기록](options-validation.md)에 구현과 통합 검증을 기록한다. 관리 설치 및 S03 선행 게이트는 유지하며 전체 S05 완료로 취급하지 않는다.

[Linux 크래시·시작 복구](guardian-validation.md): 소유 도구 자손 정리와 시작 시 journal 대조를 연결했다. 동일 ID 재개 UI는 [후속 검증](job-resume-validation.md)에서 구현했으며 백그라운드·8시간/다른 OS 검증은 남았으며 전체 S03/P0 완료가 아니다.

## 병렬 P0 후속 구현

동일 ID 재개 엔진과 bounded 이력 UI, adapter v4 typed 옵션 저장, library v2 Unix 조건부 rename을 통합한다. [재개 작업](../work-items/S03-job-resume.md), [파일 identity 실증](library-identity.md), [진단 보존 정책](diagnostic-cache-validation.md)을 따른다. guardian 종료 신호는 개선했지만 [SIGKILL 실험](guardian-fault.md)은 보호 미충족이며 출시 차단으로 유지한다. 실제 도구 공급 pin 및 FFmpeg 처리 provider는 병렬 후속 작업 중이다.

[일시 중단 통합](pause-validation.md): Paused/Cancelled 구분과 실제 실행 partial 보존·동일 ID 재개, 137 tests/가상 GUI를 검증했다. [병렬 보드](../work-items/P0-parallel-board.md)에서 현재 파일 담당을 추적한다. 관리 다운로드와 자막 typed/helper는 구현 중 또는 검증됐으나 UI/다중 artifact 완성을 대신하지 않는다. 실제 8시간 headless 시험은 진행 중이며 결과 전 완료로 승격하지 않는다.

## 실제 Windows와 관리 도구 후속

[Windows native CLI 기록](windows-cli.md)은 프로젝트 전용 GNU 도구로 실제 OS에서 초기 실패를 발견한 증거다. Canonical drive prefix와 LockFileEx contention 분류를 수정했고 최종 안정된 snapshot 검증 중이다. MSVC 릴리스·Windows GUI/IME·macOS 검증 완료로 확대하지 않는다. [관리 backend 실증](managed-tools-validation.md)의 공식 pin 실제 HTTPS 설치와 bounded inventory를 네이티브 도구 UI에 연결했으며 전체 GUI 흐름은 [작업 계약](../work-items/S04-managed-tools-ui.md)에서 추적한다.

격리 XIM의 한글 commit/삭제와 외부 입력 창/OnTheSpot 조합 표시를 분리 검사 중이다. 처음 draw 순서 가설은 수정 release에서 시각 가림을 해결하지 못했고 검정 영역은 Fcitx 별도 창으로 확인했다. 미완료 시각 조건을 IME 전체 성공이나 스택 채택으로 표현하지 않는다.

## 최신 UI와 다중 artifact 검증 범위

[다중 결과 작업](../work-items/S05-artifact-batch.md)은 core 묶음 완료, adapter v5 publication JSON(128KiB/최대3개), Unix identity 복구, after_video 자막 경로·확장자 projection과 자막 선택 UI를 연결했다. 실제 기존 yt-dlp의 loopback 자막 projection 및 합성 engine 3개 결과/자막 없음/잘못된 경로 시험을 통과했다. 실제 media+subtitle engine 통합과 가상 자막 UI 조작은 아직 남는다.

[관리 GUI 검증](managed-tools-gui-validation.md)은 exact release hash 범위에서 공식 pin 설치·SHA·활성화·다운로드 경로 적용, 중단과 foreign root 보존을 검증했다. 주 담당은 install JSON과 installed/failure 이미지를 직접 확인했고 정제된 한국어 오류 안내를 확인했다. failure JSON의 자동 GUI verified=false는 OCR을 하지 않았음을 나타내며 수정하지 않는다. 단일 pin 시험은 보관 버전 선택/제거 및 기존 active 있는 GUI 취소 시험을 대신하지 않는다. Windows 제품 suspended/atomic Job 생성은 구현·native 실증했고 주 담당이 코드/증거를 검토했다. 최신 Windows GNU 전체source snapshot4공통검증과 입력47개SHA대조가 통과했다. Windows file identity/GUI/MSVC 게이트는 유지한다.

## 최신 공통 검증 대조

Linux cargo fmt --all -- --check, cargo clippy --workspace --all-targets --locked -- -D warnings, cargo build --workspace --release --locked 성공. 전체시험과 실제미디어시험 결과는 위 기록을 따른다. Windows GNU [최종 증거](windows-process-final.json)는4공통명령exit0와source snapshot불변이며 주 담당이47개입력파일SHA를현재파일에 대조해불일치0개를확인했다. 이후신규soak시험파일추가는별도snapshot이다. macOS/WindowsMSVC/3OS GUI·IME·접근성/실제설치·원격CI를완료로승격하지않는다. 기존guardian장시간시험은5769초/97샘플에서6개정확PIDstartidentity생존을확인했고guardianRSS4620KiB였다. headless guardian범위이며8시간종료및fullengine/media/UI시험을대신하지않는다.

## 자막 수직 기능 통합 범위

[작업 계약](../work-items/S05-artifact-batch.md)과[가상GUI증거](subtitle-ui-validation.md)를대조했다. 자막선택typed옵션→snapshot/argv→after_video작은증거→ownedstaging검증→최대3artifactpublication→SQLite완료/Unix복구를연결했다. 실제기존yt-dlp/FFmpeg합성media+VTT/DBreopen1개및GUI선택/충돌/freeze/autooff를검증했으며구간자막은시작거절한다. 실제사이트extractor/공식pin별fixture/3OSGUI와source linkage는여전히남는다. Windowsnativeidentity는공개와라이브러리연결구현후실제시험진행중이다.

## 보관 도구 관리 GUI 독립 검토

[보존GUI증거](managed-retained-gui-validation.md)의select-remove JSON을직접확인해synthetic보관버전활성화/기존비활성제거/active·외부bytes보존을확인했다. hash PNG에서한국어크기·SHA검증거절과기존active/inactive표시를직접검토했다. 시험용syntheticmanifest/hash는공식도구version실행증거가아니며프로덕션임의manifest입력을추가하지않았다. 공식파일변조는ownedtempstaging만대상이며정확Rusterrorvariant/OCR은관측하지않았다.

## 이전 부분 범위: 통합 대기와 background 숨김·복귀

사용자 요청에 따라 이번 구현 범위를 통합 대기 기능과 백그라운드 UI까지로 제한했다. [숨김·복귀 계약](../work-items/S03-background-ui.md), [가상 화면 증거](background-ui-validation.md), [처리 지표 연결](../work-items/S06-processing-ui.md)을 따른다. Linux private IPC가 준비되면 창을 숨기고 동일 작업을 유지하며 명시 재실행만 기존 창을 복귀한다. 미지원 OS/unsafe·stale endpoint에서는 숨김을 비활성화한다. renderer 자원 완전 해제·트레이·3 OS GUI 전체는 완료하지 않았다.

Linux FFmpeg 전용 FIFO를 processing feed에 연결했으며 실제 합성 전체/빠른·정확 구간/자막/DB reopen 통합1개가42.21초에 통과했다. 네트워크 다운로드 B/s와 처리 배속은 분리한다. 기존 장시간 시험은 이 변경 이전 binary 범위이며 끝나기 전8시간 성공으로 쓰지 않는다. 전체 P0/공개 릴리스 완료는 여전히 아니다.

## 장시간 시험 현재 재확인

과거 기록의 두 /tmp status 파일 및 owner/guardian/worker PID handle이 모두없음을 실제확인했다. 관측타임아웃만으로종료를추정하지않고 /proc 부재로확인했으며8시간완료결과는미검증이다. 원인을재부팅으로단정하지않는다. 새로운소유시험은별도run ID와source snapshot으로추적한다.

## S00 재감사와 Windows 복귀 통합 진행

[현재S00감사](S00-current-audit.md)는최종스택판정을FAIL/미충족으로유지한다. Iced inline한글preedit와실제스크린리더증거가부족하며egui대안은후속실험중이다. Native성능baseline을[별도10초표본](evidence/S00-egui-library-baseline.json)으로수집했다. Windowsrestore용same-user protected namedpipe모듈을공유API에연결했으나현재native컴파일/실증진행중이므로완료로표현하지않는다. Linux기존activation10시험이0.62초에통과했다.

새장시간시험은host9개PID/startticks를주담당이대조해현재running을확인했다. 기록파일만으로실행중이라고주장하지않으며sandboxPID namespace와host의관측을구분한다. [영구증거](evidence/soak-restart-20261002/launch-source-manifest.json)는상한을두고보관하며8시간완료전성공으로승격하지않는다.

## Windows 복귀 native 통합 현재 증거

[Windows native복귀](windows-activation-validation.md)의같은53개input sourceSHA를주담당이현재소스와대조해4명령모두불일치0개,releaseSHA일치를확인했다. native12개IPC시험을포함한전체GNU시험통과는Windows제품GUI/IME와MSVC/macOS인수의완료를뜻하지않는다. privateDesktop제품GUI는독립담당의별도실증중이다.

S00commit-only fallback대조는도착한Commit의위젯epoch를초기화해'한글 '·취소·Latin·선택대체를복구했지만inlinepreedit가없어전체IME판정은FAIL을유지한다. [실제대조](egui-ime-validation.md)와후속Xlib reference로원인을분리한다. 제품default/스택채택은변경하지않았다.

## 공유 Windows 복귀와 후보 실험 이후 Linux 통합검증

현재snapshot에서 cargo fmt --all -- --check, cargo clippy --workspace --all-targets --locked -- -D warnings, cargo test --workspace --locked, cargo build --workspace --release --locked가모두exit0. Linux187passed/4ignored이며release38.57초, 후보alternative clippy(all-targets)54.77초exit0. 제품LinuxreleaseSHA는e9fa3ce36b3e4a1285ef0ee5930179ee4fa126451730fd0cbdcb2bae020bcc7f로기존최종GUI증거와동일하다. 문서130개검사가통과했다.

P0전체인수는미완료다. 현재독립후속은WindowsprivateDesktop제품GUI, Xlibreference preedit callback분리, 실제합성HTTP403/연결중단/동일job재개도구시험이다. 두8시간시험/archive는소유run을유지하며이전에증거없어진시험을성공으로소급하지않는다.

## 전체 P0 재개와 최신 독립 검증

이전 부분 범위를 마무리한 뒤 전체 P0 목표를 다시 진행한다. 최신 Linux4공통 검증은187passed/5ignored이며 [숨김·복귀 최신 증거](background-ui/integration-refresh-background.json)는SHA7e5fe81444a31806189e3a84d51d1ccf809f48bd3c83df1396c9d2fbabacd469에 한정한다. [제품10초성능](evidence/S00-patched-iced-library.json)은9873856B/RSS68404KiB/idle0.0% 단일 WSL2 Xvfb표본이며 장시간 예산 완료가 아니다.

주 담당이 `DLPAD_SMOKE_YTDLP=/home/joongi/.local/bin/yt-dlp DLPAD_SMOKE_FFMPEG=/usr/bin/ffmpeg cargo test -p dlpad-engine --test local_media_failures --locked -- --ignored --nocapture`를 독립 재실행했다. 1passed/23.95초/exit0: HTTP403 후 원본페이지 재추출4회 및 동일Job 새Attempt, 전송단절 후31744B partial에서 실제 Range31744 재개, exact 합성미디어 bytes와 DB reopen을 확인했다. 실제사이트·실제storage ENOSPC·8시간 영상·다른OS 인수는 대체하지 않는다. [전용 시험](local-media-failures.md)을 따른다.

현재 독립 진행은 수정winit의 Iced 제품 IME 실증, vendor 전체source SHA를 포함한 Windows GNU 재검증, 소유 temporary 파일의 실제 EFBIG/명시 injected ENOSPC 실패·재개 시험이다. EFBIG와 실제 디스크부족 errno를 같은 것으로 표시하지 않는다. S00 스택 최종 채택/실제 스크린리더/3OS GUI/실제 설치·서명·원격CI 게이트는 유지한다.

## 수정 입력 경로의 실제 제품 증거

주 담당이 [Iced 제품 preedit 화면](iced-current-ime/full/preedit.png)에서 읽히는 조합 한글/밑줄과 committed-only mirror의 한을 직접 확인하고 [실제 결과](iced-current-ime/full/result.json)의 exactcommit·BackSpace취소·Latin전환·selectionxy 및 GTK Escape 동일성/ownedcleanup을 대조했다. 오래된 고정pixel 임계값의 원실행exit1은 raw-result와collection_exit_code로 보존하며, 화면 검토 기반 preedit/IME 판정을 분리했다. 현재 Linux 한글 입력 gate는 이 특정 Fcitx/Xvfb/폰트/release 범위에서 개선됐다. 실제 스크린리더/3OS입력·접근성 없이는 최종스택을 채택하지 않는다.

[새Windows GNU 기록](windows-winit-validation.md)의278개baseline파일을현재파일과 독립SHA대조해불일치0개,각명령before==after 및4명령exit0,실제releaseSHA85cdad770ad3c1db25c25f3c50d6193636cc8b15b7d705f253b314ef48ffc59b/6807040B 일치를 확인했다. 이후 신규Linux전용disk_failure.rs는 그 snapshot 이후 파일이므로 Windows 검증을 확대하지 않는다.

최신 S10 P0 [진단clipboard](evidence/S10-current-clipboard.json)도 private Xvfb에서 explicit copy/UTF8 2957B/정제sentinel 누락을 재확인했다. 사용자clipboard/계정/외부전송은 사용하지 않았다. 다른OSclipboard·저장·모든omissionUI는 별도게이트다.

## 신규 writer 실패 회귀 포함 최종 Linux 통합

주 담당이 신규 disk_failure.rs를 직접 읽고 fmt/clippy/workspace tests/release를 재실행했다. 모두exit0,189passed/5ignored. 실제 kernel EFBIG와 명시 injected ENOSPC 시험2개가2.02초에 통과했다. 실패 상태·64KiB partial 보존·DB reopen·동일Job 새Attempt·원본재추출·무관fixture보존을 확인하며 actualstorageENOSPC로 표현하지 않는다. [실패 회귀](disk-failure-validation.md). 제품binary는 기존최신hash와동일하며 source신규테스트만 추가됐다. 문서145개검사통과.

## 현재 P0 후속: 테마 복원·실제 접근성·격리 입력

[테마 실제 화면](theme-restore/current/result.json)은 SQLite Light/Dark가 반대legacy값보다 우선하고, legacyDark/초과크기fallback·기존bytes보존·privatebus/GUI회수를통과했다. 최초renderer frame/SystemOS변경은확인하지않았다. 최신fixture-only eventtrace의Linuxfmt/clippy/workspace190passed5ignored/release와Windowsnativebuild가exit0이며새LinuxSHA80c3338775a4c0bcc017fb4ad1ef766012667fc4907f98841095b5576b23efae, WindowsSHA98c48919e79f713c0ecca74882ad4f4d5ad34beec19b0a2778e9f839e3f15ff7다. 원래일반제품 실행은 trace를활성화하지않는다.

[접근성 실제 대조](reader-validation.md)는 Iced 앱tree부재를확인했고egui후보의 실제무음Orca event처리를확인했다. 전체음성/reader사용성/3OS인수와구분한다. 수집기의원판정오류는raw를보존하고review.json으로정정했다. 주담당은최종Iced채택을진행하지않고기존Rust엔진과접근가능한native후보를연결하는 [수직기능](../work-items/S00-accessible-download-slice.md)을착수했다.

[FFmpeg 후보 실증](ffmpeg-supply-validation.md)의실제실행파일별SHA를주담당이64KiBstream으로독립대조해일치했고safeextract self-test7개를재실행했다. SHA확인첫helper는Python3.10에서없는file_digest로실패했으며stream버전재실행이성공했다. 후보두실행파일+license선택용량284847347B;archive137863452B는별도다. LGPL3+/실제codec-cut성공은대응source/notices/공급trust/3OS/managedarchiveactivation게이트를대체하지않으며배포채택은보류한다.

## Windows 실제 작업 유지와 공급 후보 엔진 통합

[Windows 현재 GUI 증거](evidence/windows-background-gui/active-normal-final.json)는 주담당이UTF8 BOM을올바르게읽고 독립대조했다. 일반제품실행(두fixture flag없음)의worker 소유Job/startFILETIME일치·숨김heartbeat5→11·동일worker복귀·primary/secondaryexit0·cleanup·inputdesktopunchangedtrue와trace파일없음을확인했다. [복귀화면](evidence/windows-background-gui/active-normal-restored.png)을직접보아작업진행·한국어GUI를확인했다. 64행anonymous eventtrace는baseline CursorLeft→press ignored, burst press captured→CursorLeft를보였으며payload key/text/URL없음을세필드검사했다. 기존tracefile create-new거절/보존시험도통과했다. 실제IME/로그아웃/절전/실제영상은여전히미검증이다.

주담당은새FFmpeg실행파일을기존engine opt-in합성미디어시험에연결했다. `DLPAD_SMOKE_YTDLP=/home/joongi/.local/bin/yt-dlp DLPAD_SMOKE_FFMPEG=/mnt/d/project/dlpad/.local-tools/ffmpeg-supply-cnu82ffa/selected/bin/ffmpeg cargo test -p dlpad-engine --test local_media --locked -- --ignored --nocapture`가1passed/22.57초/exit0으로전체·빠른/정확구간·자막·완료등록/DBreopen을확인했다. 새로운archive공급채택/배포를승인한시험은아니다.

[Deno 공급후보](deno-supply-validation.md)의안전ZIP/cache 회귀8개를주담당이재실행해통과했다. 오프라인JS/nativeversion/timeout/cancel·전용cache증거는 실제YouTube/EJS통합·3OS/notice/managedactivation 게이트를대체하지않는다. 현재실제후속구현은accessible native다운로드slice 및 해당GUI/reader시험이다.

## 접근성 다운로드 후보 실제 인수와 메모리 게이트

새 dlpad-accessible의 엔진연결·성공·실패정제보고서복사·동일Job일시중단/새Attempt재개·종료CancelWait를 실제Xvfb GUI4개로검증했다. PauseUI의 의도손실을 실제시험으로발견해request_stop(true)로수정했고주담당이새hash55ec...원시SQLite4개/보고서를직접대조했다. [수직slice](../work-items/S00-accessible-download-slice.md), [실제GUI](accessible-download-validation.md), [한글/무음reader](accessible-native-validation.md). 기존Iced제품을제거하거나P0전체UIparity로표현하지않는다.

[대기비용3회실측](accessible-performance-validation.md)은idle0%지만RSS기본196156KiB/llvmpipe138204KiB/softpipe247564KiB로80MiB제안모두FAIL이다. AT-SPI/실제Orcaeventprocessing은전체reader사용성·음성·3OS판정과다르다. budget을완화해최종stack을확정하지않고기존egui0.33.3와AccessKit를유지하는CPUrenderer후속실험을진행한다.

접근성후보 Pause수정뒤 `cargo test --workspace --all-features --locked`는196passed/0failed/5ignored, `cargo build --workspace --all-features --release --locked` exit0였다. 이는optional GPU까지포함한컴파일/단위통합검사이며실제GPU/3OS인수를뜻하지않는다. 이후공유UI추출·CPUfeature추가는별도in-progress snapshot이고모든4명령PASS로소급하지않는다.

## CPU renderer 후속 실제 근거와 새 OS 실패

동일Candidate의CPUhost를구현해실제LinuxRSS49.4MiB/idle0%10초표본과actualsuccess/failure/Pause→새Attempt재개/Close취소4case, 한글·무음Orca를검증했다. [GUI](accessible-cpu-download-validation.md), [입력/reader](cpu-native-validation.md), [비용](accessible-performance-validation.md). 메모리예산을완화하지않고기존Glowdriver/JIT경로를제거한결과다. 주담당이rawSQLite4개/보고서/완료·PausedPNG를직접대조했다.

WindowsGNUdefault4checks116passed/1ignored, CPU선택clippy/tests/release131passed/1ignored를actualprivateCLI로수행했다. 주담당은235개input과runner/binarySHA를직접대조했다. [default](windows-cpu-lock-default.md), [CPUfeature](windows-cpu-feature.md). CPU3OSlint/test/release를CI설정에추가했으나remote실행은없다.

이어WindowsactualCPUGUI가AccessKitadapter생성전창표시로panic했다. host를hidden생성→adapter준비→표시순서로수정했으며새Linux/Windowsbinary실제GUI재검증이진행중이다. 이전CLI녹색을3OSUI완료로확대하지않는다. 후보전체기능parity·실제reader사용성·macOS/MSVC/설치/8시간기타P0게이트는여전히남는다.

## CPU 파일 목록 실제 구현과 통합 (2026-10-03)

CPU 후보에 librarypanel/128개keyset목록/저장경로Apply·Default/nativeWatchworker/명시Refresh·Next 및 소유worker CloseWait를 연결했다. root는기존 engine에 협조scan취소를추가해transaction rollback/기존snapshot보존을2개회귀로확인했다. workerpanic은고정오류와stoppedack를전달하며UIthread unfinished join을하지않는다. [panel](../work-items/S01-cpu-library-panel.md), [스캔취소](../work-items/S01-library-scan-cancel.md), [통합](../work-items/S01-cpu-library-integration.md). 최초Clippy경고수정후CPUentry20시험/Clippy/release통과,Windows native153passed1ignored. 새 Linux/Windows실제GUI검증은진행중이며이전IME/reader/download증거를이새소스로확대하지않는다. 전체P0·최종S00·다른후보기능parity·macOS/설치/8시간게이트는남는다.

## CPU 후보 S10 실제 Linux 통합 완료 범위

기존core/report엔진을재사용해CPU후보에Markdown/JSON·로그edit/개별전체omit·로컬MPD구조attach/omit·copy/create-onlysave·CloseWait를연결했다. [root통합계약](../work-items/S10-cpu-diagnostic-integration.md)과 [실제GUI](cpu-diagnostics-validation.md). root는11원시JSON/FailedSQLite/attachedPNG를독립검토했다. Linux기본193/CPU242tests와release/Clippy를통과했고Windowsfixture이름충돌은AtomicU64+제한retry로수정해167tests/nativebuild를재검증했다. root237inputSHA대조불일치0. Windows/macOS진단GUI/clipboard·2MiBreportstress·최종스택/전체P0는미완료다. 현재Windowsclipboard는별도noninteractivewindowstation으로사용자clipboard를분리할수있는지새ownedcapability실험을진행하며기존WinSta0clipboard에는접근하지않는다.

## Windows private clipboard capability 결과와 설정 후속

[격리 station 실험](windows-clipboard-station-capability.md)은 CreateWindowStationW(NULL,CWF_CREATE_ONLY)가 ERROR_ALREADY_EXISTS183으로 실패했다. 새 station/desktop/clipboard 접근과 제품 GUI 실행을 진행하지 않았고 WinSta0 clipboard 접근도 없다. 기존 station 재사용·관리자 권한·사용자 clipboard 우회는 하지 않는다. Windows S10 복사 인수는 남는다. 독립적인 CPU 테마·배율 기능 이식과 안전한 설정 읽기/종료 대기 통합을 계속 진행한다.

CPU 설정 통합 현재 증거: System/Light/Dark와 session-only75–200% 선택, 창 생성 전 readonlyload, 저장 latestdrain 후 CloseWait를 연결했다. default fmt/clippy/tests197/release 및 CPUclippy/entry38/workspace268/release 모두PASS. WindowsGNU CPU188pass/1ignored과238현재input해시root독립대조PASS. [통합 작업](../work-items/S02-cpu-settings-integration.md). 실제 LinuxGUI 수집기 staticlabel 선택 오류를 수정해 검증 중이며 nativeOSThemeChanged·Windows/macOSGUI/미디어·전체 P0완료를 주장하지 않는다.

## CPU 테마·배율 boundedLinux 인수 완료

최종f052fa5b...의 [설정 GUI](cpu-settings-validation.md)에서테마3개저장/restart·양테마32px선택버튼·접근성선택후popup닫힘·200%entry양방향Tab·잠금중종료drain을확인했고root가raw와PNG를직접대조했다. 한글이동field는원자동collector exit1을보존하며조합PNG의rootmanual review와확정/취소/영문/선택교체관측을분리한다. CPU268/0/5,Windows188/0/1와238현재input/hash직접대조PASS,baselineRSS49.6MiB/idleCPU0.10%10초. System OS 실제변경·Windows/macOSGUI·전체reader·busyDB첫표시와TextEdit21→32px는남는다. 다음P0는typed품질/구간/자막/고급옵션이식이다.

## 2026-10-04 Windows 인증 다운로드 통합 검증

현재 Windows GNU에서 보호된 session cookie backend·Job terminal 확인·중단 가능한 출력 pipe·CPU 후보 인증 worker 연결을 통합했다. 신규 [합성 인증 다운로드 작업](../work-items/S02-windows-authenticated-download-fixture.md)은 root가 실제 native example 빌드 및 DownloadRepository 통합 시험을 실행하여 성공·metadata 실패·download 실패·취소 네 경우 모두 확인했다. 원본 보존, 확인된 reader 뒤 private copy 정리, DB 재열기 및 비밀 생략을 검사한다. [원시 증거](evidence/windows-authenticated-download-first/root-review.json)의 build/test와 CPU workspace all-targets Clippy 모두 exit0, 소스 before/after 및 root 현재 SHA 대조 일치다. 사용자 데스크톱은 변경되지 않았다. 실제 사이트·Windows 인증 GUI·재시작 후 쿠키 복구·최종 S00 스택·3OS 릴리스 인수는 남는다. 전체 P0 완료로 확대하지 않는다.

Windows 현재 CPU release도 native host에서 exit0로 빌드했다. [보존 증거](evidence/windows-cookie-cpu-release-current/root-review.json)는 273개 입력 소스의 before/after/current 일치, 실행 파일 7,285,248 bytes와 SHA256을 기록한다. 이 실행 파일의 실제 인증 GUI 검증은 다음 단계이며 build 성공을 GUI·IME·배포 성공으로 표시하지 않는다.

## 2026-10-04 시작 단계와 쿠키 쓰기 실패 회귀

Windows suspended child의 ResumeThread 직전에 공통 취소/전체 deadline을 확인한다. 중단 시 Job 종료·Child wait·ActiveProcesses=0 확인 후에만 Cancelled/TimedOut을 반환한다. 쿠키 backend는 일부 쓰기 뒤 WriteZero, 전체 쓰기 뒤 주입된 sync 오류, deferred copy writer 거절을 실제 임시 Windows 파일로 검증했다. [통합 원시 증거](evidence/startup-checkpoint-windows-retry/root-review.json)는 CPU workspace 시험/Clippy exit0와 source SHA 일치를 기록한다. sync 시험은 실제 디스크 sync 고장을 유발한 시험이 아니라 오류 주입 시험이다.

Linux admission/readiness 전체 deadline 구현의 첫 engine 시험은 195 passed/2 failed/2 ignored로 [실패 기록](evidence/startup-checkpoint-linux-first/result.json)을 보존했다. 기존 시간 기반 취소 시험이 readiness 이전에 중단되어 typed StartupUnconfirmed를 받았다. 실제 tool 시작을 동기화한 준비 완료 후 취소 시험과 준비 전 미확인 보존을 분리하여 수정 중이며 Linux 통합 완료로 표시하지 않는다. Windows 신규 전체 시험에서 명시 fixture가 필요한 인증 다운로드 시험은 ignored였으므로 기존 별도4mode 검증을 새 소스에 확대하지 않는다.

Linux engine library 재시험은 실제 progress/시작 marker 동기화 취소 및 preR 종료 미확인 보존 회귀를 포함하여 198 passed/0 failed/2 ignored, exit0를 확인했다. [증거](evidence/startup-checkpoint-linux-lib-retry/result.json)의 process.rs SHA는 인계 당시 동결 소스와 일치한다. 첫 실패 기록은 유지하며 공개 noR·현재 서비스 cgroup·전체 workspace·GUI 인수는 별도다.

공개 Linux startup interrupt [실제 4case](evidence/startup-interrupt-native-first/root-review.json)도 통과했다. noR150ms timeout·launcher 관측 후 취소·partial E 취소·metadata noR timeout에서 typed StartupUnconfirmed와 빠른 socket EOF를 유지하고 정확 owned child 회수를 확인했다. cookie source/copy와 실제 user-service cgroup을 직접 시험한 결과는 아니며 해당 인수는 별도다.

최신 시작 단계 소스에서도 Windows 인증 다운로드 별도4mode(success/metadata-failure/download-failure/cancel)를 재실행하여 모두 PASS, native exit0·sourcebefore/after/current 일치를 확인했다. 신규 공개driver 포함 Linux engine all-targets Clippy도 exit0다. [증거](evidence/startup-checkpoint-authenticated-regression/root-review.json). GUI·실제 사이트·재시작 recovery 인수는 별도다.

## 2026-10-04 출력 상한과 인증 GUI 현재 통합

최신 guardian helper 명시 재빌드 후 Linux engine 전체 회귀는 exit0. library201/0/2와 통합 시험을 [root 검토](evidence/guardian-reader-drain-integration/root-review.json)에 보존했다. 새 reader record/codec8시험과 Linux all-targets Clippy, workspace fmt check도 exit0이나 실제durable 복구 연결은 미완료다.

Windows [합성 인증 GUI 성공](evidence/windows-cookie-gui-auth-success-input-retry/root-review.json)은 binarySHA2049eca… 고정snapshot에서 ownedWM_CHAR 입력/readback, cookiefieldclear, Downloaded/1Attempt/40Bartifact, versioncookie없음·metadata/download동일copy, 정상copy정리·원본/sentinel보존·DB비밀생략과 사용자desktop불변을 root가 rawSQLite/files/PNG로확인했다. 이전UIA/input실패원시는보존했다. 실패·취소GUI, 최신Linuxguardian snapshot과다른OS/사이트 인수는별도다.

## 종료 회귀·journal 실제 통합 결과 추가

Linux 실제 user-service terminal22case의 raw RGCLC/EOF와 소유 PID/start78개·cgroup14개 소멸을 root가 재확인했다. [증거](evidence/reader-drain-native-terminal-first/root-review.json). cookie_reader_models Cargo 시험은 실제 anchored journal5개와 model8개 총13개 PASS다. journal은 아직 제품 launch/복구 경로에 연결하지 않았으므로 same-boot 복구 완료가 아니다. Windows immutable CPU 후보의 metadata 실패 GUI는 실제 Failed 상태·attempt1개·artifact0·private copy 제거를 확인했다. download 실패 입력 재시험은 실제 readback0으로 실패했으며 원시 증거를 보존하고 수집기를 수정 중이다. 전체 P0는 미완료다.

Windows immutable CPU 합성 GUI download-failure 및 cancel 후속 시험도 root 독립 검토를 통과했다. Failed/Cancelled 상태와 attempt1/artifact0, 취소 .part 보존, private cookie copy 제거·원본/sentinel 보존·DB 비밀 누락을 확인했다. [취소 증거](evidence/windows-cookie-gui-auth-cancel-key-envelope/root-review.json), [다운로드 실패 증거](evidence/windows-cookie-gui-auth-download-failure-key-envelope/root-review.json). report UI 및 pending restart와 최신 전체 source GUI는 별도다.

## Linux 제품 cookie journal anchor 연결

import의 별도 journal exact anchor를 recovery record2/marker digest에 연결하고 fixed owner entry·실제 proof를 검증한 뒤 소유 journal 파일만 정리한다. v1 호환·교체/추가receipt 보존·active lock 복구보류를 확인했다. 전체 engine library223 및 integration/strictclippy 통과: [검토](evidence/cookie-journal-owner-integration/root-review.json). 같은boot 기록은 계속 보존하며 실제I/B launcher와receipt writer 연결은 미완료다. Windows 보고서 readonly 접근성 오류를 제품 수정했고 LinuxCPU check는통과했으나 새Windows 실제UIA검증은 진행 중이다.

## Windows 보고서 읽기전용 접근성 수정 실제 결과

새CPU e2712ef9…/7,286,784bytes에서 UIA readOnly=true, preview 읽기/SetValue무반영, 미리보기와새export동일SHA/1478bytes, 비밀생략·sentinel덮어쓰기거절·DB/쿠키정리를root독립검토했다. [증거](evidence/windows-cookie-gui-auth-report-readonly-scope-retry/root-review.json). 테스트는소유private창높이1700/UIA에한정하며원래960x640scroll/clipboard/실제IME는미검증이다. privatePNG는물리화면이하clipped되어전체보고서렌더링을증명하지않는다. CPU+alternative 앱clippy/시험exit0.

## 2026-10-04 쿠키 복구 잠금 전 검증 회귀

[잠금 전 복구 기록 검증](../work-items/S02-cookie-recovery-before-busy.md)을 구현했다. 실제 busy journal을 유지한 hardlink 변조 회귀1개와 전체 engine library231개/guardian9개 등 integration, engine all-target clippy가 통과했다. 정상 busy는 정리 보류이며 변경 파일은 보존한다. 이는 same-boot launcher receipt 연결이나 전체 P0 완료가 아니다. 기존 moved leader TerminalUnconfirmed 최초 실패는 별도 진단 기록을 유지하며 이번 전체 회귀에서는 재현되지 않았다.

## 2026-10-04 실제 8시간 종료와 정상 크기 보고서 가시화

[독립 terminal 검증](evidence/soak-terminal-independent-20261004/root-review.json)에서 download28801.956초/guardian28804.093초, runner exit0와 소유11PID 부재, SQLite integrity/Downloaded/합성artifact/partial을 확인했다. 이전 interrupted archive는 terminal 읽기와 복사 사이의 경쟁으로 마지막 runner proof를 놓쳤으며 원시를 유지했다. 당시 고정1669 headless 합성 backend의 증거이고 최신제품/GUI/실제영상/네트워크/인증 장시간 증거가 아니다.

[정상960x640 보고서](../work-items/S10-report-focus-scroll.md)는 새 Windows immutable af89d78 snapshot에서 header→preview 실제본문2줄→저장경로의 순차 가시성, readonly/전체1478Bexport/privacy/덮어쓰기거절/cookie회수/Job0를 root가 직접 확인했다. Collector REVIEW_REQUIRED 원시는 유지하고 독립 screenshot 검토로 scoped 인수했다. Clipboard/실제IME/다른OS/전체P0는 남는다.

현재 [workspace 검증](evidence/cookie-report-workspace-20261004/root-review.json)은 기본 fmt/clippy/test/release 네 명령과 CPU test/clippy/release를 모두 exit0로 확인했다. engine233/reader models33/CPU app75 등이 통과했으며 source manifest는 종료 후에도 일치했다. ignored native fixture·Windows 신규capability 실행·macOS·전체P0는 별도다.

## FFmpeg 두 실행 파일의 실제 버전 선택

[선택 계약](../work-items/S04-ffmpeg-version-selection.md)의 구현과 Linux 실제 공식 FFmpeg/ffprobe -version 검사·active 전환·active 삭제 거절을 검증했다. [공식 fixture 결과](evidence/ffmpeg-v2-verified-selection-first/result.json) PASS/exit0, 원 archive SHA 보존. 순수 parser3 및 실제 합성 process 실패 보호1시험 통과, engine all-target Clippy exit0. 전체 최신 workspace/Windows/macOS 실행·관리 GUI·공급 license/source/notice 채택은 별도다. 전체 P0 완료로 표시하지 않는다.

## 관리 버전 선택의 취소 경계 보강

FFmpeg 선택 추가 후 Linux 기본/CPU 전체9operation 및 WindowsGNU CPU4operation이 PASS였고 source현재hash 독립대조도 일치했다. 이후 [취소 경계 작업](../work-items/S04-managed-selection-checkpoints.md)에서 Deno/FFmpeg의 전체 component SHA 검사에64KiB별 checkpoint와 commit 직전 취소 검사를 추가했다. library76PASS/1ignored 및 strictClippy PASS, 실제SQLite write busy 뒤 취소가 active rollback을 유지하는지 확인했다. 보강 이후 최신 전체/Windows/GUI 검증은 후속이고 공급 채택/전체P0 완료는 아니다.

## 관리 runtime 네이티브 화면 연결 진행

[작업 계약](../work-items/S04-managed-runtime-ui-integration.md)에 따라 CPU/accessible 도구 화면에서 yt-dlp·Deno·FFmpeg를 구분하고 runtime 검증 선택·사용·비활성 제거를 연결했다. 선택 결과는각 다운로드필드로전달하며CloseWait/작업중변경을거절한다. 공급미채택 runtime설치는disabled 및중앙안내를표시한다. 목록과선택전package조회는metadata만읽고명시선택/사용은actualversion+fullSHA를검사한다. CPUentry80시험·workspaceCPUClippy·CPUrelease PASS, actualLinuxGUI는수집중이다. 전체P0/공급채택/Windows·macOSGUI완료는아니다.

## 관리 runtime Linux GUI scoped 통과

[GUI인수](../work-items/S04-managed-runtime-gui-validation.md)에서현재CPUrelease로Deno/FFmpeg합성실제version확인·active전환·Use→각다운로드fieldliteralpath전달과version불일치prioractive/원file보존을통과했다. 빈목록·3tool전환·runtime설치disabled/pending과tool별설명도확인했다. 수집기footer/role/동일label/diagnosticdict오류의원실패를보존했다. 현재WindowsGNUCPU4전체명령및380sourcehashroot대조PASS. 자동공급notice/실행수명lease/Windows·macOSGUI/최신Linux전체workspace는후속이며전체P0완료는아니다.

## 관리 실행 잠금 기반 구현

[공유 실행lease](../work-items/S04-managed-execution-lock-foundation.md)는기존rootinstall/mutationlock과호환되는sharedcapability이며Linux실제잠금경합·외부symlink보존2회귀를통과했다. 여러실행소유자가존재하면mutation을거절하고마지막lease종료후에허용한다. [실행수명계약](../work-items/S04-managed-execution-lifetime.md)의guardian별descriptor/handlehandoff·TerminalUnconfirmed·appcrash보호와다운로드연결은후속이다. 기반API를사용자실행중삭제보호완료로표현하지않는다.

## 관리 실행 durable 보류 연결

[실행 보류 통합](../work-items/S04-managed-execution-pending-integration.md)을CPU/accessibleworker의원options/resume관리경로에연결했다. 실행전에create-only기록을sync하고확정reader정리뒤만clear한다. Drop/cleanup미확인/변경marker는보존하고다른manager의관리mutation은ExecutionPending으로거절한다. marker/lease8·CPUentry81시험및CPUworkspaceClippyPASS. actualprocesscrash/GUI/자동기록회수/Windowscurrent인수는후속이며전체P0완료는아니다.

## 관리 실행 pending Windows 수정 검증

원 Windows OpenOptions write flag 누락을 실제 수정한 뒤 [새전체검증](evidence/execution-pending-windows-open-flags-retry/result.json)의 fmt/Clippy/test/release가모두exit0/PASS였다. root의 각명령source before/after/current SHA대조불일치0, Linuxlease/marker8회귀재실행PASS. 두이전실패raw는보존하며앞선문자열수정미적용을workitem에정정했다. actualappcrash/미확인cleanup/자동pending회수/WindowsGUI는미완료다. 잔여FFmpeg notice/source17파일의실제SHA/bytes도root독립대조PASS이며세전체source와transitive자료는후속이다. 전체P0/공급채택완료가아니다.

## 관리 pending 실제 프로세스 강제 종료 scoped 인수

root가 신규example/probe코드를읽고 [독립재실행](evidence/managed-execution-crash-linux-root-repeat/result.json)을수행했다. 정상finish/actualSIGKILL두casePASS이며원Child exit0/-9와freshmanager clear/ExecutionPending을확인했다. fixture원binarySHA와수집SHA를대조했고원설치payload SHA가외부source SHA와일치하고sentinel도보존됐다. 이시험은도구reader를spawn하지않는Linuxapp-like scope이며실제다운로드crash/guardianhandoff/자동pending회수/Windowscrash를대체하지않는다. 최신Linux기본/CPU전체collector는진행중이다.

## Windows 관리 실행 original-handle 강제 종료 첫 인수

root가observer/probe코드를읽고 [첫native결과](evidence/windows-managed-execution-crash-first/native-result.json)를원fixturebinary/source/payload/sentinel현재SHA와독립대조해일치했다. build/observercompile/native명령exit0및privateinputdesktopunchanged를확인했다. 정상F/exit0뒤clear,원HANDLE TerminateProcess73뒤freshmanager ExecutionPending/pending60bytes가검증됐다. 이후ready출력boundedread보강observer는새evidence재검증중이므로첫결과를최신수집기전체검증으로확대하지않는다. 실제제품다운로드reader/guardianhandoff/자동pending회수는미완료이며Windows최초릴리스P0후속이다.

## Windows 강제 종료 수집기와 공통 전체 검증 종료

[세번째Windows시험](evidence/windows-managed-execution-crash-third/result.json)은boundedready및collectorbefore/after/currentSHA일치로2nativecasePASS를확인했다. root가원fixture·source·설치payload·sentinel의현재SHA와직접대조했다. normal0/clear,originalHANDLE terminate73/pending이며actualreader/자동회수는별도다. 이미진행중이던 [Linux공통전체검증](evidence/execution-pending-linux-workspace-current/result.json)은9operation전부exit0/PASS로종료했고root의source_final/currentSHA대조도일치했다. Windows최초릴리스범위변경뒤새LinuxGUI/OS인수는시작하지않는다.

## Windows 공식 FFmpeg mapped ZIP 실제 설치 첫 통과

[root검토](evidence/windows-ffmpeg-mapped-zip-install-first/root-review.json)에서공식171535354byteZIP의268800995byte선택세트를Windowsnative임시root에비활성설치/삭제및정확version활성선택/active삭제거절했다. 세명령exit0/sourceunchanged/privateinputdesktopunchanged,원ZIP현재SHA일치다. layout4·ZIP10회귀/enginealltargetstrictClippy도PASS. 대소문자ancestor보강뒤최신재검증/공급notice채택/GUI자동설치/관리reader복구는후속이다.

## Windows managed marker private ACL 실제 실패 인수

공통SID/strictACLhelper와관리markerprivateSD를통합했고 [ACL변조재검증](evidence/windows-managed-marker-acl-tamper-readcontrol-retry/result.json)에서전체4checksPASS/currentSHAroot대조일치했다. 원temporarymarker를실제NULL/unprotectedDACL로바꾼뒤clear거절/원recordbytes/foreignsentinel보존을확인했다. 최초시험handle권한ERROR5는원실패로보존하며READ_CONTROL보정뒤actual변조성공을확인했다. readerjournal/actualreceipt/자동회수및GUI인수는계속진행중이다.

## Windows runtime 빈목록 GUI scoped 인수

root가 [원GUI결과](evidence/windows-runtime-tools-third/gui/native-result.json)와고정c69fd4binarySHA를직접대조하고runtime-2 PNG를읽어FFmpeg탭/공급준비안내/빈목록/설치disabled를확인했다. 실제3tool왕복/WM_CLOSE/원Jobempty/privatedesktop종료/inputdesktop유지PASS다. 시작목록갱신전collector기대오류의첫2실패raw는보존한다. 선택Use/버전불일치후prioractive와field전달은별도합성GUI시험진행중이며이범위완료로확대하지않는다.

## Windows 관리 reader 수명 통합 진행

Windows 관리 실행 시작에 원 journal/8 receipt 생성·v3 marker sync와 원 파일 정리 후 재사용을 연결했다. 별도 관리 headless entry/runtime과 삭제권한 없는 guardian journal을 통합했다. [통합 계약](../work-items/S04-windows-managed-reader-lifecycle-integration.md)의 Windows GNU 전체 fmt/clippy/test/release를 실제 통과했고 최종 소스 SHA를 대조했다. 실제 다운로드 실행 adapter 및 startup 자동 회수 연결은 남는다. 기존 v2/부분 기록 보존과 전체 P0 미완료를 유지한다.

## 관리 capture 실제 helper 연결

[실제 helper 인수](../work-items/S04-windows-managed-capture-native.md)에서 새 reader Intent, 실제 helper/도구 creation identity, 독립 원 journal 잠금, 원 Job 종료, 실제 receipt와 Terminal/EOF 대조를 연결했다. Windows 격리 합성 executable의 metadata·성공·exit 7 실패·취소·정리 후 재사용이 통과했고, 마지막 수정의 Windows 전체 fmt/clippy/test/release 및 현재 소스 SHA를 독립 확인했다. 제품 DownloadSession의 모든 yt-dlp/FFmpeg/Deno 호출 전달과 동일 Job의 쿠키 결합, startup 자동 회수는 후속이다. 자동 회수를 일찍 활성화하지 않았으며 전체 P0는 미완료다.

## 제품 managed 실행 세션 전달

앱 worker의 DownloadSession에 managed capability를 전달하고 yt-dlp/FFmpeg/Deno version probe와 비인증 metadata/download를 새 guardian 경로에 연결했다. [공개 다운로드 fixture](../work-items/S04-managed-download-native.md)는 Windows 실제 공개 Repository의 합성 version→metadata→download, DB Downloaded·artifact·원 namespace 정리·재사용을 통과했으며 전체 Windows GNU fmt/clippy/test/release와 현재 SHA도 대조했다. 실제 Deno/FFmpeg 지정 및 mixed 인증 metadata/download의 paired runtime은 남는다. paired 순수 형식 검증 4개만으로 OS 종료 권한을 주장하지 않는다. 모든 reader 결합 전 startup 자동 회수는 비활성이다. 전체 P0는 미완료다.

## 인증·관리 결합 성공 경로

제품 mixed 인증 fallback을 단일 원 Windows Job의 paired capture로 교체했다. [공개 paired fixture](../work-items/S04-paired-download-native.md)의 실제 합성 성공, 두 receipt, 인증 출력 생략/DB privacy, cookie/source 보존·소유 기록 정리·재사용을 검증했고 최신 Windows GNU 전체 fmt/clippy/test/release 및 SHA를 독립 대조했다. 실패·취소·부모 crash와 startup 자동 회수는 후속이며 자동 회수는 아직 활성화하지 않았다. 이전 mixed fallback 미완료 절은 이 성공 연결 전 snapshot이며 전체 P0 미완료는 유지한다.

## 인증·관리 결합 실패·취소 실제 인수

[실패 인수](../work-items/S04-paired-download-faults.md)에서 Windows 공개 합성 다운로드의 metadata 실패41, download 실패42, actual reader 취소를 검증했다. Failed/Failed/Cancelled와 artifact0, 양 도메인 원 receipt·정리·재사용 및 인증 생략/원본 보존을 주 담당이 DB/파일/SHA로 독립 대조했다. 최초 전체 Clippy의 fixture 조건문 경고를 수정하고 최신 전체 검증은 재실행 중이다. 부모 crash와 자동 startup 회수 및 전체 P0는 미완료다.

위 실패·취소 인수의 최종 Clippy 수정 후 Windows GNU CPU 전체 네 검증과 세 actual native case 재실행 모두 PASS. 주 담당 final/current source SHA 및 DB/원본/receipt 독립 대조도 통과했다. 부모 crash·startup 회수와 전체 P0 미완료는 유지한다.

## Windows paired 부모 종료와 관리 원 기록 회수

[실제 종료 인수](../work-items/S04-paired-parent-crash-fixture.md)에서 공개 인증·관리 다운로드 중 원 부모 HANDLE만 종료하고 두 원 receipt/독립 회수·소유 기록 부재·재사용·DB Interrupted·원 Job 자연 종료를 확인했다. 주 담당 source/원본/receipt/DB를 독립 대조했다. [관리 복구](../work-items/S04-managed-startup-recovery-integration.md) 실패 보존 8회귀와 당시 Windows GNU CPU 전체 네 검증 PASS다. 앱 시작 worker 호출 연결은 최신 검증 중이며 실제 GUI crash·절전/재부팅·전체 P0는 미완료다. 앞선 자동 회수 미구현 절은 이 API 구현 전 snapshot이다.

Windows 앱 시작 및 이전 작업 재확인 worker에 관리 회수를 연결했다. managed/cookie 보류 상태를 별도로 표시하고 CloseWait/다른 worker 중 재확인을 거절한다. 최신 Windows 전체 네 명령 및 호출 회귀 5개 PASS, root source SHA 대조도 일치한다. 실제 GUI 시작 회수와 남은 전체 P0 인수는 후속이다. 이전 자동 회수 OFF 기록은 이 실제 API 인수/앱 연결 이전 snapshot이다.

## Windows 실제 GUI 시작 복구 인수

[GUI 관측기](../work-items/S04-managed-recovery-gui-observer.md)의 private desktop에서 completed 자동 회수와 실제 Interrupted 이력, pending/malformed 재확인·원 파일 보존·시작 차단을 실제 확인하고 DB/원본 SHA/정리 이름을 독립 대조했다. completed의 관측기 .NET long-path 실패는 보존하고 짧은 새 합성 경로에서 재통과했다. pending 안내가 처음 화면에 보이도록 status에 보강했으며 최신 Windows 전체 네 검증 PASS다. 새 pending 화면 재확인은 진행 중이며 전체 P0/최종 스택·실제사이트·패키징 인수는 미완료다.

보류 status 보강의 최신 실제 pending-visible GUI 재실행 PASS. 첫 viewport 경고를 직접 확인하고 원 SHA·재확인·Job 자연 종료와 최신 Windows binary/source SHA를 대조했다. 완료/손상 사례는 앞서 고정 709de3 scope이고 이번 변경은 pending 안내에 한정된다. 전체 P0/실제사이트/IME·배포 게이트는 후속이다.


### Windows 관리 runtime 전파 실제 합성 인수 (2026-10-04)

[작업 및 root 검증](../work-items/S04-windows-managed-runtime-download-native.md): 지정한 Deno/FFmpeg의 guardian version probe3회와 metadata/download 동일 argv, Attempt 구성요소 버전을 실제 Windows public 다운로드에서 검증했다. 성공 Downloaded/artifact1/receipt5, Deno 버전 오류 Failed/artifact0/receipt3 및 원 기록 정리·재사용·원본 보존을 확인했고, 최신 Windows GNU CPU fmt/clippy/test/release 모두 통과했다. 합성 실행 파일을 사용한 증거이며 실제 공식 runtime 기능/notice·source 공급 채택/실제 사이트/전체 P0 완료는 아니다.


### 복구 재시도 가용성 및 공식 Deno 오프라인 발견

[재확인 버튼 수정](../work-items/S04-recovery-retry-availability.md)은 다른 panel worker가 실행 중인 도구 화면에서 활성 버튼을 눌러도 무반응이던 조건을 recover handler와 통일했다. Windows fmt/clippy/test/release를 fresh source에서 통과했다. 실제 살아 있는 creator lease의 GUI 보존/해제 후 복구와 CloseWait GUI는 후속이며 단위 회귀를 GUI 인수로 대신하지 않는다.

[공식 Deno 오프라인 시험](../work-items/S04-windows-deno-offline-native.md)은 version/help/runhelp 및 로컬 합성 JS 계산·명시 deny 권한검사의 실제 네 명령 exit0을 확인했다. [root 부분 검토](evidence/windows-deno-offline-native-explicit-deny/root-review.json). collector는 cache32MiB 상한 초과로 실패했으며 통과로 바꾸지 않았다. cache의 node_compat_bin/node.exe가 원 Deno exe의 hardlink로 관측되어 논리 크기와 물리 중복을 구분해야 하고, 실행 후 nlink 증가가 관리 설치의 strict nlink1과 충돌하는 정책 검증이 남는다. 이 결과를 공급 채택·yt-dlp EJS·사이트·전체 P0 완료로 확대하지 않는다.


### 공식 Deno node shim opt-out의 실제 Windows 인수

[전용 작업](../work-items/S04-windows-deno-node-shim-optout.md)의 fresh 원소유 exe copy에서 DENO_DISABLE_NODE_SHIM=1을 적용해 native 파일 ID·link count1 전후 유지, SHA 동일, node.exe cache 부재와 논리 cache250,768바이트를 확인했다. [root 독립 대조](evidence/windows-deno-node-shim-optout-first/root-review.json). 실제 오프라인 계산·명시 권한 거부도 통과했다. 제품 guardian의 typed 환경 전파·소유 DENO_DIR·실제 yt-dlp EJS 호환과 공급 notice/source 채택은 별도 미완료다. Live creator fixture 읽기 권한 수정 후 최신 Windows 전체 네 검증은 통과했지만 GUI 관측기의 .NET4 verbatim 경로 처리 오류를 수정 중이며 실제 live GUI 완료로 표시하지 않는다.


### live creator lease 실제 GUI 보존·해제 인수

[fixture](../work-items/S04-managed-live-recovery-fixture.md)와 [GUI 관측기](../work-items/S04-managed-live-recovery-gui-observer.md)를 통합해 실제 Windows private desktop에서 살아 있는 원 creator lease의 startup pending/재확인 보존과 명시 release 후 done/원10기록 제거를 인수했다. root가 PNG·원 creation·보존 SHA·실제 부재·자연 Job0/input desktop 불변을 독립 대조했다. 첫 두 실패를 보존했고 최신 source Windows 전체 네 검증도 통과했다. 실제 tool/helper Busy와 CloseWait GUI, Deno typed guardian 환경·EJS·공급채택 및 전체 P0는 여전히 후속이다.


### Windows typed Deno 환경 spawn·helper wire 통합

[구현과 진단 근거](../work-items/S04-runtime-environment-propagation.md): 실제 inherited 선두 '=' 특수키 거부를 red loop로 재현 후 보존 규칙으로 수정했고, 원소유 Windows 자식의 한글 cache 경로/고정 optout 전달 및 관리·paired·cookie typed wire 왕복을 검증했다. 임시 계측을 제거한 현재 source의 fmt/clippy/test/release 모두 PASS. production None를 유지하며 helper Some 실행·원 owned cache factory·정확 관리 Deno 분류는 후속이다. 실제bundled EJS preprocessed API는 scoped PASS이지만 AST/player/사이트 검증이나 전체 P0 완료로 확대하지 않는다.

## Windows 관리 Deno 분류·소유 cache 기반 최신 통합

Windows 최초 릴리스 범위에서 분류 모듈과 소유 cache 모듈을 통합했다. [Windows 전체 fmt/Clippy/test/release](evidence/windows-deno-owned-classification-cache-corrected/result.json)는 모두 exit0이고 [root 독립 검증](evidence/windows-deno-owned-classification-cache-corrected/root-review.json)은 현재 소스 SHA/CPU artifact 및 새 native 회귀9개 성공을 대조했다. engine library292PASS/2ignored다. 명시 비활성 관리 Deno, 외부 보존, 변조/취소, Windows 원 owner/main hardlink, canonical/plain/case 내부 미존재 경로, cache 생성/reopen/Busy/partial·unknown 보존/논리 예산 보고를 확인했다. 최초 native compile import 오류 증거는 보존했다. 상세 범위는 [분류](../work-items/S04-managed-deno-classification.md)와 [cache](../work-items/S04-owned-deno-cache.md)를 따른다.

다음 인수는 이 원 cache 권한을 실제 guardian 종료까지 유지하는 capability handoff와 정확히 관리된 Deno에만 version/metadata/download 환경을 적용하는 생산 연결이다. 현재 Some 사전 거절은 유지된다. 실제 Deno payload ACL/종료미확인 보존/전체 EJS·공급source/notice, Windows GUI·IME/접근성/진단·설치 등 전체 P0 게이트는 남는다. Linux/macOS 신규 인수는 Windows 완료를 차단하지 않는 후속 범위다.

## Windows 원 cache handoff 최신 통합

[원 cache/helper 수명](../work-items/S04-owned-deno-cache-native-handoff.md)을 구현하고 [전체 Windows GNU 검증](evidence/windows-deno-cache-handoff-workspace-first/result.json)의 fmt/Clippy/test/release를 통과했다. engine304PASS/3ignored이며 [root 독립 대조](evidence/windows-deno-cache-handoff-workspace-first/root-review.json)에서 현재소스/CPU artifact를 확인했다. [실제 독립 cookie helper Some 시험](evidence/windows-deno-cache-native-handoff-corrected/result.json)은 explicitunlock/HelperAck/원 app HANDLE close 뒤 Busy/실제 child 환경/원 receipts/자연 helper exit 뒤 재획득을 확인했다. 원 app processkill 또는 managed/paired Some의 실제 인수는 아니다. cache 실패 회귀11PASS/1ignored를 통과했으며 별도 reparse 명시 시험은 symlink 생성 capability 부족으로 exit101을 보존했다.

생산 관리 Deno factory/capture/download 연결과 실제 부모kill/경쟁/모든 helper Some·EJS/공급·Windows GUI/진단/IME/접근성/설치 게이트가 남는다. 현재Some 사전거절은 유지하며 typed attachment/원 lock만으로 전체 P0 완료를 주장하지 않는다.

## Windows cache junction 실제 인수와 최종 snapshot

[새 명시 junction 시험](evidence/windows-deno-cache-junction-native-first/result.json)은 원 synthetic empty-directory HANDLE에 native mount-point 생성/GET readback→inventory Changed 거절→owner·target bytes/원 IDs 보존→원 junction만 no-follow 정리를 실제 Windows에서 통과했다. 심볼릭 링크 생성 실패를 성공으로 변경하거나 시스템 권한/설정을 바꾸지 않았다. [현재 source 전체 네 검증](evidence/windows-deno-cache-handoff-junction-workspace/result.json)은 fmt/Clippy/test/release exit0이며 engine304PASS/4ignored다. [root 대조](evidence/windows-deno-cache-handoff-junction-workspace/root-review.json)는 현재 source/CPU artifact/actual helper HANDLE-close 및 명시 junction 성공을 확인했다. 이전 handoff-workspace-first snapshot 뒤 추가된 회귀를 포함한 최신 근거다.

[관리 Deno 환경 factory](../work-items/S04-managed-deno-environment-factory.md)는 분리된 임시 소스로 준비됐지만 프로젝트 통합·native 인수·capture/download 적용 전이다. 이 factory의 계획/초안 및 전체 테스트 성공을 생산 기능 완료로 표시하지 않는다. 최초 Windows P0/S10 전체 및 릴리스 게이트는 여전히 미완료다.

## Windows 관리 Deno 제품 환경 연결 진행

앞선 factory staged/Some 사전거절 기록 이후 [제품 연결](../work-items/S04-managed-deno-product-environment.md)을 실제 통합했다. 앱 worker→공개 DownloadSession→Deno version/metadata/download와 세 helper에 원 cache authority/binding을 전달한다. 실제 Windows 합성 공개경로5case 및 factory3개는 통과했고 원 receipt/owner 보존을 주 담당이 대조했다. 첫 전체 Clippy의 새 fixture2건 수정 뒤 현재 전체 검증은 진행 중이다. 실제 Deno/EJS/site·GUI·부모kill·공급/설치와 전체 P0는 미완료다.

### 관리 Deno 제품 연결 최신 확정

[최신5개 합성 공개경로](evidence/windows-managed-deno-product-corrected-20261004T121739Z/result.json)와 [현재 source Windows 전체4명령](evidence/windows-managed-deno-product-workspace-corrected-20261004T121546Z/result.json)을 통과했다. 두 root-review에서 원 source/CPU artifact 및 receipt/owner/pending을 독립 대조했다. factory와 제품 Some 연결의 앞선 미통합/사전거절 상태는 이 결과로 갱신한다. 실제 공식 Deno helper/EJS 및 Windows S10 격리 클립보드 인수를 다음 병렬 작업으로 준비한다. 전체 Windows P0 완료는 아니다.

## 공식 Deno helper와 원 proof 실패 경계

[공식 Deno 관리 helper](evidence/windows-official-deno-managed-helper-20261004T123146Z/result.json)는 실제 Deno2.9.7 version/권한거절 JS/bundled EJS preprocessed API 및 원 receipts3개·native ID/link1·cache/node shim 보존을 통과했다. 실제 사이트/player·공급 notice완비/installer 인수는 아니다. 원 root 혼합과 factory 이후 notice 변조의 journal 전 거절을 red로 재현 후 보강했다. [공개 DownloadRepository 사전검증3개](evidence/windows-managed-download-preflight-20261004T123432Z/result.json)도 첫 도구 전 exact Policy/pristine finish로 통과했다. 최신 전체 및 public5case 재검증은 진행 중이다. S10은 실제 clipboard 격리 station capability 부족을 보존하고 복사 없는 저장·정제 native GUI 시험을 다음으로 준비한다.

### 원 proof gate 최신 통합 확정

[현재 source Windows4명령](evidence/windows-managed-download-preflight-workspace-20261004T123512Z/result.json)과 [공개5case](evidence/windows-managed-deno-proof-download-20261004T123457Z/result.json)가 모두 통과했다. 각 root-review에서 원 source/release artifact 및 actual receipt/owner/pending을 대조했다. 관리 환경 root 혼합·factory 후 변조 결함은 수정 인수를 완료했으며 앞선 '검증 진행 중' 기록을 이 결과로 갱신한다. S10 copy 없는 실제 저장·정제 GUI runner 통합이 다음 작업이다. 실제 clipboard station/IME·접근성/사이트·공급/설치·전체 P0 인수는 미완료다.

## Windows 후속 통합·실측 실패 구분 (2026-10-04)

[진단 control 통합](../work-items/S10-report-all-controls-focus-scroll.md)은 Windows 전체 fmt/Clippy/test/release와 현재 source·release SHA 독립 대조를 통과했다. 새 private GUI는 초기 JSON 및 Markdown 저장까지 성공했으나 원90초 worker 전체 budget을 소진해 후속 편집·첨부 전체 인수는 실패다. UIA provider hang으로 단정하지 않으며 시나리오별 계측과 독립 fresh fixture 분할을 준비한다.

[Windows 대기 실측](../work-items/S00-windows-idle-native-cost.md)의 `windows-idle-initial-acl-20261004T132056Z`는 신규 private ACL 생성 및3회 GUI ready를 통과했으나 Job active2가 app-only 조건과 충돌해 측정 전 실패했다. CPU/RSS 예산 통과나 초과를 주장하지 않는다. 소유 Job 멤버의 원 identity 확인 및 app/guardian 전체 비용 측정이 후속이다. 세 회차 모두 원 프로세스 종료·소유 Job 비움·private desktop 닫힘·사용자 input desktop 불변을 기록했다. Windows P0는 계속 미완료다.

## Windows 폰트 비용·저장 worker 검증 (2026-10-04 후속)

[폰트 메모리 개선](../work-items/S00-windows-font-owned-allocation.md)과 [대기 비용](../work-items/S00-windows-idle-native-cost.md)의 fresh3회 표본은 CPU 약0.31%, WS82,173,952/76,140,544/76,206,080bytes로80MiB/1% 예산을 통과했다. root는 현재 source·binary·collector SHA와 원app Job·자연exit0·desktopclosed·한글glyphPNG를 직접 확인했다. 원81.3MiB 실패는 보존하며 emptylibrary3×10초를 대형목록·IME·장시간GUI·최종스택 인수로 확대하지 않는다.

[저장 worker CloseWait](../work-items/S10-report-worker-closewait-fixture.md)의 featureunit은 actual file-open→release전ACK불가→release후ACK, callbackfail/기존파일 보존, 원eventtimeout을 통과했다. 기본OFF 선택fixture feature는 기존windows-sys0.61.2를 재사용하고 Cargo.lock은 app dependency edge 한 줄만 추가됐다(기존lock SHA 복원 대조). 시험feature의 최초Clippy 조건문 경고 수정 후 `windows-closewait-feature-corrected-20261004T134641Z` fmt/Clippy/test/release가 통과했고 root는 고정fixtureexe/source SHA를 대조했다. 실제privateGUI release/timeout시험은 진행 중이며 성공을 미리 주장하지 않는다. 일반 제품의 직전 전체 검증과 fixturefeature 근거를 구분한다.

## 2026-10-04 Windows 후속 GUI 인수

S10 원11case suite는5PASS/6FAIL이며 누적 입력90초 도달 실패를 보존한다. 소유 HWND 묶음 입력 재시험은 edit/mpd-remove/mpd-reattach/mpd-all/mpd-unsafe/mpd-large 모두 통과했고 root가 export/DB/최종 화면을 독립 확인했다. 잘못된 MPD·크기 초과는 기존 유효 첨부와 작업 증거를 보존한다. 테마는 System→Light, Light→Dark, Dark→System 실제 private GUI 저장·재실행 세case를 통과하고 SQLite 값/UIA선택/실제화면/양Run자연종료를 독립 확인했다. 실제 OS테마변경·firstpaint·clipboard·IME/reader·대형보고서 및 전체 P0 인수는 미완료다. 파일목록10k 수동변경 GUI nativePASS 후 verifier의 과도한 반복파일I/O로 outer200초 timeout이 발생했다. native 증거를 보존하고 독립 postreview를 수행하며 watcher-only 인수는 별도로 준비한다.

## Windows 자동 파일 변경 및 실제 ACL 실패 경로

2026-10-04 watch-only native는10회 안정UI 이후 수동Refresh/재진입 없이 실제 추가·삭제·rename을 반영했다. root는PNG/read-onlyDB present10k·삭제path부재·원FileIdInfo전후일치·renamedDB identity exactmatch를 확인했다. subscription등록완료는직접관측하지않았고 scanCloseWait/loadedcost는남는다. 새ownedTemp의currentSID ACL에서 direct/publiclocation create오류5를실제로확인하고 기존설정/sentinel보존, exactoriginalACL복원, 재시도성공, TempDircleanup을검증했다. 실제 Documents/시스템ACL 변경0이며 actualOneDriveprofile/unknownvolume는미검증이다. 두작업후 전체WindowsCPU fmt/clippy/test/release 네명령을통과했다. 전체P0완료는아니다.

## Windows loaded10k 비용 실패 보존

windows-library-loaded-cost-20261004-resume 실제3회10초 wholeownedJob측정은 모두BUDGET_FAIL이다. round1WS84668416bytes(80MiB초과)/CPU0.31155%, round2WS79466496/CPU1.08394%(1%초과), round3WS79773696/CPU1.24777%(1%초과). 모두원프로세스자연exit0/Job0/desktop닫힘이다. 원기준유지; quiethost/각fixtureDB10k독립후검토는남는다. 초기watchscan/metrics1Hz repaint/잔류allocation 가설을계측하며원인확정·수정완료로기록하지않는다. loaded성능과최종S00stack는미완료다.

## 2026-10-05 최신 표본·DB 상한·renderer 대조

지표 worker의 latest-only 표본 보존을 실제 UI 미소비 실패 회귀로 수정했고 동일 표시의 매초 repaint를 줄였다. 손상 library DB path/identity의 Rust 문자열 디코딩 상한도 실패 재현과8회귀 및 전체WindowsGNU CPU fmt/clippy/test/release를 통과했다. SQLite 내부 전체 메모리 비용은 별도이다.

추가 캔버스 캐시를 사용하는 동일 소스 baseline은 첫 WS84480000bytes로80MiB를 넘었다. software_host.rs의 캐시 설정 한 파일만 바꾼 direct 대조는 WS79237120/74010624/73965568bytes, CPU0.30953/0.31239/0.30923%로3회 모두 원기준을 통과했다. root는 실제 한글 PNG, 각ownedfixture3byte10k/DB10k·자연exit0/Job0/desktop닫힘과 숨김텍스처회귀를 확인하고 현재CPU후보에direct설정을 유지한다. 고정의존성/lock변경0. quietHost/장시간GUI/IME/reader/firstpaint/최종stack인수는 미완료이며 이전 비용 실패를 보존한다.

기존 fixture hook의1.84MB급 대형진단 JSON 실제Windows 저장을 시작했다. 정확2MiB, clipboard, 마지막페이지 및stress peak memory 인수는 이 시험의 단순 실행으로 완료 처리하지 않는다.

## Windows 대형 보고서 저장·마지막 페이지 실제 인수

2026-10-05 fresh private GUI의 JSON1841855B/Markdown1933102B 저장 및 first/next/last preview를 통과했다. root는 원 source·파일 SHA·Failed/단일 Attempt/Artifact 없음·영구 쿠키 부재·source/sentinel 보존·Job0/private desktop 정리와 PNG를 독립 확인했다. JSON/Markdown logs128×4096B·MPD8×256nodes이며 마지막 actual236/236페이지7982B·Next 비활성화다. [작업 기록](../work-items/S10-windows-large-report-native.md)을 따른다. 정확2MiB 및 stress peak/clipboard·전체 S10/P0 완료는 아직 아니다.

## Windows 대형 진단 비용 관측 및 스캔 종료 인수 진행

2026-10-05 대형 JSON/Markdown/preview 세 actual private GUI 구간에서 원 app 메모리를50ms/최대2000표본으로 관측했다. sampled working set 최고값은85721088/85835776/85692416bytes이고 OS lifetime peak는각87851008/93528064/87064576bytes다. active CPU9–11%는 UIA/화면 관측을 포함하며 idle1% 기준으로 판정하지 않았다. 원 source/극값/파일·DB·PNG·cleanup은 root 독립 확인했다. 첫 preview 비용 시험의 synthetic 입력124/125 실패는 보존하고 글자별prefix를 확인한 corrected fresh 시험이 통과했다. [비용 기록과 한계](../work-items/S10-windows-report-stress-cost.md)를 따른다. wholeJob 메모리·quietHost·장시간GUI·clipboard는 미완료다.

[Windows 스캔 CloseWait](../work-items/S03-windows-scan-closewait-native.md)는 원 첫 일반파일 observation INSERT 뒤 callback/취소rollback/workerACK를 위한 기본OFF feature를 통합하고 실제 Windows GNU fmt/Clippy/test/release 검증 중이다. native GUI release/timeout 인수는 아직 완료가 아니다. 새 의존성·lock변경없음이며 전체P0는 진행 중이다.

## Windows 실제 스캔 CloseWait 좁은 인수 완료

2026-10-05 [스캔 종료 작업](../work-items/S03-windows-scan-closewait-native.md)의privateGUI release/20초timeout에서첫일반파일 observation INSERT 이후ACK전창유지→취소rollback/workerACK→자연exit0/Job0를확인했다. root가20개의원alive/ACKfalse표본,DB원1행/path/size/nativeidentity동일·신규파일미공개·파일/sentinel SHA 및PNG를독립대조했다. seed GA권한이실제selfFA+inherit-onlyGA로분리된원실패를보존하고,새ownedroot에는concreteFA를적용해기존단일ACE검사를유지했다. 최종fixture/기본CPU 전체8명령과current source/artifact검증을통과했으며기본OFF제품release를복원했다. 개별OS syscall취소·대형scanpeak·전체P0는별도미완료다. 실제한글IME capability 감사를다음으로진행한다.

2026-10-05 재개: Windows owner UIthread의 실제 IME readonly 관측을 완료했다. HIMC 존재·Korean HKL/ImmIsIME·query0·context release·원 PID/HWND/thread·자연exit/Job0/private desktop 복원을 root가 대조했다. 시험feature/defaultCPU 각각 전체4검증과 release복원을 통과했다. [관측 범위](../work-items/S00-windows-private-ime.md)와 [실제조합 후속](../work-items/S00-windows-owner-ime-compose.md)을 구분한다. defaultCPU7,858,176B이고 readonlyfixture추가23,552B, 새 dependency/lock변경없다. 원 context의 open=false를 조합불가로 단정하지 않으며 실제조합·취소·확정은 진행 중이다. 전체P0 완료가 아니다.

2026-10-05 gl exactmapping 후보2ZIP 실측: 최신CPUdfae315d/7,871,488B, 두 ZIP SHA0609e30036f3a9b8ee8c668a5d55bcb7a96ee4db89e434283ee7254641575b2d 동일,12,619,818B/848entries/missing53. root bounded independent package reviewer PASS, release_ready=false. 뒤이어 GPU exactnotice3mapping을 통합했고 새후보 측정 중이므로 예상missing50을완료로집계하지않는다.

최신 S10 Job commit 실측3scenario root·독립검토 PASS: JSON54,484,992B/Markdown63,025,152B/preview53,329,920B. 원본SHA/Failed1attempt1artifact0/naturalJob0/private정리를대조했다. wholeJobRSS·quietHost·active예산과 실제reader는별도다. fixedinvalidHANDLE 실제Win32 6 negative는원statusFAIL/exit2를보존하고measuredfalse/자연종료를확인했으며baseline수치직렬화정상대조진행중이다.

새 notice6mapping을추가한최신후보2ZIP은20fb3944…/12,646,828B/858entries/missing47로동일하며독립review PASS다. lock superset의누락이며WindowsCPU실제필요package와optionalrenderer/다른OS는Cargo target/feature근거로별도감사중이다. release_ready=false유지. 최신emptyidle3회는1%/80MiB조건PASS지만loaded10k3회중첫CPU1.5552%로전체FAIL했다. 원failure/정상종료를보존하고원thread별CPU계측으로원인을조사한다.

파일목록성능후속: 원CPUdfae의60초×3회는CPU평균4.603/4.892/5.258%, fixed3/6번째10초창13~14% 반복초과를확인했다. 주CPU는originalPI main이아닌미분류worker에집중했다. 원native인과stack확정전publicreconcile 비용비교용Rust 예제1개를추가했으므로이후currentall-target전체검증은새source에서다시요구한다. 기존535source전체4PASS와후보ZIP은당시snapshot이력이다.

2026-10-05 추가 진행: library prepared INSERT와 고정 public API 비용 example을 게시했다(536소스). Windows 취소·rollback·원파일 ID·closewait 회귀12개를 실제 통과했다. 같은 10k×3회 API CPU 평균1.177083→1.15625초 감소는 GUI 대기 예산 해결 근거가 아니다. 현재 신규 전체 fmt/clippy/test/release 검증 진행 중이며, 이전535소스/dfae 빌드 증거를 최신 변경 검증으로 사용하지 않는다. 후속은 단계별 identity/SQL 비용 분리와 최신 loaded60초 엄격 예산 재검증이다. 상세 S01-library-prepared-reconcile-cost와 root-comparison-review를 따른다.

## 2026-10-05 동일 목록 UI 전달 후속

배경 동일 목록의 queue/repaint 생략과128행 Arc 공유를 통합했다. 명령 ACK·행 metadata/오류회복·root/health 변화 및 실제 scan 취소를 Windows 두 후보 각14회귀로 확인했다. 최신540소스 default CPU 전체 fmt/clippy/test/release PASS, CPU3f9d8825f1674b1c682094506f8774df208494dc10ccf1e0655f62d06b8e3667(7,888,384 bytes). 원Job4개 자연종료/격리desktop종료/input불변과sourceSHA를root대조했다. 현재 새60초×3 GUI측정은 진행중이며 P0완료가 아니다. 앞선9222 바이너리GUI 평균0.833/0.651/0.599%와메모리PASS, strict10초구간FAIL은그snapshot의증거다.

동일UI 전달 후속 GUI3회 측정이 종료됐다. 평균CPU0.728060/0.650116/0.805898%와메모리PASS, strict10초구간3회FAIL. root가 원source/CPU불변·Job자연0·격리desktop종료·3DB무결성/각10000행·합성30000원파일바이트를 실제 대조했다. UI 변경만으로 strict 비용 해소를 주장하지 않으며 다음실험은 fresh DB open 비용을 분리한다.

최신S10대형JSON/Markdown/236쪽미리보기3case를실제native/privateGUI로재검증했고root및독립검토를통합했다. exact2MiB/이번peak/clipboard/physicalreader·IME는미검증. 최신개발ZIP2개 동일성과863entry검증도완료했다. freshDBopen비용실험은별도evidencecrate/offline원registrylock으로실행했고원productionsource/CPU를보존했다. open0.2~0.35ms로캐싱변경근거가없어정책을유지한다. 공급의내장font exactpayload/grant 연결감사가다음P0작업이다.


현재18의 새 개선: 실제 키보드 도움말2필터와 진단 번호 실패·수정·비활성5필터를 격리 시험 후 제품에 통합했다. 앱은 v17보다12,800B 증가했고 설치기·CLI·의존성·Cargo.lock은 그대로다. SQL snapshot 변조·누락 회귀3/0 및 CI 안전성 관련42/0도 로컬 통과했고 hosted 실행은 미검증이다. [한글 확정 actual FAIL](evidence/settings-ime-positive-commit-isolated-20261013/root-failure-review.json)은 preview A조Z가 commit A한국어Z로 바뀌지 않은 원자료이며 새 테스트를 준비한 뒤에 발견했다. 현재 검수 성공으로 덮지 않는다.

현재33 긴 경로 후속 실제 결과:577/598UTF16·20objects/6parents·journal5/installed5·설치/검증/앱 자연0를 Root가 원46pins와 독립 대조했다. Root3PNG 직접 확인·14PNG전체 SHA 고정, 별도44장 gallery 재검수는 하지 않았다. [현재33 원 검토](evidence/windows-msvc-ui-feedback-v33-long-install-private-v1-20261016/root-review.json). 이 append는 앞의 실행 대기 상태를 갱신한다.

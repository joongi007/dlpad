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


당시 v24/source670 기록이다. Windows 네 명령은1,538 PASS/0 FAIL/42 ignored·43harness로 Root가 원로그·현재 source·3실행 파일·lock을 독립 대조했다. 최신 설치/portable ZIP도 실제1,135입력·474members SHA/CRC·456원notice·generated7를 독립 재검산했다. 정상 설치·긴경로577/598UTF-16 앱 실행·자연0와 원20객체6부모/5payload5journal 보존, 변조된실행파일 실행거부/nochild를 확인했다. [전체 검사](evidence/windows-msvc-ui-feedback-v24-whole-20261014/root-review.json), [패키지](evidence/windows-msvc-ui-feedback-v24-20261014-package-root-review.json), [긴 경로](evidence/windows-msvc-ui-feedback-v24-20261014-long-install-run/root-review.json), [변조 거부](evidence/windows-msvc-ui-feedback-v24-20261014-command-negative/root-review.json).

최신 선택 정보6조합(800/960·100/150/200%)은 고정32px·행위치/RuntimeID·원합성8MiB파일·자연app0/Job0를 대조하고, Root6/child18 총24원PNG를 직접 검토했다. 실제왼쪽정렬·작은창파일명우선폭 개선을 확인했다. metadata말줄임과 상단offscroll은 기록하고 실제pointer·IME·reader·전체UI 인수와 분리한다. 설치88원화면도 직접 팀 검토하고 전체SHA·크기를Root재검산했다. [선택 화면](evidence/current-library-selection-summary-native-v4-20261014/root-team-pixel-review.json).

최신v24 로컬CI15단계·8Cargo·7capture·2,027tracked/config source records(실제export2,026+manifest1)는Root/child독립검토를통과했다. 최신세60초평균CPU0.676534%·0.546358%·0.624538%는시작/최종Job/App원counter와실제elapsed로독립재계산했으며 메모리18구간은충족했다. 추가strict10초CPU15PASS/3FAIL(1.082014%·1.091827%·1.247925%)은보존한다. Job/App끝점읽기는비원자이며부분thread자료로operation인과를확정하지않는다. 원30,000합성3byte파일/readonlySQLite3개·6원PNG/source670·자연app0/Job0를Root전수대조했다. [최신CI](evidence/current-ui-feedback-v24-local-ci7-20261014/root-review.json), [최신비용](evidence/current-ui-feedback-v24-default18-cost-20261014/root-review.json).

v22원strictFAIL·v23빈목록worker17tick(평균wall약0.77ms)은각고정소스의이전근거이며 최신원인해결증거가아니다. 성능후속은현재실패구간의UI/layout/present와worker전체를분리한실험이며예산완화나성공재시도로대체하지않는다.

| 작업 | 현재 근거 | 남은 인수 조건 |
|---|---|---|
| UI 설정·목록 역할 정리 (P0) | 제품4상위탭·설정의기본폴더·목록읽기전용위치 통합, 최신전체검사 통과 | v22 73화면·테마저장/재시작Root검토, v24 선택24화면확인; 물리검수후속 |
| 목록 선택 정보 간격 (P0) | 고정32px·왼쪽정렬·metadata최대45%/이름폭우선 구현, 최신전체검사 및6조합/24PNG검수통과 | UIA선택전후행좌표/ID유지확인; Context sameclick/Enter회귀와물리doubleclick구분 |
| 배율·초기 배경 (P0) | 104 CPU raster와5배율회귀, 최신전체검사 통과 | 최신실제private화면 검토통과; 비용 strict진단3구간 원인분리후속 |
| 제목 기반 파일명 (P0) | 제품Core/engine/UI통합, naming15 및checkbox회귀 포함전체검사 통과 | 합성공개API10사례Root인수통과; 재개/자막·실제provider 인수후속 |
| 복사 내보내기 (P1) | 생성cap/export·UI/picker/Stop/다른화면회귀 통합, 최신전체검사 통과 | 실제합성API복사·충돌·사전취소·원본보호통과; actualGUI배선 합성8MiB·동명거절·다른탭완료 인수통과; 늦은취소/picker후속 |
| 긴 경로 앱 초기 종료 (P0) | 이전app101의OLE실패graceful fallback수정; 최신577/598경로앱창·자연0, 원20객체6부모·5payload5journal 보존 | v24 긴경로원로그·nativeID·수명Root인수, 새44화면직접검수완료; 물리drag별도 |
| 시스템 파일 열기 (P0) | 제품신원재검증·typedExplorerbroker·취소/감독회귀 통합 | 실제사용자defaultapp수명은미검증 |
| 도구 업데이트·표시 (P0/P1) | cache-first 표시/실행재검증; yt-dlp exactpin설치·명시적사용, FFmpeg/Deno승인pin | 최신Tools화면 검토통과; 실제버전전환·공급notice전체범위후속 |
| 로컬 CI/성능 (P0) | v24 15단계·8Cargo·7capture·2,027기록독립검증 | v24 세평균<1%/메모리18PASS·원counter검산; strictCPU15PASS3FAIL보존/UI·layout·present후속 |
| 설치·변조 거부 | v24 정상/긴경로 앱·감독자연0·변조실행거부Root독립검토 | 최신변조거부원로그·33pins·변조exe/다른4파일/journal5 Root검토통과; 설치88화면직접팀검수완료/물리입력별도 |
| upgrade/rollback | 미지원 요청거부·원본보존, 기능미완료 | 순수revision정책6시험Root인수; durable전환·복구·DB호환/실제rollback미구현 |
| 물리 Windows/clean OS | private/synthetic검증과분리, 사용자수동시험은나중 | 실제IME·reader·OSDPI·sleep/resume·OneDrive·freshOS |
| Git/hosted CI/앱 updater | Git조회exit128, 기존.git보존; 최신v24 로컬CI PASS | 유효repo/remote연결·hosted실행·updater; 공개릴리스별도승인 |
| 정식 배포 | unsigned개발검수ZIP 생성·검증 | 전체notice/서명·설치/제거·사용자최종인수 |

Windows NIC 관측·기본 경로·도구 설치·미리보기/구간 선택·고급 옵션·S10 P0 로컬 진단은 구현되어 있다. **S10은 최초 릴리스 범위**다. 전체P0/정식릴리스와 물리GUI인수는 아직 완료로 보고하지 않는다. 자동GUI는private desktop만 사용하고 사용자화면/전역입력으로fallback하지 않는다. 수동검수대기만으로 개발을 중단하지 않으며 승인범위밖게시·OS변경은 별도 보류한다.

v21의 후보상태와 이전 실패는 [이전 상태 기록](P0-remaining-before-20261014.md) 및 각 원증거에 보존한다. 새소스의 변경과 이전설치본의 인수를 혼용하지 않는다.

[작업 추적](../work-items/S00-settings-library-navigation-20261013.md), [긴 경로 진단](../work-items/S11-long-path-app-startup-diagnostic-20261014.md), [요구사항별 기록](P0-status.md), [이전 상태 보존](P0-remaining-before-20261014.md). 원FAIL·미실행·historical 증거를현재성공으로승격하지않는다.

현재22 Library 복사 private UI 추가 검증: 실제 합성8MiB 선택·한글 새 이름 readback·같은 폴더 복사·다른 탭 완료 표시·동명 대상 거절을 통과했다. 원본/새 파일 SHA와 full128 원수신을 대조하고 six PNG 원픽셀을 Root가 직접 확인했다. [독립 검토](evidence/current22-library-copy-private-ui-v1-20261014/root-review.json). 실제 picker·물리 IME·진행 중 취소는 이 시험 범위가 아니다.

현재33 긴 경로 후속 실제 결과:577/598UTF16·20objects/6parents·journal5/installed5·설치/검증/앱 자연0를 Root가 원46pins와 독립 대조했다. Root3PNG 직접 확인·14PNG전체 SHA 고정, 별도44장 gallery 재검수는 하지 않았다. [현재33 원 검토](evidence/windows-msvc-ui-feedback-v33-long-install-private-v1-20261016/root-review.json). 이 append는 앞의 실행 대기 상태를 갱신한다.

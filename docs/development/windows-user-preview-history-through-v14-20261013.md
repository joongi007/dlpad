<!-- Historical v14 snapshot; see current README for accepted candidate. -->
# 현재 UI 검수 후보 — 2026-10-13

현재 v14 whole은 **1,305 성공 / 0 실패 / 30 ignored**이며 source656·43harness·현재3실행 파일을 독립 대조했다. 배율 변경 뒤 최초 늦은 viewport 축소를 기존8frame 공개로 보정하며 사용자의 wheel/press/key/text/IME·페이지 이동은 감시를 해제한다. 실제 Candidate 회귀5개와 격리63화면 검수, 최신 archive·변조 실행 거부·독립27ID가 통과했다. 앱은1,536B 증가하고 설치기·CLI·Cargo·의존성은 그대로다. 설치·실행 내부 native0/Job0와 외부143 종료는 분리하며 외부 원인은 미확인이다. v13 CI·비용·긴 경로 결과는 당시 소스에 한정하며 v14로 소급하지 않는다. 물리 pointer/IME/reader/OS DPI·전체 UI/UX 인수·정식 배포는 미완료다.

[현재 전체 검사](../validation/evidence/windows-msvc-scale-viewport-reflow-whole-20261013/root-review.json), [현재63화면](../validation/evidence/current-scale-viewport-reflow-app63-20261013/root-team-visual-review.json), [현재 배율 회귀](../validation/evidence/windows-msvc-late-footer-scale-green-v3-20261013/root-review.json).

아래 v13 항목은 이전 검증 snapshot이다. 배율 일시 가림의 현재 수정 상태는 위 v14 기록을 따른다.


[설치 ZIP](../validation/evidence/dlpad-install-ui-cross-surface-v14-20261013.zip)을 새 폴더에 모두 풀고 dlpad-install.exe를 실행한다. 기본 위치를 쓰거나 바꾼 뒤 설치하고, 설치 확인 완료에서 앱 실행을 누른다. 앱을 닫을 때 원 감독 프로세스도 종료한다. [설치 없이 실행](../validation/evidence/ui-test-scale-viewport-reflow-20261013/dlpad.exe)할 수도 있다. 서명되지 않은 개발 후보이며 전체 UI/UX 인수는 아직 완료하지 않았다.

최신 v13은 설치 완료 안내·경로 편집/재검증·앱 실행 및 변조 거부, 긴 경로 설치와 독립 ID, 앱63화면 직접 검수와 로컬 CI를 확인했다. 배율 조작 일시 가림과 물리 입력·사용자 최종 평가는 남아 있다. 설치된 앱은 선택한 폴더의 registered/dlpad.exe이며 설치 확인 완료에서 앱 실행으로 시작할 수 있다.

아래는 이전 후보의 실행·검증 이력이다. [현재 검수표](../validation/UI-UX-AUDIT-CURRENT-20261013.md)와 [남은 작업](../validation/P0-remaining-20261011.md)을 먼저 읽는다.

## 이전 snapshot 기록

현재 UI 검수 후보는 [v10 설치ZIP](../validation/evidence/dlpad-install-ui-cross-surface-v10-20261013.zip)과 [최신 단독실행](../validation/evidence/ui-test-compact-network-error-20261013/dlpad.exe)이다. ZIP을 새 폴더에 모두 풀고 dlpad-install.exe를 실행한다. 설치된 앱은 선택한 설치 폴더 아래 registered/dlpad.exe에 있다. 아직 설치 후 실행 버튼은후속개발중이다. v10은 일반/긴577/598UTF16 설치·재검증 및각 독립27ID 인수를완료했고, 확대입력오류짧은안내가포함된다. Windows1284/0/30 코드검사와전체GUI/물리사용자인수는구분하며 서명되지않은개발후보다. [현재범위](../validation/UI-UX-AUDIT-CURRENT-20261013.md). 아래는이전snapshot 실행안내이다.

현재2026-10-13: [최신 설치 v7 ZIP](../validation/evidence/dlpad-install-ui-cross-surface-v7-20261013.zip)을 새 폴더에 전부 풀고 `dlpad-install.exe`를 실행합니다. 설치 없이 확인하려면 [최신 UI 검수 exe](../validation/evidence/ui-test-semantics-repaint-20261013/dlpad.exe)를 실행합니다. Windows 전체1272/0/30 및 최신 일반5파일설치·독립27ID·설치54상태/드롭다운8상태를 검토했습니다. v6 긴 경로는 이전 별도 증거이며 최신 비용·전체UI/UX·upgrade/서명/cleanOS/정식릴리스는 미완료입니다. 기존 파일은 보존하고 새 폴더에서 시험하세요. 아래는 이전 snapshot 이력입니다.

# 현재 Windows UI 검수본 — 2026-10-13

[최신 설치 ZIP](../validation/evidence/dlpad-install-ui-cross-surface-v6-20261013.zip)을 새 폴더에 모두 압축 해제하고 `dlpad-install.exe`를 실행합니다. 기본 설치 위치를 제안하며 변경할 수 있습니다. 설치 없이 화면을 보려면 [검수용 앱](../validation/evidence/ui-test-cross-surface-v5-20261013/dlpad.exe)을 실행합니다. 공통 버튼·여백·글꼴·선택 표현과 설치의다크콤보/입력/높이 수정이 포함됩니다.

Windows fmt/clippy/test/release1253/0/30 및원로그/source/artifact 대조를 완료했습니다. 실제577UTF16 긴경로5파일설치·재검증과독립27canonical/원20object·6parent ID를 확인했습니다. ZIP16,664,245B SHA256 `ae19c1b9e7bf7fcf5a6839638a95a246b554d8be9269b4399d157d423b462b78`. [통합 검사](../validation/evidence/windows-msvc-ui-cross-surface-whole-v8-20261013/root-review.json), [실제 설치 검토](../validation/evidence/windows-msvc-ui-cross-surface-v6-20261013-long-install/root-independent-review.json).

전체UI/UX 검수는 진행 중입니다. 기본Windows 메시지pointer pressed 시험과열린설치dropdown 수집기는 실패를보존하고재검증합니다. 합성egui렌더검증은물리입력검증과구별합니다. 실제upgrade/rollback·서명·clean OS·원격CI·updater는 미완료입니다. 아래는 이전snapshot 이력입니다.

이전 2026-10-12 설치 후보: [설치 ZIP](../validation/evidence/dlpad-install-ui-advanced-validity-20261012.zip)을 새 폴더에 모두 압축 해제하고 `dlpad-install.exe`를 실행한다. 진단 파일 선택·저장 결과·로그 번호 오류 문맥·고급 숫자 오류 상태 수정이 포함된다. Windows 전체1,195 성공/0 실패/30 ignored와 원 로그·소스·실행 파일을 독립 대조했고, 실제5파일 설치·재검증·현재 native ID27개를 확인했다. ZIP16,562,869B, SHA256 `700ddcf981f45966f09b5246782e025667673c6ad6435c4f259c2e3001297654`. 고급 입력의 실제 유효성·설명·시작 gate는 Light/Dark·재시작에서 검토했지만 일부 캡처 정합성·확대·전체 UI/UX 인수는 남아 있다. MPD 선택·첨부·제외의 전체 흐름도 검증 중이다. 서명되지 않은 개발 후보이며 clean OS·upgrade·정식 릴리스는 미완료다. [설치 검토](../validation/evidence/windows-msvc-ui-advanced-validity-20261012-install/root-independent-installed-review.json), [작업 기록](../work-items/S11-advanced-validity-install-refresh-20261012.md). 아래는 이전 snapshot 이력이다.

이전 2026-10-12 설치 후보: [설치 ZIP](../validation/evidence/dlpad-install-ui-open-context-final-20261012.zip)을 새 폴더에 모두 압축 해제하고 `dlpad-install.exe`를 실행한다. 진단 정보 위계·선택적 MPD 접기·최초 펼침의 기본 조작 가시성 개선이 포함되어 있다. 최초 공개는 Light/Dark960100·800200 실제 화면으로 검토했다. 실제5파일 설치·재검증·읽기 전용 기록 및 현재 native ID27개 대조를 완료했다. ZIP16,533,685B, SHA256 `0a279b91e3c25cce5e17edc5a2b3fda467e1090e383646cb881ff701e2856444`. Windows 전체1,173 성공/0 실패/30 ignored다. 페이지 문구는 다음 페이지로 바뀌었으나 최신 목록 조작 시험은 미완료다. 서명되지 않은 개발 후보이며 전체 UI/UX·clean OS·upgrade·정식 릴리스는 미완료다. [설치 검토](../validation/evidence/windows-msvc-ui-open-context-final-20261012-install/root-independent-installed-review.json), [작업 기록](../work-items/S11-diagnostic-hierarchy-install-refresh-20261012.md). 아래는 이전 snapshot 이력이다.

이전 2026-10-12 설치 후보: [설치 ZIP](../validation/evidence/dlpad-install-ui-current-final-20261012.zip)을 새 폴더에 모두 압축 해제하고 `dlpad-install.exe`를 실행한다. 이전 폴더는 덮어쓰지 않는다. 최신 영상 확인 버튼·URL 오류 안내·높이 오류 설명·진단 포커스와 Windows 접근성 수정이 포함되어 있다. 실제5파일 설치·재검증, 읽기 전용 설치 기록3단계, 현재 native ID27개를 검토했다. ZIP16,515,253B, SHA256 `d96c4123e930cfe28ae7c32d1596792d071c61f826be0387ef6703b53299febb`. Windows 전체1,165 성공/0 실패/30 ignored와 원 로그·소스·실행 파일 SHA를 대조했다. 서명되지 않은 개발 후보이며 전체 UI/UX·clean OS·upgrade·정식 릴리스는 미완료다. [새 설치 검토](../validation/evidence/windows-msvc-ui-current-final-20261012-install/root-independent-installed-review.json), [작업 기록](../work-items/S11-current-ui-install-refresh-20261012.md). 아래는 이전 snapshot 이력이다.

이전 2026-10-11 설치 후보: [전체 설치 ZIP](../validation/evidence/dlpad-install-tool-executable-picker-shell-hint-final-20261011.zip)을 새 폴더에 모두 압축 해제하고 `dlpad-install.exe`를 실행한다. yt-dlp·FFmpeg·Deno 실행 파일 선택창과 취소·설정 저장·재실행 복원을 포함한다. Windows whole1,089/0/28(43harness)와 실제private GUI5단계·SQLite5 integrity/exact경로·owner-close, 실제5파일 설치·재검증 및 현재 native ID27 조회를 root 검토했다. ZIP16,451,372B SHA256 `780e0e1f1beb1c560e9d5bdc8876bc8a8021747df9171cce856b71f9b7f6c123`. unsigned 개발 후보이며 upgrade·서명·clean OS·전체P0 완료는 아니다. [실행 파일 선택 기록](../work-items/S04-native-tool-executable-picker-20261011.md), [설치 기록](../work-items/S11-install-zip-20261011.md). 아래는 이전snapshot 이력이다.

이전 2026-10-11 설치 후보: [전체 설치 ZIP](../validation/evidence/dlpad-install-tools-first-open-20261011.zip)을 압축 해제하고 `dlpad-install.exe`를 실행한다. 수동 도구 경로 저장·자동 전환과 도구 화면 첫 진입의 설치 목록 조회를 포함한다. 최신 whole1,063/0/28, 명시 Refresh 없는 managed 선택 GUI4단계·SQLite4와 새5파일 설치/재검증을 root 검토했다. 수동 설정 기본5단계는 이전 동일 설정 정책의 별도snapshot 증거다. ZIP SHA256 `d104726294d164c9054eefb08847cf3cb7cbdd8dfcb29137e42f4dfcb74817d7`/16,423,212B. 기존 폴더 덮어쓰기/upgrade·서명·cleanOS·전체P0 완료는 미인수다. [설치 기록](../work-items/S11-install-zip-20261011.md). 아래 이전 후보 검증은 당시 snapshot 기록이다.

이전 2026-10-11 설치 후보: [설치 ZIP](../validation/evidence/dlpad-install-plan-scroll-20261011.zip), [ZIP SHA·구성](../validation/evidence/dlpad-install-plan-scroll-20261011.zip.receipt.json), [실제 설치 인수](../work-items/S11-install-zip-20261011.md).

1. ZIP 전체를 새 폴더에 압축 해제한다. payload 폴더를 함께 유지한다.
2. dlpad-install.exe를 실행하고 상위 폴더 선택 또는 새 설치 폴더 입력 후 설치한다. 기존 폴더는 덮어쓰지 않는다.
3. 설치된 폴더의 registered/dlpad.exe를 실행한다. 필요 도구는 설치 화면의 기본 해제된 선택 항목 또는 앱 도구 화면에서 설치한다.

이 후보는 unsigned 개발 설치본이다. 실제5파일 설치·재검증과 로컬 구간 저장은 확인했지만 clean OS·서명·최종 사용자 UI 인수·원격 CI·앱 updater는 남는다. 아래 링크와 수치는 이전 시험 이력이다.

# Windows 사용자 UI 미리보기: 별도 시험 폴더

[이전 네트워크 입력 검증 ZIP](../validation/evidence/windows-msvc-network-input-graph-20261008/dlpad-user-preview.zip)을 새 폴더에 풀고 아래 `$exe`를 그 안의 `dlpad-cpu.exe`로 바꾼다. 앱 UI 시험에는 설치 화면을 먼저 실행할 필요가 없다. ZIP의 `candidate-manifest.json`은 빌드 증거 메타데이터이며 개발 설치 화면에 입력할 설치 명세가 아니다.

이 안내는 직접 화면을 보고 평가할 개발 후보용이다. 정식 설치·updater·서명·clean OS 검증 완료를 뜻하지 않는다. 자동 GUI 시험은 계속 private desktop에서만 수행한다. 아래는 사용자가 직접 실행할 예시이며 에이전트가 실행하지 않았다.

현재 기본 CPU binary는 일반 `--data-dir`/`--library-root` CLI를 제공하지 않는다. 기본 실행 argv는 빈 배열이며 `--settings`는 설정 화면 시작 표시다. 개발 경로 override 환경값은 기본 CPU에서도 읽는다. 기능시험 variant용 `--fixture`는 이 일반 미리보기에 사용하지 않는다.

## 2026-10-08 네트워크 관측 입력 교정본

최신 ZIP:17,244,068B·466members, SHA256 `35cd7866634b2af640abdf69f51c8c4a95032290461bd6bbc9ceb218c655173a`. 새 폴더에 풀어 dlpad-cpu.exe를 실행한다.

네트워크 관측 설정의 인터페이스 입력을 공통32px 높이·12px 여백·중앙 정렬·focus/논리 ID 규격으로 고쳤다. Windows fmt/Clippy/test/release **957 passed / 0 failed / 17 ignored**(41 harness 합계). 실제 private GUI 첫/재실행에서 클릭·Home/End·Backspace·Tab→적용·유효/무효/빈값과앱150/200를 확인했다. [수정 화면](../validation/evidence/windows-msvc-network-input-native-20261008/native/first/network-valid-focused.png), [교정 범위](../work-items/S00-network-observation-input.md), [전 입력의 실제 수행 근거](input-coverage-audit.md).

관측 적용은 실행 중 인터페이스 선택이며 영구 저장 기능은 아니다. Windows 실제망 측정 성공이나 물리키보드/IME를 뜻하지 않는다. 진단 단일 행3곳의 공통규격 교정과전수입력은후속이다. 이전953에서검증한공개영상·idle비용은원후보근거로구분한다. 앱과CLI/개발설치화면의사용방법은아래와같다.

## 이전 2026-10-08 입력·실제 영상 점검 수정본

최신 ZIP은 **17,244,580B·466 members**, SHA256 `e639daf6bd387bb463082a4d5cd2162cea9949854f1b89be0de787aa8d128745`다. 새 폴더에 풀어 `dlpad-cpu.exe`를 실행한다. `dlpad-install.exe`는 기존 개발 설치 화면이며 선택 도구 설치도 지원한다. 정식 서명 installer/updater는 아니다.

입력 커서·여백·Segoe/Malgun 글꼴을 정돈하고 입력 중 상태 변화로 focus가 풀리는 문제, 품질 메뉴가 닫히지 않는 문제, 재생 버튼 ID 변화와 YouTube 미리보기 형식 선택을 수정했다. 두 사용자 제공 공개 영상의 준비·짧은 무음 미리보기·재생/정지·1/2초 시간 입력·품질·고급 옵션·탐색을 private Windows GUI에서 검증했다. [YouTube 화면](../validation/evidence/windows-msvc-public-youtube-playback-final-20261008/native/gui/prepared.png), [치지직 화면](../validation/evidence/windows-msvc-public-chzzk-playback-final-20261008/native/gui/prepared.png), [현재 Light 화면](../validation/evidence/windows-msvc-public-ui-final-layout-20261008/native/first/initial-download-full-960.png). 전체 영상 재생/다운로드나 모든 사이트 호환을 확인한 것은 아니다.

Windows fmt/Clippy/test/release 모두 exit0, **953 passed / 0 failed / 17 ignored**(41 harness 합계); ignored 공개망 회귀는 별도로1passed/0failed. 최종 layout63PNG/RawInput19PNG·합성 FFmpeg·실제 폴더 선택 및 library 복원을 확인했다. 합성1만 파일 목록 idle18창의 최대CPU0.786260%/WS69.55MiB는 재생·다운로드 성능 수치가 아니다. 도구3개 설치/Use/재실행 기능은 이전 원 검증과 같은 코드이며 아래 준비 안내를 따른다.

[수정·실행·실패 기록과 남은 인수](../work-items/S00-public-video-ui-walkthrough.md). 긴 영상 정밀 시간 입력 가시성·200% 정보 밀도·경로 표시·library 연결/로컬export·물리 IME/reader/OS DPI·최종 사용자 디자인 평가·정식 설치/서명/clean OS·법적 공지 완결은 남는다.

## 이전 2026-10-08 UI 정렬과 도구 설치 후보

최신 ZIP은 17,228,708B·466 members, SHA256 `384c9108656d1cc1397563b554c6d4310f4fd309e2eb32fa451263fda86eaf80`이다. `dlpad-cpu.exe`는 앱, `dlpad-install.exe`는 선택 도구 공급이 추가된 개발 설치 화면이다. 설치 명세를 직접 준비하는 기존 개발 설치 계약은 유지한다.

Windows 전체 검사 **934 passed / 0 failed / 16 ignored**. 실제 앱에서 세 도구 설치·활성화·사용과 재실행 복원을 확인했다. 설치 화면은 Deno만 선택한 실제 설치를 확인했다. 새 layout 63장·RawInput 19장, 실제 FFmpeg/합성 URL 미리보기와 정상 종료, 합성 1만 파일 목록의 비용 18구간을 검토했다. 최대 CPU 0.78527%·작업 집합 70.45MiB이며 재생·다운로드 중 성능 수치는 아니다.

[새 미리보기 화면](../validation/evidence/windows-msvc-ui-polish-url-tree-retry-20261008/native/gui/media-play-08.png), [UI 인수·후속](../work-items/S00-ui-visual-consistency.md), [도구 인수](../work-items/S03-windows-in-app-tool-install.md). 최종 사용자 UI 만족·물리 IME/reader/OS DPI·실제 사이트·긴 영상 정밀 탐색·로컬 export·정식 설치/서명/clean OS는 미완료다.

## 이전 2026-10-07 미디어 중심 UI 후보 이력

URL 자동 확인 → 영상 확인 → 구간 선택 → 저장 설정 흐름으로 변경했다. 로컬 파일 drop은 원본 읽기 전용 미리보기와 구간 선택까지 제공하며 로컬 export는 후속이다. 미리보기는 320×180·6fps·무음이며 전체 영상 플레이어가 아니다. URL은 외부 도구 설정이 필요하고 실제 사이트의 추출 성공은 이번 합성 fixture 시험으로 검증되지 않았다.

Windows 전체 검사906 passed/0 failed/16 ignored. 현재binary의 private GUI에서 합성 URL·real FFmpeg Range·빨강→파랑·정상 종료를 검증했다. 같은 UI 소스의63개 layout PNG와19개 RawInput PNG도 보존했다(바이너리는 직전905이며 현재와 다른 source는 복귀 listener 한 파일). 최신 비용18창 모두 통과, 최대CPU0.93667%·WS69.97MiB. 실제 mouse/OLE drop·IME·reader·OS DPI 및 최종 사용자 UI/UX는 별도 인수다.

ZIP SHA256: `9d5187543f27524a280c883e99f4c6a8623f2703519533d317999da7b35a539e`, 크기13,842,744B·466 members. 새 폴더에 풀고 `dlpad-cpu.exe`를 실행한다. [실제 미리보기 화면](../validation/evidence/windows-msvc-media-url-event-idle-20261007/native/gui/media-play-08.png), [비교·판단](modern-ui-audit.md), [인수와 남은 작업](../work-items/S00-media-workspace-interaction.md). 긴 영상 정밀 탐색·작은 화면의 하단 밀도·라이브러리 연결은 후속이다.

## 도구 준비

새 후보의 도구 화면에서 yt-dlp, FFmpeg/ffprobe, Deno를 설치할 수 있다. yt-dlp direct 설치는 실제 버전 검증과 활성화를 함께 수행한다. FFmpeg와 Deno는 설치 후 해당 버전을 선택하고 **이 버전 활성화 → 다운로드에 이 관리 도구 사용**을 누른다. 이후 재실행하면 빈 실행 파일 경로에만 기존 활성 선택을 복원한다. 직접 지정한 외부 경로는 보존한다.

개발 설치 화면에는 세 도구의 선택 설치 체크박스가 있다. 기본은 모두 해제이며 앱 설치 성공 뒤 체크한 도구만 다운로드한다. 앱 설치 결과와 도구 공급 결과는 따로 표시된다. 도구 공급 실패가 이미 성공한 앱 설치를 실패로 바꾸지는 않는다. 설치된 archive 도구는 앱에서 선택·사용해야 한다.

설치 프로그램의 `--tool-data`는 별도 개발 시험 프로필용 절대 경로다. 위 PowerShell의 `$data`와 같은 경로를 지정해야 그 앱 시험 프로필에서 설치한 도구를 찾는다. 일반 실행은 기본 앱 data root를 공유한다. 이 기능 추가는 개발 설치 화면을 정식 서명 설치 프로그램으로 바꾸었다는 뜻은 아니다.

## PowerShell 실행 예시

`$exe`는 전달받은 개발 ZIP에서 직접 풀어 둔 실제 `dlpad-cpu.exe` 절대 경로로 바꾼다. 폴더는 복사·쓰기 가능한 사용자 TEMP 아래 새 GUID로 분리한다. 설치 GUI/로컬 CLI를 먼저 실행할 필요는 없다.

```powershell
$exe = 'D:\내 시험 폴더\dlpad-cpu.exe'
$preview = Join-Path ([System.IO.Path]::GetTempPath()) ('dlpad-user-preview-' + [guid]::NewGuid().ToString('N'))
$data = Join-Path $preview 'data'
$media = Join-Path $preview 'media'
$documents = Join-Path $preview 'documents'
$tmp = Join-Path $preview 'tmp'
foreach ($path in @($preview, $data, $media, $documents, $tmp)) {
    New-Item -ItemType Directory -Path $path -ErrorAction Stop | Out-Null
}
$start = New-Object System.Diagnostics.ProcessStartInfo
$start.FileName = $exe
$start.UseShellExecute = $false
$start.WorkingDirectory = Split-Path -Parent $exe
$start.Arguments = ''  # 기본 화면. 설정부터 보려면 '--settings'
$start.EnvironmentVariables['DLPAD_TEST_DATA'] = $data
$start.EnvironmentVariables['DLPAD_LIBRARY_ROOT'] = $media
$start.EnvironmentVariables['DLPAD_TEST_DOCUMENTS'] = $documents
$start.EnvironmentVariables['TEMP'] = $tmp
$start.EnvironmentVariables['TMP'] = $tmp
$app = [System.Diagnostics.Process]::Start($start)
$preview
```

환경값은 이 child와 그 자식에만 적용한다. 기존 사용자 환경 설정·실제 Documents·기존 앱 DB를 바꾸는 명령이 아니다. 시험 폴더는 자동 삭제하지 않는다. 창을 닫고 작업 종료를 확인한 후 경로를 기록해 둔다.

기본 DB는 `$data\theme.sqlite`(테마/저장 위치), `$data\library.sqlite`(목록), 작업을 시작하면 `$data\jobs.sqlite`다. 라이브러리 시작 경로는 `$media`이며 설정에서 기본 저장 위치를 눌러도 override된 `$documents\dlpad`가 후보가 된다. data와 media는 서로 포함되지 않는 형제 폴더여야 한다. 저장 위치를 직접 다른 실제 경로로 바꾸면 이 격리 범위에서 벗어난다.

## 5분 확인

1. 0–1분: 기본 화면을 열고 한글, 버튼 간격, 대비를 본다. 창을 줄였다 늘려 잘림·텍스트 소실을 확인한다.
2. 1–2분: Tab/Shift+Tab으로 이동해 현재 위치 표시를 보고, 설정에서 Light/Dark/System을 바꿔 읽기 편한지 확인한다. 한글 입력 필드에서 실제 키보드 입력·지우기·커서 이동을 직접 시험한다.
3. 2–3분: 파일 목록 화면에서 현재 경로가 위 시험 폴더인지 확인한다. 비어 있는 상태의 안내와 새로고침·다음 버튼 상태를 살핀다. 실제 영상이나 자격 증명을 넣을 필요는 없다.
4. 3–4분: 입력/옵션/도구 안내를 살핀다. 도구 선택이나 다운로드를 실행하려면 별도의 원본·출력 범위를 먼저 정한다. 첫 UI 평가는 다운로드 없이 가능하다.
5. 4–5분: 창을 닫고 종료를 확인한 뒤 같은 `$start`로 다시 실행한다. 테마 선택과 시험 경로 복원을 확인한다. 불편한 점은 화면 이름·동작 순서·기대한 결과·실제 결과로 적는다.

## 확인한 source 계약 / 한계

- `accessible_cpu_main.rs`의 data override와 InstanceGuard는 새 `$data`를 사용한다. 다른 실행의 기존 data와 혼합하지 않는다.
- `accessible.rs` 초기 output은 DLPAD_LIBRARY_ROOT, 설정 초기 표시는 `--settings`다.
- `accessible_library.rs` 명시 Default 경로는 DLPAD_TEST_DOCUMENTS가 있으면 그 아래 dlpad로 해결한다. engine 기본 경로는 OS Documents/dlpad지만 위 override가 이를 피한다.
- 입력·프로세스·도구·네트워크의 모든 행동을 이 안내가 sandbox로 강제하지는 않는다. 사용자가 다른 경로를 선택하거나 도구/다운로드를 실행하면 해당 행동은 별도로 검토해야 한다.
- 실제 실행·사용자 물리 IME·reader·DPI 검증은 이 문서 작성에서 수행하지 않았다.

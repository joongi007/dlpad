# S00: 최신 Windows 다섯 화면 UI 일관성

- 상태: in-progress
- 분류: quality
- 우선순위: P0
- 요구사항: R01, R05, R06, R08, R13
- 담당/수정 범위: 주 담당 통합; 이 문서와 실제 화면에서 확인한 UI 파일. 보조 에이전트는 /tmp의 감사·collector 후보만 담당하며 프로젝트 파일은 수정하지 않는다.
- 선행 작업: Windows whole-fifth 검증, 도구 파일 선택 실제 GUI 검증

## 사용자 결과

도구 선택과 저장의 관계, 활성 도구와 다음 행동, 빈 화면의 안내를 GTMUX 기준으로 일관되게 정리한다. 기능 시험 통과를 전체 UI/UX 완료로 표현하지 않는다.

## 계약과 범위

PRODUCT.md, DESIGN.md, docs/UX.md와 기존 UI ADR을 기준으로 판단한다. 먼저 최신 MSVC CPU 실행 파일로 download/settings/library/tools/history를 가상 화면에서 관찰한다. 과거 63개 캡처는 최신 소스의 인수 근거로 대체하지 않는다. 공유 schema, Cargo workspace 및 lockfile 변경은 없다. 긴 경로의 원 값과 설정 저장 의미를 보존한다.

예상 UI 수정 대상은 accessible_tools.rs, accessible_tool_preferences.rs, accessible.rs 및 필요한 catalog 문자열이다. 실제 결함 확인 후 파일 범위를 확정한다. CI 실행 중 소스는 변경하지 않는다. 사용자 데스크톱으로 fallback하지 않는다.

## 인수 조건

- 최신 whole proof의 정확한 CPU SHA와 원본 로그를 확인한다.
- Light/Dark와 800/960 크기, 앱 배율 150/200에서 다섯 화면을 직접 검토한다. 앱 배율 관측을 실제 OS DPI 검증으로 표현하지 않는다.
- 활성 도구와 Use 행동, 세 도구 경로와 명시적 Save의 관계가 명확하며 긴 경로를 회수할 수 있다.
- 로딩·빈 목록·오류는 서로 구별되며 다음 행동을 설명한다.
- 수정 후 필요한 Windows 빌드 및 실제 가상 GUI 회귀를 실행한다. 물리 IME·포인터·reader와 사용자 최종 인수는 별도 미검증으로 남긴다.

## 검증과 handoff

후보 감사: /tmp/dlpad-five-screen-ux-audit/AUDIT.md. 최신 collector: /tmp/dlpad-current-ui-layout/collect.py. 아직 native 실행 결과와 시각 인수는 없다.

실행 예정 명령:

```sh
python3 /tmp/dlpad-current-ui-layout/collect.py --proof docs/validation/evidence/windows-msvc-tool-executable-picker-whole-fifth-20261011/result.json --output docs/validation/evidence/current-ui-polish-all-screens-layout-20261011
```

## 실제 첫 수집 결과

2026-10-11 위 명령은 exit 1로 종료했다. Windows observer 컴파일은 성공했으나 첫 Light 실행의 800×600 download에서 저장 폴더 입력의 content reachability를 확인하지 못했다. 원본 32회 wheel 기록에서 대상 중심 y565→523, anchor y105→63 이후 위치가 유지되고 대상은 하단 고정 영역 아래에 남았다. 주 담당이 `s10-scroll-failed.png`와 `initial-tools-960.png`를 직접 확인했다.

증거는 `docs/validation/evidence/current-ui-polish-all-screens-layout-20261011/`에 보존한다. 첫 프로세스는 실패 cleanup으로 owned Job 종료가 필요했으므로 natural completion PASS가 아니다. private desktop 종료·입력 desktop 보존은 원본 receipt에 기록되어 있다. 63개 수집·Dark 재실행·배율·SQLite 인수는 완료하지 못했다. 최신 five-screen 인수 성공으로 보고하지 않는다.

후속은 observer의 scroll route/element geometry와 제품 ScrollArea·고정 footer를 구분하는 분석이다. 오류 검증을 제거하거나 반복 상한을 늘려 통과시키지 않는다. 별도 도구 summary 수정 후보는 /tmp/dlpad-tools-active-summary에 있으며 아직 제품에 통합하지 않았다. 최종 CI와 8시간 시험은 별도 소스 범위로 계속 실행 중이다.

두 번째 수집은 `/tmp/dlpad-current-ui-layout/collect-v2.py`로 실행했다. 첫 실패에서 URL anchor가 y63까지 올라갔고 CentralPanel의 16px margin을 고려하면 본문 위에 wheel을 보냈을 가능성이 있다. 후보는 wheel 좌표만 navigation 하단+32와 footer 상단-32 사이로 제한하고 원 좌표·실제 좌표·경계를 기록한다. 원 32회 상한, content 가시성 판정, 실제 정리와 63개 PNG 조건은 바꾸지 않았다. 결과는 `current-ui-polish-all-screens-layout-v2-20261011`의 별도 증거로 남긴다. 원 제품과 관측기를 덮어쓰지 않았다.

주 담당은 최신 첫 실행의 Tools/History/Library PNG를 직접 보았다. History는 빈 이력에서도 긴 재개 제한 안내가 최상단을 차지하며 다음 행동 안내가 없다. Library는 폴더 선택과 두 줄의 빈 목록 안내가 있어 비교 기준으로 삼는다. Tools는 빈 설치 상태보다 HTTPS/SHA와 배포 target 정보가 앞서 읽힌다. 이 관찰은 Light 960 화면에 한정하며 전체 테마·상호작용 인수를 뜻하지 않는다. 화면별 역할을 유지하면서 상태와 다음 행동을 먼저 설명하도록 후속 범위를 정한다.

두 번째 수집 중 full-800 PNG를 직접 확인한 결과, 중심점 검사만으로는 입력 전체 가시성을 증명하지 못했다. path geometry는 center y510/height32이지만 하단 footer가 입력을 가린다. 따라서 v2의 해당 boolean을 전체 입력 가시성 PASS로 승격하지 않는다. 후속 observer는 대상 전체 사각형과 실제 코드의 navigation/central margin 및 footer margin을 대조해 더 엄격하게 검사한다. 현재 v2 원본은 보존하며 제품 결함 여부는 재시험으로 구분한다.

## 2026-10-12 구현 통합

주 담당이 후보의 전체 diff와 원본 SHA를 확인한 후 accessible.rs, accessible_tool_preferences.rs, accessible_tools.rs, catalog.rs 네 파일을 통합했다. 각 실행 경로 바로 아래에 동일 도구의 Manual/Automatic 모드를 배치하고 공통 Save/status는 세 그룹 뒤에 유지한다. 원 changed/revision/queue/late-save 처리와 picker busy/close gates는 그대로다. 활성 도구 버전·상태·Use는 한 행으로 묶고 정확한 canonical 경로는 기본 접힌 읽기 전용 상세와 명시적 복사 행동으로 남긴다. 새 의존성·schema·lock 변경은 없다. 자동 생성 접근성 numeric ID 보존을 주장하지 않으며 새 GUI 회귀가 필요하다.

기존 최종 CI는 변경 직전 소스에 대해 실제 7개 명령을 통과했고 독립 root-review를 보존했다. 이번 UI 변경을 포함한 결과로 승격하지 않는다. 새 Windows whole 검증을 `windows-msvc-five-screen-ui-polish-whole-20261012`에 실행 중이다.

v2 실제 observer는 63개 PNG와 두 프로세스의 natural 종료를 기록했지만 collector가 원래 `settings-result.json` 대신 `result.json`을 읽어 실패했다. 전체 입력 가시성 약점도 있어 시각 인수는 아니다. v3는 전체 사각형 판정을 강화했으나 반쯤 가려진 URL을 scroll anchor로도 거절해 `No actual visible owned scroll anchor`로 실패했다. v4는 타깃 전체 가시성 조건은 유지하고 wheel anchor만 본문과 실제 교집합이 있는 Edit의 교집합 중심으로 분리한다. 이번 UI 변경 이후 새 whole proof로 실행한다. 모든 원본 실패와 정리 기록을 보존한다.

첫 UI whole 실행은 fmt 성공 후 Clippy에서 E0433/E0425로 실패했다. 주 담당의 통합 복사에서 반복 변수로 마지막 설정 파일을 가리킨 채 도구 화면 후보를 쓴 것이 원인이었다. accessible_tool_preferences.rs와 accessible_tools.rs 각각의 명시적 대상 경로로 올바른 후보를 복원했다. 원 실패 로그를 보존하고 `windows-msvc-five-screen-ui-polish-whole-second-20261012`에 네 검사를 다시 실행한다. 제품 정책이나 검사 조건을 완화하지 않았다.

두 번째 UI whole의 fmt/clippy/test/release는 실제 exit 0으로 종료했다. test 원본은 43 harness, 1,089 passed / 0 failed / 28 ignored다. 전체 화면 인수 완료로 승격하지 않는다. 후속 화면 collector v5는 입력 필드가 화면 밖으로 사라져도 원 HWND의 navigation/footer와 client geometry로 확인한 본문 내부에만 wheel을 보낸다. 대상 전체 사각형 가시성 검사는 v3처럼 유지하며, 코드로 알려진 margin 가정과 실제 좌표를 원 receipt에 기록한다. 관측기 보강과 제품 UI 수정을 구분한다. 최신 whole proof를 고정한 실제 v5 수집을 `current-ui-polish-all-screens-layout-v5-20261012`에 시작했다. 현재 시각·SQLite 독립 인수는 대기 중이다.

주 담당의 whole 독립 검토는 `root-review.json`에 PASS_SCOPED로 보존했다. 최신 CPU는 9,658,880B(SHA bfb863eafda8fd97f7d24f626bdedc6c050a6d9ad23a75dfc6d5ee5ce452bedb)로 이전보다 5,120B 증가했다. 새 dependency/thread/I/O worker는 없다. 접힌 경로 상세에는 경로 TextEdit를 만들지 않고 명시적 펼침에만 생성한다. 실제 idle 비용은 별도 최신 인수가 필요하며 작은 binary 증가로 성능 인수를 대신하지 않는다.

v5는 실제 C# 컴파일에서 CS0136으로 실패했고 앱은 실행하지 않았다. body 경계의 내부 `bounds`와 이후 target의 `bounds` 이름이 충돌했다. v6는 후자를 `targetBounds`로 변경한 것만으로 새 수집을 시작한다. 원 v5 실패와 natural host 종료를 보존하며 제품 source/전체 빌드 인수에는 영향이 없다.

v6는 실제 compile 성공 후 작은 창의 저장 폴더 전체 가시성에서 실패했다. body 중심 y280으로 보낸 32회 wheel에서 대상 y565→551 이후 멈췄다. root가 실패 PNG를 직접 봤으며 제품 max-scroll 결함으로 단정하지 않는다. 기존 성공 S10 관측기와 달리 move와 wheel 사이 대기가 없고 위치도 다르다. v7는 move 후 150ms Pump 한 변수만 변경해 재실행한다. 위치·상한·전체 타깃 가시성·정리 조건은 동일하며 모든 실패를 보존한다.

실제 v11 도구 회귀는 save/reopen 후 Use 단계에서 접힌 경로를 예전 plain Text로 찾다가 실패했다. root는 managed-refreshed PNG에서 활성 상태/Use/접힌 경로가 의도한 위치에 있음을 확인했다. v12는 접힘을 실제 확인하고 펼침 행동 후 readonly Edit의 정확 경로를 확인하도록 보강할 예정이다. save/reopen 수집만으로 전체 다섯 단계 성공이나 DB 인수를 주장하지 않는다.

v7의 move 뒤 대기만 추가한 실제 시험은 32회에서 대상 y565가 그대로여서 실패했다. 대기만으로 해결된다고 주장하지 않는다. v8는 같은 대기·y 위치·전체 가시성 검사를 유지하고 wheel x만 원 성공 S10 관측기처럼 본문 왼쪽 x24로 바꾼 대조다. body/preview 위치 차이를 분리하며 원 실패 및 cleanup을 보존한다. 이 시험으로 실제 제품 전역 스크롤 정상 여부를 증명한 것은 아니다.

v12 도구 실제 회귀를 시작했다. v11의 compact 화면을 그대로 검토한 뒤 실제 Toggle와 readonly ValuePattern으로 펼친 경로를 원 canonical fixture 경로와 대조하며 Use·저장·재실행 검사를 유지한다. 클립보드 복사를 실행하지 않으며 복사 성공을 주장하지 않는다.

v8의 wheel x 대조도 실제 전체 입력 가시성에서 실패했다. 위치·간격 추정만으로 제품이나 관측기의 원인을 확정하지 않는다. 다음 단계는 기본 OFF bounded fixture로 실제 native CursorMoved/CursorLeft/MouseWheel과 egui 입력·ScrollArea offset/inner rect/content size를 대조하는 것이다. 추가 수정 범위는 accessible.rs/software_host.rs의 개발 계측이며 새 schema·dependency·thread·디스크 I/O를 도입하지 않는다. 원 pointer route를 바꾸거나 CursorLeft를 무시하지 않는다.

v12의 실제 다섯 phase는 오류 없이 종료되어 REVIEW_REQUIRED를 반환했다. root는 펼친 경로 PNG와 readonly 값의 canonical 일치를 직접 확인했다. 원 로그·DB·전체 phase 독립 검토는 실행 중이며 전체 UI 인수를 의미하지 않는다.

v12 종료 후 root가 실제 5phase 원 receipts·raw private Job 종료·DB5·모달10·현재 source/artifact pins를 독립 reviewer로 대조했다. 반환값은 SCOPED_RECEIPTS_VERIFIED_ROOT_PIXEL_REVIEW_PENDING였으며 이후 root가 terminal5·파일 dialog3·펼친 경로1 PNG를 직접 확인했다. `windows-tool-executable-browser-private-v12-ui-polish-20261012/root-review.json`에 PASS_SCOPED_ACTUAL_GUI_AND_DATABASE를 보존했다. 새 배치의 실제 선택/취소/저장/재실행/managed Use·원 창 닫기의 기능 인수이며 전체 다섯 화면·스크롤·물리 포인터·복사·사용자 디자인 인수는 아니다.

후속 수정 범위에 accessible_history.rs와 catalog.rs를 추가한다. 실제 최신 History 빈 화면에서 확인한 긴 기술 안내를 접을 수 있게 하고, 재개 가능한 작업에는 기존 제한을 계속 표시한다. 오류를 빈 목록으로 동시에 안내하지 않으며 상태별 다음 행동을 정리한다. 원 refresh/next/Resume eligibility와 job 의미를 보존하고 실제 egui 접근성 출력의 실패 상태 회귀를 검사한다.

계측 준비에서 기존 C# 관측기의 CreateProcess는 standard stderr를 전달하지 않는다는 제약을 확인했다. env flag만 켜면 trace를 얻는다고 주장하지 않는다. instrumented observer의 별도 소유 pipe·명시적 handle-list·유한 reader/EOF 정리 계약을 준비하며 기존 제품 입력 경로와 ordinary GUI 관측기를 변경하지 않는다. 계측 빌드/실행은 아직 미수행이다. 현재까지 실패한 스크롤 시험을 성공으로 바꿔 쓰지 않는다.

작업 이력 UI와 실제 HistoryPanel.show/AccessKit 실패 회귀 3개를 root가 전체 후보 diff·원본 SHA 대조 후 통합했다. 소유된 corrupt 임시 DB의 실제 읽기 실패, 로딩/빈 목록 문구의 구분, 기존 행 보존 및 Resume/refresh/next gates를 검사한다. 테스트 UI 출력은 물리 GUI 인수를 대신하지 않는다.

스크롤 계측은 기존 Windows first-paint-fixture와 정확한 DLPAD_TEST_SCROLL_TRACE=1에서만 활성화한다. 첫 실제 wheel 전에는 Candidate 프레임 예산을 차감하지 않고 이후 최대128프레임의 숫자만 stderr에 기록한다. Host는 pointer/wheel 이벤트가 있는 프레임만 유한 예산을 사용한다. URL·경로·입력 텍스트는 기록하지 않는다. 새 dependency/thread/디스크 파일 I/O는 없다. 일반 후보는 예산0이고 env 조회도 초기화 시 한 번뿐이다. 실제 원 CursorLeft를 무시하거나 입력 의미를 바꾸지 않는다. 시험 계측의 관측 비용을 일반 GUI 비용으로 승격하지 않는다.

root가 Rust1.94 rustfmt로 네 파일을 정리한 후 `windows-msvc-history-ui-scroll-trace-whole-20261012`에 Windows 네 검사를 실행했다. 아직 결과는 없다. 별도 debug 계측 observer는 stdout/stderr handle allowlist와 bounded reader를 준비했으나 컴파일·실행 전이다. 이번 변경 이후에는 이전 whole/UI receipts를 최신 소스의 인수로 표현하지 않는다.

검증 상태와 수정 근거를 실제 결과 확인 후 기록한다. P0 또는 정식 릴리스 완료를 뜻하지 않는다.

최신 History fix source의 전체 화면 관측은 natural exit0/REVIEW_REQUIRED로 종료했다. root가 원 로그·current source/artifact/pins·두 lifecycle·필수63+추가6 PNG IHDR/SHA 및12조 경로 geometry를 독립 검토해 root-semantic-review.json에 scoped PASS를 보존했다. root는 최신 Dark 이력800과 다운로드 전체800도 직접 확인했다. 원 physical pointer/OS DPI/모든pixel/사용자 디자인 인수는 false를 유지한다. 앞선 오류 native case4PNG 직접 인수와 결합하되 실제 로딩 미관찰을 완료로 표현하지 않는다.

동일 History v5 관측기를 최신 제품에 실행한 v6-product-fix는 실제 empty/error 두 case natural0/Job0으로 종료했다. root가 원 raw 로그·현재source/모든pins·실제corrupt bytes/sentinel/noDBcreate·네 PNG SHA를 독립 검증하고800/960 네 캡처를 직접 확인해 root-review.json에 scoped PASS를 보존했다. 제품 변경으로 기존 오류 진입 실패가 해결됐으며 관측기 조건을 완화하지 않았다. 실제 로딩은 두 크기에서 관찰되지 않아 loading pixel 인수는 false이고 Resume 실행/물리 입력도 미검증이다. 전체 UI 관측기는 계속 실행 중이다.

History recovery navigation whole의 Windows 네 검사가 실제 exit0이다. root가 원 로그·43harness/1,101passed/0failed/28ignored·현재source 및 original GNU/lock/release pins를 독립 검증했다. CPU는9,668,608B로 직전과 동일 크기이며 SHA는d03ef200b554880f4d11660598de67c57e9a95995fa3f0bb35d72de59870f74f다. 수정 전 실패를 재현한 동일 v5 History 관측기와 최신 전체 화면 관측기를 새 output에서 병렬 실행한다. 관측기 조건 완화 없이 제품 변화로 오류 진입이 해결되는지 확인한다. 결과는 아직 대기 중이다.

root가 History recovery navigation 후보 전체 diff와 원 bytes/SHA를 검토해 두 제품 파일을 통합했다. 이력 화면 진입은 ready와 독립이며 read refresh/next와 Resume 권한을 분리했다. Resume 표시·take_resume 실행·start의 기존 ready/worker/closewait 조건을 유지한다. 실제 corrupt startup Candidate→navigation→read retry와 128행 Panel의 read/Resume 분리 회귀를 추가했다. 기존 native v5 오류 진입 실패가 수정 전 재현 증거다. `windows-msvc-history-recovery-navigation-whole-20261012` 네 검사를 실행한다. 이전69 UI/설치ZIP 인수는 변경 전 소스에 한정하며 최신 소스 인수로 승격하지 않는다.

반응형 scroll-contract v5 전체 관측은 exit0/REVIEW_REQUIRED로 종료했다. 필수63+추가6 PNG의 semantic inventory와 원 두 native cleanup을 collector가 기록했으며 root 독립 검토를 준비한다. root는 최신 Light 다운로드960/설정960와 Dark 이력800/도구960/재실행 파일목록800 다섯 대표 캡처를 직접 보았다. 좁은 경로 전체 폭과 목록 가로 배치를 확인했다. 모든69 screenshot 및 실제 hover/OS DPI 인수를 주장하지 않는다. corrupt DB의 이력 navigation ready gate 수정은 아직 후보 단계다.

History v5의 empty case는 실제 두 크기 state 인수를 통과했지만 corrupt DB case의 화면 진입은 실패했다. root가 실제 source와 screenshot을 대조해 제품의 navigation `clicked && self.ready` 조건을 원인으로 확인했다. 복구 확인 실패 시 활성화된 이력 버튼을 눌러도 화면이 열리지 않는다. History show의 동일 ready gate가 read-only refresh/next와 Resume도 함께 차단한다. 후속 범위는 accessible.rs/accessible_history.rs에서 읽기 조회와 재개 권한을 구분하는 실제 제품 수정이다. 읽기 동작의 DB 생성 금지와 원 데이터 보존을 유지하고 Resume/take_resume/start는 기존 ready gate를 유지한다. 실제 Candidate navigation 실패 상태 회귀와 native corrupt fixture를 인수 조건으로 추가한다. 실행 중인 전체 UI proof를 변경하지 않도록 후보는 `/tmp`에서 준비한다. 기존 설치 ZIP은 이 문제 수정 전 source이며 최신 최종 인수로 승격하지 않는다.

최신 History v3는 탐색 버튼의 bounds 클릭이 본문용 EnsureContent를 호출해 navigation을 본문 밖으로 거절하면서 실패했다. 원 실패/cleanup을 보존하고 navigation은 원 HWND client 전체 containment, refresh는 본문 containment로 구분하는 `/tmp` v4를 준비한다. 전역 입력이나 사용자 데스크톱으로 우회하지 않는다. 반응형 전체 화면 v4는 기존32px/fullrect를 유지하며 현재 실행 중이다. latest whole에 묶인 의존성 graph 세 명령은 종료했고 fresh preview/payload ZIP 생성을 시작했다. 최신 UI/설치 인수 완료나 기존 사용자 링크 교체는 아직 하지 않는다.

최신 ordinary GUI 첫 수집은 기존 관측기의 항상 같은 행 조건 `Actual same32px path row`에서 실패했다. root가 실제 최신 다운로드960 PNG를 직접 확인했으며 경로는 카드 전체 폭, 선택은 다음 행에 표시됐다. 변경된 DESIGN의 좁은 폭 예외에 맞춰 관측기를 수정하되 각 입력/버튼32px와 실제 전체 가시성 검사를 유지한다. History v2는 `No-focus action pattern unsupported`에서 실패해 원 UIA node/control pattern을 조사한다. 두 앱의 owned cleanup과 원 실패를 보존하며 성공 인수로 승격하지 않는다. 설치 graph 수집은 최신 whole를 고정해 별도 실행 중이다.

responsive second whole의 실제 fmt/clippy/test/release가 모두 exit0이며 root가 원 로그·Job 종료·현재source·GNU/lock·세 release 산출물 pins를 독립 검증했다. 43 harness / 1,097 passed / 0 failed / 28 ignored다. CPU는9,668,608B로 직전history/trace ordinary release보다512B 증가했다. 새 dependency/thread/schema 변경은 없다. 실제 idle 비용은 별도 게이트다. 최신 proof로 History empty/error v2와 전체 화면 release-focus v2를 별도 private desktop에서 병렬 실행한다. 두 native 결과는 아직 대기 중이다.

첫 responsive whole은 Clippy drop_non_drop에서 실패했다. 테스트 closure의 불필요한 명시 drop만 제거했으며 검사 예외를 추가하지 않았다. 별도 second output에서 네 검사를 재실행한다. 원 실패를 보존한다.

root가 focus69 독립 reviewer를 직접 실행했다. 원 native 두 phase와 필수63/추가6 reachability PNG의 정확 manifest·경로 전체 사각형을 확인해 `root-semantic-review.json`에 보존했다. 원 collector FAIL은 그대로다. root는 Dark800 다운로드 전체와 재실행 설정800 PNG를 직접 확인했다. 경로 입력은 고정 footer와 겹치지 않았으며 설정 행과 접힌 상세 배치를 확인했다. 아직 모든69 pixel 인수를 주장하지 않는다. 이 증거는 responsive folder 변경 전 소스이며 current_source_matches=false를 명시적으로 유지한다.

root가 staged path_picker.rs 전체 diff와 원 SHA를 확인해 제품에 통합했다. 좁은 영역에서는 경로 입력 전체 폭과 다음 행 선택 버튼을 사용하고 넓은 영역은 가로 배치를 유지한다. 실제 Context/AccessKit 회귀는 268/720 폭 geometry, resize 후 입력 ID·focus·텍스트 이벤트, disabled/미지원 선택 버튼을 검사한다. caret 위치 전체 인수를 주장하지 않는다. DESIGN의 기존 항상 가로 행 문장을 실제 반응형 계약으로 갱신했다. `windows-msvc-responsive-folder-ui-whole-20261012` 네 검사를 실행 중이다.

앞선 release-focus native는 두 phase REVIEW_REQUIRED/natural 종료였으나 collector의 exact63 고정 PNG 조건에서 실패했다. 실제69 파일을 보존하고 필수 화면 및 추가 focus 캡처를 별도 독립 검토한다. History empty/error 첫 시험은 C# CS0170 미초기화 Rect로 앱 실행 전 실패했다. `/tmp` v2는 해당 초기화만 수정하며 원 실패를 보존한다. 최신 제품 source proof가 나온 뒤 다시 실행한다.

진행 중인 ordinary release-focus 시험에서 실제 저장 폴더 Edit의 UIA 포커스 ACK와 전체 사각형 가시성을 관찰했다. 원 기록은 client top451/bottom483, 경계56..504, height32, in_owned_content=true다. 이는 원 마우스 wheel 시험 실패와 별도 경로이며 전체 시나리오 terminal/63 PNG/다른 화면 성공을 아직 뜻하지 않는다. 원 receipt는 `current-ui-release-focus-layout-20261012/gui/first/s10-focus-scroll.json`에 보존한다.

저장 경로 읽기 편의성 후속 범위에 path_picker.rs의 folder_field를 추가한다. 300px 설정 카드의 경로 입력이 약147px로 줄어드는 실제 관찰에 따라 좁은 영역에서만 폴더 선택을 다음 행에 배치하는 후보를 준비한다. 원 입력 ID/길이 상한/포커스 scroll/비활성화/선택 정책을 유지하고 넓은 파일 목록은 기존 가로 행을 유지한다. 후보는 아직 제품에 통합하지 않았으며 실행 중인 release-focus GUI의 source를 먼저 고정한다. 인수 조건은 800/960 실제 입력·버튼 전체 가시성, 긴 경로의 입력·선택·원 값 보존, 공통32px 크기와 카드 폭 준수다.

root가 최신 계측 실행의 initial-history-960.png를 직접 확인했다. 빈 작업 안내와 다운로드 URL 다음 행동이 보이고 재개 안내는 접혀 있어 이전의 긴 본문을 제거했다. 이 직접 검토는 Light 960 빈 상태에 한정한다. 오류/재개 가능/800 크기/테마별 상태는 별도 native 수집이 필요하다. 하위 담당은 `/tmp`에서 History empty/error 전용 실행 collector와 키보드/UIA 전체 입력 가시성 collector를 각각 준비하며 제품 파일을 동시에 수정하지 않는다.

실제 private v8 계측 수집은 종료했다. 원 stderr 38,996B의 SHA/EOF와 두 producer 각각128줄 상한을 검증했다. GUI 자체는 저장 폴더 전체 가시성에서 FAIL이며 owned child Job 강제 정리를 기록했다. 원 입력에서 CursorMoved 뒤 CursorLeft가 wheel 전에 발생했고, 실제 download frame은 pointer=None, offset=0, content height818, inner y64..488을 기록했다. 따라서 휠은 전달됐지만 이 시험에서 hover 대상이 사라져 소비되지 않았음을 확인했다. 일반 사용자의 물리 스크롤 결함 또는 정상 완료로 확대하지 않는다. 원 실패는 보존하고 UIA/키보드 포커스로 실제 입력 전체 가시성을 별도 검증한다. 원 CursorLeft나 제품 입력 정책을 변경하지 않는다.

별도 scroll-trace debug 빌드는 실제 exit 0으로 종료했다. 시험 바이너리는 24,302,592B / SHA 1838806b72fee53f7b02a050e71ab99d36e2db18062d9399c2461f26908f7271이며 일반 release 산출물을 대체하지 않았다. 실제 private GUI 계측은 다음 단계이며 빌드 성공으로 스크롤 성공을 주장하지 않는다.

최신 history/scroll-trace whole의 Windows fmt/clippy/test/release는 모두 실제 exit 0이다. root가 원 로그의 owned Job 종료와 현재 source/기존 GNU/lock/release artifact pins를 독립 대조했다. 43 harness에서 1,095 passed / 0 failed / 28 ignored이며 `windows-msvc-history-ui-scroll-trace-whole-20261012/root-review.json`에 보존했다. 추가 History 렌더링 회귀는 두 앱 대상에서 실행됐다. 전체 UI의 시각·스크롤 인수는 여전히 미완료다. 별도 `windows-msvc-scroll-trace-debug-build-20261012` 시험 빌드를 시작했으며 ordinary release를 보존하도록 검사한다.

## 2026-10-12 주 담당 최신 확대·설정·도구 화면 직접 검토

최신 current-ui-history-fix-layout-20261012의 first/initial-settings-960, first/initial-tools-800, first/initial-zoom-200-download, first/initial-zoom-200-settings, restart/initial-library-full-800 PNG를 추가 직접 확인했다. 200% 설정 화면의 상단 설명 일부가 scroll viewport 경계에 걸려 보인다. 실제 포커스 공개에 따른 scroll offset인지 레이아웃 결함인지 구분하기 전에는 정상·버그 어느 쪽으로도 확정하지 않는다. UI 감사 담당 diagnostic_inputs는 읽기 및 /tmp 보고서만 수행하고 프로젝트 수정은 주 담당이 통합한다. diagnostic_cache는 S05 실제 CloseWait 실패를 독립 읽기 진단한다. 제품 파일 동시 수정은 없다. 69장 전체의 시각 인수, 물리 hover/scroll/IME/OS DPI 및 사용자 평가 완료를 주장하지 않는다.

## 2026-10-12 도구 누락 안내의 실제 복구 행동 교정

독립 감사의 최신 다운로드 800/150/200 캡처와 Tools/DownloadEnvironment 원 코드를 주 담당이 대조했다. 기존 '도구 화면에서 실행 파일을 지정' 안내는 실제 browse가 있는 '다운로드 도구 및 인증 설정'과 불일치했고, yt-dlp tab 선택은 활성 도구 사용과 달랐다. catalog.rs의 MEDIA_TOOLS/MEDIA_TOOL_REQUIRED를 관리 설치 후 사용 또는 다운로드 도구 및 인증 설정의 실행 파일 선택이라는 두 실제 대안으로 교정했다. 새 권한·자동 설치·worker/schema/dependency 변경은 없다. 인수는 다음 source 전체 검사와 800/150/200 실제 줄바꿈·복구 행동 확인이다. 단순 문구를 복제한 테스트는 추가하지 않는다. 현재 README의 history-fix 설치 ZIP은 이 추가 문구 교정 이전이며 다음 검증·패키징 후에 갱신한다.

## 2026-10-12 설정 포커스 설명 보존 회귀

담당 후보 accessible_settings.rs의 원 SHA를 대조했다. 주 담당이 실제 Appearance.show/AccessKit Focus 회귀를 먼저 기존 control-only Center 스크롤에 붙여 RED 검사한다. 원 200% 화면의 clip을 row 손상이라 단정하지 않으며, 전체 설명·label·control이 들어갈 때 그 문맥을 최소 이동으로 보존하는 UX 개선이다. 큰 row는 control만 최소 공개한다. 원 ID·disabled·popup·worker·scale 정책과 수정 전 설치 ZIP은 유지한다. 전체 source가 검사 중 변경되지 않도록 수정 후 검사한다.

실제 RED 결과: windows-msvc-settings-focus-context-red-20261012/test.log에서 원 Center 동작은 200%/960px의 화면 테마 label bounds [8,37.5]–[64.4,52.5]가 clip [0,43.5]–[468,149.5] 밖이라는 실제 회귀로 실패했다(test exit101, 원 Job natural/final zero, active0). 컴파일 오류가 아닌 인수 조건 실패다. 주 담당이 full-row minimal reveal 후보를 통합하고 불필요한 closure drop을 제거했다. GREEN 및 전체 strict 검사 결과는 아직 대기한다.

첫 GREEN 통합 whole은 fmt 통과 후 Clippy에서 회귀 closure의 불필요한 mut로 실패했다. 원 실패 로그를 보존하고 mut를 제거해 fresh whole-v2에서 같은 -D warnings 검사를 재실행했다. 실패를 허용하거나 Clippy gate를 완화하지 않았다.

whole-v2는 mut 제거로 달라진 closure 줄바꿈 때문에 fmt에서 실패했다. 주 담당이 고정 rustfmt1.94로 staged 파일을 포맷하고 복사해 수정했다. 두 실패 원본은 보존하며 whole-v3에서 전체 검증한다.

whole-v3의 실제 fmt·strict Clippy는 exit0이다. 실제 accessible/CPU 두 harness에서 actual_settings_focus_keeps_stacked_label_help_and_control_in_200_percent_viewport가 모두 성공했다. 이전 control-only RED와 같은 800/960의 200% viewport/초기 scroll/실제 AccessKit Focus 입력에서 label·설명·control 전체 공개를 검사했다. 이는 Context 회귀 GREEN이며 가상 OS 캡처 인수는 별도로 남는다. 전체 workspace 테스트·release는 아직 실행 중이므로 최종 whole 성공을 주장하지 않는다.

whole-v3의 원 test.log에서 주 담당이 43개 harness의 test result를 합산해 1,103 passed/0 failed/28 ignored와 두 실제 focus 회귀 성공을 확인했다. 원 Job natural/final-zero, final-active0도 test.log에 기록됐다. 릴리스 및 전체 source/artifact 최종 대조는 terminal 결과 후 별도 수행한다.

whole-v3 최종 결과: Windows fmt/clippy/test/release 모두 실제 exit0, 43 harness/1,103 passed/0 failed/28 ignored/source623. 주 담당이 require_current_whole과 /tmp/dlpad-review-preferences-whole.py로 원 명령·로그·Job 정리·현재 source·GNU 원본·Cargo.lock·release artifact를 독립 대조했다. CPU 9,669,120B/SHA d5904fe70dadbec9f9d08caf91825771a6c0e47fb6766ab7a0ebb082b602b338(+512B vs 이전 history-fix), installer 4,310,016B/SHA1a57da1eb4bbb8b7438abf9457104faeb5df4bf4f08bc3b3464488b7524f33b2, CLI974,848B/SHA불변이다. 새 dependency·lock 변경은 없다. 가상 GUI/idle 비용/새 설치 ZIP은 이 whole 성공으로 대신하지 않는다.

## 2026-10-12 800폭 확대 GUI 인수 확장

담당 diagnostic_inputs가 /tmp/dlpad-ui-800-zoom-focus에서 기존 필수63+추가쌍 observer를 유지하고 800×600의150/200%·5화면·Light/Dark focus 전후 및 설정 펼침을 별도48PNG/receipt로 준비했다. 프로젝트 수정·native 실행은 주 담당만 수행한다. 주 담당 읽기 검토에서 첫 실행 initial/selected 단계가 동일 create-only 확장 폴더를 중복 생성할 수 있음을 찾아 initial 단계에만 실행하도록 후보를 교정했다. 확장 UIA focus·전체 bounds·설정 Toggle ACK를 실제 대조하며 물리 hover/wheel/OS DPI 성공으로 주장하지 않는다. 긴 복구 안내의 raster 줄바꿈은 별도 직접 검토한다.

최신 std GUI가 생성한 first/initial-zoom-200-settings와 download PNG를 주 담당이 직접 확인했다. 설정의 테마 label·설명·Combo는 전체 표시되어 이전 얇은 글자 조각 문제가 개선됐다. 반면 같은 캡처에서 배율 Combo는 viewport 아래이며, 선택 직후 실제 focus가 어느 widget에 있는지와 추가 스크롤로 접근 가능한지를 구분하기 전에는 확대 UI 완료로 처리하지 않는다. 복구 안내는 200%에서 두 줄로 읽히며 실제 도구 설정 위치를 안내한다. 이 중간 캡처 관찰은 terminal cleanup·전체PNG review를 대신하지 않는다.

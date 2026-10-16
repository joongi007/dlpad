# S10: 모든 진단 제어 요소의 포커스·ancestor 스크롤

- 상태: in-progress (root 제품 통합 완료; Windows 전체 검증 진행, 실제 GUI 인수 대기)
- 분류: fix
- 우선순위: P0
- 요구사항: R13, R08
- 담당/수정 범위: diagnostic_cache; `/tmp/dlpad-accessible-diagnostics-all-focus.rs` 및 본 문서만
- 선행 작업: report-focus-scroll; 실제 windows-s10-report-save-content-clip-20261004T130318Z 초기 JSON 성공/Markdown 재선택 실패

## 사용자 결과

키보드·접근성으로 Markdown/JSON, 로그 편집·생략, MPD, 저장·페이지 제어를 선택하면 해당 제어가 diagnostic-controls와 바깥 download-content에 순차 노출되어야 한다. 실제 제품 화면 인수 전 완료로 표시하지 않는다.

## 계약과 범위

AGENTS와 기존 agent diagnose 설정/읽기 계약을 따르고 diagnose SKILL.md의 재현→가설→계측→수정→회귀 루프를 적용했다. 피드백 루프는 root가 이미 실행한 private Windows standalone S10 collector다. 실제 초기 JSON export 뒤 Markdown target y=-21/anchor63·scroll 불변 실패를 보존한다. 가설 우선순위: 제어 gained_focus의 ancestor 전달 누락(현재 source 확인), nested viewport/UIA clipping 차이, observer wheel anchor 선택. 기존 export/readonly preview만 전달하고 다른 제어 요소는 전달하지 않는 source 차이를 가장 직접적인 수정 대상으로 삼았다. 새 GUI/Cargo 실행은 root source freeze 때문에 수행하지 않았다.

원 프로젝트 `crates/app/src/accessible_diagnostics.rs` SHA256 `552693a542ddcd95bf45ca90ae7483d188c485f29346f11043687189fc63bf2e`에서 /tmp 사본을 만들었다. 현재 사본 SHA256 `d18ae052d4c1c445e86fd447ade0b132a4f4226e690beea0779d8fc8523084d6`. root가 원 source SHA 일치 확인 후 통합하며 기존 project Rust/Cargo/shared APIs는 root 소유다.

## 수정 내용

각 show 시작 시 exact AccessKit ScrollIntoView 원 NodeId를 widget 소비 전에 최대32개 fixed stack array에 보존한다. reveal_control은 실제 생성된 Response ID와 exact-match하고 response.enabled이며 gained_focus 또는 exact request일 때만 scroll_to_me Center와 ancestor_target을 설정한다. 접근성 ScrollIntoView action도 enabled actual node에 광고한다. 레이블·auto-ID 추측이나 비활성 제어를 실행하는 hook은 없다.

format/copy/log index/edit/omit/remove/edit text/apply/MPD path/attach/각 MPD section header·omit/all MPD/export/save/previous/page DragValue/next에 같은 경로를 연결한다. readonly preview는 기존 explicit exact request·max320 viewport·8KiB pages를 유지하며 focusable로 바꾸지 않는다. mutation active/can_mutate/pending/stop/worker-report ownership 조건과 disabled save/attach·channel 흐름은 그대로다. 긴 I/O와 cache/DB/process 작업을 UI thread에 추가하지 않았다.

## 인수 조건

의미 있는 headless 회귀 초안2개를 사본에 추가했다. 실제 DiagnosticPanel이 생성한 AccessKit Button node에서 format/edit/omit/remove/apply/all-MPD ID를 얻어 Focus와 ScrollIntoView 요청을 보내고 finite ancestor target과 preview 불변을 확인한다. mutate=false의 실제 disabled omit node에 exact request를 보내면 ancestor target이 없어야 한다. 기존 readonly-preview exact/unrelated/Focus 회귀도 유지한다. 이 tests는 아직 컴파일·실행하지 않았다.

root는 scoped 테스트/default·CPU Clippy·fmt와 실제 latest frozen Windows binary의 standalone S10 collector를 다시 실행해야 한다. 특히 정상960x640에서 export 하단→Markdown 상단·log index/edit field·MPD header/omit·저장·페이지의 actual bounds/ancestor offsets/owned PNG와 접근성을 대조한다. 자동 UIA 성공만으로 실제 paint PASS를 주장하지 않는다. 원 collector 실패와 clipboard ERROR5/183은 보존하며 visible fallback은 없다. 동시 ScrollIntoView 요청은 첫32개만 보존하며 초과분은 이 frame의 ancestor 전달에 사용하지 않는다. 초과분을 다음 frame에 자동 보존한다고 주장하지 않으며 이 상한의 실제 접근성 host 적합성도 root 검토 사항이다.

## 검증과 handoff

root는 원본과 사본의 위 SHA를 각각 대조한 뒤 `crates/app/src/accessible_diagnostics.rs`에 통합했다. `windows-diagnostics-focus-workspace-20261004T131658Z`에서 Windows fmt와 Clippy가 종료 성공했고 test/release는 진행 중이다. 실제 Windows 격리 GUI 재검증 전에는 기능 인수 완료로 표시하지 않는다. 이번 통합은 Cargo/schema 변경을 포함하지 않는다.

후속 종료 확인: 위 workspace collector의 fmt/Clippy/test/release 모두 성공했다. root는 `source_final`과 현재 snapshot 및 release SHA `503ce73fce57017a71f79606c0a8b09cf9c71a773ccda3db96e0cf652b9cf543`를 직접 대조하고 evidence의 `root-review.json`에 기록했다. `windows-s10-all-controls-focus-20261004T131828Z` 격리 GUI 시험을 시작했으며 종료 전 성공으로 표시하지 않는다.

실제 GUI 종료 결과: collector exit2/FAIL. 초기 JSON972bytes와 Markdown1479bytes 저장 기록이 생성돼 이전 Markdown 재선택 실패 지점을 넘었다. 이후 `UIA worker deadline exceeded`로 전체 편집/첨부 인수를 완료하지 못했다. owned Job 비움과 원 프로세스 종료, input desktop 불변은 기록됐지만 `private_desktop_closed=false`이므로 정상 종료 성공으로 표시하지 않는다. root는 후속 자동화 호출 계측을 별도로 맡겼으며 타임아웃을 늘리는 것으로 기능 완료를 주장하지 않는다.

개별 rustfmt exit0, 프로젝트 source 수정 없음. 사본 source/회귀 작성만 완료했다. root 독립 통합·실제 Windows compile/test/GUI 전까지 수정 기능과 인수는 미완료다. observer의 별도 좌표 문제가 재현되면 이 패치 성공을 추정하지 않고 원시 evidence로 다시 분리한다.

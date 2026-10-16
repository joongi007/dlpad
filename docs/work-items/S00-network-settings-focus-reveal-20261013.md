# S00: 네트워크 설정 조작의 포커스 공개

- 상태: completed
- 분류: UX
- 우선순위: P0
- 요구사항: R01, R08, R11
- 담당/수정 범위: windows_auto — /tmp 후보; root — 공유 source/Cargo/native 통합 검증
- 선행 작업: 네트워크 density 통합의 actual metrics88/0/1 검증 및 기존 rate preset 포커스 회귀

## 최종 인수 — 2026-10-13

이 작업의 네트워크 배치·포커스·수동 실패 안내 수직 범위를 완료했다. 최종whole-v2 CPU776d17c…/1,235/0/30와 fresh network light/dark private-v3, root-scoped-pixel-review를 연결했다. root는 actual960Light100 auto와800Dark200 Apply after PNG를 직접 보았고 고유Name/Value/원RuntimeID/fullbody/선택보존/healthy RXTX/Off/Manual 실패 안내/Auto복귀 및 자연Job0를 원로그·pins·source와 대조했다. 물리IME/reader/OSDPI/실제routechange/지속저장/전체UI 인수는 이 국소완료 범위 밖이다. 아래 검증 기록은 각 시점의 이력이며 과거 준비/실패를 현재완료로 소급하지 않는다.

## 사용자 결과

Combo, 목록 새로고침, 관측 적용에 키보드/접근성 포커스가 처음 도착하면 해당 조작 전체가 본문 안으로 스크롤된다. 이미 유지 중인 포커스 때문에 사용자의 후속 wheel 스크롤을 되돌리지 않는다. 선택·초안·갱신 요청은 focus만으로 변하지 않는다.

## 계약과 범위

PRODUCT/DESIGN, ADR-0001의 네이티브 UI 계약을 유지한다. 현재 density 통합 crates/app/src/accessible_metrics.rs의3response에 gained_focus→scroll_to_me(None)만 추가한다. worker/API/DB/NetworkMode/채널/관측rate/기한/epoch/StopWake는 변경하지 않는다. 기존 Apply global stable scope와 수동 Editor ID, Combo salt를 유지한다. native collector는 bright960100/dark800200의 명시 scope를 추가하되 원 strict Value/RuntimeID/healthy/natural-lifecycle 조건을 유지한다.

## 인수 조건

실제 MetricsPanel settings를 fixed top/footer 사이 ScrollArea에서 렌더링해 각 조작 FocusAction만으로 body clip에 전체rect를 공개하고, 원 선택/revision/refresh/draft 및 NodeID를 유지한다. 같은 포커스의 후속 wheel은 실제 offset을 옮기며 다음 frame에서 snap back하지 않는다. 기존 한글 입력/focus/disabled/worker 회귀를 유지한다.

새 current-whole/CPU의 private healthy GUI를 light100-960 및 dark200-800 fresh 출력에서 각각 실행한다. theme/scale은 실제 Invoke/Toggle 및 Value ACK로 선택하며 OS DPI나 사용자 desktop을 변경하지 않는다. 네트워크3controls는 focus only→full original body rect/선택/RuntimeID ACK를 요구하고 focus 검수에 wheel/ScrollIntoView fallback을 사용하지 않는다. 기존 Auto/alias/rate/목록/Off/Manual 없는 이름/복귀 및 자연Job/WM_CLOSE/MTA/InputDesktop 검사는 완화하지 않는다. provider bounds 외 실제 PNG의 clip은 root가 직접 검토한다.

## 검증과 handoff

/tmp 후보 준비 단계다. Python AST, C# delimiter 정적검사 및 production settings method 외 source 동일성만 검사했다. native/.NET/Cargo/fmt/clippy/test/build를 실행하지 않았다. 회귀 후보도 아직 실행하지 않았다. root가 RED/수정후검증·whole4 및 두 private scope·PNG를 인수한다. 이전 density/v4 PASS를 새 focus 동작 PASS로 소급하지 않는다.

Root가3control의focusedge와새실제ScrollArea회귀만통합했다. worker/API/schema/lock은보존했다. actualWindows 검증은다음단계이며아직성공으로표시하지않는다.


2026-10-13 통합 검증: `python3 /tmp/dlpad-network-settings-focus-targeted-20261013.py`는 실제 Windows metrics 실행에서33성공/1실패로 종료했다. 포커스로 조작 전체를 공개하는 assertion은 통과했으며, 후속 wheel과 다음 frame offset의 동일성 assertion이 실패했다. 원 실패 로그를 보존하고 egui wheel 잔여 보간과 실제 focus snap을 분리 중이다. 실패를 성공으로 집계하지 않는다.

같은 설정 단위에서 없는 수동 연결은 fresh/same-revision Failed 표본에 한해 고치는 방법을 inline으로 표시한다. 관측 alias caption은 `선택한 연결`로 고쳐 실패한 선택을 현재 연결로 오인하지 않게 한다. 수정 파일은 accessible_metrics.rs와 중앙 catalog.rs이며 stale/이전 revision 실패에는 이 안내를 표시하지 않는다. 기존 worker/API는 유지하고 실제 GUI collector에서 없는 이름과 안내의 ACK를 함께 확인한다.


후속 v2: vendor/egui/src/input_state/mod.rs는 Point delta 길이8 이상을 unprocessed_scroll_delta에 넣어 exponential 보간한다. ScrollArea.animated(false)는 scroll_to 요청에만 적용한다. 원 -80 wheel 다음 frame도 같은 방향의 입력이 남을 수 있으므로 동일성 assertion에 적합한 direct-smooth Point -6으로 fixture를 고쳤다. full bounds 공개·focus/NodeID/선택/초안 보존·wheel 실제 이동·후속offset 동일성 조건은 유지했다. production의 gained_focus 조건을 완화하지 않았다.

`python3 /tmp/dlpad-network-settings-focus-targeted-v2-20261013.py` 실제 Windows90/0/1이며 두 app target에서 새 실제 focus 회귀가 통과했다. root가 원 raw 로그/lifecycle/current source를 독립 대조하여 root-scoped-review.json에 기록했다. 원 v1 실패는 그대로 보존했다. fresh whole은 windows-msvc-ui-focus-plan-whole-20261013에서 실행 중이며 새 GUI 인수는 아직 남는다.


독립 검토 결과를 root가 읽었다. 원 -80 run은 offset 숫자/방향을 기록하지 않았으므로 input smoothing 원인은 소스 기반 추론이며 원 실행의 이동 방향을 실측했다고 주장하지 않는다. Point -6 새 실행의 PASS는 직접 근거다. production gained_focus edge 및 기존 fullbounds/tolerance/선택불변 조건이 유지된 것을 대조했다. 검토 문서는 `/tmp/dlpad-network-focus-wheel-review-20261013/REVIEW.md`이며 큰 notched/물리 OS wheel은 별도 미검증이다.


최신 통합 whole: `python3 scripts/windows_msvc_workspace_probe.py --configuration docs/validation/evidence/windows-msvc-advanced-input-validity-whole-v2-20261012/configuration.json --output /mnt/d/project/dlpad/docs/validation/evidence/windows-msvc-ui-focus-plan-whole-20261013 --reuse-owned-target`의 fmt/clippy/test/release4검사 PASS다. actual43harness 합계1,235성공/0실패/30ignored이며 root가 `/tmp/dlpad-review-preferences-whole.py ... 1235 30`로 원 로그·현재 source map·실행 파일·GNU 원본을 독립 대조했다. CPU9,821,184B SHA256 `8de4fcef475f579826d52e60461018897479259d04be5a9a0209124582ec319b`. 이전 CPU보다4,096B 증가한 UI batch 효과이며 dependency 추가는 없다. actual GUI 인수와 사용자 최종 UI 평가는 별도다. 현재 fresh private rate/네트워크 light/dark3scope를 병렬 실행 중이다.


actual private 네트워크 검수: light100-960와 dark200-800의 compile/native/collector 및 root 원 source/log/pins 검토 PASS. 첫 helper compile의 Rect definite-assignment 실패는 보존하고, /tmp v3에서 두 Rect 초기화만 수정했다. fresh v2 출력 두 scope에서3controls focus-only/fullbounds/RuntimeID/선택 보존, healthy automatic alias/rate, Off, 없는Manual 선택·초안·실제inlinefailure ACK, Auto복귀, 자연exit0/Job0/InputDesktop보존을 확인했다. root가 Light auto 및 Dark missing PNG를 직접 보았고 body/footer 스크롤경계를 좁게 인수했다. network-view.json으로 최종960×640Light100%,800×600Dark200%를 검증했다(run initial geometry는960×640). reader/physical input/OSDPI/routechange/persistence/전체UI 인수는 미검증이다. 증거는 windows-msvc-network-focus-light-private-v2-20261013와 dark-private-v2다. 이후 Reset focus 수정의 source/CPU는 별도 새whole·native검증으로 구분한다.


최종 Reset 포함 whole-v2: 동일 runner의 fresh `windows-msvc-ui-focus-plan-whole-v2-20261013` fmt/clippy/test/release PASS다. root 원로그/source/artifacts/GNU원본 독립검토 PASS이며43harness 합계1,235/0/30이다. CPU9,821,184B SHA256 `776d17c165e859734b57e126414889a510946f302a3cc5ed90df2d9f2c2933d0`. Reset 수정은 binary크기를 늘리지 않았지만 SHA는 달라졌으므로 첫 whole·GUI 증거를 이 source로 소급하지 않는다. 최신CPU의 fresh rate/light/dark3native scope를 병렬 실행 중이다.

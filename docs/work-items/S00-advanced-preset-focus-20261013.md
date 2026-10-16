# S00: 고급 속도 preset의 키보드 포커스 공개

- 상태: completed
- 분류: UX
- 우선순위: P0
- 요구사항: R07, R08
- 담당/수정 범위: root — accessible_options.rs/catalog 통합·회귀·native 검증; advanced_units — /tmp 후보만
- 선행 작업: 속도 단위 actual Form30회귀/whole1227 및 실제 GUI v5 관측

## 최종 국소 인수 — 2026-10-13

최종whole-v2 CPU776d17c…/1,235/0/30 및 actual targeted32옵션/2Candidate를 연결했다. 새 private rate-v2의 Light800200·선택Dark800200·재시작Dark800200에서21state/15actualreadonlyplan·preset/unit/무효gate/Reset/focus·원ID·자연Job0를 확인했다. root가 원source/pins/logs/3phasevalidator를 독립 실행하고 LightReset/DarkPreset/restartDarkReset PNG를 직접 보았다. actual missing-tool/relative-tool purepreview와 실행거절은 Candidate 회귀가 확인하고 native15plan은 owned nonexistent 절대toolpath fixture를 사용한다. 실제도구설치/다운로드는 실행하지 않았다. core bytes보존은 실제Form회귀 증거이며 GUI가 core private값을 직접 읽었다고 주장하지 않는다. managed30103/30159 미등록·physicalreader/IME/OSDPI/전체UI 인수는 별도 미완료다. 이 작업의 사용자단위·포커스·pure미리보기 국소범위를 완료했다. 아래 기록은 시점별 이력이다.

## 사용자 결과

실행 계획을 보거나 스크롤한 뒤 속도 preset/단위 조작으로 키보드 포커스를 이동하면 조작이 화면 안에 나타난다.

## 계약과 범위

가상 화면에서 원 preset5 MB/s HasKeyboardFocus=true인데 client top−834/bottom−770이며 body96~410 밖에 남는 실제 실패를 관측했다. 무효입력/속도bytes/Start gate/원ID를 유지하고 focus가 새로 얻어진 때에만 필요한 스크롤을 요청한다. 같은 포커스에서 사용자가 스크롤한 위치를 매 frame 되돌리지 않는다. 새 dependency/schema/lock 변경 없음.

## 인수 조건

실제 Form ScrollArea에서 화면 밖 preset/단위 포커스 이동→viewport 공개, 같은포커스 idle→스크롤 유지. 기존 실제 단위/무효값/plan회귀와 fresh800200 private GUI에서 원ID/기하/선택을 검증한다.

## 검증과 handoff

원 Windows GUI v5 실패를 보존했다. 아직 수정·회귀 성공으로 표시하지 않는다.

Windows actual Form/ScrollArea 회귀만 먼저 추가한 RED 실행은 exit101, 원bounds가body밖에남는 실패를 재현했다. 그 뒤 Root가 두 control의 gained_focus 공개를 통합했다. 원false판정을 약화하지 않았으며 GREEN actual 옵션회귀는 다음 단계다.

GREEN Windows actual 옵션회귀32/0/0 exit0. Root가두app target의새 focus-only/5control/fullbody 및heldfocus/manualwheel 회귀와 기존 옵션회귀의 원log/snapshot를대조했다. fresh전체검사·nativeGUI는 아직남았다.


최신 통합 whole: `python3 scripts/windows_msvc_workspace_probe.py --configuration docs/validation/evidence/windows-msvc-advanced-input-validity-whole-v2-20261012/configuration.json --output /mnt/d/project/dlpad/docs/validation/evidence/windows-msvc-ui-focus-plan-whole-20261013 --reuse-owned-target`의 fmt/clippy/test/release4검사 PASS다. actual43harness 합계1,235성공/0실패/30ignored이며 root가 `/tmp/dlpad-review-preferences-whole.py ... 1235 30`로 원 로그·현재 source map·실행 파일·GNU 원본을 독립 대조했다. CPU9,821,184B SHA256 `8de4fcef475f579826d52e60461018897479259d04be5a9a0209124582ec319b`. 이전 CPU보다4,096B 증가한 UI batch 효과이며 dependency 추가는 없다. actual GUI 인수와 사용자 최종 UI 평가는 별도다. 현재 fresh private rate/네트워크 light/dark3scope를 병렬 실행 중이다.


추가 actual GUI 실패: 최신CPU의 first Light800×600/app200%에서 preset5·단위KB의 실제plan/readback은 통과했지만 overflow 뒤 Reset 버튼은32번 focus ACK에도 clienttop-38/bottom26로 body96..410 밖에 남았다. root가 s10-focus-scroll-failed.png를 직접 확인했다. collector/root lifecycle 원 실패를 보존한다. 같은 수직 단위의 Reset focus 공개를 추가하며 source는 accessible_options.rs만, click/reset/옵션 실행 정책은 유지한다. advanced_units가 /tmp fixture-only/minimal patch를 독립 제출하고 root는 actualbaselineRED→수정→통합검증을 전담한다.


Reset baseline: `python3 /tmp/dlpad-reset-focus-red-20261013.py`는 exit101이다. 실제 테스트가 Reset bounds[-1408,-1376]와 body[56,192] 불일치를 재현했다. source/lifecycle 보존이며 원 FAIL을 유지한다. root는 Reset response gained_focus에서만 scroll_to_me를 추가해 클릭 초기화 정책을 유지했다. `/tmp/dlpad-reset-focus-green-20261013.py`로 전체 accessible_options 회귀를 실행 중이다. production 파일은 accessible_options.rs 한 곳이며 공유schema/lock/dependency 변화는 없다.


Reset 수정후 `/tmp/dlpad-reset-focus-green-20261013.py`: actualWindows32/0/0 PASS, 두 app target의 focus 회귀가 Reset을 포함해 통과했다. root가 현재source·원raw/ownedlifecycle·원2instances를 독립 대조하여 fresh evidence/root-scoped-review.json에 기록했다. 새 whole-v2는 windows-msvc-ui-focus-plan-whole-v2-20261013에서 실행 중이다. 사용자 최종평가/실제GUI 재검수는 아직 남는다.


최종 Reset 포함 whole-v2: 동일 runner의 fresh `windows-msvc-ui-focus-plan-whole-v2-20261013` fmt/clippy/test/release PASS다. root 원로그/source/artifacts/GNU원본 독립검토 PASS이며43harness 합계1,235/0/30이다. CPU9,821,184B SHA256 `776d17c165e859734b57e126414889a510946f302a3cc5ed90df2d9f2c2933d0`. Reset 수정은 binary크기를 늘리지 않았지만 SHA는 달라졌으므로 첫 whole·GUI 증거를 이 source로 소급하지 않는다. 최신CPU의 fresh rate/light/dark3native scope를 병렬 실행 중이다.

# Proposed S00 virtual-library keyboard boundary regression

범위: R03 목록/ R08 UX 및 DESIGN80·132의 가상화 키보드 탐색 인수. 수정후보는 accessible_library.rs 기존 tests 모듈에 append-tests.rs 한 테스트와 private helper 세 개만 추가합니다. production/schema/worker 변경 없음.

테스트: 실제400×300 logical /200% Context viewport warmup을 확인합니다. 실제 wheel로 첫 행만 공개한 bootstrap 뒤 실제 AccessKit Focus로 최초 row를 선택하고, 이후 scroll state나focus ID를 직접 바꾸지 않고 Tab 및ArrowDown 각각 독립 Context에서128개 경로를 순서대로 탐색합니다. 각 행이 materialize/focus/fullclip/32px 및고정footer 안에 있어야 하며 path ID는서로다르고 원Arc rows불변/탐색 중preview0, 마지막Enter만127번 preview를 요청합니다.32bootstrap/3settle/128행 상한이 있습니다.

중요 범위: bootstrap이 wheel이므로 toolbar에서 목록까지 키보드만으로 진입할 수 있다는 인수는 아닙니다. 별도 keyboard-entry 회귀가 필요합니다. 이 후보는 virtual-boundary 단절의 실제RED 여부만 판단합니다. 결과가실패하면 실제focus ID/논리pathindex를 근거로 좁은 navigation seam을 결정하며 모든128행상시생성등 제안은하지않습니다. Arrow와Tab 어느쪽성공도다른쪽성공을대신하지않습니다.

원본 SHA8b6be6c0929230928db2ac40405260186cf5f145339a9682b6ef26a9be324910. 컴파일/Context/native 실행은하지않았으며root 실제검증이필요합니다. 실제physicalkeyboard/reader/UIAOS인수는아닙니다.

담당: 주 담당 통합·Windows 실행, diagnostic_cache는 /tmp 테스트 후보만 작성. 담당 파일: crates/app/src/accessible_library.rs의 tests 모듈, 본 작업 문서. 의존성: 진행 중 GUI source freeze 종료 및 실제 테스트 검토. 인수: Tab·Arrow의128행 실제 focus/clip/ID/Enter 정책 또는 실패 원인 기록; 물리 keyboard/reader는 별도 미검증. 현재 상태: 후보 검토 중, AccessKit incremental update 반영을 보완하며 프로젝트 테스트 코드는 아직 변경하지 않았다.

### 증분 접근성 트리 시험 보완

후보 검토에서 AccessKit TreeUpdate가 전체 tree가 아닌 증분이라는 점을 확인했다. 모든 warmup/wheel/key/settle frame을 병합하되 현재 root children graph에 도달하는 노드만 유지해 삭제된 가상 행의 stale cache를 허용하지 않는다. focus 원값과 최종 실제 paint clip 검사는 유지했다. 수정 후보 SHA256 `074ddf53dd79430b358f8a112f168e4c2798784998b56edad29b9a0305d15731`. 아직 프로젝트 통합/컴파일/실행 전이며 GUI source freeze 종료가 의존성이다.

### Windows 실제 회귀 시작

GUI 실행 두 개의 terminal을 확인한 뒤 주 담당이 검토된 tests-only 후보를 통합했다. `windows-msvc-real-footer-scale-virtual-keyboard-red-20261012`의 actual_ filter 실행 중. 실패/통과 결과는 아직 확정하지 않는다.

### 실제 RED 결과

`windows-msvc-real-footer-scale-virtual-keyboard-red-20261012/test.log`: 컴파일 성공,31 passed / 2 failed / 2 ignored. 실제 Candidate800×600 Scale control [16,203..176,235]이 실제 painter clip bottom203을 벗어난다. library Tab은 path4 가상화 경계에서 다음행 focus ID를 얻지 못했다. Arrow 검증은 Tab 실패로 도달하지 못했으므로 통과로 주장하지 않는다. 소유 Job natural-zero/forced=False/input desktop unchanged. 설정과 목록 수정 후보를 파일별 담당으로 분리해 준비 중이다.

### Tab/Arrow 분리 진단 실행

Tab·Arrow harness를 독립 분리하여 한쪽 실패가 다른쪽 실행을 막지 않게 했다. focus mismatch에는 실제 root에서 도달하는 focused ID/role/label/bounds 및 가상 row 범위를 기록한다. 앞선 path4 노드는 존재했으므로 미생성 행이 원인이라고 단정하지 않는다. 생산 정책은 변경하지 않았으며 기존128행 순서/fullclip/Enter 조건은 유지한다. `windows-msvc-real-footer-keyboard-focus-diagnosis-20261012` 실행 중.

### 독립 키 진단 실제 결과

`windows-msvc-real-footer-keyboard-focus-diagnosis-20261012/test.log`에서 Tab path4는 focused role Window/label None이 되어 focus를 잃었다. target row4는 실제 current tree와bounds201..233에 존재했으므로 미생성 행 문제라고 단정할 수 없다. ArrowDown은 row1 focus assertion을 통과한 뒤 full glyph clip에서 실패했다. 서로 다른 두 경로 모두 미해결이며 path-ID drift/egui focus의 사라진 widget 처리/실제 clip 수치를 추가 조사한다. 실제 Candidate footer 회귀도 동일하게 실패했다.

### 경로 ID 교정 통합

egui Ui::new_child의 push_id seed는 parent.next_auto_id_salt를 포함해 virtual range가 바뀌면 같은 path Button의 auto ID도 바뀐다. 기존 show_rows의 skip_ahead_auto_ids 보정은 manual range에 없다. UiBuilder::id를 사용해 현재 부모 namespace와 path를 포함한 고정 child scope로 교정했다. 가상 행 수/disabled/선택/Enter/preview 상태 정책은 유지했다. Tab·Arrow 독립 회귀를 실제 Windows에서 다시 실행한다.

### 경로 ID 교정 실제 결과

`windows-msvc-library-stable-path-keyboard-green-20261012/test.log`: Tab은128개 경로 순서·focus·가시성·ID·마지막 Enter preview까지 통과했다. ArrowDown은 row1 full glyph clip에서 실패해 목록 전체 인수는 미완료다. actual test 첫 UI harness1 passed / 1 failed; Job natural-zero/forced=False/input desktop unchanged. 경로 ID 교정이 Tab 단절을 해결한 효과는 확인했으며 방향키 clip은 별도 계측한다.

### 방향키 실제 경계와 focus-once 교정

`windows-msvc-library-arrow-actual-clip-diagnosis-20261012/test.log`: ArrowDown 이후 row1 ID는 유지되지만 bounds223.5..255.5/glyph232..247이 실제 clip bottom241을 넘고 다음5frame에도 이동하지 않는다. cardinal focus는 egui end_pass에서 변경되어 gained_focus-only reveal이 전이를 놓친다. last_focused_row Option<Id>로 새 관찰 포커스만 한 번 공개한다. 동일 ID에서의 의도적 wheel은 자동 복귀시키지 않으며 가상행 수·경로 ID·disabled·Enter·worker 정책은 유지했다. 임시 진단 계측을 제거하고 원래 Tab/Arrow128 회귀를 재실행한다.

### Tab·Arrow128 실제 GREEN

`windows-msvc-library-arrow-focus-once-green-20261012/root-review.json`: 두 UI harness에서Tab·Arrow 각각128개 path순서/focus/fullglyphclip/ID 및 마지막Enter preview 회귀 총4 passed / 0 failed를 주 담당이 raw Job guard/current source snapshot과 독립 확인했다. cfg(test) 계측 제거. physical keyboard/reader/toolbar부터keyboard-only진입은 별도 미검증이다. 전체4검사 `windows-msvc-real-viewport-keyboard-ui-whole-20261012` 실행 중.

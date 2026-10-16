# S05: Plan header 키보드 인수 엄격화

- 상태: completed (Linux private GUI plan header keyboard activation 범위)
- 분류: validation
- 우선순위: P0
- 요구사항: R07, R01
- 담당 파일: soak_recovery_audit — scripts/cpu_plan_toggle_probe.py, 이 문서
- 의존성: 실제 frozen CPU binary 및 private Xvfb/AT-SPI, root native 실행

## 범위와 인수 조건

GrabFocus성공/실제FOCUSED/64bitstate/Enter·Space각open-close를모두assert한뒤에만keyboard_verified를기록한다. Action탐색지원범위는별도로보고하며pointer나action을keyboard성공으로대체하지않는다. 기존SHA/flags/privateXvfb/noDesktop/nojob/notool/noclipboard계약을유지한다. Pythoncompile만수행하고GUI는root가실행한다. 다른script/Rust/Cargo는수정하지않는다.

## 구현과 검증

state_bits는정확2uint32를64bit로조합하고각word범위를검증한다. Return/space각각GrabFocus의exit0/booleantrue,3초boundedFOCUSED대기,EXPANDABLE/초기collapsed,실제previewopen+EXPANDED/FOCUSED,실제previewclose+collapsed/FOCUSED를필수assert한다. 모든key검증후keyboard_verified=true이며실패를pointerreset으로구제하는분기는제거했다.

Action discovery0/실패및ineffectiveactivation은action_scope로별도기록하고개별activation_verified를남긴다. Keyboard검증은action지원의PASS가아니며root보고에서scope를분리해야한다. 기존pointer/plan내용/expectedSHA/validate-only/require-expanded-state flags및ownedXvfbcleanup/nojob/toolchecks는유지한다.

`compile(Path(...).read_text(), ..., 'exec')` actualexit0,Pythoncompile PASS. GUI/process/Cargo실행은없다. Sourcefreeze SHA256 `6e7f9e4080e2f149a016679a0c9a46dfd15eab23f0d473ebdf3364cc8e56c139`. 실제nativeFOCUSED/Enter/Space결과는root가fresh evidence로실행해독립인수한다. Compile성공은키보드인수성공이아니다.

## 실제 root native 후속 인수

[최초 실패](../validation/evidence/p0-plan-keyboard-strict-first/result.json)는JSONDecodeError/keyboard_verified=false로보존했다. [계측 후 실제 second](../validation/evidence/p0-plan-keyboard-strict-second/result.json)는passed=true/keyboard_verified=true/close_exit0/expanded_state_verified=true다. Current harness SHA `feb679df8d55e5c6c54557dbddfc3773a9e98fe9945c79e1272c6ab67eba98cb`,동일immutableCPU SHA `ca8e969f028fdd50f5c7f9ae19c3c71928d7740daa5c93189ec100ee9f5fc77e`로실행됐으며원시JSON을독립대조했다.

Return/space각각GrabFocus true와nativeFOCUSED상태1124080384,open1124081408(EXPANDED+FOCUSED),close1124080384를확인했다. 실제13planread는absent/valid-json로기록됐고각key의유효planopen/previewabsentclose를strictassert했다. Action0지원/activation은별도scope이며키보드의대체증거로쓰지않았다.

완료범위는ownedLinuxXvfb/privatebus의**plan header를GrabFocus한뒤Return/space로open-close하는activation**이다. 처음부터Tabnavigation,다섯옵션helpheader키보드,screenreader실제발화,Windows/macOS인수는남는다. 최초실패의제품원인은확정하지않으며second성공을제품버그수정완료로표현하지않는다. Agent는원시자료읽기/문서만수정했고실제GUI는root가실행했다.

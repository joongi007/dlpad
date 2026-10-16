# CPU 실행 계획 헤더 토글 진단

기존 R07 GUI의 `DoAction exit0 / preview 미관측`은 이번 최소 대조에서 제품 토글 버그로 재현되지 않았다. [footerfix](evidence/cpu-plan-toggle-footerfix/result.json), [독립 반복](evidence/cpu-plan-toggle-repeat/result.json)은 각각 exit0/PASS다. 기존 실패 raw와 원인을 보존하며 전체 reader 접근성을 완료로 판단하지 않는다.

실행: `python3 scripts/cpu_plan_toggle_probe.py --binary target/release/dlpad-cpu --output docs/validation/evidence/cpu-plan-toggle-repeat --font /mnt/c/Windows/Fonts/malgun.ttf --expected-sha 166946017de0b3d0589bf3d02bd864d004d90f3457ff2e314fe5e10a5f865417`. privateXvfb/DBus/임시roots만 사용했고 사용자desktop/클립보드/영상/실계정/도구실행은 없다. binary 시작/종료SHA 일치, source/probe/Cargo제품수정없음. [소스대조](evidence/cpu-plan-toggle-repeat/source-inputs.json)는 두 번째 시험 중 sample 이후 종료 대조다.

피드백루프는 pointer open/close 기준 → native action index별 실제preview출현 → reset → GrabFocus/Enter open/close → Space open/close다. 단순 readonly text 조회나 action 반환을 토글 성공으로 확대하지 않고, preview node가 없던 상태에서 실제 생기고 다시 없어지는지 확인했다. 각 경계PNG도 보관했으며 action-0.png의 펼쳐진 삼각형을 직접 확인했다.

초기 가설순위와 결과:

1. 잘못된 node/action index: header role43, NActions property1, index0 GetName `click`, DoAction true. 잘못된 action 종류 가설은 배제했다. 최초 별도 수집기는 존재하지 않는 GetNActions method를 불렀으며 Properties.Get NActions로 교정한 raw를 보존했다.
2. egui header input 소비누락: 같은 node의 native click이 실제 preview를 생성했고 pointer/Enter/Space 양쪽 토글을 두 fresh run에서 확인했다. 제품 input 소비 버그는 이 최소 조건에서 재현되지 않았다.
3. callback repaint 지연: action후0.6초 관측에서 이미 실제 preview가 나타나므로 추가 지연/단순pointermotion probe는 실행되지 않았다. 이 조건의 wake 실패는 지지되지 않지만 모든 지연상황을 배제하지 않는다.

최초 최소 pointer 기준 자체가 실패했다. 실제 header center549px가 fixedfooter542px 시작 아래여서 click이 가려졌다. 공통 helper는575px까지 허용하며 앞선 수집기의48node상한 및 긴 multiline 전체bounds 중심 조건도 미관측을 일으켰다. 전용 harness만 center530px 상한/bounded256node 직접읽기로 교정했다. shared helper와 제품 source는 변경하지 않았다. 최종 두 run에서는 pointer와nativeAction과Enter/Space 모두 open→close 전이를 동일하게 확인했다. jobs.sqlite/합성도구marker없음, ownedapp/bus/log cleanuptrue, 정상 exit0다.

남는 접근성 한계: header GetState1124075776는 open/close에서 그대로이며 key focus후1124079872로만 바뀌었다. 실제 expanded/collapsed 상태 메타데이터를 게시했다고 주장하지 않는다. 헤더 role은 버튼으로 보이며 스크린리더가 펼침 상태를 안내하는지는 별도 S00 R08 게이트다. metadata/state 개선은 shared UI/bridge 책임자가 별도 인수 가능한 기능으로 판단해야 한다. 이번 조사에서는 토글 버그 수정이나 Cargo/vendor 변경이 필요하다는 근거가 없다.

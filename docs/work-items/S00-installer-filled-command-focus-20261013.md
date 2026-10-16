# S00: 설치기 filled 명령의 키보드 포커스 대비

- 상태: completed (scoped)
- 분류: UX
- 우선순위: P0
- 요구사항: R06, R08
- 담당/수정 범위: root — 통합/실제 Windows 검증/본 기록; installer_batch — /tmp 후보; crates/app/src/installer_main.rs
- 선행 작업: v14 CI 종료와 frozen 원소스 검증

## 사용자 결과

밝은 테마의 설치·앱 실행 버튼에서 키보드 포커스 테두리를 배경과 구별할 수 있다.

## 계약과 범위

현재 Light primary의 fill과 안쪽 focus pen은 같은 PRIMARY여서 literal 대비1:1이다. 이는 소스 render-role 결함이며 실제 OS focus 전달 시험 성공을 의미하지 않는다. filled 명령201/204/212의 안쪽 선만 기존 밝은 foreground로 바꾼다. neutral/checkbox/disabled/HC native fallback·1.5px·handlers·ID·폰트·공유 token·schema·Cargo 유지. 현재 작은 흰 글자 대비는 literal 계산5.227/4.656/7.490으로 별도 결함을 확인하지 않았다.

## 인수 조건

원동작의 대비3:1 미만을 실제 Windows 회귀에서 재현하고 두 테마·idle/hot/pressed filled 역할의 대비3:1 이상을 독립 수학으로 확인한다. implementation을 그대로 복제한 return-equality 테스트는 추가하지 않는다. 실제 GDI/physical focus·reader는 별도이다. 통합 후 fmt/clippy/test/release와 설치 후보를 갱신한다.

## 검증과 handoff

/tmp/dlpad-installer-filled-role-focus-candidate-20261013 후보는 정적 rustfmt만 검증했으며 RED/GREEN 실제 실행 전이다. 기존 v14 CI에 포함된 문서를 수정하지 않고 추가한 후속 작업이다. v14 export snapshot 밖이며 별도 문서 검사가 필요하다. 원FAIL·원소스를 보존하고 현재 구현으로 과거 결과를 소급하지 않는다.


실제GDI 회귀3개는 Default desktop을 명시거부하며 일반cargo test에서ignored로 분리하고 privatehost에서 --ignored로 전부 실제 실행한다. 원동작production은 그대로이며 test-only 첫compile은 E0277 u32&i32 fixture타입오류로 실패했다. 제품RED가 아니므로 원로그/source를 보존하고 mask/shift 타입만 명시해 교정했다. [원compile 실패](../validation/evidence/windows-msvc-installer-focus-actual-paint-baseline-20261013/root-failure-classification.json).


교정된 원GDI baseline은 실제2성공1실패로 renderer의 role201/DPI96 idle focusstroke와fill 모두COLORREF0xbe6f00(대비1:1)을 재현했다. 최소filledrole 선 수정 후 Light/Dark×96/192×idle/hot/pressed36case·neutral12case·실제disabled HWND 및localHCfallback의3시험이 성공했다. 물리포커스와OSHC 실제전환 인수는 아니다. 포맷 복사본의 하위module해석 실패는 별도도구fixtureFAIL이며 원소스는 적용하지 않았고 skip_children=true로 두파일만 포맷했다. [실제RED](../validation/evidence/windows-msvc-installer-focus-actual-paint-baseline-v2-20261013/root-failure-review.json), [실제GREEN](../validation/evidence/windows-msvc-installer-focus-actual-paint-green-20261013/root-review.json). 최종포맷회귀·전체Windows/패키지는 진행 중이다.


## 최신 통합 인수

v15 final whole-v2 Windows fmt·strictclippy·test·release와 source656/43harness/현재3PE/GNU/lock은 root 독립 대조PASS:1,307/0/33. private GDI3은 별도명시실행3/0/0이며 일반시험33ignored 중3은 이격리전용시험이다. app63·3DB·자연0/Job0와 팀전체직접열람(root7+2추가), 최신ZIP16,726,197B/선택graph258/notice누락0·원payload5/archiveCRC를 확인했다. direct설치·전이·재검증·앱/감독자연0 및outer0, 변조refusal자연1/자식0, 독립27ID도root 대조했다. app9,837,056B로v14보다1,024B감소, installer4,399,616B동일; dependency/schema/Cargo변화없음. Runtimecost와전체physicalUI 인수는별도다. 원nestedouter143/원clippyFAIL은보존하며원인미확인이다. Dark200 하단명령은 별도scroll작업에서검수한다.

[최종 whole](../validation/evidence/windows-msvc-command-focus-and-browse-whole-v2-20261013/root-review.json), [현재63](../validation/evidence/current-command-focus-and-browse-app63-20261013/root-team-visual-review.json), [direct설치](../validation/evidence/windows-msvc-ui-cross-surface-v15-20261013-command-happy-direct/root-review.json), [현재27ID](../validation/evidence/windows-msvc-ui-cross-surface-v15-20261013-command-independent/root-review.json).

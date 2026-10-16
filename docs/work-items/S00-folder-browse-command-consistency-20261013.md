# S00: 폴더 선택 버튼의 공통 명령 스타일

- 상태: completed (scoped)
- 분류: UX
- 우선순위: P0
- 요구사항: R06, R08
- 담당/수정 범위: root — 통합/실제 검증/본 기록; windows_auto — /tmp 후보; crates/app/src/path_picker.rs
- 선행 작업: v14 CI 종료와 frozen 원소스 검증

## 사용자 결과

저장 폴더 선택 버튼도 다른 명령과 같은 높이·글꼴·hover·포인터 표시를 사용한다.

## 계약과 범위

path_picker235의 raw egui Button은 공통 secondary/cursor 적용을 우회한다. 기존 공통 컴포넌트를 사용하며 ID·라벨·disabled·Response·dialog worker·파일 소유권·핸들러는 유지한다. History/Tools action identity·keyboard 도움말 개선은 섞지 않는다. Rust native·schema·Cargo unchanged.

## 인수 조건

실제 Context로 enabled/disabled·hover cursor·한글 배치·기하를 확인한다. 테스트는 별도 불변 계약을 확인하고 구현 복제 테스트는 추가하지 않는다. 원 handler/ID 보존과 기존 관련 회귀·전체 Windows 검사를 확인한다. 물리 포인터·폴더 dialog OS 입력은 별도이다.

## 검증과 handoff

후보는 /tmp 준비 단계이며 아직 제품 코드 변경이나 실제 Windows 시험을 하지 않았다. frozen v14 export 밖에 추가한 후속 문서이다. 기존 CI 문서는 동결 유지하며 이번 문서는 별도 검사한다. 완료하지 않은 시험을 성공으로 기록하지 않는다.


원동작 actual Context은 흰 배경#FFFFFF와 공통secondary#F5F5F5 차이로0성공1실패(Rust101/자연Job0)를 재현했다.최소shared secondary 연결 후 Light/Dark×320/800·hover/press/release·available/disabled·AccessKit click refusal·ID/input preservation의 실제1회귀가 통과했다. 원로그/source/GNU/lock을 root가 대조했으며 이후 설치기test-only module 추가는 이scope와 분리한다. 전체Windows 검사·최신패키지 갱신은 진행 중이다. [RED](../validation/evidence/windows-msvc-folder-browse-baseline-20261013/root-failure-review.json), [GREEN](../validation/evidence/windows-msvc-folder-browse-green-20261013/root-review.json).


## 최신 통합 인수

v15 final whole-v2 Windows fmt·strictclippy·test·release와 source656/43harness/현재3PE/GNU/lock은 root 독립 대조PASS:1,307/0/33. private GDI3은 별도명시실행3/0/0이며 일반시험33ignored 중3은 이격리전용시험이다. app63·3DB·자연0/Job0와 팀전체직접열람(root7+2추가), 최신ZIP16,726,197B/선택graph258/notice누락0·원payload5/archiveCRC를 확인했다. direct설치·전이·재검증·앱/감독자연0 및outer0, 변조refusal자연1/자식0, 독립27ID도root 대조했다. app9,837,056B로v14보다1,024B감소, installer4,399,616B동일; dependency/schema/Cargo변화없음. Runtimecost와전체physicalUI 인수는별도다. 원nestedouter143/원clippyFAIL은보존하며원인미확인이다. Dark200 하단명령은 별도scroll작업에서검수한다.

[최종 whole](../validation/evidence/windows-msvc-command-focus-and-browse-whole-v2-20261013/root-review.json), [현재63](../validation/evidence/current-command-focus-and-browse-app63-20261013/root-team-visual-review.json), [direct설치](../validation/evidence/windows-msvc-ui-cross-surface-v15-20261013-command-happy-direct/root-review.json), [현재27ID](../validation/evidence/windows-msvc-ui-cross-surface-v15-20261013-command-independent/root-review.json).

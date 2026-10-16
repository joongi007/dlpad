# S07: 원 선택 검사에서 소유 Windows 프로세스 시작

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R11, R08
- 담당/수정 범위: root — 공유 accessor·facade·문서·통합·native검증; metrics_recovery — /tmp active_launch.rs 단일파일 runtime/tests 초안; soak_recovery_audit — /tmp 보안/실패경로 독립검토; diagnostic_cache — 현재CLI증거인수 후 fixture/관측 초안
- 선행 작업: 명시active 선택 scoped DONE. 현재CLI actual sourcefreeze동안 제품Rust수정없음.

## 사용자 결과

명시3경로 선택을 readonly검사한 원capability에서 정확한앱을 소유Job에suspended생성하고 원image를대조한후시작한다. 자동탐색/upgrade를도입하지않는다.

## 계약과 범위

원선택caps/원exe pin을시작과수명동안유지한다. root가 pub(super) 원등록root·원exe File을한정노출한다. 별도 active_launch.rs에서 CreateProcess(argv), 상속handles0, 원Job/Process/Thread만관리; shell/PIDreopen/globalinput0. 실제 childimage원nativeID+SHA를resume전에검사한다. 정상/취소/기한/cleanup미확인상태를구별하고실패파일자동삭제0. 새deps/workspace/lock변경없음. 최초CLI작업인수뒤통합한다.

## 인수 조건

preCancel/pastdeadline spawn0; binding/namespace/differentID거절; suspended후resume전image/ID/SHA 대조; 원Job선행bind; 실패생성child는unresumed원process종료/terminal 확인. resume후ownedcancel/naturalexit와원Job0, cleanup미확인보존; 실제privateGUI/원files+sentinel보존과root독립검토. visibledesktopfallback0, TEMP합성만. GUI자동실행서비스/서명/업그레이드P0전체완료주장없음.

## 검증과 handoff

계약초안은 /tmp/dlpad-active-launch-contract.md. 현재후속읽기/runtime초안 단계이며실제실행전이다. retainedcaps·fullhashI/O 비용과세부API는root가초안검토후확정한다.

root가 공유accessor/module을통합했다. launch_inputs는 원registeredrootfinalpath+고정dlpad.exe와원File참조만전달한다. 원PI보호·Resumeattempt/mayRun·256각loopcancel 세보강을/tmp재동결뒤반영했다. sharedschema/lock·새dependency변경0. 아직compile/native검증전이며DONE아니다.

최초nativewhole fmtPASS 뒤clippy는 collapsible_if/field_reassign_with_default 두lint로FAIL(101)이었다. 원실패증거를보존하고root가중첩if와STARTUPINFOW initializer를교정한뒤새whole로재검증한다. 이를테스트PASS로집계하지않는다.

실제fixture 파일담당은 metrics_recovery /tmp/windows_selected_process_fixture.rs, diagnostic_cache 독립읽기QA, root 프로젝트example/합성AMD64helper·collector통합으로분리한다. 실제세case는 전용helper 자연종료17·현재CPU publicresume뒤취소·caller wait과거기한이며 실제GUIpixels/readiness/전체startupstall인수와구분한다.

교정후전체Windows4검사/root독립PASS, 설치89(새unresumed2)/755passed13ignored. CPUbb89e182…/7,917,568B와기본CLI5dc933c3…/723,456B이며크기불변. 실제3case example을통합하며독립fixtureQA의 wait_failure 상세/owneddesktopprefix/argv·namespace상한보강을반영했다. actualresume3case는아직미검증이다.

새actualexample의최초whole은fmtPASS/clippy manual_range_contains 1lint FAIL(101). root가1024UTF16 desktopname 범위를Range::contains로교정하고원실패를보존한다. 기존runtime755PASS와새example인수실패를구분한다.

scoped DONE: 현재source575 whole fmt/clippy/test/release와 설치89/755passed13ignored, 실제13operations/3typedcase Natural17·Cancelled126·TimedOut126 및원30nativeID·21파일/notice/sentinel/cursor17보존을root·독립검토했다. [actual](../validation/evidence/windows-selected-process-real-20261006-continue/root-completion-review.json), [whole](../validation/evidence/windows-selected-process-final-workspace-20261006-continue/result.json). childimage는runtime resume전원nativeID/SHA 검증의성공근거이며C#새조회는보존설치namespaces대조다. 실제GUI/실제payloadwork/sharedstartupstall/정식installer·upgrade는미검증으로후속한다. 최초lint실패들은보존했다.

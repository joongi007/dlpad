# S00: egui IME 취소 transaction과 선택 복원

- 상태: done
- 분류: bug
- 우선순위: P0
- 요구사항: R06
- 담당/수정 범위: root는이문서·Cargo.toml/Cargo.lock·vendor/egui전체provenance·appentry/통합/Windows전체검증. metrics_recovery는 /tmp/dlpad-egui-ime-patch/의state.rs/builder.rs초안만수정. diagnostic_cache는 /tmp/ime_cancel_regressions.rs의actualContext회귀만수정. 같은파일동시수정없다.
- 선행 작업: actualWindows nativecanceltrace의빈Commit/잔류UIFAIL, 동일binarynonemptyCommitPASS.

## 결정과 범위

기본OFFnative계측에서cancel은Enabled→Preedit한글→emptyPreedit(None)→emptyCommit→Disabled, commit은동일순서의nonemptyCommit이었다. dropped0/overflowfalse를확인했다. egui-winit번역의Disabled/emptyCommit가실제eguiTextEdit의버퍼를원상복원하지않는코드를대조했다. nativeevent위조없이입력위젯 transaction경계에서수정한다.

hostemptyPreedit추가만으로는기존선택문자열복원이불가능하다. 앱wrapper는privateIME상태/galley/undo와중복복원및한frame불일치문제가있다. root는현재정확egui0.33.3cache를vendor로고정하고state/builder두파일의boundedIMEbaseline·원text/커서/undo복원을채택한다. version/새의존성은변경하지않으며Cargo.patch/lock은root가조정한다. upstream변경사항과source/licenseprovenance를보존한다. 공개issue/PR/릴리스는게시하지않는다.

## 비용과 인수

원text최대2MiB와expectedpost최대2MiB를activecomposition에서만저장하며추가상한4MiB, Arc로stateClone은참조만복제한다. snapshot은serde/debug/일반로그로노출하지않고commit/cancel/focus수명종료에서반환한다. snapshot불가/초과의preedit는원selection을파괴하지않으며commit기본경로를보존한다. 외부text변경과일반편집충돌을강제rollback하지않는다. undo는원사용자history를유지하고preedit중간값을영구undo checkpoint로남기지않는다. 소스vendor용량과실제binarydelta는통합후측정한다.

정확actualnative번역순서의동일Context회귀를원dependency에먼저실행해실패를보존하고,새vendor에서green을확인한다. 빈값/삽입cursor/기존선택/역방향/Unicode/multipreedit/splitframe/nonIME/focus/외부mutation/oversize/UndoRedo를검증한다. 최신Windows전체4명령/defaultfeature복원과nativecancel/commit의원context복원·UIA/PNG·naturalexit/Job0·desktop보존을root가대조한다. 실제물리키IME/reader/전체P0는별도게이트다.

2026-10-05 좁은버그인수 완료: 원본egui에서 동일회귀4PASS/5FAIL 후vendor수정9PASS; 실제IMM취소UI빈값·확정UI한글/nativecontext원복/자연exit0/Job0는root독립검토PASS다. [native취소](../validation/evidence/windows-ime-compose-cancel-patched-20261005-resume/root-review.json), [native확정](../validation/evidence/windows-ime-compose-commit-patched-20261005-resume/root-review.json), [fixture전체4검증](../validation/evidence/windows-ime-cancel-patched-feature-20261005-resume/root-review.json), [현재defaultCPU전체4검증](../validation/evidence/windows-ime-cancel-current-default-20261005-resume/root-review.json). 현재defaultexe7,856,640B,계측featureexe7,893,504B로차이36,864B이며새dependencyversion없다. baseline retained4MiB상한은원text+expected값만이며allocator/galley/undo일반비용을포함하지않는다. 원오류·원bridge미채택·fixture최초실패를보존했다. 물리키IME/reader/전체S00최종스택/전체P0는별도미완료다.

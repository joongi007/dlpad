# S00: Windows IME 취소 후 조합 문자열 잔류 수정

- 상태: done
- 분류: bug
- 우선순위: P0
- 요구사항: R06
- 담당/수정 범위: root는 이 문서·software_host.rs·공유entry/Cargo·통합검증. diagnostic_cache는 /tmp/ime_input_bridge.rs의입력bridge/실제Context회귀초안. metrics_recovery는 /tmp/ime_text_transaction_design.rs 또는md의원선택복원설계초안만담당한다. projectedits는root만수행하며동일파일동시수정없다.
- 선행 작업: actualWindows IMM합성cancel에서nativecontext복원성공/actualUIA·PNG한글잔류FAIL, 동일binarycommitPASS. S00-windows-owner-ime-compose 원실패보존.

## 결과와 인수

Windows 실제 IME취소후원committed텍스트와커서·선택을보존하고확정은정확히한번반영한다. 우선원nativeIme이벤트순서를기본OFF boundedtrace로확인해bridge후보원인을분리한다. 실제event를위조하지않고UI번역경계만조정한다. nativecontext API성공을UI성공으로대체하지않는다.

빈값·기존문자열의삽입cursor·기존선택치환취소·같은batch/splitframe 확정·중복Disabled·nonIME혼합을actualeguiContext에서검증한다. 원선택텍스트는egui가preedit로이미삭제하므로emptypreedit만으로복원완료를주장하지않는다. 입력transaction이필요하면원field권한·최대크기·singlefocus·snapshot수명·비밀Debug생략을명시하고독립편집을임의Undo로삭제하지않는다. 실제Windows격리GUI에서cancel/commit·context복원·원프로세스자연exit/Job0·desktop보존·UIA·PNG를root가대조한다. 물리키변환/전체reader/전체P0는별도미완료다.

2026-10-05 좁은버그인수 완료: 원본egui에서 동일회귀4PASS/5FAIL 후vendor수정9PASS; 실제IMM취소UI빈값·확정UI한글/nativecontext원복/자연exit0/Job0는root독립검토PASS다. [native취소](../validation/evidence/windows-ime-compose-cancel-patched-20261005-resume/root-review.json), [native확정](../validation/evidence/windows-ime-compose-commit-patched-20261005-resume/root-review.json), [fixture전체4검증](../validation/evidence/windows-ime-cancel-patched-feature-20261005-resume/root-review.json), [현재defaultCPU전체4검증](../validation/evidence/windows-ime-cancel-current-default-20261005-resume/root-review.json). 현재defaultexe7,856,640B,계측featureexe7,893,504B로차이36,864B이며새dependencyversion없다. baseline retained4MiB상한은원text+expected값만이며allocator/galley/undo일반비용을포함하지않는다. 원오류·원bridge미채택·fixture최초실패를보존했다. 물리키IME/reader/전체S00최종스택/전체P0는별도미완료다.

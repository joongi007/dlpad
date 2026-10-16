# dlpad egui IME 취소 패치

정확 egui 0.33.3 registry source/upstream commit44cdd653e2317d300fb8a6c9c36b03f23991e803를 보존한다. DLPAD_SOURCE.json은 원fileSHA와 변경2파일을 기록한다. 공식 고정commit LICENSE-MIT/APACHE를 함께 보존한다. 원 .cargo-checksum.json은 patch 전registry원문의기록이며 수정후전체checksum으로표현하지않는다.

state/builder만 조합전text·선택의boundedephemeraltransaction을 추가한다. 빈Commit/취소에서원상복원,nonemptyCommit에서확정,외부변경강제rollback거절,조합중undo저장차단을 연결한다. 원2MiB+expected2MiB와Arc참조clone을사용하며serde/debug에snapshot을노출하지않는다. 새version/dependency는없다. 최초원Context회귀4pass/5fail을보존했고패치Context/native 인수는진행중이다.

[작업계약](../../docs/work-items/S00-egui-ime-transaction.md)을따른다. 물리IME키변환/reader/전체P0완료가아니다. 공개upstreamissue/PR는게시하지않았다.


2026-10-13: 서로 다른 Preedit/Commit 문자열의 역방향 선택 확정 문제를 수정했다. Preedit에 저장하는 cursor range는 실제 반환 preview range와 같고, 성공 취소 시 원 문자열·directed cursor와 원 commit anchor를 함께 복원한다. Commit의 선택 이동·외부 변경 거부 및2MiB snapshot/expected 상한은 유지한다. 기존9회귀는 조합·확정 문자열을 분리해 강화했으며 실제 Settings8개 조합과 stale selection2개를 별도로 검사했다. Source-only v1 후보가 새 split Disabled/Commit 회귀를 일으킨 실패도 보존했다.

[현재 작업](../../docs/work-items/S00-settings-ime-positive-commit-20261013.md)의 native Context 검증 범위이며 실제 OS 키보드 IME/reader 인수는 별도다. 새 version/dependency/state allocation/thread는 없고 기존 range copy만 갱신한다. DLPAD_SOURCE.json의 원 registry SHA·upstream commit/license와 changed_files(state/builder)는 원본 provenance로 유지한다.

## 2026-10-15 AccessKit 문자열 값 변경

같은0.33.3의 builder는 대상 Action::SetValue String을 enabled mutable TextEdit에 적용한다. UIA가 전달한 문자열 요청을 누락하던 실제 Context 9개 중7실패를 수정했으며 한글·문자수/단일행·undo·foreign target/data·readonly/disabled·다른 focused input·조합 중 외부 변경/늦은 Commit 회귀를 확인했다. 실제 private URL 및 검색 입력 readback도 v27에서 확인했다. 원 IME transaction·registry/upstream checksum·license·changed_files provenance는 그대로 유지한다.

SetValue capability는 수정 가능한 입력에만 광고하며 최대2MiB 문자열을 검사한다. 다른 입력의 키보드나 포커스를 소비하지 않고 입력 변경 시만 상한 있는 일시 문자열을 만든다. 새 dependency/version/Cargo.lock/state schema/thread는 없다. 전체 v27 Windows 검사1593/0/42와 scoped private readback 근거이며 물리 IME/reader·전체P0 완료는 아니다. [작업](../../docs/work-items/S00-native-text-value-action-20261015.md)을 따른다.

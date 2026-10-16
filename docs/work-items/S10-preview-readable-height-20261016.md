# S10: 작은 창 진단 미리보기의 읽을 높이

- 상태: done-scoped — 실제 RED/GREEN·제품 통합·전체 검사·fresh PE 100/200 화면 확인
- 분류: bugfix / UX
- 우선순위: P0
- 요구사항: R13/R08, S10 로컬 진단·정제 미리보기
- 담당/수정 범위: field_reveal. 새 scratch `current32-s10-preview-height-investigation-20261016/`의 RED/GREEN accessible_diagnostics.rs·accessible.rs 및 이 새 문서만. 제품 채택은 Root.
- 선행 작업: current32 S10 JSON export/collision 원 functional 검수와 Root4PNG 직접 읽기.

## 사용자 결과

실패 보고서 미리보기가 세 줄로 눌리지 않고 최소6줄을 읽을 공간을 갖는다. 뒤 저장 경로와 편집 입력도 기존 바깥 스크롤·키보드 focus로 접근할 수 있어야 한다.

## 계약과 범위

[DESIGN](../../DESIGN.md)과 [진단 계약](S10-failure-diagnostics.md)을 유지한다. 큰 재설계 없이 nested ScrollArea의 실제 font 기반 최소 높이만 검토한다. 최대320·bounded 페이지·원 identity/readonly·정제/저장/충돌·worker 수명은 유지한다. 새 deps/lock/schema/제품/원 evidence 변경 없음. cfg(test) 실제 geometry tuple만 추가하며 release 계측은 없다.

## 인수 조건

실제 Main Context의960×640100/200 기하에서 원 preview action 후 actual parent clip 안에6개 이상의 완전한 nonempty paint 행과 기존 padding을 확인한다. 원 focus를 빼앗지 않고 뒤 저장 입력·log editor는 실제 Focus로 고정 navigation/footer 사이에 드러난다. RED가 의미 실패를 재현하고 GREEN/기존회귀/strict 검사를 Root가 통합 검증한 뒤 actual 새 PE PNG까지 직접 읽는다. 극소창 등 추가 범위를 임의 통과시키지 않는다.

## 검증과 handoff

실제 수행은 source/locked egui 원 계산·폰트 default 경로 읽기와 scratch rustfmt4파일뿐이다. Context/Cargo/native 미실행. 원 기능 성공은 유지하되 UI 전체 통과로 확대하지 않는다. Root 실행 명령·핀·미검증 경계는 scratch HANDOFF.md/source-pins.json에 있다.


## Root 실제 재현·수정 검증

원1차 호출은 sandbox WSL interop `UtilBindVsockAnyPort socket failed1`로 Windows 시험 전 실패했다. 사용자 허용 private native 검증 범위의 require_escalated 호출은 승인됐으며 자동 승인 거절은 없었다. RED2 시험은 labelled_by 관계를 직접 label만으로 찾던 collector 오류 때문에 geometry 전 unique node0으로 실패했다. 두 원본은 보존했다.

같은 실제Main 회귀의 resolver를 기존 labelled_by→Label/value 관계로 바로잡은 RED3은100% viewport64px/row17.28125/완전3행으로 실제 의미실패를 재현했다. 첫 GREEN은100%8행이었지만200% inner높이150.25가150인parent body를3.25px 침범해 실패했다. 본문clip−기존2*item_spacing으로 목표8행높이만 반응형 제한한 GREEN3은 원6행/전체부모포함/뒤focus 조건을 바꾸지 않고100%150.25px/8행·200%134px/7행을 실제 통과했다. 각각 native Cargo101/0와자연Job0·input/private 종료·원Live불변을 확인했다. [RED3](../validation/evidence/current32-s10-preview-height-red-v3-20261016/preview.log), [첫 GREEN 실패](../validation/evidence/current32-s10-preview-height-green-v1-20261016/preview.log), [최종 GREEN](../validation/evidence/current32-s10-preview-height-green-v2-20261016/root-review.json).

Root가 정확2파일을 원백업 후 Live에 채택했다. source675/lock/의존성은 유지한다. 새 whole33 fmt·strict all-targets clippy·test·release가 실제 성공했고1,670 PASS/0 FAIL/42 ignored/43harness를 Root가 원로그/새3PE/source와 독립 대조했다. [새 전체검토](../validation/evidence/windows-msvc-ui-feedback-v33-whole-20261016/root-review.json). 실제100/200 private 화면과 패키지 검수는 별도 진행한다.

Root가 새 CPU의 실제100/200 private 진단 미리보기·JSON971B 저장·같은 경로 거절/원파일·DB 보존을 독립 대조했다. 10PNG를 모두 직접 읽고 최소6개 완전한 nonempty JSON행 및 뒤 저장/편집 접근을 확인했다. [실제 화면 통합 검토](../validation/evidence/current33-s10-preview-composite-root-20261016/root-review.json). 원 실패 collector와 부분100 성공을 보존하며 전체P0·물리clipboard/IME/reader·모든 OS를 주장하지 않는다.

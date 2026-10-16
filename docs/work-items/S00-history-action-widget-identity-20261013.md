# S00: 작업 이력 명령의 안정된 widget identity

- 상태: completed (scoped)
- 분류: UX
- 우선순위: P1
- 요구사항: R06, R08
- 담당/수정 범위: root — 통합/실제검증/본 기록; windows_auto — /tmp 후보; crates/app/src/accessible_history.rs
- 선행 작업: 기존 P0 Job/managedtool typed정책·상한 구현; 현재v16 whole/native source동결 해제

## 사용자 결과

이력 갱신·정렬 뒤에도 키보드/접근성 명령의 대상을 원래 작업에 연결한다.

## 계약과 범위

source-only 감사에서 row action의 자동/인덱스 ID가 entity key를 갖지 않는 위험을 찾았다. 실제Context로 목록 재배치시 원node/Response와요청대상을 확인하여 제품오작동과가능성을구분한다. 이미있는 Job/설치identity만사용하고 core상태·worker/삭제·schema/공유UI API/Cargo는변경하지않는다. Root외에livefile을수정하지않으며 두파일의 담당을나눈다. 새tooltip/Remove사유/대규모레이아웃을섞지않는다. ROADMAP의eligiblelocalUI개선 범위이며S08/S09확장stage완료를주장하지않는다.

## 인수 조건

실제render에서 기존2개entry→순서변경/앞삽입/한entry제거 후같은entityaction의widget/nodeID가유지되고 action이올바른entity에연결되는지 확인한다. 단순idhash값복제시험은추가하지않는다. 실제원동작이실패하지않으면 제품버그로선언하지않고 기존보장범위를기록한다. 채택시관련실패경로·Windows전체·현재GUI를확인한다. 물리reader/keyboard는별도이다.

## 검증과 handoff

현재는/tmp후보 준비단계이며 아직실제RED/GREEN/제품코드변경을하지않았다. 현재v16 sourceguard검증이끝날때까지 Root도live변경을하지않는다. 원P0/전체UI 인수미완료를이후작업완료로대체하지않는다.


실제original Context에서 B의원nodeClick이 목록재정렬뒤 Resume(A)를전달해Rust101/0성공1실패로재현했다. 최소entity-key scope후 원대상전달·제거node거부는PASS이나 metadata행 geometry회귀1개는실패해통합을보류한다. bounded관측은label과Resume 중심차7px/버튼오른쪽851.8125를기록했으며 원geometry대조전제품/시험원인을확정하지않는다. 원FAIL을보존한다.


동일800×600/8frame original metadata장면은32px로다음줄(8,170)-(72,202)에배치해PASS했다. 첫keyedwrapper는같은frame4에서46px·우측851.8125로실제회귀RED였으므로폐기했다. 기존64×32desired space를부모wrappedlayout에서먼저확보한뒤keyedscope를생성하는최소수정을채택했다. 실제rowband 매핑회귀는32높이/800내전체control/paintclip 검증을추가했으며 viewport/tolerance를늘리지않았다. 최종actual_history_8/0/0(원ID대상/removednode거부/focus/metadata/기존실패경로)이PASS이고Windows전체/sourcefreeze는진행중이다. [원배치](../validation/evidence/windows-msvc-history-metadata-original-20261013/root-review.json), [후보회귀](../validation/evidence/windows-msvc-history-metadata-keyed-20261013/root-failure-review.json), [최종회귀](../validation/evidence/windows-msvc-entity-action-identities-green-v2-20261013/root-review-v2.json).


현재v17 whole1,319/0/33/source656 및current3PE·ZIP·63화면·direct정상/변조outer0·독립27ID·설치8pixels를root통합검증했다. 빈목록PNG로행명령을증명하지않으며실제Context의원node→entity전달·접근성FocusACK·행geometry/paint회귀범위를완료한다. 물리keyboard/reader/전체UI인수는별도이다. [전체](../validation/evidence/windows-msvc-entity-action-identities-whole-20261013/root-review.json).

# S00: 작은 창 긴 도움말의 키보드 스크롤

- 상태: candidate / integrated verification pending
- 분류: accessibility
- 우선순위: P0
- 요구사항: R08, R10
- 담당/수정 범위: field_reveal는 scratch help-keyboard-scroll-v2, Root는 ui_components.rs 통합·전체/Windows 검증·문서.
- 선행 작업: source31 작은 popup 제한; 원 Cargo terminal 후 S에 단계적 적용.

## 사용자 결과

작은 창의 고정된 긴 도움말을 원 정보 버튼 포커스를 유지하며 PageUp/Down·Home/End로 읽는다.

## 계약과 범위

기존 popup 열림·logical owner focus·overflow가 모두 참인 보호 도움말만 키를 소비한다. 자동 tooltip·일반 input·다른 owner·닫힌 popup에는 키를 전파한다. 기존 Escape/reentry·입력 보호·자연 clamp를 보존한다. 의존성/schema/feature 변경 없음. 짧은 키 처리만 추가하고 daemon/worker/상시 timer를 만들지 않는다.

## 인수 조건

실제 Context에서 처음/끝 glyph·양방향 Page·반복 clamp·원 ID/입력 rect·Escape6·닫힘/다른 owner의 키 전파를 확인한다. private native는 원 HWND에 제한한 키 메시지·처음끝 원 캡처·원 입력 불변·자연종료로 별도 검증한다. Context 통과를 native 성공으로 대체하지 않는다.

## 검증과 handoff

v1 테스트 helper의 private API 사용을 v2에서 기존 공개 NodeId 변환으로 보정했다. child rustfmt0, Cargo/native 미실행. Root RED→GREEN→기존 Help 회귀→strict clippy 및 최신 전체/실제 화면은 아직 대기다. 원 wheel 실패는 보존하며 원인을 확정하지 않는다.

Root 실제 Windows: `root-help-keyboard-stage-v1/run.py help-keyboard-red-v1 20261016`는 test101/1PASS1FAIL(고정 도움말의 PageDown 미소비)을 재현했고, GREEN 동일 명령은 test0/2PASS0FAIL이다. 원 HWND/private Job 자연0·최종0·사용자 input 불변 및 Live/S 불변을 확인했다. [GREEN 원결과](../validation/evidence/managed-tool-isolated-help-keyboard-green-v1-20261016/result.json). 기존 전체 Help 회귀·strictclippy·Live통합·실제native는 아직 대기다.

Root 실제source32 native100/200 두씬을완료했다.3owner×Escape6/원ID·input/browse/value/포커스·자연0/Job0와39원pins·38PNG·12raw재해시,8원PNG직접검토를확인했다.200%FFmpeg/Deno의End원마지막문장전체·Home처음복귀와owninput/browse비가림은인수했다. 다른footer/내용을overlay할수있으며보편비가림·physicalIME/reader/실마우스/전체UI는미검증이다. [Root검토](../validation/evidence/current32-help-keyboard-private-v1-20261016/root-review.json).

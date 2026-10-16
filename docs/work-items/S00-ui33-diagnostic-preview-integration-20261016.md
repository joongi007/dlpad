# S00: 진단 미리보기 가독성 수정의 제품 통합

- 상태: done-scoped — 통합·전체 Windows 검사·패키지·설치·실제 진단 화면 확인
- 분류: bugfix / integration
- 우선순위: P0
- 요구사항: R13, R08, R06, R11
- 담당/수정 범위: Root / accessible.rs·accessible_diagnostics.rs, 새 whole/패키지/실제 private 화면 evidence, 관련 상태·검수 문서. 공유 Cargo/lock/schema 변경 없음.
- 선행 작업: v32 JSON 저장/충돌 성공·실제64px/3줄, RED3 의미실패·GREEN2 실제Main100/200 PASS

## 사용자 결과

진단 자료 미리보기가 작은 창에서 세 줄로 눌리는 문제를 고친다. 폰트의 실제 행 높이와 본문 크기에 맞추며 저장/편집은 바깥 스크롤과 키보드로 접근한다.

## 계약과 범위

S10-preview-readable-height의 원 실패를 보존하고 GREEN3 두파일만 채택했다. 일반 화면은8행 높이를 목표로 하며 작은 본문은 실제 clip와 기존 item spacing으로 제한한다. 최대320·원12행 문서·bounded 페이지/readonly/정제/export worker·field ID는 유지한다. 런타임 추가 비용은 화면의 font row-height/clip 계산이며 추가 I/O·스레드·repaint wake나 의존성이 없다. 실제 geometry 계측은 cfg(test)다.

## 인수 조건

Root 원2파일 백업·source675 중정확2파일/lock 불변·RED101/자연Job0와 GREEN0/자연Job0·6행/부모 clip전체/뒤 focus 확인. Windows 전체 fmt/clippy/test/release를 새 source로 실행한다. 새 PE의 실제 private JSON/저장/충돌/DB 보존과 캡처를 확인한다. 새 ZIP은 원 bundle/notice/graph/CRC/SHA 연결과 설치 검증 뒤에만 최신으로 안내한다. 기본 성능이나 물리 IME/clipboard·전체P0는 별도다.

## 검증과 handoff

Root 채택 receipt `.local-tools/session-recovery-20261013/root-ui33-integration-v1/receipt.json`. 원64px RED와 첫 GREEN의200%3.25px clip침범·접근성 label resolver 시험오류·첫 WSL sandbox interop 실패를 각각 보존했다. 최종GREEN Context는100%8행/200%7행·뒤focus 접근 PASS다. 이후 실제 제품 검증은 아래 완료 기록을 따른다.


새 whole33 실제 fmt/clippy/test/release0,1,670 PASS/0 FAIL/42ignored/43harness. Root는 원 raw/harness/source675·정확2파일변경·lock 불변·3PE와 GNU원본을 재대조했다. CPU10,361,344B(이전보다1,024B 증가), CLI/설치기는 원v32와 byte동일이다. [Root 전체 검토](../validation/evidence/windows-msvc-ui-feedback-v33-whole-20261016/root-review.json). 이 단계 뒤 fresh PE 100/200과 새 graph·패키지 검수를 완료했다. 현재 최신 검수 ZIP은33이다.


새 ZIP7/467member CRC/SHA·선택258/notice456 및 생성payload 전체 연결 검사를 통과했다. 실제 설치/제거/동일위치 재설치/cold검증/installedCPU 자연0와Root33pins·두journal각5/최종payload5·직접4PNG를 확인했다. 현재33 실제진단100/200·10PNG/DB/971B·원실패보존도 scoped 통과했다. README와 사용자 실행 안내는v33으로 갱신한다. 새기본CPU성능·긴경로/로컬CI·물리OS/전체P0 인수는 별도다.

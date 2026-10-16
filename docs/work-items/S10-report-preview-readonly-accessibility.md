# S10: 미리보기 읽기 전용 접근성 수정

- 상태: done
- 분류: fix
- 우선순위: P0
- 요구사항: R08, R13
- 담당/수정 범위: root — accessible_diagnostics.rs, accessible.rs, 이 문서
- 선행 작업: Windows private report GUI 실제 readonly=false 증거

## 사용자 결과

편집 불가능한 보고서·실행 계획 미리보기를 접근성 API에서도 읽기 전용으로 표시한다.

## 계약과 범위

egui interactive(false) 설정은 유지하고 해당 response ID의 AccessKit node에 ReadOnly flag를 명시한다. vendor·dependency·Cargo schema·사용자 desktop은 변경하지 않는다.

## 인수 조건

CPU/alternative compile·clippy, 새 Windows immutable binary의 실제 UIA IsReadOnly=true 및 보고서 preview/save 개인정보 생략 검증. 일반960x640 scroll은 별도 인수다.

## 검증과 handoff

root 구현 후 실제 검증 진행. 기존 immutable binary의 false 결과는 보존한다.

Root Linux CPU cargo check exit0. Windows GNU CPU release는 exit0/280개 입력 before-after-current 일치, 새binary SHAe2712ef9589ca0524ff6223d1c5e8f2175a32eceb88c9399b6b643f77097791c/7,286,784bytes. report-readonly-windows-release에manifest·binary·rootreview를보존했다. 실제GUI는 수집기의CS0160/CS0136 compile실패로 앱시작전에중단했으며 source수정후fresh재시험예정. 제품UIA readOnly=true 검증은 아직미완료다.

최종 root 실제 새Windows GUI는 readonly=true 및 TextPattern text/SetValue무반영을 확인했다. preview/export SHA동일·1478bytes·비밀과private경로생략·sentinel덮어쓰기거절·Failed/Attempt1/artifact0/쿠키copy제거를원시파일·SQLite와독립대조했다. [검토](../validation/evidence/windows-cookie-gui-auth-report-readonly-scope-retry/root-review.json). CPU+alternative strictclippy와app시험도exit0. 격리창을1700높이로늘린UIA검증이며 기본960x640scroll·clipboard·실제IME·다른OS는별도다. PNG는물리화면높이아래검은영역으로clipped되어전체미리보기렌더링증거로쓰지않는다.

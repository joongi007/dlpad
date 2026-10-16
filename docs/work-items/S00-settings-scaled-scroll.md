# S00: 확대된 설정 화면의 스크롤 접근

- 상태: done
- 분류: bug
- 우선순위: P0
- 요구사항: R06, R08
- 담당/수정 범위: root; accessible.rs settings branch, 본 문서 및 전용 검증 문서. cpu_scaled_native_probe.py/증거는 stress_recovery 독점
- 선행 작업: CPU150/200% 실제 private GUI 실패 재현

## 사용자 결과

확대·작은 창에서 아래 설정과 한글 입력 검증 필드에 실제 스크롤로 접근한다.

## 계약과 범위

PRODUCT/DESIGN/ADR0001·impeccable product register를 읽는다. 하루 중 다른 작업 옆의작은창과야간확대사용을위해기존Light/Dark/System·native컨트롤을유지한다. 608eeeb System200%960×640에서footer y519/관측적용y551/IMEy662로잘림을확인했다. settings에bounded ScrollArea를추가하며session-onlyscale/테마저장정책/Cargo/schema변경없다. 사용자desktop/숨은action/창확대우회없음.

## 인수 조건

150/200%Light/Dark/System에서actualscroll 후한글필드·적용button이contentclip안에존재, 실제입력·키보드focus·theme저장·소유정상종료. source/binary freeze 및기존100%실행계약을구분한다. 다른OS·전체IME판정은별도게이트.

## 검증과 handoff

원시실패는evidence/cpu-scaled-native/system-200-1에보존. 좁은설정scroll수정뒤새snapshot으로실제GUI재검증한다.

## 통합 담당 검토

root가 scroll-matrix의6개실제휠/IME/도움말bounds와각18개원PID경로부재를재확인했다. [독립대조](../validation/evidence/cpu-scaled-native/root-review.json). 현재새서비스backend빌드는별도전체회귀중이며이GUI결과로미검증OS나S00전체완료를주장하지않는다.

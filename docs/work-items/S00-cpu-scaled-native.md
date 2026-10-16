# S00: CPU 배율·테마별 네이티브 입력 회귀

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R06, R08, R11
- 담당/수정 범위: stress_recovery — scripts/cpu_scaled_native_probe.py 및 전용 validation/evidence/본 작업 기록
- 선행 작업: 608eeeb CPU release READY, 기존 기본 배율 IME 검증

## 사용자 결과

960×640에서 밝게·어둡게·시스템 테마와 150/200% 배율의 필수 설정 접근 및 실제 한글 입력을 검증한다.

## 계약과 범위

ADR-0001, DESIGN, S02 계약 기준. 테마만 저장되며 배율은 세션 한정이다. 제품·공유 harness·Cargo 수정 없음. Xvfb·DBus·Fcitx·임시 데이터는 이 시험 소유로 분리한다. 숨겨진 Action, 창 확대, 사용자 desktop/clipboard 사용으로 실패를 우회하지 않는다.

## 인수 조건

실제 ComboBox 선택, theme.sqlite 값, 배율에 따른 실제 입력란 높이, 상태 패널 상단까지 필수 입력란 전체 가시성을 기록한다. 보이는 경우 ASCII GetText, 한글 조합/확정/취소, Latin 전환, 선택 교체 및 Tab focus metadata를 확인한다. 잘림은 실패로 보고한다. PNG만으로 입력 성공을 판단하지 않는다.

## 검증과 handoff

608eeeb 고정 snapshot 6개 case를 실행: 150% 세 테마 실제 입력/취소/Tab PASS, 200% 세 테마 필수 control/IME 잘림 FAIL. docs/validation/cpu-scaled-native-validation.md와 evidence/cpu-scaled-native/에 실제 결과를 보존한다. 제품 root 담당에게 clip 근거를 전달했다. 새 layout binary READY 후 같은960×640 재검증이 남는다. 최신 guardian source와 이 고정 binary의 결과를 구분한다. 전체 OS/전체 reader/원시 XIM callback은 본 범위 밖이다.

후속 검증: root settings-content ScrollArea가 포함된 immutable CPU `/tmp/dlpad-settings-scroll-binary-cgg8vee9/dlpad-cpu` SHA `1fe649add2558aca14b30785b7498fa4820986099e639d15ea5f716e86689f0e` 12,404,344B를 읽기만 사용한다. 기존 608eeeb 실패 raw는 불변. 실제 휠 후 관측 적용·IME·IME 안내문 전체 bounds와 PNG를 확인하고 실제 입력/Tab을 반복한다.

최종 제한 scope: immutable1fe649 scroll-*-2 6case 실제휠 후 관측 적용·IME·안내문 전체접근, 실제한글·취소·Tab PASS/exit0. 개별PID/start와runtime잔존0 독립확인. 원608fail 및 scroll-*-1 collector실험fail 보존. docs/validation/cpu-scaled-native-validation.md 참조. Python구문/check_docs322 PASS. 전체OS/reader/키보드스크롤/후속 backend 소스검증은 남는다.

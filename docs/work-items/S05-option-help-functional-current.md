# S05: 도움말 변경 후 옵션 실제 기능 회귀

- 상태: completed (Linux CPU 합성 GUI 3case)
- 분류: validation
- 우선순위: P0
- 요구사항: R07, R08
- 담당/수정 범위: root; 이 문서 및 p0-option-help-functional-current evidence
- 선행 작업: CPU 도움말 가시성/고정 release ca8e969f snapshot

## 사용자 결과

높이 선택·정확 구간 자르기를 실제 worker argv/DB에 적용하고 구간과 자막 충돌은 새 작업 없이 거절한다.

## 계약과 범위

기존 cpu_options_probe의 합성 tool, 임시 media/data, 소유 Xvfb를 사용했다. 사용자 데스크톱·Documents·실제 영상·계정 쿠키는 사용하지 않는다.

## 인수 조건

height는720 selector, accurate는밀리초구간과force-keyframes argv, conflict는Job/worker미생성. 각binarySHA 전후일치/소유child회수/정상종료를 요구한다.

## 검증과 handoff

root 실제 height/accurate/conflict 각exit0/PASS를확인했다. 원시case result/argv/DB/PNG는 p0-option-help-functional-current에보존한다. 다른OS·키보드전체·실제사이트로확대하지않는다.

# S11: v26 설치·재설치와 새 기본 CPU 실행 검수

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: installer_batch 신규 B/v26-install-reinstall-launch-private-v1/helper와 본 문서; Root actual native/통합 검토
- 선행 작업: whole-v5 actual PASS, Root review, 현재 defaultCPU 3PE와 새 ZIP/generated7 실제 생성 검토

## 사용자 결과

새 패키지로 소유 임시 설치→검증→제거→같은 위치 재설치→설치기 재시작/새 기록 검증을 재확인한다. 설치된 새 앱의 실제 private HWND/UIA를 관측하고 앱과 supervisor의 자연 종료를 확인한다.

## 계약과 범위

원 v25 helper/증거와 source670 v24 긴 경로33pin 검증은 불변이다. 이번에는 최소 정상경로 helper 하나로 재설치와 새 CPU 실행을 준비하며 v26 long path는 미검증으로 둔다. 원 privateDesktop/Job/PID/creation/MTA/명시 child env/짧은 synthetic data·media/Light seed/sentinel/320초를 유지한다. 실제 사용자 폴더·default/visibleDesktop·전역 registry·계정/영상/네트워크·global input을 사용하지 않는다. Live/S/Cargo/schema 수정과 공개 배포는 제외한다.

## 인수 조건

whole-v5/source/current3/Root raw43와 ZIP7→generated7→manifest5 정확 연결, 모든 CRC/size/SHA 전후 대조. old/new journal5 각각 보존·최종 payload5·원 source7 보존, actual Install201/재시작 후 cold matching Verify203 성공. 첫 installer0+fresh supervisor0+actual app0/총process3/Job0·inputdesktop 불변. Started를 GUI-ready로 승격하지 않고 실제 HWND/UIA 관측과 구분한다.

## 검증과 handoff

Python AST 및 원 LaunchLifetime 함수/payload validator byte identity 통과. C# compile/native/package 실행은0회다. Root만 새 currentwhole/ZIP을 바인딩해 exact argv로 실행한다. 준비 실패이면 원증거를 보존하며 userDesktop으로 fallback하지 않는다. B/HANDOFF에 actual long 역사 범위와 미검증, 측정필드 제한·비용·원본핀을 기록했다.

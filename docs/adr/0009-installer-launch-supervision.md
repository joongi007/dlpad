# ADR 0009: 설치 창과 설치된 앱의 실행 수명

- 상태: Accepted
- 날짜: 2026-10-13
- 관련 요구사항: R08, R11
- 선행: [앱 설치 기반](0005-app-install-foundation.md)

## 문제

설치 완료 뒤 앱 실행 버튼이 없다. 기존 typed 실행 API의 소유 프로세스·Job을 즉시 해제하면 실행한 앱도 종료되므로 설치 창 종료와 프로세스 종료를 구분해야 한다. 설치 bool이나 경로 문자열만으로 실행 권한을 만들 수 없다.

## 결정

설치/재검증 완료 시 설치 버튼은 비활성화하고 앱 실행을 활성화한다. 클릭하면 worker가 bundle·journal·active selection의 원identity/record/실행파일을 다시 검사한 후 existing launch_explicit_selection으로 시작한다. shell·Explorer·무소유detach·관리자권한·자동실행을추가하지않는다.

Started는 원processID/creation의 resume ACK이며 GUI 준비 증거가 아니다. 완료된 실패를 queued Started보다 우선하고 모든 실패 outcome/원exitcode를 보존한다. 대기 중 닫기/중지는 취소하여 원owner 종료를확인한다. Started 뒤 정상설치창닫기는 앱을취소하지않고 HWND만닫으며 원설치process가bounded supervisor로 앱과Job의자연종료까지살아있다. 비정상message loop종료와owner포기는취소한다. 알려지지않은cleanup은원capability를보존하고새실행/변경을막는다.

최초실행·inspection은UIthread밖, cap1알림/단일worker, startup상한1–600초(실제300초), runtime에는임의앱종료deadline을두지않는다. 기존wait_cancelled의100ms kernel wait를사용한다. dependency/version/lock 변화는없으며프로그램크기·실제idle메모리/비용은통합후측정한다.

## 인수와 한계

Windows typecheck/clippy/test/release, 실제격리5파일설치→검증→실행버튼→설치HWND없음/원owner생존/실제앱HWND→원앱닫기→둘의자연종료·Job0을확인한다. 실패·취소·중복·stale/tamper·즉시nonzero와마지막실패보존을검증한다. 합성appdata/media는설치namespace밖의소유sibling이며사용자Documents·실영상·계정·쿠키를사용하지않는다.

HWND를닫은뒤늦은앱실패의오류창재표시는별도이며, Started로최초화면/정식배포완료를주장하지않는다. Windows whole1,295/0/30과 v11 실제 설치·실행의 양쪽 자연종료, 별도 exe 변조 거부·자식 미생성·실패 receipt 유지가 통과하여 이 수명 구조를 채택한다. 설치기 크기는 48,128바이트 증가했으며 runtime 추가 비용은 아직 측정하지 않았다. [작업 계약](../work-items/S11-installer-launch-affordance-20261013.md).

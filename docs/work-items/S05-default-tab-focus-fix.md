# S05: 기본 Iced Tab 포커스 이동

- 상태: completed (URL/저장경로 실제 Tab 이동 수정 범위)
- 분류: fix
- 우선순위: P0
- 요구사항: R01, R07
- 담당/수정 범위: root — crates/app/src/main.rs, scripts/default_tab_navigation_probe.py, 이 문서
- 선행 작업: p0-default-tab-native-first 실제 URL xy 재현

## 사용자 결과

기본 화면의 Tab/ShiftTab으로 focusable 입력 사이를 이동한다. 현재 URL에서 Tab 뒤에도 다음글자가 URL에 입력되는 실제 문제를 수정한다.

## 계약과 범위

Iced ignored keyboard Tab에만 focus_next/previous task를 연결한다. Ctrl/Alt/Logo 조합과 widget이 이미 처리한 키는 건드리지 않는다. 기본Iced네이티브 유지/새dependency없음. 도움말문자열/Windowsjournal은 별도 담당.

## 인수 조건

격리 GUI의 URL x→Tab→y 입력이 URL xy로 남지 않고 다음저장경로로 이동해야한다. ShiftTab URL복귀 및 원래툴/DB미실행·소유자원회수를 확인한다. 전체앱버튼/화면밖포커스/reader/타OS는 별도 범위다.

## 검증과 handoff

Root가 원 first PNG(url-x/after-tab/after-y)를 직접 읽어 URL x→xy 유지 및 URL focus outline을 확인했다. 원시보존. 실제 수정후 immutablebuild/동일fixture 재실행 pending.

Root 실제 수정후 build fmt0/release0, binary10095488B/SHA1cc9f5644d7af2138670c089ea5a3551ee5cfe89e7a113536d2ab227cc8bc360. 기존10082464B 대비13024B 증가; 이벤트subscription과기존Icedfocus task만 추가하며dependency/상주worker/I/O없음.

수정후 p0-default-tab-fixed-native-second collectionexit0/nojob/notool/ownedcleanup/binarybeforeafter동일. Root가 after-tab/after-y/after-shift-tab PNG를 직접 확인했다: URL x 유지, 저장경로에 y입력, BackSpace복원후 ShiftTab URLcaret복귀. root-review.json에범위를 기록했고 collector keyboard_navigation_verified=false는 전체계측미지원뜻으로 원시그대로 유지한다. 기본2input이동 visual범위 PASS. 전체controls/화면밖포커스/reader/다른OS는 남는다. 최신Linux/Windows전체검증은 windows-journal-default-tab-workspace-* terminal로 별도 확인한다.

현재source 전체Linux7+2toolversion과 WindowsGNU4명령 모두exit0/입력beforeafter+현재SHA일치를 root가 확인했다. 증거 windows-journal-default-tab-workspace-first/result.json 및 windows-journal-default-tab-workspace-native-first/result.json. 전체키보드 인수로 확장하지 않는다.

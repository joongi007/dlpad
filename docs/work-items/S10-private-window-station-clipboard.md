# S10: 사용자 clipboard와 분리된 Windows 복사 시험

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R10, R13
- 담당/수정 범위: root 작업 기록·prototype 게시/native 실행·공유host 조정·통합 검증; diagnostic_cache /tmp C# prototype/primary API 읽기만
- 선행 작업: private desktop의 기존 clipboard 접근 실패 보존; 최신 목록 GUI 측정 종료 후 native 실행

## 사용자 결과

정제 보고서 복사를 사용자 clipboard 변경 없이 별도 WindowStation에서 실제 검증할 수 있는지 확인한다.

## 계약과 범위

새 임시 WindowStation/desktop을 현재 사용자 권한으로 생성하여 그 내부에서만 합성 clipboard를 읽고 쓴다. 기존 interactive clipboard 내용은 조회하거나 변경하지 않는다. 기존host/desktop보안정책·관리자권한·전역입력·실제비밀을 사용하지 않는다. capability 실패는 보존하고 사용자환경 fallback 없이 종료한다. 최초 standalone 격리 가능성 시험 뒤에만 최신제품 복사/원diagnostic fixtures 통합을 판단한다.

## 인수 조건

서로 다른 원WinSta HANDLE·privateWinSta identity 및 실제 clipboard write/read 원byte 일치, owned handle/process/desktop의 정상 종료, 기존 사용자WinSta clipboard를 대상으로하는 OpenClipboard/GetClipboardData/EmptyClipboard 호출0. 실제productcopy/reader/IME 완료는 별도다.

## 검증과 handoff

격리 가능성 prototype 착수. 현재 CPU60초×3회GUI 측정과 native 실행을 겹치지 않는다. 원clipboard/desktop 접근 오류를 우회했다고 주장하지 않는다.


### 권한 경계 확인으로 actual 작업 보류

[Microsoft CreateWindowStationW 계약](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createwindowstationw)은 명시적 이름 지정에 Administrators 그룹 조건을 명시한다. 기존 named accessDenied5/unnamed create-only183 실패는 이미 보존돼 있다. 동일 권한 동작을 반복하거나 기존 station을 채택하는 fallback·새 logon·관리자 전환·전역clipboard로 우회하지 않는다. 이 actual 격리 실행만 보류하고 허용된 목록/성능/보고서/공급 개발은 계속한다. 필요한 다음 결정은 별도 승인된 격리 Windows 환경 또는 관리자 범위의 실제 시험 제공이며 현재 자동 승인이 거절한 동작은 아니다. Prototype은 미실행 초안으로만 보관한다.

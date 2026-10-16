# S00: 빈 상태 안내와 확대 화면 포커스 후속

- 상태: in-progress — 빈 상태·로딩·재시도 문구와 도구 중복 제목은 구현; 전체 controls keyboard 인수는 후속
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root 예정 — accessible_library.rs, accessible_tools.rs, accessible.rs, catalog.rs, private GUI observer, DESIGN.md
- 선행 작업: S00-windows-user-ui-rework의 현재 소스 고정 및 새 후보 검증

## 사용자 결과

파일 목록의 조회 중·빈 결과·실패를 구분하고 다음 행동을 안내한다. 좁은 창과 확대 화면에서 고급 입력으로 이동하면 해당 제어가 완전히 보인다. 도구 화면에서 설치 상태와 다음 행동을 먼저 찾는다.

## 계약과 범위

기존 비동기 scan/취소·경로 정책·도구 다운로드 정책·라이선스 필수 안내는 유지한다. UI 문자열은 catalog에서 관리한다. 목록 상태를 새로운 DB schema로 만들지 않는다. 도구 제목 중복과 설명 순서를 개선하되 제한/라이선스 문구를 삭제하지 않는다. shared 인터페이스와 최종 통합은 root 담당이며 같은 파일 병렬 수정은 하지 않는다.

## 인수 조건

합성 TEMP에서 조회 중/실제 빈 결과/경로 실패/비어 있지 않은 결과를 구분한다. private desktop에서 800x600·앱 배율150/200의 고급 옵션·도구 설정·설정 아래 항목에 포커스를 보내 전체 제어와 표시가 보이는지 확인한다. 단순 캡처나 URL/시작 버튼의 가시성만으로 전 controls 키보드 인수로 표시하지 않는다. Windows 전체 검사와 새 고정 binary 회귀를 수행한다. 실제 물리 IME/reader/OS DPI는 별도 사용자 인수다.

## 검증과 handoff

2026-10-07 독립 PNG 검토에서800폭 고급 옵션 버튼의 초기 viewport 하단 일부 잘림, 빈 목록 안내 부재, 도구 제목/설명 중복이 관측되었다. 넓은 화면의 footer·기본 입력 위계 개선과 구별한다. 후속은 아직 구현/검증 전이며 현재 후보의 완료 범위에 넣지 않는다.

2026-10-07 두 번째 UI 피드백의 [폴더 선택·레이아웃 작업](S00-native-folder-picker-layout.md)에서 상태별 안내·경로 browse·중복 도구 제목 제거·기술 설명 disclosure를 통합했다. 좁은 창 focus scroll은 유지한다. 모든 controls의 키보드 접근·물리IME/reader 인수를 현재 layout PNG만으로 완료하지 않는다.

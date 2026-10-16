# S00: 빈 파일 목록의 다음 행동 안내

- 상태: in-progress
- 분류: quality
- 우선순위: P0
- 요구사항: R05, R08
- 담당/수정 범위: 주 담당 crates/app/src/catalog.rs, crates/app/src/accessible_library.rs 및 이 문서
- 선행 작업: 현재 Library 빈 화면 root 픽셀 검토

## 사용자 결과

확정된 빈 목록에서는 폴더 선택·새로고침을 안내하고, 파일이 있어야 사용할 수 있는 미리보기 행은 접는다. 로딩·오류·파일 미선택·선택 상태의 기존 안내와 동작을 유지한다.

## 계약과 범위

중앙 catalog를 사용한다. worker·DB·미리보기 요청·원본 보존 정책과 폴더 선택 버튼은 유지한다. 새 의존성·schema 변경 없음. 기존 최신 설치 ZIP에는 이번 수정이 포함되지 않으며 후속 빌드에서 교체한다.

## 인수 조건

실제 egui 렌더링의 빈 목록·미선택·로딩·오류·선택 5상태에서 안내와 버튼 활성화·Click 결과·원본 rows 보존을 확인한다. Windows 전체4검사와 실제 빈 화면 캡처·문서 검사를 수행한다.

## 검증과 handoff

/tmp 후보 rustfmt와 root 소스 핀 대조를 확인했다. 실제 통합 검증은 진행 중이며 완료로 보고하지 않는다.

Windows whole fmt·clippy·test·release 종료0, 1145/0/30 및 root 원본 로그·소스·artifact 대조 통과. 두 앱 구성의 실제5상태 렌더링 회귀를 통과했다. [whole](../validation/evidence/windows-msvc-library-empty-action-whole-20261012/root-review.json), [23화면 raw/provider·SQLite 검토](../validation/evidence/current-ui-library-empty-scoped-20261012/root-independent-provider-sqlite-review.json). Light800150/Dark960150은 root 직접 확인했다. 초기800200에서는 안내가 viewport 아래라 실제 스크롤 후 glyph 확인을 이어간다. 물리 hover/IME·전체23픽셀은 미검증이다.

최종 스크롤 도달 수집35 및 독립검토 종료0. Light/Dark/재시작×800/960×100/150/200 18개 상태의 empty/help 전체 body bounds를 확인했다. 주 담당이 세 테마 전이 단계의200% 두 폭 안내 캡처6개와 Light100/150 네 화면을 직접 확인했다. [도달 검토](../validation/evidence/current-ui-library-empty-scoped-v3-20261012/root-independent-reach-provider-sqlite-review.json). 기능·변경 영역 검증을 인수했으며 문서 재검사와 새 설치 ZIP 반영은 이어간다. 빈 viewport의 스크롤바 appearance와 페이지 문구 개선은 별도 후속이며 전체 UI/UX 인수로 확대하지 않는다.

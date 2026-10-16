# ADR-0011: 목록 전체 기록의 bounded 검색

- 상태: Proposed, Windows 후보 구현·검증 중
- 날짜: 2026-10-14
- 관련 요구사항: R03, R08

## 문제

파일 목록과 작업 이력에 검색창이 없어 원하는 항목을 찾기 어렵다. 현재 화면의128행만 필터링하면 뒤 페이지의 항목을 놓치지만 사용자는 전체 검색으로 오인할 수 있다.

## 결정 후보

현재 성공 관측 DB의 파일명·경로, 이력의 작업 ID·버전1 StoredOptions에 실제 저장된 URL을 SQL parameter와 literal substring으로 검색한다. 제목을 저장하지 않는 이력에서 제목 검색을 약속하지 않는다. query는 trim한 UTF8 최대256바이트이며 NUL을 거절한다. ASCII 대소문자는 구별하지 않고 다른 Unicode는 SQLite 기본 literal 비교를 사용한다. schema·의존성·lock은 바꾸지 않는다.

검색 결과는 keyset128행씩 기존 단일 worker와 capacity1 mailbox로 전달한다. 검색창은 명시 검색/Enter/지우기를 제공하고 입력 중 매프레임 조회하지 않는다. 표시 중인 applied query와 draft를 구분하고 늦은 이전 요청 결과를 새 검색으로 게시하지 않는다. 실패는 기존 결과·오류를 남기며 빈 결과로 대체하지 않는다.

읽기는 기존 DB를 readonly/no-follow로 열고 생성·migration하지 않는다. SQLite busy_timeout5초는 잠금 대기 한도이며 전체 query의 실행 기한이 아니다. 문자열 부분 검색 비용은 기록 수에 비례할 수 있다. 조회가 끝나기 전 Stop 완료를 주장하거나 UI thread에서 join하지 않는다. 대량 DB의 interrupt/실측은 별도 후속이다.

## 권한과 비용

검색은 읽기 의도이며 현재 파일 존재·열기/삭제/내보내기·작업 재개 권한이 아니다. 기존 실제 실행 worker의 경로·file identity·옵션·작업 상태 재검증을 유지한다. 검색 결과는128행, 요청 payload는256바이트이고 새 상시 worker/네트워크/전체 DB의 UI 메모리 적재는 없다.

## 검증 상태

구현 중이다. 한글·공백·%/_/quote literal·128행 경계·clear·corrupt/oversize/readonly 실패·late generation·키보드·작은 화면을 실제 명령으로 검증한 후 기록한다. 물리 IME/reader/UI 사용자 인수는 단위 시험으로 대신하지 않는다. v25 설치본에는 포함하지 않았다.

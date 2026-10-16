# S00/S03: v28 도움말·검색 private native 검수

- 상태: proposed / prepared, 실행 전
- 분류: bug
- 우선순위: P0
- 요구사항: R04/R05/R08/R10/R11
- 담당/수정 범위: field_reveal 담당은 B/v28-help-search-native-prep-v1 scratch만; Root가 미래 whole28 검토와 실제 native 실행/원증거 대조
- 선행 작업: executable focus group·Escape dismissal·search result reveal 통합, 실제 새 전체 Windows gates와 Root 독립 검토

## 사용자 결과

Escape로 도움말을 닫았을 때 제목뿐 아니라 동일 본문 tooltip도 사라지고 정보 버튼 focus는 유지된다. 실행 파일 정보 버튼 focus는 의미 라벨·입력·찾아보기를 함께 노출한다. 검색 완료 시 빈 안내와 이력 ID 라벨이 비focusable Text 그대로 읽히며 검색 action focus를 빼앗지 않는다.

## 계약과 범위

기존 v27-help-private-v2 / search-private-v8 helper는 원자리 보존하고 해시를 기록했다. 런타임에 whole28 원자료, Root source count/합계, 네 operation 로그/Job, 현재 source snapshot, 실제 release 3개, lock 및 helper를 읽어 before/after 바인딩한다. 미래 hash/count를 미리 기록하지 않는다. 기존 소유 HWND/PID/start/Job/private desktop/timeout/원 fixture 보존 guard는 유지한다. Help FullDescription은 실제 native COM30159 BSTR를 owner RuntimeID로 읽으며 managed default/name fallback이 없다. 환경 Edit은 purpose label과 동일 accessible name으로 유일하게 찾고 browse는 정해진 세 caption으로 찾는다. label/info/Edit/browse 모두 현재 Download의 header/footer 사이 full body bounds를 검사한다.

## 인수 조건

Help Light100/960 1scene, 실제8PNG. Invoke 뒤 제목·본문 Text, Escape 뒤 둘 다 없음+owner focus6개100ms settled 표본을 요구한다. 환경3행은 모두 full body 및 inline 라벨 정렬을 요구한다. Search Light200/800 1scene, 실제7PNG/140합성 파일/저장 DB 유지. 기존 첫 행·match·clear는 기존 focus guard 유지. 새 empty/HistoryID/URL 세 단계는 순수 Text/nonfocusable/full body를32회100ms 내 기다리고 실제 action focus와 label/nav RuntimeID를 보존한다. 이 passive 구간은 SetFocus/Invoke/wheel/키보드 입력을 보내지 않는다. 성공해도 pixel review required이며 물리 hover/IME/reader·OS DPI·전체 UI 인수를 대신하지 않는다.

## 검증과 handoff

준비 단계 Python 전 파일 compile() syntax만 통과했다. collect/run/native/Cargo는 실행하지 않았다. Live/S/docs/evidence/원 helper 변경0. Root 실행은 help/argv-template.json 또는 search/argv-template.json. 원 출력은 fresh exact namespace이며 이미 존재하면 중단한다. Helper runtime 실패는 제품 실패와 설치/transport/setup 실패를 분리하여 보존한다. 같은 원자료를 덮어쓰거나 visible desktop/global input으로 fallback하지 않는다.

Root Help-v3 actual2ops0/8PNG/source674/40pins raw독립검토와 환경3장 직접확인을 마쳤습니다. 환경3행의 label/info/edit/browse 전체bbox는body내부이나 포커스tooltip이일부입력을덮습니다. 별도occlusion완료아님. 원v2는Escape6표본중Boolean실패이나표본이assert이후저장되어실패유형을알수없습니다. 원FAIL은보존하고 v3의새진단을고정2회추가반복해성공할때까지재시도하는판정을하지않습니다. 각실패표본·원창focus·실패직전화면을보존합니다. source/PE/helper불변이며이번반복결과를물리/전체GUI/P0인수로승격하지않습니다.

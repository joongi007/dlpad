# S05: 필요한 도구 설치로 안내하고 원 작업으로 복귀

- 상태: in_progress / 기존 부분 동선 분석·scratch 후보 준비
- 분류: UX feature
- 우선순위: P0 사용자 후속
- 요구사항: R05/R07/R08
- 담당/수정 범위: p0_audit scratch accessible.rs/accessible_tools.rs 후보; Root catalog·공유인터페이스 통합·전체검증
- 선행 작업: source31 검수 후보 보존; 기존 show_tool_recovery/open_tools/도구 캐시·실행 재검증 계약

## 사용자 결과

기능에 필요한 도구가 없으면 이름·필요 이유와 설치하러 가기 행동을 제공한다. 해당 도구의 설정 페이지를 바로 열고 명시 설치 후 원래 화면으로 돌아와 URL·구간·선택 파일을 유지해 이어서 쓴다.

## 계약과 범위

기존 설정의 관리 설치·외부 실행 파일 선택·공급 trust·검증 gate를 재사용한다. 버튼 이동만으로 도구를 자동 설치하거나 유료/인증 작업을 시작하지 않는다. missing/확인실패/무효경로/지원불가를 구별하고 잘못된 문제에 다운로드를 권하지 않는다. Local에는 불필요한 yt-dlp를 요구하지 않는다. DB/schema/lock변경없다.

## 인수 조건

실제 Context 이동·대상도구 선택·원화면복귀와draft유지,설치취소/실패후복귀,이미설치됨/검증중/종료중/지원불가를확인한다. 실행직전재검증은유지한다. UIA/실제OS 검수 범위를 구별한다.

## 검증과 handoff

기존 미리보기 도구 설치 링크는 있으나 전체 필요조건 안내·특정 도구 선택·복귀는 부족하다. 현재 후보 작성 중이며 구현/검증 완료가 아니다. source31 ZIP 예정 후보에 이 추가 요청은 아직 없다.

Root가 후보5code+중앙catalog를 S에 통합하고 실제 Windows `feature_tool_`를 실행했다.9개 중8PASS1FAIL이며 managed hash거절 원문이 실제 Context tree에 없다는 실패를 보존한다. 원 사전분류는 Rejected였으므로 UI lifecycle/조회반영/표시를 조사 중이다. 작업시작·자동설치 없이 draft/로컬intent/이동·복귀/취소cache8회귀는 통과했지만 통합완료가 아니다. Live/패키지31 변경0이다.

후속v2 실제960100/800100은 사유glyph/설정이동/복귀32px 모두 통과했다.800200은 최초가상viewport의사유/action을본뒤Click후panelNone으로실패했다. Root는잠긴response.rs에서RoleLabel은set_value임을직접확인했고기존label조회실패는시험오류로분리했다. 현재200%실패는미해결이며실제viewportseed/settle후원ID/disabled/rect/클릭결과를별도검증한다.

Root v4 실제 전체앱은490PASS/0FAIL/6ignored/필터0이다. 실제200% inner viewport104logical과사유행107logical을직접관측해oversized로구분했고원사유와설정동작각glyph에기존사용자scroll로도달했다.100%는두필수내용동시fullrow를확인했다.200%설정복귀행72logical/clip204에서필수glyph/원URL·구간보존을확인했다. paintclip110을실제viewport로사용하지않았으며성공을위해조건을완화하지않았다. strictclippy/Live통합·현PEprivate·패키지는아직후속이다.

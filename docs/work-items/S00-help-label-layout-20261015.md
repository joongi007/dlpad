# S00: 설명 아이콘과 입력 라벨의 정렬

- 상태: in-progress
- 분류: bugfix
- 우선순위: P0
- 요구사항: R07, R08
- 담당/수정 범위: Root app/accessible.rs; 별도 ui_components.rs 정보 아이콘 작업은 advanced_units 담당
- 선행 작업: 기존 whole-v5 및 실행 중 native/패키지 종료, 원 증거 보존

## 사용자 결과

도움말이 떠 있는 물음표나 독립된 버튼 줄처럼 보이지 않고 설명할 입력 라벨에 붙는다. 선택 설명은 hover/focus와 클릭으로 확인하고 필수 오류는 본문에 유지한다.

## 계약과 범위

다운로드 환경의 yt-dlp, FFmpeg, Deno 도움말을 각 필드 라벨 옆으로 옮긴다. 입력의 명시 ID와 labelled_by, 파일 선택·worker 정책은 유지한다. 버튼의 클릭 영역32px와 접근성 제목·설명·popup 동작은 공통 컴포넌트의 독립 작업을 따른다. DB/schema/dependency 변경은 없다.

## 인수 조건

작은 폭에서 라벨·아이콘이 붙고 입력·찾기 버튼과 겹치지 않는다. 설명만 있는 독립 줄을 없애고 동일한 keyboard/hover 도움말과 명시 입력 ID를 보존한다. 필수 쿠키 보존 안내와 validation은 숨기지 않는다. 실제 Context와 private 캡처를 별도로 확인한다.

## 검증과 handoff

scratch 후보 준비 중이며 Live 미반영이다. 사용자 거절 이전 whole-v5와 v26 패키지 후보를 최신 UI 인수로 사용하지 않는다. 향후 새 source/PE/설치본을 연결해 검증한다.

Root 통합 검증: 원 정보 아이콘 RED는 실제 Context 0 통과/1 실패로 missing vector를 재현했다. 최초 GREEN은 기존 help 2 통과/새 focus 시험1 실패이며 원 증거를 보존했다. Escape 같은 pass 후반의 focus 복원보다 앞선 paint를 검사한 시험 시점을 다음 settled frame으로 교정했다. 새 GREEN-v2는 정보 아이콘·기존 keyboard/pointer 총3개와 입력 레이아웃1개 모두 통과했고 strict Windows workspace/all-targets clippy도 exit0이었다. 각 원로그/Job0/private/input 및 Live/S 불변을 Root가 대조했다. UI 두 파일은 이 결과 뒤 통합했으며 새 whole-v27와 private 시각 검수는 실행 전후를 별도로 기록한다. 실제 사용자·reader 인수는 미완료다.

새 whole-v27 실제 네 명령 exit0, source674, 1,593 통과/0 실패/42 ignored/43 harness. Root가 원로그·현재 소스·3PE·lock을 독립 확인했다. CPU는 10,306,560바이트 SHA1576b78a5f01c78515ed3349a918038baafa973f698ee04bae9fe1a66be6d6f2로 직전 whole-v26-v5보다3,072바이트 늘었다. CLI/설치기 바이트는 같다. 새 dependency/lock/schema는 없고, 도움말마다 벡터3개를 그리며 runtime 전체 비용 절감은 측정하지 않았다. 실제 private Help/Tools/Search/Progress 화면은 실행 중이며 전체 UI 사용자 인수로 승격하지 않는다.

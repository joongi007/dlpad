# S00: 도구 설치 버튼 글자 표시 재현

- 상태: in-progress
- 분류: UX
- 우선순위: P0
- 요구사항: R04, R08
- 담당/수정 범위: root — /tmp 전용 native observer, 실제 evidence, 원인 확인 후 관련 UI source
- 선행 작업: 사용자 설치 버튼 글자 제보, UI/network 통합 빌드

## 사용자 결과

설치 버튼의 글자가 정상적으로 읽히고 focus/hover에서 잘리거나 깨지지 않는다.

## 계약과 범위

소유 임시 data와 private desktop에서만 실행한다. 설치 실행 없이 세 도구의 실제 버튼을 Light100/Dark200/restartDark150에서 관측한다. actual Name·PID/HWND/creation·RuntimeID·geometry와 PNG를 남긴다. 전체 UI/물리 입력/OS DPI 인수로 확대하지 않는다.

## 인수 조건

실제 glyph와 paint clip을 root가 PNG에서 확인한다. 제보가 재현되지 않으면 재현되지 않은 범위를 기록하고 수정 완료로 주장하지 않는다.

## 검증과 handoff

현재 /tmp observer 후보의 Python 구문 검사만 통과했다. native compile/GUI/실제 PNG는 최신 통합 빌드 통과 후 실행한다.

- 최신 UI/network 통합 whole 및 독립 검토 통과 후 실제 private desktop에서 세 설치 버튼의 이름·focus·hover·배율별 PNG 관측을 시작했다. 설치·외부 다운로드는 실행하지 않는다. 아직 픽셀 검수 완료는 아니다.

- native44811의 first/restart 각 실행은 REVIEW_REQUIRED였고 세 도구×3phase×focus/hover18PNG를 생성했지만 최종 원128 media 조건이 다른 시험에서 남아 있어 overallFAIL이었다. root가 실제 Light100 yt-dlp/Dark200 FFmpeg/restartDark150 yt-dlp를 직접 열었으며 누락 glyph는 재현하지 못했다. 내부 흰 focus frame의 과도한 비중과200%에서 도구 탭이 scroll 밖으로 밀리는 UI 문제를 확인했다. 원 실패/픽셀 SHA 보존 후 다음 UI 조정을 진행한다.

- root가 primary의 두꺼운 내부 흰 focus frame을 표면과 분리된1.5px 외곽 accent indicator로 변경했다. 설치 action은 도구 역할·출처 아래에 배치해 확대 화면에서 주 동작으로 이동할 때 선택 탭이 밀려나는 부담을 줄인다. 다운로드/설치 용량·버전/라이선스 정보는 보존하고 설치 worker/권한/실행 의미는 바꾸지 않는다. 후속 관련 회귀·빌드·캡처를 진행한다.

최신 whole1227 고정 Windows private GUI는 exit0, 3테마/배율×3도구의 focused/hovered18PNG를 수집했다. Root가 Light100 yt-dlp focus, Dark200 FFmpeg hover, Dark150 yt-dlp focus를 직접 확인했다. 표본에서 한글·Latin 글리프 누락은 재현되지 않았고 두꺼운 내부 흰 테두리 제거 및 확대 tool tab/action/cost 동시 가시성을 확인했다. 설치/다운로드 실행·전체 픽셀·물리 입력 인수는 이 scope가 아니다.

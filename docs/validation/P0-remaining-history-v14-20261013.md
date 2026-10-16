<!-- Historical v14 snapshot; see current README for accepted candidate. -->
# Windows P0 잔여 작업 — 2026-10-13

현재 v14 whole은 **1,305 성공 / 0 실패 / 30 ignored**이며 source656·43harness·현재3실행 파일을 독립 대조했다. 배율 변경 뒤 최초 늦은 viewport 축소를 기존8frame 공개로 보정하며 사용자의 wheel/press/key/text/IME·페이지 이동은 감시를 해제한다. 실제 Candidate 회귀5개와 격리63화면 검수, 최신 archive·변조 실행 거부·독립27ID가 통과했다. 앱은1,536B 증가하고 설치기·CLI·Cargo·의존성은 그대로다. 설치·실행 내부 native0/Job0와 외부143 종료는 분리하며 외부 원인은 미확인이다. v13 CI·비용·긴 경로 결과는 당시 소스에 한정하며 v14로 소급하지 않는다. 물리 pointer/IME/reader/OS DPI·전체 UI/UX 인수·정식 배포는 미완료다.

[현재 전체 검사](evidence/windows-msvc-scale-viewport-reflow-whole-20261013/root-review.json), [현재63화면](evidence/current-scale-viewport-reflow-app63-20261013/root-team-visual-review.json), [현재 배율 회귀](evidence/windows-msvc-late-footer-scale-green-v3-20261013/root-review.json).

아래 v13 항목은 이전 검증 snapshot이다. 배율 일시 가림의 현재 수정 상태는 위 v14 기록을 따른다.


현재 코드의 Windows fmt·strict clippy·전체 테스트·release는 **1,297 성공 / 0 실패 / 30 ignored**다. 43 harness와 source656의 원로그·소스·3실행 파일을 root가 대조했다. 선택 버튼의 밝은 테마 대비와 설치 완료 제목·다음 행동 안내를 포함한다. [현재 전체 검사](evidence/windows-msvc-installer-completion-copy-whole-20261013/root-review.json).

최신 v13에서 완료 안내·경로 변경·재검증·앱 실행·양쪽 자연 종료, exe 한 바이트 변조 거부·실패 보존, 일반/긴 경로 각각27 native ID를 검증했다. 앱63화면을 직접 검수하고 로컬 CI15단계·7captures·8Cargo와 export를 독립 대조했다. 이 범위를 전체 물리 UI 인수로 확대하지 않는다. S10의 P0 로컬 진단은 최초 릴리스에 포함한다.

| 남은 작업 | 현재 상태 | 완료 조건 |
|---|---|---|
| 최신 설치·앱 UI/UX | v13 설치 전이·실행/실패·63화면 검수 완료, footer 줄바꿈 뒤 배율 조작 일시 가림 추적 | 배율 조작 일시 가림·scroll/focus·물리 pointer/IME/reader, 사용자 최종 평가 |
| 로컬 CI·hosted 전환 | v13 실제15단계·7captures·8Cargo 및 export 독립 대조 완료 | Git/remote 준비 후 hosted 실행; 현재 증거는 로컬/export 당시 snapshot |
| 기본 성능 후속 | v13 reported3평균 목표/메모리 통과, strict18 CPU17성공/1실패 보존·원자료/3DB/3만fixture 독립 대조 | 정확한 인과/기준 환경 및 설치 감독 비용; 평균과strict 판정 구분 |
| upgrade/rollback | 미지원 명령의 거부·원본 보존 완료, 실제 기능 미완료 | typed revision/active 전환·반복 복귀·부분 기록/취소 실패 경로 |
| 물리 Windows 인수 | 합성·격리 증거와 구분 | 한국어 IME·reader·OS DPI·sleep/resume·OneDrive·clean OS |
| 배포 조건 | 개발 설치 후보까지 | 서명·정확한 notice·정식 설치/제거·사용자 최종 평가 |
| Git·hosted CI·updater | 실제 Git 조회 exit128, 기존.git 보존 | 유효 저장소·remote와 CI 연결 후 updater; 공개 배포는 별도 승인 |

Windows 네트워크 수신·송신 관측, 기본 경로, 도구 설치와 P0 로컬 진단은 이미 구현되어 있다. 위 인수의 미완료를 해당 기능 전체의 미구현으로 해석하지 않는다. 전체 P0와 정식 릴리스는 아직 완료하지 않았다.

[앱·설치 현재 검수표](UI-UX-AUDIT-CURRENT-20261013.md), [요구사항별 기록](P0-status.md), [원문을 보존한 이전 snapshot 이력](P0-remaining-history-through-20261013.md). 과거의 ‘현재/최신’ 표현과 수치는 해당 snapshot의 기록이다.

# Windows P0 잔여 작업 — 2026-10-13

검증된 최신 배포 검수 후보는 v18이다. 기록·도구 목록 identity 수정을 포함한다. 키보드 도움말·진단 번호 오류 연결·컴파일 SQL binding을 통합했다. 실제 Settings의 선택 한글 확정 시험에서 새 semantic FAIL을 확인해 수정 중이며 물리 UI/UX·정식 릴리스 인수는 별도다. 전체 P0는 미완료다.

검증된 v18은 Windows1,327/0/33·43harness·source657이며 네 필수 명령을 실제 실행한 증거다. 최신63화면 팀 직접 검수, ZIP·원파일/기록 보존, 정상 설치·앱 실행 및 변조 거부 outer0/private Job0, canonical 현재27ID 독립 대조, Dark200 스크롤 후 다섯 명령의 직접 픽셀 검수가 완료됐다. [whole](evidence/windows-msvc-keyboard-diagnostic-help-whole-20261013/root-review.json), [현재 검수표](UI-UX-AUDIT-CURRENT-20261013.md).

| 남은 작업 | 현재 상태 | 완료 조건 |
|---|---|---|
| 추가 UI 안정성 (P2) | 목록 identity 회귀·통합은 완료; 도움말/진단 번호 연결 통합; 한글 확정 semantic FAIL 후속 진행 | hover/focus 도움말 충돌·진단 오류 association·정확 caret reflow/IME commit 실제 검증 |
| UI/UX 최종 인수 | v18 합성 격리63화면·설치8프레임 직접 검수; 긴 경로 표시/도구 경로 label 후속 | 물리 pointer/IME/reader·OS DPI·사용자 직접 최종 평가 |
| 최신 CI·hosted 전환 | v13 로컬15단계·7captures·8Cargo/export 당시 PASS, v14 외부143/최종기록 부재 미완료 | 최신 소스 로컬 CI 및 유효 Git/remote 이후 hosted 실행 |
| 기본 성능 후속 | v13 reported평균/메모리와 strict18 CPU17통과/1실패 분리·원자료 전수 대조 | 원FAIL 유지, 정확한 인과/기준 환경과 감독 프로세스 비용 |
| 긴 경로 최신 검증 | v13 577/598UTF16 설치·재검증·독립27ID 당시 PASS | 최신 후보의 별도 설치 검증; 긴 경로 앱 실행 미검증 |
| upgrade/rollback | 미지원 요청의 거부/원본 보존 확인, 실제 기능 미완료 | typed revision/active 전환·반복 복귀·부분기록/취소 실패 경로 |
| 물리 Windows·clean OS | 합성·private 증거와 구분 | 한글 IME·reader·OS DPI·sleep/resume·OneDrive·fresh OS |
| 정식 배포 | unsigned 개발 설치 후보 | 서명·정확한 공지/notice·설치/제거·사용자 최종 인수 |
| Git·hosted CI·updater | Git exit128, 기존.git 삭제/재구성 없이 보존 | 유효 저장소/remote/CI 연결 후 updater; 공개 배포는 별도 승인 |

Windows NIC 관측·기본 경로·도구 설치·미리보기/구간 선택·고급 옵션·S10 P0 로컬 진단은 구현되어 있다. S10은 최초 릴리스에 포함한다. 위 미완료 인수를 해당 기능 전체의 미구현으로 해석하지 않는다. 전체 P0나 정식 릴리스 완료는 주장하지 않는다.

[요구사항별 기록](P0-status.md), [이전 snapshot 보존 이력](P0-remaining-history-through-20261013.md), [v13 CI](evidence/current-completion-copy-local-ci7-20261013/root-review.json), [v13 비용](evidence/current-completion-copy-default18-cost-20261013/root-review.json), [v13 긴 경로](evidence/windows-msvc-ui-cross-surface-v13-20261013-long-install/root-review.json). 과거 문서의 “현재/최신”은 그 snapshot 시점에 한정한다. 실패 원인 미확인인 외부143/strict1FAIL을 WSL·앱·observer 원인으로 추정해 확정하지 않는다.


현재18의 새 개선: 실제 키보드 도움말2필터와 진단 번호 실패·수정·비활성5필터를 격리 시험 후 제품에 통합했다. 앱은 v17보다12,800B 증가했고 설치기·CLI·의존성·Cargo.lock은 그대로다. SQL snapshot 변조·누락 회귀3/0 및 CI 안전성 관련42/0도 로컬 통과했고 hosted 실행은 미검증이다. [한글 확정 actual FAIL](evidence/settings-ime-positive-commit-isolated-20261013/root-failure-review.json)은 preview A조Z가 commit A한국어Z로 바뀌지 않은 원자료이며 새 테스트를 준비한 뒤에 발견했다. 현재 검수 성공으로 덮지 않는다.

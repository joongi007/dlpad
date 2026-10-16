<!-- Historical v16 snapshot. Current candidate is linked by README. -->
# Windows P0 잔여 작업 — 2026-10-13

v16 배포 검수 후보와 진행 중인 entity-action identity 수정 소스를 구분한다. 아래 수치는 검증된 v16에 한정하며 후속 소스·빌드에 소급하지 않는다. 전체 P0·전체 물리 UI/UX·정식 릴리스는 미완료다.

검증된 v16은 Windows1,309/0/33·43harness·source656이며 원3검사와 같은소스의 별도 release 복구를 결합한 증거다. 최신63화면 팀 직접 검수, ZIP·원파일/기록 보존, 정상 설치·앱 실행 및 변조 거부 outer0/private Job0, canonical 현재27ID 독립 대조, Dark200 스크롤 후 다섯 명령의 직접 픽셀 검수가 완료됐다. [whole](evidence/windows-msvc-app-filled-focus-whole-recovered-20261013/root-review.json), [현재 검수표](UI-UX-AUDIT-CURRENT-20261013.md).

| 남은 작업 | 현재 상태 | 완료 조건 |
|---|---|---|
| entity-action identity 통합 | v16 이후 실제 RED/GREEN·source 수정 진행 중 | 실제 새 whole/raw·패키지·현재 GUI identity 검증 후 후보 교체 |
| UI/UX 최종 인수 | v16 합성 격리63화면·설치8프레임 직접 검수; 긴 경로 표시/도구 경로 label 후속 | 물리 pointer/IME/reader·OS DPI·사용자 직접 최종 평가 |
| 최신 CI·hosted 전환 | v13 로컬15단계·7captures·8Cargo/export 당시 PASS, v14 외부143/최종기록 부재 미완료 | 최신 소스 로컬 CI 및 유효 Git/remote 이후 hosted 실행 |
| 기본 성능 후속 | v13 reported평균/메모리와 strict18 CPU17통과/1실패 분리·원자료 전수 대조 | 원FAIL 유지, 정확한 인과/기준 환경과 감독 프로세스 비용 |
| 긴 경로 최신 검증 | v13 577/598UTF16 설치·재검증·독립27ID 당시 PASS | 최신 후보의 별도 설치 검증; 긴 경로 앱 실행 미검증 |
| upgrade/rollback | 미지원 요청의 거부/원본 보존 확인, 실제 기능 미완료 | typed revision/active 전환·반복 복귀·부분기록/취소 실패 경로 |
| 물리 Windows·clean OS | 합성·private 증거와 구분 | 한글 IME·reader·OS DPI·sleep/resume·OneDrive·fresh OS |
| 정식 배포 | unsigned 개발 설치 후보 | 서명·정확한 공지/notice·설치/제거·사용자 최종 인수 |
| Git·hosted CI·updater | Git exit128, 기존.git 삭제/재구성 없이 보존 | 유효 저장소/remote/CI 연결 후 updater; 공개 배포는 별도 승인 |

Windows NIC 관측·기본 경로·도구 설치·미리보기/구간 선택·고급 옵션·S10 P0 로컬 진단은 구현되어 있다. S10은 최초 릴리스에 포함한다. 위 미완료 인수를 해당 기능 전체의 미구현으로 해석하지 않는다. 전체 P0나 정식 릴리스 완료는 주장하지 않는다.

[요구사항별 기록](P0-status.md), [이전 snapshot 보존 이력](P0-remaining-history-through-20261013.md), [v13 CI](evidence/current-completion-copy-local-ci7-20261013/root-review.json), [v13 비용](evidence/current-completion-copy-default18-cost-20261013/root-review.json), [v13 긴 경로](evidence/windows-msvc-ui-cross-surface-v13-20261013-long-install/root-review.json). 과거 문서의 “현재/최신”은 그 snapshot 시점에 한정한다. 실패 원인 미확인인 외부143/strict1FAIL을 WSL·앱·observer 원인으로 추정해 확정하지 않는다.

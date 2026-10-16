<!-- Historical v16 snapshot. Current candidate is linked by README. -->
# 앱·설치 UI/UX 검수 — 2026-10-13

v16 배포 검수 후보와 진행 중인 entity-action identity 수정 소스를 구분한다. 아래 수치는 검증된 v16에 한정하며 후속 소스·빌드에 소급하지 않는다. 전체 P0·전체 물리 UI/UX·정식 릴리스는 미완료다.

| 항목 | 실제 확인한 범위 | 남은 인수 |
|---|---|---|
| Windows 코드 검사 | 원3검사+동일소스 release 복구, 1,309/0/33·43harness·source656·현재3실행 파일/lock 독립 대조 | 연속 실행이라고 주장하지 않음; 후속 소스는 새 whole 필요 |
| 앱 화면 | v16 격리63PNG 팀 직접 검수·첫/재실행 자연 종료·원소스/DB 대조 | 물리 입력·사용자 최종 평가 |
| 채워진 버튼 포커스 | native installer와 앱의 실제 RED/GREEN 회귀; private GDI3/0/0 별도 | 실제 OS 포커스/hover·reader 인수 |
| 설치 완료 전이 | 준비→완료→target 변경 준비→원target 재검증 완료; Light100/Dark200 제목·안내 직접 픽셀 | clone LOGFONT fit는 근사 측정, 원HFONT 측정 아님 |
| 확대 설치 명령 | Dark200 owned scroll 후 명령201/202/203/204/212 전체 row/UIA와 실제 픽셀 검수 | 물리 휠/키보드 동작 |
| 설치 후 실행 | v16 outer0·앱/감독 자연0·Job0·원7source/5payload/journal 보존 | resume ACK는 GUI-ready 신호 아님; 실제 물리/clean OS |
| 변조 거부 | owned exe1byte만 변조→Run 거절·Run 비활성/Verify 활성·자식 미생성·설치기 자연1·outer0/Job0 | 실제 upgrade/rollback |
| 최신 파일 | v16 ZIP16,726,709B·selected graph258·선택 notice 누락0, canonical 현재27ID 재조회 | 서명·전체 법적 공지·정식 배포 |
| 긴 경로 | v13 577/598UTF16 설치·재검증·원20objects/6parents·독립27ID 당시 PASS | 최신 v16 소급 인수 없음, 긴 경로 Run 미검증 |
| 성능 | v13 raw18구간·3DB/3만fixture 대조, strict17PASS/1FAIL 유지 | 실패의 정확한 인과/기준 환경·감독 비용 |
| 로컬 CI | v13 당시15단계/7captures/8Cargo 및 export 대조 | v14 CI 외부143/최종기록 부재 미완료, 최신 CI·hosted 전환 |

[현재 whole](evidence/windows-msvc-app-filled-focus-whole-recovered-20261013/root-review.json), [63화면 팀검수](evidence/current-app-filled-focus-app63-20261013/root-team-visual-review.json), [설치·실행](evidence/windows-msvc-ui-cross-surface-v16-20261013-command-happy/root-review.json), [실행 거부](evidence/windows-msvc-ui-cross-surface-v16-20261013-command-negative/root-review.json), [현재27ID](evidence/windows-msvc-ui-cross-surface-v16-20261013-command-independent/root-review.json), [설치 직접 픽셀](evidence/windows-msvc-ui-cross-surface-v16-20261013-command-happy/root-direct-pixel-review.json), [최신 ZIP 검토](evidence/windows-msvc-ui-cross-surface-v16-20261013-package-root-review.json).

기본 저장 경로·네이티브 폴더 선택, 선택 도구 설치/활성화, Windows NIC 수신·송신 관측, 미리보기·구간 선택과 고급 옵션 설명/실행 계획은 구현되어 있다. S10 P0 로컬 진단은 최초 릴리스 범위다. 미완료 인수가 이 기능 전체의 미구현을 뜻하지 않는다.

긴 경로 전체를 동시에 표시하는 방식과 도구 데이터 경로 label은 좁은 UX 후속 검토다. 물리 pointer/IME/reader·OS DPI·고대비·sleep/resume·OneDrive·clean OS는 격리 합성 시험과 구분한다. 원 v1/v2 FAIL과 과거 layout/외부143 기록은 보존하며 최신 성공으로 덮지 않는다. IsOffscreen 미지원 값만으로 실제 잘림을 단정하지 않는다.

현재 entity-action identity 수정은 실제 RED/GREEN 및 통합 whole을 진행 중이다. 이전 layout 실패 이력과 수정 회귀를 구분하며, 새 whole·파일·GUI가 검증되기 전 최신 배포 후보를 바꾸지 않는다. [전체 잔여](P0-remaining-20261011.md), [검수 작업](../work-items/S00-ui-cross-surface-audit-20261013.md).

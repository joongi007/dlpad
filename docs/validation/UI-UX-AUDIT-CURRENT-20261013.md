# 앱·설치 UI/UX 검수 — 2026-10-13

검증된 최신 배포 검수 후보는 v18이다. 기록·도구 목록 identity 수정을 포함한다. 키보드 도움말·진단 번호 오류 연결·컴파일 SQL binding을 통합했다. 실제 Settings의 선택 한글 확정 시험에서 새 semantic FAIL을 확인해 수정 중이며 물리 UI/UX·정식 릴리스 인수는 별도다. 전체 P0는 미완료다.

| 항목 | 실제 확인한 범위 | 남은 인수 |
|---|---|---|
| Windows 코드 검사 | 네 명령 실제 실행, 1,327/0/33·43harness·source657·현재3실행 파일/lock 독립 대조 | 후속 소스는 새 whole 필요 |
| 목록 버튼 | 실제 B→A 오작동/disabled ID 변화 RED, 수정 후 기록8/tool17 및 metadata32px/800clip PASS | 물리 키보드·reader, nonempty native 행 별도 |
| 앱 화면 | v18 격리63PNG 팀 직접 검수·첫/재실행 자연 종료·원소스/DB 대조 | 물리 입력·사용자 최종 평가 |
| 채워진 버튼 포커스 | native installer와 앱의 실제 RED/GREEN 회귀; private GDI3/0/0 별도 | 실제 OS 포커스/hover·reader 인수 |
| 설치 완료 전이 | 준비→완료→target 변경 준비→원target 재검증 완료; Light100/Dark200 제목·안내 직접 픽셀 | clone LOGFONT fit는 근사 측정, 원HFONT 측정 아님 |
| 확대 설치 명령 | Dark200 owned scroll 후 명령201/202/203/204/212 전체 row/UIA와 실제 픽셀 검수 | 물리 휠/키보드 동작 |
| 설치 후 실행 | v18 outer0·앱/감독 자연0·Job0·원7source/5payload/journal 보존 | resume ACK는 GUI-ready 신호 아님; 실제 물리/clean OS |
| 변조 거부 | owned exe1byte만 변조→Run 거절·Run 비활성/Verify 활성·자식 미생성·설치기 자연1·outer0/Job0 | 실제 upgrade/rollback |
| 최신 파일 | v18 ZIP16,741,557B·selected graph258·선택 notice 누락0, canonical 현재27ID 재조회 | 서명·전체 법적 공지·정식 배포 |
| 긴 경로 | v13 577/598UTF16 설치·재검증·원20objects/6parents·독립27ID 당시 PASS | 최신 v18 소급 인수 없음, 긴 경로 Run 미검증 |
| 성능 | v13 raw18구간·3DB/3만fixture 대조, strict17PASS/1FAIL 유지 | 실패의 정확한 인과/기준 환경·감독 비용 |
| 로컬 CI | v13 당시15단계/7captures/8Cargo 및 export 대조 | v14 CI 외부143/최종기록 부재 미완료, 최신 CI·hosted 전환 |

[현재 whole](evidence/windows-msvc-keyboard-diagnostic-help-whole-20261013/root-review.json), [63화면 팀검수](evidence/current-keyboard-diagnostic-help-app63-20261013/root-team-visual-review.json), [설치·실행](evidence/windows-msvc-ui-cross-surface-v18-20261013-command-happy/root-review.json), [실행 거부](evidence/windows-msvc-ui-cross-surface-v18-20261013-command-negative/root-review.json), [현재27ID](evidence/windows-msvc-ui-cross-surface-v18-20261013-command-independent/root-review.json), [설치 직접 픽셀](evidence/windows-msvc-ui-cross-surface-v18-20261013-command-happy/root-direct-pixel-review.json), [최신 ZIP 검토](evidence/windows-msvc-ui-cross-surface-v18-20261013-package-root-review.json).

기본 저장 경로·네이티브 폴더 선택, 선택 도구 설치/활성화, Windows NIC 수신·송신 관측, 미리보기·구간 선택과 고급 옵션 설명/실행 계획은 구현되어 있다. S10 P0 로컬 진단은 최초 릴리스 범위다. 미완료 인수가 이 기능 전체의 미구현을 뜻하지 않는다.

긴 경로 전체를 동시에 표시하는 방식과 도구 데이터 경로 label은 좁은 UX 후속 검토다. 물리 pointer/IME/reader·OS DPI·고대비·sleep/resume·OneDrive·clean OS는 격리 합성 시험과 구분한다. 원 v1/v2 FAIL과 과거 layout/외부143 기록은 보존하며 최신 성공으로 덮지 않는다. IsOffscreen 미지원 값만으로 실제 잘림을 단정하지 않는다.

목록 identity는 실제 RED/GREEN·whole·최신 파일·GUI를 통합 검증했다. 비어 있는 History/Tools PNG를 행 명령 동작의 증거로 취급하지 않으며 실제 Context 명령 전달/포커스/paint 회귀를 따로 확인했다. 키보드 도움말의 원 누락 및 다른 hover 충돌을 재현·수정해 경로 입력과 도구 찾기에 연결했고, 진단 번호 오류 association 및 컴파일 SQL binding도 통합했다. 도움말 전체 caller·긴 경로 popup·물리 입력은 별도 후속이다. [전체 잔여](P0-remaining-20261011.md), [검수 작업](../work-items/S00-ui-cross-surface-audit-20261013.md).


[통합 전 v17 입력 감사](evidence/current-input-caret-source-audit-20261013/root-review.json)는7종입력8source와당시 전체로그의20기존filter/40PASS를대조했다. 새semantic수정/물리입력완료와구분한다. [목록 회귀](evidence/windows-msvc-entity-action-identities-green-v2-20261013/root-review-v2.json), [키보드 도움말](../work-items/S00-keyboard-context-help-20261013.md), [진단 번호](../work-items/S00-diagnostic-index-error-association-20261013.md), [리소스 binding](../work-items/S11-compiled-resource-source-binding-20261013.md).


현재18의 새 개선: 실제 키보드 도움말2필터와 진단 번호 실패·수정·비활성5필터를 격리 시험 후 제품에 통합했다. 앱은 v17보다12,800B 증가했고 설치기·CLI·의존성·Cargo.lock은 그대로다. SQL snapshot 변조·누락 회귀3/0 및 CI 안전성 관련42/0도 로컬 통과했고 hosted 실행은 미검증이다. [한글 확정 actual FAIL](evidence/settings-ime-positive-commit-isolated-20261013/root-failure-review.json)은 preview A조Z가 commit A한국어Z로 바뀌지 않은 원자료이며 새 테스트를 준비한 뒤에 발견했다. 현재 검수 성공으로 덮지 않는다.


[공통 제어·호버·글꼴 source 전수 inventory](evidence/current18-production-controls-source-audit-20261013/root-review.json)는 actual16파일 및 지원 source pin을 독립 대조했다. 단순 textual callsite89개와 실제 화면 상태를 구분했고, diagnostic/library/options의 남은 hover-only 도움말은 별도 후속 후보로 기록했다.

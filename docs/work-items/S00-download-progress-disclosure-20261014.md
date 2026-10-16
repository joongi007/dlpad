# S00: 다운로드 진행률과 설명의 단계적 표시

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R01, R07, R08
- 담당/수정 범위: Root — engine metrics.rs/download.rs의관측field와template, app accessible.rs/media_workspace.rs/accessible_cpu_main.rs/accessible_main.rs/catalog.rs, CONTEXT/ADR/사용자안내. advanced_units — display_rate.rs/accessible_metrics.rs. windows_auto — ui_components.rs. installer_batch — accessible_tools.rs/tool_update_ui.rs. 각자별도workitem/후보/백업사용.
- 선행 작업: v25 재설치인수/현재frame진단원자료검토terminal

## 사용자 결과

다운로드화면에서현재파일전송진행률을바로확인하고읽기좋은B/KB/MB/GB 속도를본다. 선택도움말은? hover/focus/click에서읽고필수오류/필드라벨/다음행동안내는본문에남긴다.

## 계약과 범위

Rust native/프로세스argv/worker만I/O/기존3초관측freshness를유지한다. 전송100%는후처리/최종파일검증성공이아니며current파일범위로표시한다. total미상/무효/만료는fake0%나100%로표시하지않고진행량확인중으로구분한다. estimate는추정으로명시한다. UI관측field추가는DB/저장schema변경아니며정확공유계약은Root가조정한다. decimal속도는현재속도제한설정과동일1000기준. 새의존성/feature/lock없음, 상시idle animation추가없음.

## 인수 조건

known0/중간/전송끝/estimate/unknown/만료/비정상카운터와취소/성공후Reset을구분한다. 800/960·100/200%·Light/Dark에서진행률/행동/상태strip겹침없음을Context와실제privateGUI로검수한다. ?hover/focus/Enter/Space/openclose와필드ID/caret보존을검사한다. 속도·도구작업은독립구현후Root전체4검사와원로그/actualcurrentbinary/설치ZIP를확인한다. 사용자desktop/실제영상/실제Documents는시험하지않는다.

## 검증과 handoff

원v25 설치ZIP는재설치버그만수정하고앱UI는v24와바이트동일이며기존프로그램으로새UI완료를주장하지않는다. currentsource671 frame진단은actual0/rawRoot검토(48pins/10000files/1DB2PNG)후종료. mean0.598319%,diagwindow6CPU/5·6WS실패는보존하며default성능성공을새diag로대체하지않는다. 이제새UI소스수정진행중이며향후whole/GUI/ZIP는아직미실행이다.


2026-10-14 구현 중: engine의 고정 argv 진행률 template에 current downloaded/total/estimated_total/status 관측을 추가했다. UI는 실행 중 Download 행동 영역에 현재 파일 범위의 bar·속도·ETA·?를 표시한다. unknown과 후처리는 퍼센트를 꾸며내지 않고, finished는 최종 파일 확인 완료로 표시하지 않는다. 정확 total이 estimate에 우선한다. UI 설명 버튼·십진 속도 formatter·Tools 상태/cache 후보를 Root 격리 source에서 통합 검증 중이다.

실제 Windows 선택 검사 v1: transfer_progress_tests 3개 통과. app 컴파일은 ui_components test fixture의 새 Help enum 분기 누락 E0004로 실패했다. 원로그를 보존했고 담당자가 누락 분기를 수정한 v2 후보로 새 실행을 시작했다. v1 실패를 통과로 기록하지 않는다. v2 전체 app 테스트는 아직 종료 확인 전이다.

새 사용자 검색 요청은 S03 별도 수직 기능으로 병렬 진행한다. 현재 앱 목록에는 검색창이 없다. 현재 페이지 필터를 전체 검색으로 주장하지 않고 bounded DB 조회를 worker에서 수행한다. catalog/module 등록은 Root 담당이며 schema/lock 변경은 없다. 검색 포함 새 패키지는 아직 생성하지 않았다.


추가 실제 관측: v2 app 전체436PASS/3FAIL/6ignored. 신규 진행률 fixture는 최초 sizing frame에서 취소 버튼 paint를 읽던 설정 오류를 두 번째 정상 layout frame 검사로 고쳤다. v3 진행률 회귀 실제1PASS/0FAIL/40개(viewport/theme/zoom/phase)조합이지만 동시에 Live source를 수정해 source-unchanged guard 실패이며 후보 인수로 승격하지 않는다. v4 도움말 두 회귀 실패 유지, v5 Escape 뒤 own button focus 복귀·reorder/stable ID/키보드 도움말 actualPASS, pointer 회귀는 실패로 유지하고 실제 입력·hit-test 자료를 수집했다. 원 실패/namespace를 덮어쓰지 않는다. 예상 total 근접값도 100%로 반올림하지 않도록 한 자리 소수 퍼센트를 내림 표시하는 추가 회귀를 준비했다. 공통 문서 검사 실제1108/114 notices PASS 이후 수정 내용은 최종 재검사 대상이다.

2026-10-15 v26 실제 private 진행 시험에서 URL 문자열 setter의 원held/fresh readback이 첫 관측에 성공해 기존 무반영 실패가 해결됐음을 좁게 확인했다. 이어서 worker 미출현 실패는 보존했다. 진단 mock은 version/metadata exit0, download 호출0이었으며 Root 별도 native raw-byte probe가 합성 C# metadata의 한글 출력이 UTF-8이 아님을 확인했다(synthetic-metadata-utf8-byte-probe-v1-20261015). 제품 JSON 파서의 검증을 완화하지 않고 합성 도구의 출력 인코딩을 명시하는 새 후보를 준비했다. 실제9phase 진행/취소 인수는 새 실행 전까지 미완료다.

# S04: 수동 도구 경로의 설정 복원

- 상태: complete (수동 설정 저장·복원과 명시 managed Use 범위)
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R07, R08, R13
- 담당/수정 범위: root 공유 인터페이스·통합·검증; timeline_precision은 /tmp에서 기존 settings/store 및 UI 연결 후보만 작성한다. 예상 파일은 crates/app/src/accessible.rs·accessible_settings.rs 및 좁은 tool preference 모듈과 관련 문서다. Cargo/schema/lock 변경은 root만 조정한다.
- 선행 작업: 수동 도구3개 입력 실제 인수·managed active 설치 저장 계약

## 사용자 결과

직접 고른 도구 경로가 앱 재실행 후 유지되고, 자동 탐색으로 돌아가는 행동이 명확해진다. 잘못된 새 입력이나 저장 실패가 기존 유효 설정을 덮어쓰지 않는다.

## 계약과 범위

기존 Rust native·긴 I/O UI thread 분리·argv·외부 도구 보존을 유지한다. 시작 환경 override와 명시적 UI 선택·managed active·PATH의 우선순위를 문서화한다. 사용자 기존 데이터 대신 owned 임시 settings를 시험한다. 이 작업은 실제 버전 검증/도구 설치/삭제의 소유권을 변경하지 않는다.

## 인수 조건

세 도구의 수동/자동 선택과 재실행·환경 override를 확인하고 invalid draft·DB 실패/손상·중복 저장·종료 중 worker ACK 실패 경로를 검증한다. managed Use와 수동 입력 충돌의 동작을 기록한다. 새 저장 성공은 실제 재실행 증거 전 완료로 표시하지 않는다.

## 검증과 handoff

현재 인수에서 수동 값은 재실행 시 초기 환경값으로 돌아가는 사실을 확인했다. 기존 저장 adapter를 감사해 좁은 후보를 작성한다. 아직 구현 완료가 아니다.

## 2026-10-11 실제 통합 시작

root는 accessible.rs·accessible_tool_preferences.rs·accessible_cpu_main.rs·accessible_main.rs·catalog.rs를 담당한다. release_preparation은 /tmp 후보만 작성하여 파일 충돌 없이 종료했다. 기존 theme.sqlite 설정 key를 재사용하며 Cargo/schema/lock 변경은 없다. 직접 편집(빈값 포함)은 Manual로 전환하고, 명시 환경값은 빈값도 복원보다 우선한다. 자동 선택은 앱에서 검증하여 선택한 managed 도구로 돌아간다. GUI의 PATH 탐색 연결은 현재 지원되지 않아 도움말에서 제거했고 별도 후속으로 남긴다. 저장/조회는 lazy worker에서 수행하며 종료는 ACK까지 기다린다. 후보의 rustfmt만 확인했으며 새 Rust/native 인수는 아직 수행 전이다.

## 실제 Windows 통합 회귀

첫 MSVC 시험은 타입 오류 2개로 exit101이었으며 원 로그를 보존했다. root 수정 후 저장 모듈8개(실제 SQLite 쓰기 잠금·원 설정 보존·active 저장 종료 ACK 포함), 관련 UI 모듈127개, CPU 바이너리 전체223passed/0failed/3ignored를 실행했다. source620과 실제 raw byte 수·전체 원 SHA·natural Job0·private desktop close·input desktop 불변을 root 재검증했다. evidence: windows-msvc-manual-tool-preferences-{second,app-first,cpu-first}-20261011의 root-review.json. 전체 workspace와 실제 저장/재실행 GUI 인수는 진행 중이며 아직 기능 완료가 아니다.

## 실제 GUI에서 발견한 Windows prefix 오류

첫 전체검사1049/0/28(43harness)은 통과했지만 실제 새 private GUI에서 preference 조회 실패로 Save가 비활성화됐다. collector의 ElementNotEnabled 및 forced-cleanup 결과를 FAIL로 보존하며 GUI 인수를 통과로 표시하지 않는다. InstanceGuard의 실제 canonical data root는 Windows verbatim disk prefix를 갖는다. 새로운 checked 함수가 prefix만 파일 조회한 것이 원인이며 기존 appearance checked_path와 동등하게 prefix는 조합만 하고 실제 root 및 모든 후속 경로를 검사하도록 수정했다. canonical_windows_data_prefix_allows_worker_load_save_and_restart 실제 Windows 회귀는 수정 전1FAIL/exit101을 확인했다. 후속 GREEN 및 새 전체검사/GUI는 진행 중이다.

두 번째 실제 GUI는 Save의 keyboard focus가 부모 scroll을 움직이지 않아 실패했다. source의 prefix 오류와 달리 설정 읽기는 성공하고 Save는 enabled였다. root가 before/failed PNG 두 장을 직접 확인했고 실제 AccessKit Focus+bounded parent clip 회귀1FAIL→1PASS를 확인했다. 새 컨트롤에 exact ScrollIntoView 및 gained_focus의 부모 reveal을 연결하며 disabled control은 새로운 reveal action을 부여하지 않는다. 회귀를 Focus/ScrollIntoView 두 route로 확대하고 시작 환경 우선 도움말을 보강했다. 세 번째 whole/actual GUI 인수는 진행 중이다. collector의 전체Combo 수3 가정은 품질Combo가 있어 틀렸으므로 도구label association3개만 엄격 검증하도록 /tmp v2에 별도 교정했고 이전 실패를 보존한다.

세 번째 whole1053/0/28은 fmt/clippy/test/release 및 두 reveal route를 통과했다. 실제 privateGUI의 Save→재실행→explicit empty 환경3개 phase는 자연exit0/Job0와 actualSaved status/readback을 확인했다. Auto phase는 provider association 수집 가정으로 실패했고 이전 자료를 보존한다. 실제 adapter에서 ComboBox.LabeledBy는 null이고 labelled_by가 현재 Name(yt-dlp/FFmpeg·ffprobe/Deno)으로 resolve됐다. /tmp v3는 관측된 유일 이름으로 엄격 연결하며 다른 품질combo·추측순서를 쓰지 않는다. root가 Saved PNG를 직접 보고 콤보 왼쪽 정렬이 서로 다른 것을 발견하여 공통label112/콤보160/높이32 native row로 정돈했다. 전체5단계 GUI 인수와 새 whole은 아직 진행 중이다. 독립 읽기 리뷰 /tmp/dlpad-tool-preferences-review.md는 확정 수정필수 결함을 찾지 않았고 actual queued controller 연속 저장은 후속 검증으로 남겼다.

## 네 번째 whole와 실제 GUI 관측

최신 whole fmt/clippy/test/release 1,053 passed/0 failed/28 ignored, root source620·raw Job0·원 SHA 검증 통과. actual GUI save/reopen/empty 3 phase는 정상 완료했고 보존한 실제 SQLite 각각 integrity ok 및 저장값을 root 조회했다. auto phase는 자동 선택 action을 찾지 못해 실패했으며 전체 기능 인수로 표시하지 않는다. 실제 캡처에서 label allocation이 내용 크기로 축소되어 Combo 열 정렬이 남는 문제를 발견하여 후속 수정 중이다. GUI 원 로그·실패·캡처는 보존한다.

## 실제 UI 열 정렬 RED/GREEN

처음 시험은 Label 대신 Combo를 잘못 선택해 geometry 증거가 아니었으며 원 실패 로그를 보존했다. 정확한 AccessKit Label 역할·labelled_by ID·ComboBox 역할로 교정한 real-red는 실제 left edges 48.59375/110.125/46.5625로 실패했다. 레이블 child minimum width112를 고정한 green은 실제 Controller::show의 320/480pt 부모에서 3 Combo 열·폭·수직 중심·행 겹침·부모 경계 검사 1 passed/0 failed. root raw/source/natural Job0를 대조했다. 현재 수정 whole 재검증 중이며 이전 graph는 이전 snapshot 인수로 보존한다.

## 최신 whole 및 private GUI 5단계 root 인수

whole-fifth: fmt/clippy/test/release 1,055 passed/0 failed/28 ignored(43 harness), 원 source620/로그/Job0/SHA root 검증 통과. CPU9,625,600B SHA3d7c4f5f69b4f8857b13c80c6f8e92b538a616060c8c92513299da6c0f627c78. 기존 diagnostic CPU 대비84,992B 증가하며 새 Cargo 의존성·lock/schema 변경 없음; worker는 lazy/용량1 channel/active1+queued1 정책이다. GUI-fifth actual save/reopen/empty/auto/reopen-auto 모두 자연exit0/원Jobactive0·MTA/handles/private desktop/input desktop 검증, 각 실제 closed SQLite integrity ok 및 정확한 Manual/Automatic 행 확인. root Save·Auto PNG 직접 열어 열 정렬을 확인했다. Auto의 provider Invoke 후 popup이 열린 캡처는 물리 포인터 닫힘 인수가 아니다. 물리IME/실제managed cache 충돌 GUI는 미검증이며 controller 실제 상태회귀 범위와 구별한다. 현재 설치 ZIP 결합 검증 중이다.

## 세 경로 동시 실제 인수

windows-tool-preferences-three-paths: 세 distinct synthetic executable path를 GUI에서 Manual Save→reopen→yt-dlp emptyenv→yt-dlp만AutoSave→reopenAuto 전체5phase/naturalexit0 Job0 통과. root가 원source620/원SHA/raw와closedSQLite5 integrity·정확한3Manual 및 FFmpeg/Deno 기존값 보존을 독립검토했다. managedUse/cache 충돌 actual은 다음 synthetic 표준ToolManager fixture로 검증 중이다. 기존 Windows managed download example을 source620/locked offline MSVC build만 수행했다(제품 source/Cargo 변경 없음). 최신idle비용은8시간 실험과 동시probe가 없는 조건을 확보한 뒤 확인하며 이전binary값을소급하지 않는다.

관리 충돌 GUI 첫 실행: 표준 existing Windows managed example 실제 synthetic Version/Metadata/Download·finish-and-reuse를 수행했고 새 originalownedmanager/data를 보존했다. ManualSave와restart는 통과했으나 fixed header의 도구 tab y24를 content-only viewport top40 조건에 잘못 적용한 collector scroll에서 실패했다. 실제Scroll/tree로그로 collector 문제를 확인하여 header-specific exact UIA navigation을 교정한다. product 버그/전체managed 인수로표시하지않으며 원 fixture·실패 로그를보존한다.

## 관리 선택 인수의 관찰 코드 교정

두 번째 실행은 C# 미초기화 Rect 컴파일 실패, 세 번째는 tab Invoke 후 실제 페이지 전환 미확인으로 실패했다. 네 번째는 exact header focus/selection과 실제 페이지 요소 확인으로 도구 화면 전환을 확인했으나 inventory 없이 Use action을 찾지 못했다. 도구 화면의 명시적 설치 목록 갱신을 포함하는 후속 관찰을 준비한다. 실패 자료와 원본 synthetic manager를 보존하며 제품 결함이나 성공으로 단정하지 않는다.

## 실제 managed Use 충돌 인수 통과

windows-tool-preferences-managed-conflict-seventh-20261011/root-review.json PASS_SCOPED. 표준ToolManager의 원본 synthetic 활성2026.09.30 경로를 파일handle의 canonicalname/nativeidentity에 고정하고 실제GUI4단계 Save/reopen/Use/reopen-managed 및 closedDB4 integrity·정확3slot을 root검토했다. 명시Use는yt-dlp만Automatic저장하며 FFmpeg/Deno 기존저장Manual을 보존한다. 편집중FFmpeg draft는Use 직후유지되고 재시작하면 원 저장Manual로 돌아온다. 원 manager/activationDB/manifest/tool/sentinelSHA와 nativeIDs·source620·자연Job0 보존을 확인하고 actualrefreshed/재시작PNG를 직접 보았다. 다섯 번째는 canonical표기 비교, 여섯 번째는 .NET4 extendedpath FileStream거절인 collector실패를 보존했다. 제품코드 추가수정 없음. 최신 idle비용은 장시간시험 종료 후 독립 실행할 인수로 남긴다.

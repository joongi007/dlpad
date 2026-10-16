# S00: 진단 보고서의 정보 위계

- 상태: in-progress
- 분류: UX
- 우선순위: P0
- 요구사항: R08, R13
- 담당/수정 범위: release_preparation — /tmp/dlpad-diagnostic-information-hierarchy-20261012/ 후보; root — crates/app/src/accessible_diagnostics.rs, crates/app/src/accessible.rs의 부모 화면 회귀, 중앙 catalog 통합, 이 문서
- 선행 작업: 현재 설치 후보 인수와 시각 감사

## 사용자 결과

실패 보고서를 확인하고 복사·저장하는 기본 흐름을 먼저 읽고, 전문 MPD 구조는 필요할 때 펼친다.

## 계약과 범위

PRODUCT/DESIGN과 GTMUX의 목적별 정보 위계를 따른다. S10 로그 선택·편집·정제·구조 추가·제외·페이지 기능과 privacy 정책을 유지한다. 안정적 widget ID·focus 공개·worker gate와 읽기 전용 미리보기를 보존한다. 공유 schema와 Cargo는 변경하지 않는다. 프로젝트는 root만 수정한다.

## 인수 조건

실제 Panel에서 접힌 고급 구조 설정이 기본 복사/저장 흐름을 가리지 않는다. 펼친 후 기존 모든 조작을 수행할 수 있고 데이터·초안이 보존된다. 실제 Context와 Light/Dark 확대 화면의 공개·스크롤·footer 충돌을 검증한다.

## 검증과 handoff

시각 감사에서 목적이 다른 버튼·전문 입력이 평면적으로 경쟁하는 문제를 확인했다. release_preparation에 /tmp 후보를 할당했다. 구현·실행·사용자 전체 UI/UX 인수는 아직 완료하지 않았다.

- root가 실제 Panel의 기본 Copy/Save 접근과 MPD 첨부·제외 기본 접힘, 보고서 보존을 검사하는 회귀를 먼저 추가해 RED46971을 실행했다. report worker는 panic 여부와 관계없이 원 stop ACK로 회수한다. 새 UI runtime은 아직 통합하지 않았다. 후보는 rustfmt parsing만 통과했으며 root가 실제 compile/Context/native를 수행한다.

- RED46971 exit101: 실제 Panel에 전문 ATTACH_MPD 버튼1개가 기본 상태에 노출되어0개 요구에 실패했다. 원 로그/source 불변·소유 Job 자연 종료를 확인했다. root가 후보 production prefix만 통합하고 원 테스트와 새 회귀를 보존했다. format/copy→readonly preview·페이지→저장→로그 편집→접힌 선택적 MPD 구조로 재배치했다. 같은 frame에서 Save/Attach가 report를 worker에 넘긴 뒤에는 report 존재를 다시 확인해 나머지 변경 조작을 비활성화한다. 중앙 catalog에 선택적 MPD disclosure 문구를 추가했다. 새 의존성/schema/Cargo 변경은 없다. GREEN86454 실행 중이며 기존 전체 진단 회귀·ID/키보드·확대 native 인수는 아직 남는다.

- 새 기본흐름 GREEN86454 exit0: 실제 accessible/CPU 두 harness에서 기본 Copy/Save와 전문 MPD 접힘·원 report 보존 회귀 통과. 전체 진단Context7191은14성공/4실패: 기존3개 입력 회귀가 기본 접힘에서2개를 관측했고 MPD 모두 제외 기본 노출을 기대한 버튼 회귀가 실패했다. 전체 통과로 기록하지 않는다. release_preparation에 실제 disclosure Expand 후 기존 고급 입력/포커스/disabled 인수를 유지하고 숨겨진 ID·보고서/초안 보존을 검증하는 /tmp 후속 후보를 할당했다.

- diagnostic_inputs에 원 S10 normal Metadata7·두 export·readonly/DB/privacy/소유 lifecycle을 유지하는 /tmp native observer 후보를 할당했다. 기본 MPD 접힘은 별도 관측하고 기존 MPD 작업은 실제 header Toggle/Invoke→fresh expanded controls로 계속 검사한다. root가 최신 whole 완료 후 실제 실행·PNG를 검증한다. 현재 사용자 설치 ZIP에는 이 새 진단 재배치가 아직 없다.

- root가 tests-only 후속 후보를 통합했다. 기존 고급 입력3개와 MPD 제외 버튼 검증 전에 actual header ActionExpand→fresh frames→실제 첨부 버튼 도달을 검사한다. 입력은 행 순서 대신 실제 labelled_by의 유일한 의미 관계로 식별하여32px/정렬/limits/disabled/report 조건을 유지한다. 새 actual Expand/Collapse·편집 초안·숨겨진 MPD ID의 Focus 무변경과 export/editor ID 보존 회귀를 추가했다. Context-v2 실행 중이며 모델 state를 직접 open시키거나 원 실패 기준을 삭제하지 않는다.

- Context-v2 91860 exit101:19개 중15개 통과/4개 실패. 실제 Expand 후 MPD 버튼 회귀는 통과했다. 의미별 입력 helper가 Label.label을 읽어0개를 관측한 것을 확인했다. 잠긴 vendor/egui/src/response.rs의 Role::Label은 set_value를 사용하므로 root가 정확한 labelled_by Node ID·Role::Label·value를 대조하도록 수정했다. 임의 좌표/문구 추측 fallback을 추가하지 않고 원 geometry/입력/ID 기준을 유지했다. Context-v3 78947 실행 중이다.

- Context-v3 78947 exit0: 실제 accessible/CPU 두 harness에서 진단19개 회귀 모두 통과했다. actual header Expand 후 기존3개 입력의 테마/배율/32px·glyph·limits·disabled·focus/scroll, 기본 MPD 접힘, disclosure 전후 export/editor ID·편집 초안 보존과 숨겨진 MPD ID의 Focus 무변경을 확인했다. source 불변·소유 Job 자연 종료. 현재 통합 Windows 전체를 시작하며 native 확대 화면 인수는 그 다음이다.

- 첫 전체32971은 fmt/clippy 통과 후 test 실패로 중단됐다. actual_candidate_diagnostic_field_reveal_paints_full_input_in_parent_clip이 기본 접힌 MPD 입력을 바로 찾던 기대에서 실패했다. 전체 성공/릴리스 빌드 완료로 세지 않는다. root가 해당 MPD 경로에만 실제 header Expand와 fresh animation frames를 추가했고 원 Focus/ScrollIntoView 한 번 요청·전체 background/glyph clip 기준은 유지했다. 부모 Context12729를 실행 중이다.

- 부모 Context12729 exit0: accessible/CPU 두 harness에서 실제 Candidate 안의 LOG_INDEX/MPD_PATH/EXPORT_PATH 각각 Focus/ScrollIntoView로 전 background/glyph가 부모 clip 안에 들어오는 기존 회귀를 통과했다. source 불변·소유 Job 자연 종료. 전체-v2를 시작했으며 native 확대 화면·실제 MPD 첨부/제외는 다음이다.

- 전체-v2 69789 및 독립 대조31499 exit0: fmt/clippy/test/release 통과,43 harness/1,171 성공/0 실패/30 ignored. current source·원 raw 로그·GNU 보존과 실행 파일 SHA/크기를 독립 대조했다. CPU9,716,224B SHA956341f3681d7784339bcf7f3587cda7e7a52eff98a3d1dd98f283cd0e3ad45a(직전보다1,536B 증가), 설치 companion978,432B와 installer4,311,552B. 새로운 runtime 의존성은 없다. native Light89166/Dark96417을 시작했고 기본 위계·포커스·보고서 보존의 실제 PNG 검토가 남는다.

- native Light89166/Dark96417과 독립74046/83921 exit0. 원6phase·보고서 bytes/privacy/DB/lifecycle 검토는 통과했다. 그러나 root가 직접 본 default 양테마 PNG는 열린 진단 body가 viewport 아래에 있어 기본 조작이 보이지 않는다. root-scoped-pixel-review.json은 basic_diagnostic_controls_visible=false로 기록했다. 전체 native 시각 인수로 세지 않고 S00-diagnostic-open-context에서 부모 disclosure의 한 번 공개를 보강한다. MPD transaction 실제 native 인수도 별도로 남는다.

# S00: 키보드 도움말·진단 입력 오류 통합

- 상태: completed (scoped)
- 분류: UX
- 우선순위: P2
- 요구사항: R06, R08, R13
- 담당/수정 범위: root — ui_components.rs, path_picker.rs, accessible.rs, accessible_diagnostics.rs, 관련 문서 및 전체 Windows 검증; advanced_units — /tmp 공통 도움말 후보; installer_batch — /tmp 새 검증 collector 준비; windows_auto — /tmp 별도 입력 감사 후보
- 선행 작업: v17 설치·앱 검수, 별도 소유 export의 tooltip 2필터 및 diagnostic 5필터 실제 Windows 통과·원본 보존 검토

## 사용자 결과

키보드로 경로 입력과 도구 찾기 버튼에 접근해도 선택적 도움말을 읽는다. 다른 버튼을 마우스로 가리키면 해당 버튼 도움말이 우선한다. 진단 로그 번호가 잘못되어 작업이 거절되면 해당 입력에 오류를 연결하고 올바르게 수정하면 그 오류만 해제한다.

## 계약과 범위

기존 Response/ID/선택·커서/비활성 정책, 필수 인라인 오류, 실제 파일·worker·DB 동작을 유지한다. 공유 도움말은 focused/nonhover에서만 최대16개 hover ID를 복사하고 Context lock을 해제한 뒤 응답을 조회한다. 초과 시 native hover에 맡긴다. 새 thread/timer/I/O/의존성/Cargo 변경은 없다. SQL 리소스 검증 binding은 S11 별도 작업의 root 담당이다. 사용자 데스크톱 자동시험은 금지한다.

## 인수 조건

원 RED와 후보 GREEN을 보존한다. 제품 caller를 통합한 뒤 새 source domain으로 fmt/clippy/workspace tests/release를 실행한다. 설치 payload와 private desktop 앱 화면을 새 source/PE에 고정해 검수한다. 물리 키보드·IME·reader·OS DPI와 actual Context 이벤트 검증을 구분한다.

## 검증과 handoff

v17은 이미 1319/0/33, 앱63화면 및 설치 정상·변조거절 검수를 마친 별도 후보다. 새 기능은 통합 중이며 v17 증거를 새 source에 재사용하지 않는다. 모델 용량 오류로 초기 병렬 재개 3개가 실패했고 공유 통합은 root가 이어서 수행한다. 독립 후보 준비는 재시도한다.


포맷 후보 준비에서 path_picker의 folder_picker.rs와 folder_picker_worker.rs를 각각 누락해 실제 native exit1 두 번을 보존했다. 이는 setup 실패이며 제품 회귀 GREEN이 아니다. 두 모듈을 모두 갖춘 v3 포맷은 actual native0/Job0/private desktop/source unchanged로 완료한 뒤 포맷된 네 파일을 채택했다. 새 Windows whole 검증을 실행 중이다.


v18 통합 인수: 네 Windows 실제 명령 1327/0/33·43harness·source657, 실제 현재3PE·lock·원로그 독립 대조, 새 ZIP·정상 설치/앱 자연 종료·변조 거부·canonical27ID·앱63 및 설치8 PNG 직접 검수를 완료했다. 공유 Cargo/schema/의존성 변경은 없다. [현재 whole](../validation/evidence/windows-msvc-keyboard-diagnostic-help-whole-20261013/root-review.json). 추가 Settings distinct-commit semantic FAIL은 별도 후속 작업이며 이 scoped 성공으로 덮지 않는다. 전체 P0·물리 IME/reader/OS DPI·full UI 인수는 미완료다.

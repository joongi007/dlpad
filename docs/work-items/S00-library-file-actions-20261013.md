# S00: 파일 목록 선택·미리보기·시스템 열기

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R02, R08, R11
- 담당/수정 범위: advanced_units — accessible_library.rs, 새 library_open.rs 후보; root — Main/catalog 통합
- 선행 작업: 사용자 v20 피드백; core 공유 schema는 root 조정

## 사용자 결과

더블클릭/Enter는 같은 미리보기 gate. 선택 파일 기본앱 열기와 폴더 열기는 명시적 행동으로 제공하고 실제 원본/경로를 worker에서 재검증한다. 자동시험은 OS 기본 desktop으로 실행하지 않는다.

## 계약과 범위

후보만 독립 namespace에 수정하고 root가 제품 소스와 공유 catalog/schema/worker를 통합한다. 기존 데이터 삭제·덮어쓰기/visible desktop 자동 GUI/공개 배포는 포함하지 않는다.

## 인수 조건

더블클릭/Enter는 같은 미리보기 gate. 선택 파일 기본앱 열기와 폴더 열기는 명시적 행동으로 제공하고 실제 원본/경로를 worker에서 재검증한다. 자동시험은 OS 기본 desktop으로 실행하지 않는다.

## 검증과 handoff

실제 실행과 미실행을 구분해 기록한다. 계획을 완료로 보고하지 않는다.

## 2026-10-14 주 담당 실제 회귀 검토

`managed-tool-isolated-feedback-library-green-20261014`: Windows MSVC 소유 private desktop/Job에서 native helper 4개, double-click 1개, 시스템 열기 명령 1개, 원 row snapshot/worker ACK 1개 총7/0. 주 담당이 raw terminal guard, 필터별 nonzero 실행 수, 소스·후보 불변과 로그 hash를 대조했다. 기본 앱 실제 실행과 물리 GUI 인수는 아니다.

v2 선택 상세 정보와 v4 Windows NOASYNC를 함께 검사한 `managed-tool-isolated-feedback-library-v2-20261014`는 helper4/0, 목록38/2 FAIL. 선택 이후17px 행 이동과 안내 누락을 원자료로 보존했고 기존 assertion을 완화하지 않은 Root 후속 후보에서 수정 중이다. v4는 메시지 pump 없는 worker에서 ShellExecuteEx의 NOASYNC 요구를 따른다. OS 호출10초는 호출 전후 deadline 검사이며 hard wall cancellation이 아니다. 사용자 player/Explorer를 종료하지 않는다.

후속 `managed-tool-isolated-feedback-library-v3-20261014`는 기존 실패 assertion을 유지한 목록40/0+helper4/0. strict let-chain/Report명 정리 뒤 재검사40/0+4/0 및 all-target strict clippy를 통과하고 v21 개발 소스에 통합했다. fresh whole와 새 native 화면 인수는 진행 중이다.

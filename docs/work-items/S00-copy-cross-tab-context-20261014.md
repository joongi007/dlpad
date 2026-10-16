# S00: 복사 진행과 화면 전환 UI 검증

- 상태: in-progress
- 분류: verification
- 우선순위: P0
- 요구사항: R02, R08, R11
- 담당/수정 범위: windows_auto — 독립 시험 후보; root — accessible.rs/accessible_library.rs의 cfg(test)·통합·원로그 검증
- 선행 작업: S04-library-named-copy-export-20261013

## 사용자 결과

다운로드·설정·파일 목록으로 이동해도 복사 준비/중단 상태를 확인한다. Stop은 worker ACK까지 완료가 아니며 완료 알림을 한 번 소비해도 자세한 실패 결과를 보존한다.

## 계약과 범위

기존 actual Candidate와 cap1 Library command/terminal을 사용한다. 새 detached thread·Shell·외부 원본·schema·renderer를 추가하지 않는다. 준비/중단 transport는 fixture이며 실제 파일 복사/진행률 퍼센트 인수와 구분한다.

## 인수 조건

800/960px·100/200%·Light/Dark에서 실제 Context·CPU raster·현재 root-pruned AccessKit을 검사한다. 탭 전환·32px Stop 노출·반복 disabled 동작·잘못된 serial 거부·실패 stage facts 유지·CloseWait와 worker ACK 구분을 확인한다. 실제 GUI/OS/물리 포인터 시험으로 대체하지 않는다.

## 검증과 handoff

green-v4에서 CloseWait1PASS/matrix1FAIL로 busy Refresh의 UI 상태를 발견했고 원자료를 보존했다. backend는 bounded pending을 유지했으나 UI Refresh를 busy 중 비활성화했다. green-v5/v6/v7의 두 actual copy Context 시험은 각각2PASS0FAIL이며 Root 전체 Main/Library/Settings·strict와 독립 원자료 검토가 남아 있다. 현재 제품 설치본에는 미통합이다.

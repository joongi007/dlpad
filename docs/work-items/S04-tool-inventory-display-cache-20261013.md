# S04: 도구 목록 캐시 표시와 행동 재검증

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R02, R08, R11
- 담당/수정 범위: windows_auto — accessible_tools.rs 캐시 후보; root — update 후보/현재 copy 병합
- 선행 작업: 사용자 v20 피드백; core 공유 schema는 root 조정

## 사용자 결과

탭 전환 시 도구별 최근 표시 캐시를 즉시 보여주고 갱신중/실패/오래됨을 드러낸다. 설치·사용·제거는 exact 실제파일/버전/취소/CloseWait를 worker에서 다시 확인하며 cachedpath/row가 실행 권한이 되지 않는다.

## 계약과 범위

후보만 독립 namespace에 수정하고 root가 제품 소스와 공유 catalog/schema/worker를 통합한다. 기존 데이터 삭제·덮어쓰기/visible desktop 자동 GUI/공개 배포는 포함하지 않는다.

## 인수 조건

탭 전환 시 도구별 최근 표시 캐시를 즉시 보여주고 갱신중/실패/오래됨을 드러낸다. 설치·사용·제거는 exact 실제파일/버전/취소/CloseWait를 worker에서 다시 확인하며 cachedpath/row가 실행 권한이 되지 않는다.

## 검증과 handoff

실제 실행과 미실행을 구분해 기록한다. 계획을 완료로 보고하지 않는다.

## 2026-10-14 실제 격리 검토

`managed-tool-isolated-feedback-cache-green-v2-20261014`는 실제 Context 캐시 즉시 표시·조회 실패 유지/재정렬된 entity 조작2개, 탭 이동 뒤 origin 결과와 stale ACK 거부1개, 연결 끊긴 worker의 정확한 슬롯 실패1개 총4/0. 주 담당이 각 raw terminal, nonzero 필터 수, 소스·후보 불변과 로그 hash를 독립 대조했다. 캐시 후보 시험이며 제품 통합·전체 Windows·물리 GUI 완료가 아니다. 이전 helper namespace 컴파일 실패 기록은 유지한다.

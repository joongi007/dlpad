# S04: 영상 제목과 이해 가능한 파일명

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R02, R08, R11
- 담당/수정 범위: installer_batch — download naming/publication 후보; root — Options schema/UI/통합
- 선행 작업: 사용자 v20 피드백; core 공유 schema는 root 조정

## 사용자 결과

새 작업의 제목 기반 파일명을 지원하고 기존 작업/ID staging/실행 핀/취소/복구를 유지한다. 충돌 시 기존 파일을 덮어쓰지 않으며 Windows 예약 이름/잘못된 문자/한글/긴 경로/동명 영상/자막을 검사한다.

## 계약과 범위

후보만 독립 namespace에 수정하고 root가 제품 소스와 공유 catalog/schema/worker를 통합한다. 기존 데이터 삭제·덮어쓰기/visible desktop 자동 GUI/공개 배포는 포함하지 않는다.

## 인수 조건

새 작업의 제목 기반 파일명을 지원하고 기존 작업/ID staging/실행 핀/취소/복구를 유지한다. 충돌 시 기존 파일을 덮어쓰지 않으며 Windows 예약 이름/잘못된 문자/한글/긴 경로/동명 영상/자막을 검사한다.

## 검증과 handoff

2026-10-14: Root 격리 소스에서 Core optional FinalNameStyle/StoredOptions, engine execute/resume naming snapshot/publication, UI 기본 체크를 통합했다. feedback-naming-v2에서 Core1+readable9+naming5가 실제15/0으로 통과했고 Root 원로그/guard/소스 핀을 대조했다. feedback-copy-ui-v1의 실제 egui checkbox 두 시험도2/0이다. ID staging과 legacy JSON/argv는 보존한다. 영상 제목 뒤 full job ID로 이름을 구별하고 Windows 예약 문자/길이를 제한한다. 다운로드 시작 전 임의 사용자 파일명 입력은 아직 포함하지 않으며 이름 지정은 P1 복사 내보내기에서 제공한다. 현재 제품 소스/설치본에 새 naming 후보는 미통합이며 전체 Windows/시각 인수는 남아 있다. 실패한 strict 원자료도 보존하고 publication matches 문법만 동등하게 수정한 feedback-copy-strict-v2의 종료0을 별도 보존한다.

2026-10-14 현재 통합 상태: 후보는 제품 source669/v22 설치본에 포함되었다. Windows전체네명령1,524/0/42와73화면/정상·긴경로설치 Root검토를 통과했다. 앞 문단의 미통합 표현은 당시 격리 후보의 기록이다. 제목다운로드·이름지정복사 공개API 실제실행은 [별도 합성시험](S02-windows-v22-title-copy-api-smoke-20261014.md)에서계속검증한다. 물리picker/실영상/늦은취소·전체사용자인수는별도다.

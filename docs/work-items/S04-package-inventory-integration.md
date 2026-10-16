# S04: 관리 archive package 목록·명시 선택 통합

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R08
- 담당/수정 범위: root — tools.rs/selection.rs/공유 계약; metrics_recovery — package_inventory.rs/전용 작업 문서
- 선행 작업: bounded archive extraction 전체 회귀 PASS

## 사용자 결과

다중 component package를 하나의 검증·선택 단위로 조회한다. 실행 파일 이외의 auxiliary/notice가 바뀌면 같은 package를 정상 선택하지 않는다.

## 계약과 범위

InstalledPackage{package,directory,main:InstalledTool}. archive-package.json와 기존 manifest.json은 일치하는 main Executable을 표현한다. exactcomponents+2metadata allowlist를 bounded 검증한다. install은inactive이며 select 때 기존active DB를 원자전환한다. 새schema/Cargo는root만 조정하며 같은 파일 동시수정은 하지 않는다.

## 인수 조건

정상 다중 component 및 변조/외부 파일/누락/unknownschema/한도/불일치에서 기존active 유지. root가 workspace와 실제 filesystem fixture로 독립 확인한다. 설치·native삭제·GUI전체는 후속이다.

## 검증과 handoff

구현 및 실패 인수 중. 기존 archive 추출 PASS를 선택/삭제 흐름 완료로 계산하지 않는다.

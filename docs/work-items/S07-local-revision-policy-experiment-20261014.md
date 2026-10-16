# S07: 로컬 설치 revision 정책 실험

- 상태: done (순수 정책 실험 범위)
- 분류: architecture-experiment
- 우선순위: P0 설치 전환 준비
- 요구사항: R08, R11
- 담당/수정 범위: windows_auto — 분리된 revision_schema/revision_policy 및 시험 bridge; root — 공유 계약 검토·실행·통합 판단·문서
- 선행 작업: 현재 설치·등록·active selection의 원 신원 검사 API, v22 전체 검사와 정확한 소유 소스 복사본

## 사용자 결과

기존 설치의 업데이트·되돌리기에서 이전 설치와 기록을 잃지 않는 상태 전이 계약을 검증한다. 이번 실험은 설치를 실제 전환하는 기능을 제공하지 않는다.

## 계약과 범위

기존 active selection v1과 fresh 설치 journal을 변경하지 않는다. Proposed 순수 정책은 generation/predecessor와 원 설치·대상 설치의 정확한 binding을 검사한다. 실패·취소·불명 상태에서는 기존 transition 자료를 반환하며 삭제나 실행 권한을 만들지 않는다. 두 파일은 독립 시험 workspace에서만 컴파일한다. 원 Cargo.lock·제품 파일·설치된 앱·사용자 데이터는 변경하지 않는다. 기존 serde/serde_json/sha2와 engine의 잠긴 버전을 사용한다. 새 제품 dependency나 런타임 worker는 추가하지 않는다.

## 인수 조건

실제 Windows에서 순수 정책 여섯 시험을 실행하고 Root가 원로그·6/0/0·current whole/source669·원 provider2014와 독립 잠금파일의 버전/체크섬을 검토한다. 알려지지 않은 필드·중복·부분 기록·오래된 head·overflow·취소·잘못된 rollback binding을 거절한다. 시험 통과를 실제 durable commit/launcher/DB downgrade/설치 복구 성공으로 확대하지 않는다.

## 검증과 handoff

Root가 `v22-revision-policy-tests-v2/run_root.py prepare`를 실행 중이다. 이어 setup의 offline generate-lockfile, test의 `cargo test --lib --target x86_64-pc-windows-msvc --locked --offline revision_`를 수행한다. private host의 자연0/Job0 및 원 자료 전후 검사를 유지한다.

제목 다운로드 합성 도구의 CLR 실패는 여섯 순수 정책과 기능적으로 독립이다. 정확한 실패 원자료·원 소스·원Job 종료만 검증하고, 기능 성공으로 재사용하지 않는다. 실제 전환 구현에는 별도 namespace/단일 writer lease/확정 또는 불명 commit/복구/앱 데이터 호환 계약이 필요하다. 앱 updater는 Git/CI 연결 이후라는 사용자 지정 순서를 유지한다.

실제 결과: owned bridge prepare·offline lock setup·Windows test를완료했다.6/0/0,compile11.13초·natural0/Job0를확인했고 Root가 원raw/source669/2,014provider파일·compileSHA와독립lock의모든registry버전/체크섬을대조했다. 원RootCargo.lock은불변이다. [Root 검토](../validation/evidence/current22-revision-policy-test-v2-20261014/root-review.json). 새 제품의dependency/worker/릴리스binary 비용은0이며별도debug시험산출물만생성했다.

첫 bridge prepare는현재config의D:/경로를backslash전용h.winpath에전달하여setupFAIL했다. Root가원driver를보존하고PureWindowsPath로tool/target경로를정규화하는별도driver를작성했다. 첫Root감사도tomllib없는LinuxPython에서setupFAIL했으며기존CargoLock의flat package metadata를읽는추가의존성없는검토로대체했다. 실제시험실패와구분하고두원스크립트를보존했다. 실제durable전환·복구·launcher·DB호환은이순수실험의완료범위에포함하지않는다.

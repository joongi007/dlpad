# S07: 현재 Windows 배포물 notice 누락 해소

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R11
- 담당/수정 범위: root — mapping·통합·검증·문서; soak_recovery_audit — /tmp 현재 lock/cache/notice 누락 감사; 후속 source 수집/검증은 파일 담당을 추가 기록한 뒤 진행
- 선행 작업: 기존 notice inventory와 current Cargo.lock 보존; 현재 native CLI 인수 source freeze 완료 전 제품/scripts 변경 금지

## 사용자 결과

Windows 개발 패키지의 누락 notice를 정확한 source/version에 연결한다. 전체 lock에 있는 package와 현재 Windows CPU normal/build 도달 package를 구분하되 도달하지 않는다는 이유로 누락을 숨기지 않는다.

## 계약과 범위

먼저 scripts/windows_dependency_notice_scope.py, docs/licenses/upstream/manifest.json, 기존 package inventory/evidence와 현재 Cargo.lock 및 소유 개발 cache를 읽어 누락 목록을 좁힌다. 원 cache/외부 설치를 수정·삭제하지 않는다. 정확 version/revision/license source/hash를 확인한 자료만 mapping에 추가하고 서명·공개 배포·법적 완비를 추정하지 않는다. 신규 dependency/lock 변경 없음. source 및 notice 파일 담당과 인수 명령은 감사 결과 이후 확정한다.

## 인수 조건

현재 lock hash와 exact package keys 보존, 원 license bytes/source/revision/size/SHA 연결, duplicate/unknown/middle tamper 실패 회귀, 재현 가능한 두 개발 ZIP/내장폰트/SBOM inventory 비교. 해결하지 못한 누락과 clean OS/서명 인수는 그대로 기록한다.

## 검증과 handoff

현재 읽기 감사 초안 단계. 기존 snapshot package 결과를 현재 CPU 또는 최신 CLI 포함 패키지에 소급하지 않는다. 공개 릴리스는 수행하지 않는다.


현재 Windows normal/build 도달 감사: app259 packagekeys, CLI 기본feature87이며 CLI⊆app. 원 Windows 두 tree에서 알려진 wholelock 누락과 도달집합의 교집합0을 확인했다. 전체lock43개 누락은 그대로이며 법적 completeness나 actual linked set을 뜻하지 않는다. 최초 CLI `cli-console`을 enginefeature로 요청한 cargo tree exit101은 원자료로 보존했고 defaultfeatures로 재실행했다. probe 계약 확장 전 tree 원입력을 inputs에 SHA 그대로 보존해 독립 검토는 해당 historicalsnapshot 범위에서 PASS다. 최신 source full 검증은 companion whole proof와 새 ZIP에서 별도로 완료했다. 따라서 notice mapping 추가는 Windows현재도달 누락 해소에 필요하지 않았지만 전체lock43source의 후속 수집은 미완료로 유지한다.

[원tree·보존입력·독립범위](../validation/evidence/windows-current-app-cli-notice-tree-default-20261006-continue/independent-historical-review.json). 현재 mapping28sources/37keys·wholelock inventory526은 변경하지 않았다. offWindows 누락 수집 때문에 Windows-first 구현을 지연하지 않는다.

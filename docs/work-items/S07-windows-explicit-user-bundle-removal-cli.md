# S07: 명시 개발 bundle 제거 CLI

- 상태: DONE (명시 개발 제거 CLI 범위; UI·정식 installer는 후속)
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — 공유 계약·통합·Windows 전체/실제 CLI 최종 검증; metrics_recovery — `/tmp/dlpad-remove-user-bundle-cli-stage/` 파싱·보고 초안. 프로젝트 파일 수정은 root만 수행한다.
- 선행 작업: [제거 엔진](S07-windows-explicit-user-bundle-removal.md)의 Windows 전체·별도 native 인수. 현재 엔진 source는 freeze하고 CLI는 /tmp에서만 준비한다.

## 사용자 결과

개발 명령 `remove-user-bundle`에 bundle과 외부 journal 두 경로를 명시하면 완전한 현재 일치를 검사하고 고정 설치 파일만 제거한다. 기존 15개 명령은 유지한다.

## 계약과 범위

공유 API는 root가 담당하는 explicit removal factory를 사용한다. 수정 예정 파일은 `crates/engine/src/bin/dlpad-local-install.rs`, 개발 CLI 안내와 이 작업 기록이다. 별도 argv parser와 bounded JSON 보고에 현재 disposition 수락·부재 관측·mutation 가능성·반환 전 caps를 분리한다. 취소/기한은 같은 caller 제어를 사용한다. 외부 journal/source/data/media/tools는 보존하고 자동 제거·경로 추측·historical ownership을 제공하지 않는다. 새 dependency/schema/lock 변경은 없다.

## 인수 조건

엄격한 로컬 절대 경로·필수 인자·중복/unknown/UNC/relative 거절과 유한 기한을 검증한다. 정상 제거는 현재 outer 부재를 확인한 뒤에만 성공을 보고한다. factory/remove 실패 단계와 실제 accepted counts·보유 caps를 구별하며 미관측 값은 null로 보고한다. Windows 전체 검증, 실제 TEMP 합성 CLI 성공·취소/기한·foreign 거절 및 원 자료/native ID 대조와 root 독립 통합 검토를 완료한다.

## 검증과 handoff

2026-10-06 /tmp 구현·파싱 초안을 준비했다. 프로젝트 반영·컴파일·실제 CLI 실행은 아직 하지 않았다. 제거 API의 원 cap 획득·부분 retry 인수와 별개로 UI 제거·upgrade·clean OS·정식 installer 및 전체 P0는 남는다.

2026-10-06 통합: root가 원 CLI와 /tmp original byte 일치를 확인하고 검토한 16번째 명령을 반영했다. 중복 journal 옵션 시험도 추가했다. fmt0 후 MSVC전체4검사0, 41 suites841/0/13·기존15와새parser 회귀 PASS다. source591의새정확map, CPU11bb9b…8,710,656B·CLI65e0fe…974,848B·installer65c80d…759,296B를확인했다. CLI는직전engine proof의934,912B보다39,936B(39KiB) 증가했고CPU/installer는같은binary SHA다. 새 dependency/lock 변화없음. 실제collector에서는 삭제전12files/sourceSHA와6mode native 관측, 별도정상otherbundle/journal cold검사와 mixed거절, 기존15 parser 및새negative를실행중이다. 아직실제CLI 인수완료아니다.

2026-10-06 인수: 실제 collector45명령/CLI36행(성공12·예상실패24), root 추가 native20명령(19 readonly query·최대3병렬)을 독립 raw 검토했다. 삭제전 원7files/4roots native 관측과 stage SHA, 정상 outer Windows부재, 원 source/journal nativeID·SHA, foreign/torn/mixed 거절 accepted0과 별도 정상 reference의 canonical/domain/phase binding을 확인했다. 기존15 parser 거절도 실제 검증했다. [전체 검증](../validation/evidence/windows-msvc-bundle-removal-cli-workspace-20261006-continue/root-review.json), [실제 CLI](../validation/evidence/windows-msvc-bundle-removal-cli-real-20261006-continue/root-review.json), [독립 검토](../validation/evidence/windows-msvc-bundle-removal-cli-real-20261006-continue/independent-review.json). 삭제된 metadata 원 canonical bytes의 독립 보관·CLI 실행 중 취소/기한은 이 proof로 인수하지 않았다. 원caps 재시도·실제 disposition 뒤 실패는 엔진 회귀 범위다. 논리적 pending ReadRoot 수가 전체 OS HANDLE 수로 오해되지 않도록 다음 UI 통합에서 unpublished API/JSON 이름을 명확히 하고 전체 검증을 재실행한다.

# S11: Windows CI 원시 실행 증거 수집

- 상태: in-progress
- 분류: tooling
- 우선순위: P0
- 요구사항: R13
- 담당/수정 범위: root; scripts/ci_capture_operation.py, scripts/test_ci_capture_operation.py, CI 관련 문서. 하위 에이전트는 /tmp 후보만 담당했다.
- 선행 작업: ci_whole_receipt 형식·경로 검증과 Windows whole 증거

## 사용자 결과

Windows CI에서 각 Cargo 작업의 원시 stdout/stderr와 실제 소유 Job 종료 기록을 수집한다. 형식 검사만 통과한 자료를 실제 실행으로 표시하지 않는다.

## 계약과 범위

headless argv 실행, gated worker의 Job 선배정, 각 로그 8MiB와 64KiB 읽기 상한, 타임아웃과 자연 active0 확인을 유지한다. 공유 Cargo/schema/lock은 변경하지 않는다. 사용자 데스크톱·원격 계정·공개 배포에는 접근하지 않는다.

## 인수 조건

실제 Windows의 정상 종료와 남은 자손·출력 초과·query 실패·타임아웃은 구분한다. 원시 바이트를 변형하지 않고 실패 산출물을 보존한다. root가 native 실행과 원본 SHA를 확인하기 전에는 완료로 표시하지 않는다.

## 검증과 handoff

하위 에이전트의 /tmp 후보는 순수 실패 경계 7개를 통과했다고 보고했다. 루트 검토·프로젝트 통합·실제 Windows 실행은 진행 중이다. 원격 CI 인증, GUI 인수, 법적 공지·릴리스 완료는 false로 유지한다.

Root가 두 스크립트를 통합하고 CI 관련 회귀44개를 실제 실행하여 통과했다. Windows Python3.14.5의 원 native 회귀7개와 실제 Cargo fmt 캡처에서 gated worker Job 선배정·natural active0·두 raw stream drain·원본/소스SHA를 확인했다. `windows-ci-capture-producer-native-20261011/root-review.json`은 이 범위 PASS_SCOPED다. workflow의 기존 핵심7개 작업을 Windows에서 producer로 대체했고 중복 재실행은 추가하지 않았다. 실패 raw/lifecycle는 always artifact에 보존한다. 나머지6개 작업의 producer 전체 실행·actual descendant timeout fault injection·원격 CI 실행은 아직 미검증이다. 기존 whole receipt의 private_job_cleanup_verified/remote_execution_authenticated false는 변경하지 않는다.

## Windows 파일 식별 실제 회귀와 실패 주입

첫 native fault 실행은 불변 파일을 `original opened file changed`로 거절해 실패했다. 실제 Windows Python3.14.5에서는 path stat의 ctime이 birth time과 같고 열린 fd fstat의 ctime은 ChangeTime으로 관측됐다. `windows-ci-receipt-stat-20261011`의 두 원시 기록을 보존했다. 원 guard를 느슨하게 하지 않고 원 HANDLE의128bit FileId/volume·생성/수정/변경 시각·크기·속성을 before/after 및 pathname 재개방과 대조하도록 교정했다. retainedfd fstat의 before/after 검사도 유지한다. 공유 Cargo/schema/lock과 앱 런타임은 변경하지 않았다.

동일 native whole-format 회귀의 RED1error/exit1과 GREEN1pass/exit0를 원 Jobnatural0와 함께 확인했다. Windows 전체회귀는11pass/1skip, Linux 전체 CI회귀44pass다. Windows symlink 생성권한1314는 시스템 설정을 바꾸지 않고 skip으로 구분했다. 앞선 전체 native 회귀의 실패를 보존한다. Windows Python의 기본 temp가 사용자 설정으로 Public Documents 아래 CreatorTemp를 선택한 것을 발견했으며, 해당 시험은 새 고유 fixture만 생성/정리했다. 이후 native 회귀는 프로젝트 `.local-tools/windows/tmp`를 직접 지정하여 이 경로를 사용하지 않는다. 실제 Windows symlink 검증을 완료로 표시하지 않는다.

교정 뒤 `windows-ci-capture-native-faults-second-20261011/root-review.json`에서 actualraw nonUTF/NUL bytes, parent exit0+살아 있는 자손의 거절, stdout8MiB+1 초과의125/상한 보존,1초timeout125, 실제 invalid HANDLE query 실패125를 root 검토했다. 실패는 legacy success record를 발행하지 않았고 outer originalhost Job은 natural0으로 종료했다. 테스트 syntheticargv는 실제 whole validator가 Cargo fmt 증거로 거절했다. 이들5개 native 경계는 원격CI나 전체Cargo7작업 성공을 증명하지 않는다. 새 pin 교정은68c4849 Git snapshot 이후 변경이다.

## 독립 검토 후 identity 경계 보강

root는 ci_whole_receipt.py와 test_ci_whole_receipt.py를 담당한다. diagnostic_cache는 /tmp 후보만 제공했다. 성공 query의 zero128ID는 identity로 채택하지 않으며, binary alias 예약 ID와 hash 시점 ID를 전후·최종 재검사로 결합한다. 같은 bytes의 다른 실제 파일 교체와 마지막 source 검사 중 교체 실패 회귀를 추가한다. point-in-time 검사이며 atomic/transient-change 배제·remote CI 인증을 주장하지 않는다. 후보 pure 테스트 16개는 root 재검증 및 실제 Windows 인수 전이다.

root는 보강 후보를 읽고 실제 통합했다. Linux CI pure48passed/0failed와 Windows 원 Python의 native15passed/1skip/0failed를 확인했다. skip은 실제 symbolic-link 생성 권한1314로 별도 표시하며 검사 통과로 주장하지 않는다. actual hardlink·파일 교체·samebyte ID 교체·zeroID 거절·128bit ID 보존이 검증됐다. 원 log/SHA/source620/natural Job0/input desktop 불변을 root 확인한 windows-ci-receipt-identity-hardening-native-20261011/root-review.json을 남겼다. 원 실패와 이전12개 시험 근거를 보존하며 remoteCI/wholeconstructor 실행 인증·릴리스 완료는 여전히 false다.

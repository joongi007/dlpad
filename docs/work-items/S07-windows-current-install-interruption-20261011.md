# S07: 현재 Windows 설치 중단과 새 프로세스 검사

- 상태: completed (현재 create-only 설치 인수 범위)
- 분류: verification
- 우선순위: P0
- 요구사항: R03, R10, R11
- 담당/수정 범위: root 실행·통합; 기존 scripts/windows_msvc_native_installer_control_collect.py 및 windows_native_installer_control_probe.cs를 사용한다. 새 cold consumer 후보는 release_preparation이 /tmp에서만 준비한다. 공유 Cargo/schema/lock 변경 없음.
- 선행 작업: plan-scroll whole1013/0/26와 실제 새 payload5파일 설치·재검증 인수

## 사용자 결과

설치 도중 중단하거나 닫으면 원래 worker 종료를 확인한 뒤 partial과 intent 기록을 보존한다. 이후 새 프로세스의 검사와 재설치는 미완료 파일을 성공으로 표시하거나 덮어쓰지 않아야 한다.

## 계약과 범위

ADR0005의 create-only·원 HANDLE 권한 계약을 유지한다. 별도 private desktop, 앱 소유 Windows TEMP의 128MiB 합성 입력과 sentinel만 사용한다. 실제 worker의 소유·생성시각을 확인한 copy checkpoint에서 시험한다. 사용자 데이터, visible desktop, powerloss·OSsleep, 자동 partial 재개, upgrade는 인수 범위가 아니다.

## 인수 조건

현재 binary cancel/close 각각 실제 중간 복사·원 worker ACK·natural exit0/Job0·private desktop 종료·원본 SHA/nativeID·partial prefix·intent-only를 확인한다. cold consumer는 기존 partial에서 성공/READY/등록을 만들지 않고 namespace·원본을 유지해야 한다. 가능한 새 설치는 다른 fresh target에서만 수행한다.

## 검증과 handoff

기존 phase fault와 journal 회귀는 현재 whole에서 통과했다. 현재 installer GUI 실패 경로는 새 증거로 확인하며, 역사적 시험 결과를 현재 실행으로 표시하지 않는다. 원 프로젝트 .git은 계속 invalid/empty이고 보존한다. 최신 source bundle a66ed4f는 이 후속 검증 추가 전 snapshot이다.

현재 [cancel](../validation/evidence/windows-msvc-installer-current-copy-cancel-20261011/root-review.json)과 [close](../validation/evidence/windows-msvc-installer-current-copy-close-20261011/root-review.json)는 각각 root-review PASS_SCOPED다. partial 18,939,904/19,660,800 bytes의 원 source prefix, intent-only namespace, sentinel 및 원본 SHA를 root가 별도로 확인했다. 실제 worker 종료 ACK·natural exit0·Job0·private desktop 종료가 기록됐다. 새 프로세스 검사 후보는 네 명령의 예상 종료값과 원본 보존을 확인했지만, 거절 원인의 typed 출력 검증을 추가하기 전에는 인수 완료로 표시하지 않는다.

추가 실제 실행: cold consumer 네 원 process 종료값 1/1/0/0, natural Job0 및 원 HANDLE/creation time/native ID 증거를 검토했다. root가 모든 기존 byte SHA/크기와 새 payload2파일 및 journal5단계를 별도 확인했다. [최종 인수](../validation/evidence/windows-msvc-installer-cold-consumer-first-20261011/root-acceptance.json), [typed 거절](../validation/evidence/windows-msvc-installer-cold-rejection-20261011/root-review.json). 검사 FAIL/InvalidData와 같은 root prepare FAIL/AlreadyExists·mutation=false를 raw 출력으로 확인했다. 원 실행 후보는 evidence/inputs에 보존했고 재현 helper3개를 scripts에 통합했다. 프로젝트 상대 py_compile은 기존 __pycache__의 읽기 전용 제약으로 실패했고 `/tmp` pycache prefix로 실제 재실행을 통과했다. 자동 재개·upgrade·물리 전원 차단 인수는 포함하지 않는다.

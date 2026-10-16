# S07: 기존 설치 루트의 변경 없는 생성 거절

- 상태: in-progress
- 분류: fix
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: diagnostic_cache — /tmp/dlpad-existing-root-refusal-20261012/ 후보; root — crates/engine/src/app_install/windows_caps.rs, crates/engine/src/app_install/user_bundle.rs, 이 문서
- 선행 작업: 현재 설치 후보 인수 완료와 source 고정 해제

## 사용자 결과

기존 폴더에 설치를 시도하면 해당 폴더를 보존하고, 정확한 생성 충돌 거절에서는 변경이 없었다는 결과를 제공한다.

## 계약과 범위

원 native FILE_CREATE의 정확한 STATUS_OBJECT_NAME_COLLISION만 변경 없는 거절로 분류한다. io ErrorKind만으로 판단하지 않는다. 생성 이후 ACL·identity 검증 오류와 자식 생성 실패는 보수적인 변경 가능성을 유지한다. 공개 create_new 계약과 소유 HANDLE·검증·취소 정책을 보존한다. Cargo/schema는 변경하지 않는다.

## 인수 조건

실제 Windows 원 디렉터리 identity/bytes가 충돌 후 보존되고 native 원 생성 충돌에서만 분류 값이 참이다. 없는 부모와 다른 오류는 충돌로 오인하지 않는다. 실제 bundle CLI의 동일 root 거절 후 모든 payload·namespace·native marker ID를 다시 대조한다.

## 검증과 handoff

현재 CLI smoke는 AlreadyExists인데 변경 가능성=true로 보고하여 엄격한 false 기대에 실패했다. 제품의 보수적 값과 시험 기대를 확인했고 정확한 native 충돌 구분 후보를 준비했다. root가 기존 Windows 회귀에 변경 가능성=false를 요구하여 먼저 재현한다. 원 실패 산출물과 기존 사용자 데이터는 보존한다.

- 실제 RED77008 exit101: 원 existing_outer_never_adopted에서 mutation=false 요구가 실패했다. 원 로그/source 불변·소유 Job 자연 종료를 보존했다. root가 정확한 native FILE_CREATE 충돌 분류와 outer factory만 통합했다. 공개 create_new는 기존 io Result wrapper를 유지하고 정확한 충돌 이외에는 기존 보수적 값을 유지한다. 원 directory identity 보존 충돌·없는 부모 오분류 거절 회귀를 추가하고 Windows 전체 app_install 대상 GREEN54767을 실행 중이다. Cargo와 공유 schema 변경은 없다.

- GREEN54767은133개 중132개 통과/1개 실패. 원 transaction은 factory 호출 직전 target hint를 설정해 변경 가능성을 true로 계산했고 상위 OR가 정확한 collisionfalse를 다시 true로 덮었다. root가 이 정확한 outer collision 증거에서만 상위 OR를 생략하도록 고쳤다. 다른 실패는 기존 OR를 유지한다. GREEN-v2 69992 exit0: app_install 실제 Windows133개 모두 통과, source 불변·소유 Job 자연 종료. 최신 전체와 CLI 재검증은 아직 남는다.

- diagnostic_cache가 최신1173 whole과 새 ui-open-context-final payload를 묶은 /tmp/dlpad-fresh-publication-open-context-final-20261012 runner를 준비했다. root가 원 collector diff와 strict 현재 source/5파일 검사 경로를 확인했다. 기존1165 실패 증거를 보존하며 새 package 완료 뒤 root가 실제 실행·독립 대조한다. 현재는 새 CLI 인수 미완료.

- 최신1173 정확payload의 실제 smoke24521 exit0, 독립 검토92291 exit0 PASS_SCOPE. 원 prepare/cold inspection/AlreadyExists 거절에서 mutationfalse를 확인하고 native ID 전후·namespace·5파일 bytes/SHA 보존을 대조했다. 증거 windows-msvc-fresh-publication-ui-open-context-final-20261012/root-independent-review.json. 원1165 실패는 그대로 보존한다. 전체 upgrade/rollback·역사적 ownership·서명·정식릴리스는 이 범위의 완료가 아니다.

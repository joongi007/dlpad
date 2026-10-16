# S03: keeper sibling 및 P0 옵션 도움말 전체 통합 검증

- 상태: completed (고정 source의 Linux 통합 검증 범위)
- 분류: validation
- 우선순위: P0
- 요구사항: R07, R08, R10, R11
- 담당/수정 범위: root; 이 문서 및 cookie-keeper-options-workspace-first 증거
- 선행 작업: keeper sibling 실제2PASS, P0 CPU/default 도움말 구현

## 사용자 결과

다른 서비스 보존 및 옵션 도움말 변경을 같은 source에서 검증한다.

## 계약과 범위

Rust/Cargo 입력 동결, 기본/CPU 공통7검증과 실제 cookie 서비스/다운로드/복구/초기실패 회귀를 실행한다. 격리 GUI는 별도 고정 CPU 바이너리의 결과다. 다른 OS/실제재부팅/전체P0 완료로 확대하지 않는다.

## 인수 조건

각 명령 실제 성공과 source-before/after 일치. ignored 시험은 필터로 명시 실행하고 실제 개수를 확인한다. 실패는 보존하고 원인을 조사한다.

## 검증과 handoff

실행 중.

필터 점검에서 cookie_download_gate 및 cookie_recovery_gate의 시험 이름은 actual_ 접두사가 없음을 확인했다. 현재 pipeline의 해당 두 필터 실행은 실제 시험이0개라면 인수로 인정하지 않는다. 기존 서비스/초기실패 필터는실제이름과일치한다. pipeline 종료 후 native_authenticated_/sameboot_ 필터로4+2개를 별도 명시 실행하고 결과를 연결한다.

Root가기본/CPU7명령exit0/source일치와서비스3·초기실패2actualPASS를대조했다. 잘못된actual_필터의다운로드/복구0시험은제외한다. 별도native-corrected의외부다운로드4PASS를확인했고첫summary내부worker1선택오류는root-review에분리했다. recovery-corrected는실제parent2PASS+fixtureenv없는내부worker noop1로outer3PASS다. 결과분류오류를원시재검토해root-review에남겼으며제품실패는없었다. 다른OS/재부팅/실제사이트/전체P0는별도다.

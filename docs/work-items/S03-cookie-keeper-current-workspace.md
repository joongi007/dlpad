# S03: EOF keeper 현재 workspace 검증

- 상태: completed (고정 second source 범위)
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R10, R13
- 담당/수정 범위: root; 이 문서와 cookie-keeper-workspace-current 증거만 수정
- 선행 작업: S02-cookie-eof-keeper-integration

## 사용자 결과

초기 인증 시작 실패 정리 변경의 전체 회귀를 확인한다.

## 계약과 범위

Rust 소스 동결. schema/Cargo 변경 없음. 기존 유효하지 않은 .git 보존. 실제 재부팅과 다른 OS 인수는 별도다.

## 인수 조건

기본/CPU fmt, clippy, test, release 성공과 소스 해시 일치.

## 검증과 handoff

실행 중. 실패 로그 보존. 이전 keeper 도입 전 통과를 현재 통과로 취급하지 않는다.

첫 실행 fmt/clippy exit0, engine lib252PASS/1FAIL/2ignored. 새 nonce 양성 fixture가34자리였으며32자리 제품 parser는 정확히 거절했다. root가 양성 fixture만32자리로 수정했다. 원 실패 증거는 그대로 보존하고 새 second evidence에서 전체 검증한다.

Second evidence의 기본 fmt/clippy/test/release가 각각 exit0이며 현재까지 Rust/Cargo 입력 해시가 일치한다. CPU feature clippy/test/release는 같은 pipeline에서 아직 실행 중이다. ignored native와8h 시험은 기본 workspace 성공으로 통과 처리하지 않는다.

Second pipeline 기본/CPU의 fmt, clippy, test, release 전체7명령 exit0 및 source-before/after SHA 일치를 root가 실제 결과로 확인했다. 이후 debug sibling barrier 변경은 별도 snapshot과 검증 대상이며 이 통과에 포함하지 않는다.

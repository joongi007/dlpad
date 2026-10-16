# S11: 실패한 현재 CI 전용 빌드 캐시 회수

- 상태: done-scoped
- 분류: verification-infrastructure
- 우선순위: P0
- 요구사항: R11
- 담당/수정 범위: Root 실행·통합, search_reveal은 `.local-tools/session-recovery-20261013/current32-failed-ci-cache-cleanup-v1/` 후보만 담당. 제품 코드·공유 Cargo/lock 변경 없음.
- 선행 작업: 현재32 lowdebug-v2 CI의 실제 종료와 원자료 보존

## 사용자 결과

시험용 빌드 캐시가 D 드라이브를 채워 후속 P0 검증을 막는 상태를 해소한다.

## 계약과 범위

실제 cpu-release는 moxcms/pxfm 출력 쓰기에서 os error112, cargo101로 종료했다. outer1, inner/outer Job natural empty와 사용자 desktop 불변은 독립 확인한다. 원 실패를 성공으로 변경하지 않는다. 이번 실행이 새로 만든 정확한 fresh-source/target 하나만 회수한다. 소스·registry·실행 로그·캡처·기존 Windows 도구·최신3PE·사용자 자료는 삭제하지 않는다.

## 인수 조건

원 config·실패 operation/log·exact root 소유권, native reparse/identity, 컴파일러 종료를 확인하고 회수한다. 전후 source675/lock/기본3PE 및 원 로그가 일치하고 실제 여유 공간 차이를 기록한다. 논리 크기를 물리 회수량으로 추정하지 않는다. 이후 검증은 새 증거 경로를 사용한다.

## 검증과 handoff

Root의 후보 독립 검토와 실제 실행 대기. PDB/다른 과거 캐시 정리와 CI 재실행은 별도 결정이다. 자동 승인 거절은 없으며 이번 실패는 디스크 부족이다.

실제 Root v3 실행은 `--delete-exact-failed-ci-target`로 통과했다. native14,913파일/1,726디렉터리/16,638entry·원rootID와 no-reparse를 확인했다. 논리5,557,317,909B, actual free798,720→5,207,900,160B이며 hardlink336 때문에 논리량과 실제 회수량은 다르다. debug4,284,518,775B 중 incremental1,685,426,953B/debugPDB843,304,960B, release1,272,799,134B를 기록했다. 원 source675/기본3PE/로그 핀 보존 확인. [실제 결과](../validation/evidence/current32-failed-ci-cache-cleanup-v3-20261016/result.json).

v1은 helper 인자 누락으로 native 미실행/삭제0, v2는 native 재검사 PIN_CHANGED/삭제0이었다. 원 실패를 보존하고 v3은 조건을 바꾸지 않고 path/필드 계측만 추가했다. v3에서 같은 조건이 통과했으므로 v2 변경 원인은 확정하지 않는다. 프로젝트 밖 데이터·과거 공급/설치 도구·사용자 미디어 삭제0이다. 자동 승인 거절은 없었다.

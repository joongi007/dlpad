# S03: 게시된 파일의 복구 완료 저장 SQLITE_FULL

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R06, R13
- 담당/수정 범위: diagnostic_cache /tmp cfg(test) 초안; root crates/engine/src/download.rs 테스트 module·본 문서·원 Windows 실행·전체 통합.
- 선행 작업: 이전 native full4 terminal 및 public create_job FULL 인수 완료.

## 사용자 결과

이미 게시된 합성 파일의 완료 기록 복구가 SQLite 용량 상한으로 실패해도 저널과 원 파일을 보존하고, 공간 제한 해제 후 정확히 한 번 완료 복구한다.

## 계약과 범위

원 publication::Plan prepare/json/publish와 Verifying journal로 published-but-uncommitted 경계를 seed한다. 같은 제품 retained SQLite connection에 실제pagecount quota를 설정하고 완료 UPDATE에만 64KiB 할당하는 test-only trigger로 실제 SQLITE_FULL을 발생시킨다. public recover를 수정 없이 실행한다. 제품API/schema/dependency/런타임 seam0이며 SQLite논리quota와 실제filesystemENOSPC를 구분한다.

## 인수 조건

실제DiskFull 오류코드, 원Verifying JSON/planned/attempt/Queued/staged+final byte보존, trigger 할당rollback0. quota/trigger해제 후 recover1→0, 재열기 Downloaded/attempt1/artifact1 및 integrityok, 원final/sentinel보존. Windows native와 sourceSHA 통합 확인.

## 검증과 handoff

이 작업의 논리 SQLite quota 회귀 구현·실측은 완료했다. 실제 execute_inner 마지막save와 filesystempublication 용량실패 인수는 별도다.

원Windows exact1 PASS/0FAIL,0.04s. 실제quota의DiskFull을확인하고원Verifying/plan/attempt/source/final/Queued보존과공간제한해제후recover1/0·reopen을인수했다. [원회귀](../validation/evidence/windows-sqlite-published-full-first-20261005-resume/published-full.json). 현재기본CPU 전체fmt/clippy/test/release4 PASS와535source SHA 불일치0를root확인했다. [원전체](../validation/evidence/windows-published-full-current-default-20261005-resume/result.json). 전체P0/물리ENOSPC/actualexecute_inner마지막save는미검증이다.

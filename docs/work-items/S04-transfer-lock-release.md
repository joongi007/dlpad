# S04: 상속 descriptor 이후 설치 lock 해제

- 상태: done
- 분류: bug
- 우선순위: P0
- 요구사항: R05, R08
- 담당/수정 범위: diagnostic_cache agent; crates/engine/src/tools/direct_download.rs, 이 계약, managed-tools-validation.md
- 선행 작업: staging capacity transfer lock

## 사용자 결과

실패/종료된 설치의 lock을 상속된 descriptor 때문에 남겨 다음 설치가 WouldBlock으로 거절되지 않게 한다.

## 계약과 범위

주 담당 workspace 통합 실패2건을 먼저 조사하도록 위임했다. 실제 오류 출력만 추가해 engine 전체 반복에서 length16 expected UnexpectedSize 대신 Io WouldBlock code11을 재현했다. Unix flock은 shared open-file description의 clone/fork가 남으면 원본 close만으로 해제되지 않는다. TransferLock Drop에서 명시 unlock한다. 일반 오류와200ms 의도적 timeout 기대/제품120초 상한은 유지한다.

## 인수 조건

cloned descriptor가 남은 동안 contender lock이 거절되고 guard Drop 후 같은 contender가 획득하는 재현 시험, 전체 engine 반복, clippy를 통과한다. other OS를 검증했다고 표현하지 않는다.

## 검증과 handoff

수정 후 engine111 pass/1ignored(4.53초), 전체5회 연속 통과. 일반 hash fixture의 원래1초 deadline을 복원한 뒤 전체3회 추가 통과(총8회);200ms version timeout/제품120초 상한 유지. engine all-target clippy 통과(10.30초). 다른 OS 실제 시험은 별도 담당이며 이 기록에 포함하지 않는다.

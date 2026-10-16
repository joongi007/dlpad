# S00: 긴 Windows CLI 출력의 소유 표식 보존

- 상태: done
- 분류: bug
- 우선순위: P0
- 요구사항: R01
- 담당/수정 범위: root; scripts/windows_current_workspace_probe.py, 이 문서와 검증 기록.
- 선행 작업: Windows scan fixture 전체 회귀 시험의 원 로그.

## 사용자 결과

긴 회귀 시험 출력 때문에 검증기가 성공한 실행의 소유 표식을 잃지 않는다.

## 계약과 범위

8MiB 로그와64KiB terminal tail 상한을 유지한다. host가 자식 출력 전에 쓰는 소유 header는 별도 최대4096B prefix에 보존한다. 전체 로그를 메모리에 적재하지 않는다. native ownership·격리 desktop·input desktop 불변 조건을 제거하지 않는다.

## 인수 조건

원65650B 로그에서 tail에는 없는 실제 첫줄 소유 header를 prefix에서 찾는다. 표식이 없는 자료를 성공으로 바꾸지 않는다. 수정 후 실제 Windows 전체 CLI 회귀와 release를 다시 검증한다.

## 검증과 handoff

windows-scan-closewait-feature-first-20261005-resume/test.log 첫줄은 원pid9924/private-desktop=true이며 Cargo exit0과 새fixture시험 성공을 확인했다. 기존64KiB tail에서는 첫줄이 밀려 rawcollector passed=false였다. 원실패를 보존한다. root가 bounded prefix 보존을 구현했으며 수정 후 실제 재검증은 후속이다.

원65650B 로그의 tail에header없음/prefix에원9924한개를 재현했고 header없는자료/문장내유사표식 거절도 확인했다. windows-scan-closewait-feature-seed-fixed-20261005-resume의 실제전체4명령이 모두원PID/private desktop/input불변과source일치를 보존하며 통과했다. root가최종snapshot/새10회귀/lock불변/releaseSHA를 독립 대조했다. 수집기 좁은 결함 인수를 완료했고 제품전체P0 완료는 아니다.

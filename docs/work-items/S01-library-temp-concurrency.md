# S01: TEMP 스캔 중 독립 읽기·쓰기와 취소

- 상태: done
- 분류: verification
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: root 격리 evidence/실행·통합 문서; diagnostic_cache /tmp fixture 초안
- 선행 작업: Windows fresh metadata·lazy TEMP index 회귀

## 사용자 결과

주기 스캔에 CPU 대기를 넣기 전에, 마지막 게시 목록 읽기와 취소 rollback에 미치는 영향을 실제 Windows SQLite로 확인한다.

## 계약과 범위

기존 기본 OFF library-scan-closewait-fixture만 격리 manifest에서 활성화한다. 첫 TEMP INSERT 이후 동일 library.sqlite 별도 연결로 bounded read와 setting write를 수행한다. 원 busy timeout 5초, 소유 합성 파일 두 개, cap1 channel, 외부 Job 기한을 유지한다. 제품 Cargo/schema/lock 및 jobs/theme DB는 변경하지 않는다.

## 인수 조건

읽기는 기존 정확 한 행을 반환한다. 쓰기는 실제 성공 또는 Busy/Locked만 허용하고 시간·코드를 기록한다. 취소 뒤 재열기 목록·합성 파일·sentinel을 보존한다. publication 단계와 GUI 응답성·전체 P0 인수는 별도다.

## 검증과 handoff

아직 실행 결과를 완료로 주장하지 않는다. 격리 offline lock/clippy/release/owned native 실행 후 root가 원 guard/source/fixture pins를 검토한다.

실제 native offline lock/clippy/release/fixture4명령 PASS. root source/CPU/fixture2pins·원 Job/private/input guard 및 두 파일/sentinel 재읽기 PASS. read0.0003712초·writer0.0096892초 성공, 취소/reopen 정확 보존. [원증거](../validation/evidence/windows-library-temp-concurrency-native-20261006-continue/result.json). 최초 runner 잘못된 lock 인자 및 제한 WSL interop socket 실패는 앞선 디렉터리에 보존했다. 최종 publication 잠금/GUI는 미검증이다.

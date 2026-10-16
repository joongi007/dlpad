# S00: 소유 CI24 incremental 캐시 보존과 개발 디스크 공간 확보

- 상태: done (소유 incremental cache 백업·공간 확보)
- 분류: maintenance
- 우선순위: P0
- 요구사항: R10, R11
- 담당/수정 범위: Root — 소유CI24 fresh-source/target/debug/incremental, /tmp 전용 백업, B/reinstall-root-v1 보고서, 이 작업 기록
- 선행 작업: CI24 모든15단계의원natural0/Job0, 전체Rootreview확인; feature진단첫실행StorageFull 112/101 확인

## 사용자 결과

검증된설치본·원증거·코드·Git·사용자데이터는보존하고이번작업의순수compiler cache만백업해개발드라이브공간을확보한다.

## 계약과 범위

대상은자체CI24 source export에생성한debug/incremental 7,446파일/2,578,186,745B만이다. debug guardian·release실행파일·.git·source·evidence·관리도구·미디어·임시프로필은대상이아니다. /tmp 소유백업archive를만든뒤전체tar bytecompare/원stat비교/archiveSHA를통과해야정확대상만제거한다. 실패시원자료를보존한다. 파일identity복원이나다른cache전체삭제를주장하지않는다. activefeature진단첫루프는이미종료됐고CI24 compiler는전15단계종료된원cache다.

## 인수 조건

원정확namespace/plain경로/정상완료CI증거 확인, 전체archive=original bytecompare0/원snapshot불변/archiveSHA를확인한다. 삭제후target정확부재와release/debug executable보존을대조한다. feature첫원FAIL을보존하고실제공간확보뒤새namespace에서같은4명령을검사한다.

## 검증과 handoff

feature첫현재source671 fmt/clippy0 후engine aggregate_staging set_len의actualStorageFull 112로test101,미측정이다. 재설치v25일반whole1542/0/42와ZIP/native인수는기존성공범위로유지하며feature실패를숨기지않는다. root원CI24증거는PASS15/1538/0/42이고incremental은최종Rootreview의filepin대상이아니다. 백업·비교·공간확보는진행중이다.


실제실행 python3 -B .local-tools/session-recovery-20261013/reinstall-root-v1/relocate_ci24_incremental.py exit0. archive2,591,610,880B SHA75e5a74c54b307bbc6a0632b475f5f9cad25bcc5272b6053b3cfd311357ca916,tar전체bytecompare0/원snapshot불변뒤원정확incremental만제거했다. 확보후free2,918,322,176B. Root는별도CI24 release/debug 실행파일6개의원SHA/size/stat ID/nlink를전후대조해불변을확인했다. userdata/소스/.git/notice/설치본/evidence는삭제하지않았다. 원cache의fileidentity복원을주장하지않고/tmp archive와전체before manifest를보존한다.

# R04: Windows 실제5GiB 전송·완료등록 수직시험 후보

- 상태: done-scoped / 실제 합성5GiB byte-axis Root 독립 검증
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R06, R11, PERFORMANCE 필수시나리오3
- 담당/수정 범위: p0_audit는 `.local-tools/session-recovery-20261013/large-byte-axis-prep-v1/ 및 Root v2–v6 후보/` 준비만, Root가 실제 빌드·실행·독립 검토. 공유 schema/Cargo/lock·제품 코드 변경 없음.
- 선행 작업: current32whole/Rootreview/source/기본3PE·원syntheticseed/toolpins 일치, Root의D빌드공간확보·C새ownedTEMP쓰기자동승인검토. CI/Cargo와순차

## 사용자 결과

실제5GiB HTTP바이트가production다운로드·publication·SQLite Downloaded/재열기·u64 progress를통과하는독립시험. 8h duration/벽시계와분리한다.

## 계약과 범위

후보의 SAFETY-CONTRACT/HANDOFF/argv를 따른다. 원8h ToolPin/guardian/processrunner/workingSet함수와단순loopback서버기반, 새source/schema/dependency0. 원scratchpackage/deps/lockbyte동일+고유bin만. readonly원seed778,515B는후처리0; 가상freebox뒤zero64KiB를생성. production예산수정0. 새CprivateDirectoryknownfolder/currentSIDACL/retainedparent+rootHANDLE, Root독립검토전artifact성공/실패보존. D큰artifact쓰기0·기존userdata삭제0.

## 인수 조건

정확5,368,709,120B 실제파일의fullstreamSHA/원prefix·extendedfreeheader·zerotail, sparseflag0, productionDownloaded/artifactbytes/DB再열기recover0, 실제ProgressFeed>u32max/최종5GiB, HTTP완료Rangeunion全coverage. 원NativeIdentity와HASH전후불변; source/3PE不변. 900초/外1020초/12GiB중복포함전송/256requests/901samples/파일64·고유bytes5GiB+32MiB 상한. timeout/cancel/unknownownershipfail/추가tool0. 전체8h·실사이트·활성외부toolmemory인수로승격금지.

## 검증과 handoff

실제실행: PythonAST2PASS·원scratchCargo.lockbyte동일정적검사뿐. layout Rust회귀2개코드준비,未실행. Cargo/CSC/네트워크/큰파일0. Root run.py인자와자동승인검토후실행. source32고정gate는실제proof에서count/hash/totals를읽는다. Dfree<2GiB면빌드/출력dir생성전거부; 이여유는최악빌드공간보장이아니며Root공간조정필수.

## 실제 실행과 보완

원 v2는 standalone 도구가 소유 tmp에 runtime을 푸는 중 총64파일 제한으로 실패했다. 이후 v3은 작업 파일64와 별도 tmp4096파일/512MiB·전체288디렉터리/깊이12로 제한했다. 제품 예산 변경이 아니라 재현된 fixture 분류 오류 수정이다. 원 v1의 동결된 SAFETY-CONTRACT는 준비 당시 기록이며 실제 승인 범위는 v6 ROOT-ACCEPTED-SCOPE에 명시했다.

v3/v4 metadata ConnectionAborted10053은 시험 서버의 accepted socket 처리에서 좁혔다. 작은 Windows delayed-client 회귀가 수정 전1실패, `set_nonblocking(false)`를 적용해 새 package/bin으로 실제 컴파일한 뒤1통과했다. v5는 새 컴파일을 확인하지 못해 판정에서 제외했고 원 실패를 보존했다. 복사 mtime/공유 cache가 원인이라는 추정은 확정하지 않는다. [재현·수정 원검토](../validation/evidence/large-byte-accept-delayed-client-regression-v2-20261016/root-review.json).

Root의 v6 run.py 실제3operations는0/Job natural empty/forced false/입력 desktop 불변이다. 미디어 body5,368,709,120B가 HTTP 전체구간으로 전송됐고, 실제 원 파일·publication·Downloaded1/attempt1/artifact1·DB 재열기 recover0·u64 진행률을 통과했다. 전체 fixture157.30초,16 progress samples, 최종bytes5GiB다. ffprobe는5GiB/28,800초이며 짧은decode만 검사했다. 원778,515B synthetic media+extended free atom+zero padding의 byte 축이다.

Root가44pins·원 로그/샘플·readonly 실제 DB를 대조하고, 별도 Windows 읽기 전용 HANDLE로5GiB 전체 SHA와 원128bit NativeID를 다시 확인했다. 원seed+header+zero의 SHA도 별도 Python으로 계산했다. 모두 `e0c9a26fcf187a40939d3b73172b686af15aa8d83e1777c954be26cf4c13fb52`다. [Root 독립 검토](../validation/evidence/r04-large-byte-axis-v6-20261016/root-review.json).

실제 tmp peak139파일/23,815,375B이고 작업 파일64 제한을 유지했다. owner WS 관측은 전송 중16표본뿐이며 전체157초·외부도구·shipping UI 메모리 인수로 사용하지 않는다. 실제 사이트/8시간 전체decode·새8시간 벽시계/slow disk·전체P0는 별도다. 이전8시간 실험 근거를 대체하지 않는다. 새 C TEMP 전용 root와5GiB 결과를 보존했고 기존 사용자 자료·제품 source675/기본3PE/공유 lock은 변경하지 않았다. scratch root package/bin만 고유 이름으로 바꿨으며 Cargo.lock의 그 이름 외 바이트가 원과 같아 새 registry dependency0이다. 제품 실행 크기/상시비용 증가0.

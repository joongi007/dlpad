# P0 로컬CI 공간 준비: 원 잔여 incremental76 읽기 후보

- 상태: proposed / Root 실행 대기
- 분류: validation
- 우선순위: P0
- 요구사항: R11
- 담당/수정 범위: p0_audit / 새B의원rowmanifest·두batchnativehelper·인계만
- 선행 작업: Root S07/Cargo/CI/native 모두terminal,기존12삭제는Root근거이며원88/12문서보존

## 사용자 결과

원88의남은76을새경로·정확명세로관찰해CI실행전공간확보결정의근거를준비한다. 삭제를수행하거나회수량을보장하지않는다.

## 계약과 범위

원scan_rows88 - 원12 = 원순서76; 각38,중복0. 원row/treeSHA·mtime/count/bytes/nlink유지. 원nativecode는Manifest경로/SHA/Names만,collector는manifest/expected38만변경. 원cap모두유지. 두batch합계6,659entry/210dirs/논리5,245,521,886B.

## 인수 조건

실제각native의owner marker/ancestor 및descendant no reparse/nativeIDs/count/mtime/bytes 및shareREADonly·60k출력/90초/20k entries·256dirs/depth8을만족해야한다. stale모두거절하며후보그자체를삭제권한으로쓰지않는다. Root가두결과union76및삭제직전원treeSHA재검증을수행. 원PE/증거/cache자격/프로젝트밖userdata삭제0.

## 검증과 handoff

PythonAST/partition/원native변경상수만검사. 실제CSC/native/delete0. 각batchargv.json을Root가순차실행하고모든Cargoterminal후판정한다. hardlink로실제회수미확정. 제품runtime/deps0,두compile+host상한330초.

Root 조정: 현재Cargo9714 terminal1/후속clippy미실행, 다른Cargo없음. 실제S07권한실패자료는보존하고캐시는증거/PE와분리한다. 원v24CItarget실측8,606,642,176B보다여유가작아추가읽기검증부터진행. 실제삭제는Root별도소유권/원metadataSHA재검증및전체컴파일러종료확인후수행한다.

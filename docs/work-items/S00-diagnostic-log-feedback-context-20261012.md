# S00: 진단 로그 번호 오류 문맥

- 상태: completed
- 분류: UX
- 우선순위: P0
- 요구사항: R08, R13
- 담당/수정 범위: root — accessible_diagnostics.rs(재현 후 필요시), /tmp 격리시험, 이 문서
- 선행 작업: 현재1193 whole 및 저장 피드백 인수

## 사용자 결과

잘못된 로그 번호로 수정·제외를 누르면 해당 조작 가까이에서 오류를 확인한다.

## 계약과 범위

현재 Logs 피드백은 큰 편집창과 적용버튼 뒤에서 표시된다. 잘못된999999 번호·수정 클릭의 actual screenshot으로 먼저 확인한다. 별도 편집 적용오류와 구분하고 worker/report/입력ID/focus/scroll을 보존한다. 공유schema/Cargo변경없다.

## 인수 조건

로그 번호 오류가 해당 조작과 같은viewport에서 전체 읽히고, 적용오류는 편집 적용 문맥에서 보인다. 결과삽입으로 입력초안/selection을 잃지 않는다. 실제Light/Dark·작은창/확대와 회귀검증을 완료한다.

## 검증과 handoff

source 점검으로 가시성 문제가 의심되어 현source 격리GUI 재현후 수정여부를판정한다. 아직구현/인수미완료.

- 원source actualLight15378 exit0. invalid-log-index-after-action.png에서오류는100%화면에보이지만index/action과editor/apply뒤로분리되어있음을root직접확인했다. 미가시성으로과장하지않는다. UI-only scope를LogIndex/LogEdit로분리해번호action바로뒤/편집적용바로뒤에각각한번표시하도록수정했다. 기존worker/report/문구/입력ID/Cargo유지. native·whole회귀는새source에서아직미실행.

- pinned1.94 rustfmt exit0. Windows whole 새output windows-msvc-diagnostic-log-context-whole-20261012 실행을시작했다. 완료전기존1193proof를현재source증거로사용하지않는다.

- whole69727 exit0: fmt/clippy/test/release collector passedtrue. 독립whole와새source actualLight v2 재현을시작했다. 아직로그위치최종가시성/확대인수는미완료다.

- 독립whole42166 exit0:43harness/1193passed/0failed/30ignored와원4로그/source/GNU/artifact를대조했다. CPU9,761,280B SHA98a8a9d6bda593e1fad5c2eefbae0ef7239df6f1486883e96a0e58e4c1d754c3이며전feedbackCPU대비512B증가. 새dependencies/Cargo/lock변경없고UI-only scope1개추가비용이다. actualLight35882 실행중.

- 최신Light35882 exit0. root직접invalid-log-index-after-action.png확인: 번호action바로아래오류가전체읽히며기존editor/apply와분리된다. actual100%범위이고Dark/200%아직미완료. 독립Light및Dark실행을이어간다.

- Light독립49148 exit0로원raw/source/pins/DB/naturalguard를대조해root-independent-review.json에통합했다. 이review의workflowguard가오류paint를대신하지않으며root는actualPNG로별도문맥을확인했다. 실제800640/app200% 후보Light70833 실행중이며Dark100%15427도동일handle재관측중이다.

- Dark15427 exit0/독립92667 exit0. actualDark100%도root직접캡처에서번호action바로아래오류확인. Light800640/app200%70833 exit0:번호·3action·전체오류가같은body에읽히는actualPNG를root확인하고SHA/proof기록. Dark200%후속실행중이며편집적용오류/전체UI인수는미완료다.

- Light200% 독립95246 exit0, 원오류label1회/actual800640PNG/currentsettings200%/source·byte·DBguard대조해root-independent-review통합. Dark200%61025 exit0이며root직접캡처에서번호/action/전체오류를같은body에서확인하고root-scoped-pixel-review기록했다. Dark독립8363진행중. apply-edit쪽오류와fullUI범위는여전히미완료다.

- 로그번호scope외에apply편집scope actual인수가남아있어/tmp새200%scenario에잘못된번호999999 상태의정제후반영클릭·원label/PNG를추가했다. 기존번호action·보고서저장/기존파일보존guard를유지하고최신1195whole로Light실행시작. native결과/직접PNG는아직미확인이다.

- applyLight74910 exit0/최신1195source보존. root가800640/app200% invalid-log-apply-after-action.png에서적용버튼·전체오류가같은body안에읽히는것을직접확인하고SHA/pixelproof기록. 별도원provider/PNG/currentwhole/byte·DBguard독립88077진행중이며Darkapplyscope인수는남는다.

- Lightapply독립88077 exit0를root-independent-review로통합했다. Darkapply89700새실행중이며최신1195source·whole/raw/원보고서/파일보존조건동일. 전체로그UI인수는Dark결과후검토한다.

- Darkapply89700 exit0. root직접actual800640/app200% PNG확인:적용버튼·전체오류가같은body에읽히며footer와겹치지않는다. 원PNG SHA/pixelproof기록/독립54256진행중. root는고급입력의중복Textorigin이accessible.rs1937시작조건reason과accessible_options.rs318inlineerror일수있음을source로대조했고nativeparent순서범위를후보로위임했다.

- Darkapply독립54256 exit0/root-independent-review통합후현재로그번호/편집적용오류의문맥수정범위를인수했다. 실제번호action100/200양테마와apply200양테마/rootPNG·원provider/보고서·파일보존·natural·원전체검사를대조했다. 로그문맥수정의수직작업완료이며전체UI/UX·물리IME·모든로그편집경계인수를뜻하지않는다.

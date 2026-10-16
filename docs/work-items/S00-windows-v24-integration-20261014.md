# S00: Windows v24 실제 선택 글자 정렬 통합

- 상태: done / v24 scoped 통합 검증
- 분류: integration / validation
- 우선순위: P0
- 요구사항: R03, R08, R10, R11
- 담당/수정 범위: Root — 단일Live 파일·전체검사·native/패키지/문서; installer_batch — 분리renderer후보; advanced_units — 분리nativehelper; windows_auto — 분리패키지helper
- 선행 작업: source23 전체1536/0/42,실제6UIAgeometry/24원화면검수,새alignment50회귀PASS

## 사용자 결과

선택 파일명·metadata를왼쪽정렬하고작은창에서이름을우선한다. metadata최대45%/남은이름열,고정32px와hover/description/선택후원행좌표를유지한다.

## 계약과 범위

accessible_library.rs 한파일만 Root가변경한다. 기존두cell에서explicitmainAlignMin/crossCenter/main+crossjustify를사용하고실제그리기좌표회귀1개를추가했다. Core/engine/catalog/DB/schema/Cargo/lock/deps/worker/cadence는변경하지않는다. source670이며 Rootadoptionreceipt에backup/원SHA/새SHA를기록했다. 기존v22/v23산출물과원FAIL은보존한다. Git exit128을보존하고.git을재구성하지않았다.

## 인수 조건

새 Windows fmt/strict clippy/workspace test/release 및원source/로그/3PE 독립검토. 새privateUIA6scope(800/960·100/150/200)/24PNG 원픽셀로left alignment·filename priority/ID32px/nooverlap/footer/원8MiB파일보존/자연Job0 확인. 새ZIP원파일·전체CRC/notice provenance·payload7검토 및native정상/긴경로실행. 가상화면만 사용하며물리pointer/IME/reader/DPI/clean OS/정식서명·공개배포는구분한다.

## 검증과 handoff

단순halignMin의원v5제품미해결49PASS1FAIL을보존했다. explicitcell layout으로수정한v6은같은paint회귀·기존49를합쳐50PASS/0FAIL이다. 현재Live채택과whole24실행중,새UI/패키지는아직미인수다. 원23실worker17ticks는별도historicaldiagnostic이며current24/기본성능목표PASS로승격하지않는다.

### 최신 실행 결과

Windows 네 명령은 모두 exit0이며 실제 1,538 PASS / 0 FAIL / 42 ignored,43 harness/source670이다. Root가 원로그·현재 소스·실행 파일 세 개·lock을 독립 대조했다. 현재 새 private UI 여섯 장면과 설치 패키지 여섯 단계는 병렬 실행 중이며 성공을 미리 판정하지 않는다.

### 화면 인수

v24 private UIA여섯장면은전후행좌표/ID·32px·원합성8MiB파일·자연app0/Job0를Root독립대조했다. Root6selected/child18other 총24원PNG를직접보고 전체SHA/bytes/dimensions를Root재검산했다. 선택이름이왼쪽기준선으로맞고800px/200%에서도전체합성파일명이보인다. metadata말줄임/상단offscroll범위는기록하며물리pointer·doubleclick·IME·reader인수는아니다. [화면팀검토](../validation/evidence/current-library-selection-summary-native-v4-20261014/root-team-pixel-review.json). 새패키지여섯단계exit0,독립archive감사와정상/긴경로native는현재진행중이다.

### 패키지·실제 설치 인수

Root는새패키지6phaseexit0 및1,135exactinputs/474archive membersCRC/456원notice를재검산했다. 실제정상/긴경로/변조driver모두exit0,원33pins/current670/current3를대조했다. 정상app/supervisor자연0/Job0,긴577/598UTF-16·원20objects6parents/5payload5journal 보존,변조77→76/nochild/natural1와다른4파일·원7generated·journal보존을확인했다. Childreadonly三role리뷰도각exit0이나Root의독립확인이완료판정근거다. 설치88화면분담검수·문서check·최신CI/비용은남아있다.

### 통합 완료 범위

새actualsource24 전체1538/0/42, 선택24원화면,설치88원화면을Root/child가직접분담검수하고Root가전체SHA/bytes/dimensions를재검산했다. 새두ZIP/normal/long/negative검증과문서체크1094문서/114원notice는통과했다. 물리사용자검수/cleanOS/정식배포/전체P0완료는아니다. 최신로컬CI와비용은별도 [CI작업](S00-windows-v24-local-ci-20261014.md)/[비용작업](S00-windows-v24-default-cost-20261014.md)에서진행한다.

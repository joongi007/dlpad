# S05: 영상 상세에서 기본 앱으로 영상 열기

- 상태: in_progress / 기존 경로 분석·scratch 후보 준비
- 분류: feature
- 우선순위: P0
- 요구사항: R03/R05/R08 및 사용자 영상 상세 열기 요청
- 담당/수정 범위: search_reveal scratch media_workspace.rs/library_open.rs/external_open_broker.rs/engine media_preview.rs 후보; Root accessible.rs·두 main·catalog.rs 공유 통합·실행 검증
- 선행 작업: source30 native/package 실행 종료; 원 Prepared local pin·기존 library worker의 열기 검증 계약

## 사용자 결과

영상 상세에서 로컬 영상을 기본 연결 앱으로 열고, 연결 앱이 없으면 OS 앱 선택 동작을 제공한다. 파일 목록에서 연 영상뿐 아니라 로컬 드롭·export 결과도 같은 검증 경로를 따른다. 파일이 없는 원격 URL은 로컬 열기로 오인하지 않는다.

## 계약과 범위

Workspace는 사용자 의도만 내보내고 원 Prepared HANDLE/identity를 worker에서 다시 확인한다. UI thread I/O와 shell 문자열을 도입하지 않으며 기존 외부 viewer를 다운로드 소유 Job에 넣거나 종료하지 않는다. Windows private automation에서 사용자 desktop의 viewer 실행을 우회하지 않는다. macOS/Linux adapter 가능성을 기록하되 미검증 OS를 지원 완료로 보고하지 않는다. 공유 Cargo/lock/schema/catalog는 Root가 조정한다.

## 인수 조건

정상 파일·삭제/교체/권한실패·원격 URL·연결 앱 부재·선택 취소·중복 클릭·종료 중 동작을 구별한다. 버튼 hover/focus/disabled와 좁은 창 배치를 확인한다. 실제 OS chooser 미검증은 남긴다.

## 검증과 handoff

현재 기존 파일 목록의 SystemOpen은 있으나 Workspace 상세에는 없음을 확인했다. Windows no-association은 현재 오류이며 OS 선택 창 구현은 아직 없다. scratch 후보와 통합 검증 후 실제 명령·결과를 갱신한다.

## 실제 격리 검증

Root가v2후보를읽고앱소유modal SHOpenWithDialog를배제한Explorer handoff와원Prepared pin/worker경로를선택했다. exact NoAssociation일때만동일deadline으로chooser1회,다른오류/취소/시간초과에추측재실행없다. 선택회귀는Tools4를제외한16개가실제통과했다(기존ignored2는미실행). native viewer/chooser성공을이합성검사로대체하지않는다. S에명시hover/disabled안내를추가했고strictclippy실행중이다. 새고정상태worker는한번에1개·채널1·기존10초협조deadline을쓰며OS외부viewer를소유/종료하지않는다. 아직Live/사용자ZIP에미포함이다.

Root가실제S검사·원소스674/현재12파일·lock불변을대조해source31에채택했다. source675의새Windows전체검사실행중이며현재사용자v30 ZIP과구별한다. 격리최종strictworkspace/all-targetsclippy는0이다.

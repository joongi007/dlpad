# S03: 목록 전체 DB 읽기 전용 검색

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: installer_batch engine/library.rs·download.rs, 본 문서/B.list-search-engine-v1; windows_auto UI/worker/list_search, Root공유통합
- 선행 작업: windows_auto scratch/API 이관, 기존 keyset/read-only/busy/cursor·progress-template 계약

## 사용자 결과

현재화면128개를넘는전체DB 파일명/경로와이력ID/저장된URL을literal부분검색한다. 검색해제는원전체목록·첫cursor로복귀한다. 다른세대결과게시/Stop/CloseWait는UIworker담당과연결한다.

## 계약과 범위

검색어trim후256UTF8byte/NUL거절·SQLparameter·키셋검색128행·readonlyNOFOLLOW/NO_MUTEX와busy_timeout5s. 5초전체검색완료보장아님; substring선형scan비용/조회중cancelACK종료대기는명시한다. ID/StoredOptions v1 URL만조회하고 argv snapshot/advanced JSON은검색하지않는다. NULLlegacy는ID만; 선택된malformed/oversized는오류로자료보존, empty결과로숨기지않는다. Schema/Cargo/core/의존성변경0. Root최신download progress-template 보존.

## 인수 조건

129+행DB다음페이지한글·%_quote literal/trim/clear/keyset128·해당root분리·readonlymissing비생성/손상·busy실패·oversize/NUL/잘못된cursor·URL과snapshot구별/NULLlegacy/손상형거절과원DBbyte보존. UI세대fresh/stale는UIcaller시험과Root통합한다.

## 검증과 handoff

scratch인계받고actualLive원본을새B에별도backup했다. 미실행후보SQL을정리해실제Live소유2파일만좁게overlay하며nativeRoot통합전sourcefreezeACK를한다. 현재native/Cargo0.

### 구현·정적 검증 완료와 Root 인수 대기

Live두파일 API이식후 Root검사freeze동안B회귀만작성하고v8terminalACK후최종overlay했다. 이전v7 Sourceguard교차는실행통과로승격하지않고원FAIL scope보존한다. read_library_search_page_readonly/library_search_page/open_readonly/history_search API를UI담당에게전달했다. Rootprogress-templateliteral exact보존/legacy library256·검색128분리. 140행·한글특수문자·literal/clear/keyset·NULLlegacyIDonly·snapshot비검색·boundedmalformed/refusal·readonlymissing/corrupt/busy원자료보존5회귀를구현했다.

실제file-onlyrustfmt1.94 exit0, LiveSQL추출PythonSQLite 사전관측PASS. RustAPI/Windows/native 실행은아직0이며Root통합인수대기다. 5초는busy_timeout만이며전체선형SQLscan walltime·즉시cancel보장은하지않는다. UI세대/Stop/CloseWait 게시거절은windows_auto가검증하고검색backend는schema/mutation/권한을추가하지않는다. 원본/source-pins/patch/HANDOFF/sql-preflight는B/list-search-engine-v1에보존한다.

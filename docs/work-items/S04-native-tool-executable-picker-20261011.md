# S04: 수동 도구 실행 파일 선택

- 상태: completed
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R07, R13
- 담당/수정 범위: root accessible.rs·catalog.rs·native picker 공유 인터페이스 통합 및 전체 검증; release_preparation /tmp 후보만 작성. 동일 프로젝트 파일 병렬 수정 없음. Cargo/lock/schema는 root 조정.
- 선행 작업: 수동 도구 선택 저장, 기존 native 폴더 선택 계약

## 사용자 결과

yt-dlp, FFmpeg, Deno 경로 입력 옆에서 실행 파일 선택 창을 열 수 있다. 선택한 경로는 해당 수동 입력에 반영되며 기존 저장 동작으로 재실행 후 복원한다.

## 계약과 범위

기존 asynchronous native picker 및 source/owner 종료 계약을 재사용한다. UI thread 긴 I/O 없음. 취소·실패·종료 중 결과는 기존 draft를 보존한다. 선택은 도구 설치·신뢰·삭제 권한을 부여하지 않으며 기존 실행 전 검증을 유지한다. 중앙 문자열과 GTMUX input/button 정렬·hover/focus/disabled 상태를 적용한다. 활성 CI 검증 중 source를 수정하지 않고 독립 /tmp 후보를 준비한다.

## 인수 조건

실제 private Windows에서 세 필드 선택·취소·한글/긴 경로·owner-close 및 저장/재실행을 검증한다. 좁은 폭에서도 입력·버튼이 겹치지 않고 keyboard label/focus를 유지한다. 파일 선택 필터가 실행 파일 선택을 지원한다. 전체 fmt/clippy/test/release와 새 installer snapshot 결합을 root가 확인한다.

## 검증과 handoff

읽기 감사로 세 필드가 text input만 제공함을 확인했다. 기존 Download/Library 폴더 picker와 수동 설정 저장 인수는 유지된다. 후보 작성 중이며 구현/실제 GUI 인수 전 완료로 표시하지 않는다.

root는 frozen CI7 및 독립Win32 재조회가 끝난 뒤 후보를 원accessible.rs SHA와 대조해 통합했다. 추가3module·entry2개·중앙catalog이며 Cargo/lock/schema/의존성 변경없다. 단일transient STA worker/channel1 및 기존owner Arc·취소 ACK를 유지한다. 초기 relative/invalid/missing parent는 worker에서 bounded hint로만 판단하고 current_exe 부모로fallback한다. source 타입/전체/실제private 인수는 후속이며 현재 설치ZIP은 이전806bf snapshot이다.

첫 whole는 fmt 통과 뒤 clippy에서 새 회귀의 존재하지 않는 Store::get_setting 호출 및 lint로 실패했다. 원로그/실패receipt를 보존했다. root는 실제 Store::setting API로 교정하고 nested if/Boolean/question-mark lint를 동작 동일하게 수정한다. 테스트/release는 아직 실행되지 않았으며 fresh 전체 재검증한다.

새 picker 통합 whole-second fmt/clippy/test/release 모두 통과: 1,085passed/0failed/28ignored/43harness, root source623 및 원raw/Job0/원SHA 확인. CPU9,652,736B SHA dcd3e53ef08238fd593f5841e1b3b2bbb4564533a17c34746b31595d8f964db5는 직전9,625,600B보다27,136B(26.5KiB) 증가했다. 신규 Cargo/lock/의존성/DBschema 없음, worker/channel1은 명시 선택 동안만 실행되며 실제idle비용은 후속이다. C# observer 실제compile 및 root원pins 검토도 통과했다. 실제private browser 선택/cancel/Save/restart/owner-close 및 새 설치snapshot 인수는 실행중으로 아직 완료가 아니다. 현재README 설치ZIP은 이전검증806bf 후보를 유지한다.

첫 actual GUI observer는 native modal을 찾지 못해 실패했다. 기존 비동기 worker는 hidden STA proxy를 owner로 사용하지만 새 observer가 dialog의 immediate owner를 main HWND로 요구한 계약 불일치를 source에서 확인했다. 이 원실패는 forced fixture cleanup이며 정상 GUI 인수로 처리하지 않는다. 원소스/observer/로그를 보존하고 exact main→originalsamePID/STA/class proxy→dialog chain을 기록·검증하는 fresh observer를 준비한다. arbitrary ancestor/foreignPID/visible desktop fallback은 허용하지 않는다. 제품 버그 여부는 실제 chain 관측 후 판단한다.

source 감사에서 원폴더worker와 새실행파일worker가 같은 static Win32 class 이름에 다른 WNDPROC를 등록하는 실제 충돌 가능성을 발견했다. root가 folder_picker_worker.rs/path_picker.rs/tool_file_worker.rs의 기존 strict등록을 작은 helper로 추출하고 실제RegisterClass/GetClassInfo 단일회귀 RED를 통합했다. 같은 process의 원module/procedure를 검증하며 GUI/foreignclass변경 없이 충돌을 재현한다. GREEN은 도구class만 DlpadToolExecutablePickerProxy로 분리한다. 아직RED실행전이다. 새1085ZIP은 bounded검토만 완료했고 사용자현재링크로 채택하지 않았다.

actual RED: 단일native등록시험0passed/1failed, 원exit101/naturalJob0/noForced/source623를 root검토했다. 다른 WNDPROC의 기존class 재사용 strict거절을 재현했으며 원실패를보존했다. GREEN은 toolclass DlpadToolExecutablePickerProxy와 worker이름만 변경하고 기존 folderclass/등록검사/owner lifecycle을 유지한다. root는 red→green diff의 정확2문자열 변경을 확인했다. GREEN/scoped 및 freshwhole는 후속검증한다.

GREEN 실제등록 단일시험1passed/0failed 및 root원raw/source/Job0 검토통과. whole-third는fmt통과후새cfgtest helper 위치의 items_after_test_module lint에서실패했다. root는helper를tests모듈앞으로옮겨경고를수정하고freshwhole-fourth로재검증한다. 원 실패는보존하며전체시험통과로소급하지 않는다. 새v3observer 실제compile0(상속된unusedfieldwarning2) 및 root원pins검토통과, 실제GUI는후속이다.

whole-fourth 실제 fmt/clippy/test/release 모두 통과, root 원raw/source623/원도구SHA/actual artifact 검토 완료: 1,087passed/0failed/28ignored/43harness. 새 CPU9,653,760B SHA2f45fd0b43a86eaa6b1a8adc4cc5f7ebc6cd4e74536f7fb0a96bff527782f2a3. 실제 v3 private GUI와 fresh fixed graph/package 실행 중이다. v3 pure7 및 package guards5 root실행 통과. 실제 선택창·설치 인수 전 기존 설치 링크 유지.

v3 실제GUI는 첫 폴더 choose의 native filename exact readback에서 실패했다. 실제 원PID/STA의 DlpadFolderPickerProxy chain은 관측됐지만 선택 완료 인수는 아니다. 원 result/log/소유 process cleanup 및 입력desktop unchanged 기록을 보존했다. root는 관측 값을 기록하는 fresh observer를 준비하며 generic normalization/visible fallback 없이 원인 분리한다. 제품 수정의 RED/GREEN 및 whole-fourth 통과는 별도 인수로 유지한다.

새 fixed ZIP16,451,372B SHA95fde017efbca215a7b0cd252538231f154d70379f8c5fcd5858abe41c45f297 생성. root independent bounded archive/proof 검토 통과(실제graph258/86/258, selectednotice 누락0, 전체lock appendix 누락43 별도한계 유지). actual5file 설치 실행 중이며 완료판정 전이다. v4 observer의 정확W setter/readback은 native/provider 동일경로를 확인했지만 폴더선택 후 UI 문자열 정확비교에서 실패했다. 원실패 보존, actual Shell 경로표현과 원파일 ID를 함께 계측하는 후속으로 원인분리한다.

새 fixed 실제설치와다시검증 collector0, root33원핀/source623/installed5SHA/journal5/등록namespace/원rawJob0 독립검토통과. installed.png·verified.png 실제caption을 root직접확인했다. 별도post-Drop Win32ID재조회 및 파일선택actualv5는후속이다. v5는genericnormalize대신 actualSIGDN_FILESYSPATH와 원HANDLE FileID128/정확nativeFinalPath를 함께묶어시험한다. 제품source변경0. 현재사용자ZIP링크는 GUI인수전기존유지.

v5 실제폴더선택·취소는 actualShell문자열/원FileID128/정확FinalPath 인수가 통과했다. 첫tool은modal부재에서실패했고 v6계측이 before failurelabelfalse→Invoke후150ms failurelabeltrue를확인했다. 제품tool 초기화 오류이며GUI전체반복을중단하고 실제 constructor scoped회귀로축소한다. 현재 whole-fourth1087/설치5인수로 이기능의GUI완료를주장하지않는다. 기존모든실패/raw를보존한다.

post-install nativeID再조회는 compile0후native1/자연Job0에서실패했다. 원failure receipt/log를보존하며현재ID독립조회완료로표시하지않음. 원helper의failure출력에단계가없어 boundedstage계측후fresh출력으로원인분리한다. 설치5SHA/재검증/직접PNG 인수는별도범위로유지한다.

root는 원tool_file_dialog SHA ad614aff... 확인후실제constructor 단일회귀를통합했다. new(None)와current_exe hint를Show없이같은ownedthread에서생성하고 cfgtest 단계HRESULT를기록한다. production flags/type/콜순서변경0. scopedRED실행중이므로source동결후현whole-fourth는이전source범위이며추후freshwhole필요. 최신CPU CI7 첫시도는exportedsource에서제외된evidence의빈parent가없어Cargo전거절됐다. 원실패보존및freshouteroutput 재시도준비, source코드오류와혼동하지않음.

명칭 red인 scopedconstructor출력은실제시험1PASS였으며RED재현으로표현하지않는다. newNone/absolutehint 실제COM단계HRESULT전부0, Show=false. rootraw/source/Job0 검토완료. 제품문제는constructor밖 worker/Show/owner경로로축소하며 추가instrumentation을준비한다.

root가실제v6 invoke-timeout PNG를직접보았고 기존toolfield는확장정규경로(\\?\D:\Project...)임을확인했다. 기본current_exe 힌트성공만으로이형식을검증하지않았으므로 실제std::fs::canonicalize(current_exe) case를동일constructor회귀에추가한다. Shell초기힌트파싱가설이며아직실패재현/수정완료아님. 준비된ignoredworker Show시험은필요할때만통합한다.

실제 canonical extended 힌트 회귀는0passed/1failed, SHCreateItemFromParsingName HRESULT0x80070057로재현했다. root가원raw/exit101/자연Job0/source623를독립검토했다. 검증된diskprefix 초기폴더힌트에만 extended prefix를제거하여Shell에전달하며선택결과/원파일권한/검증은유지한다. GREEN은동일실제COM3case 모두생성·해제성공(1passed/0failed, Show없음), root원source/raw검토통과. freshwhole-fifth와실제GUI후속인수진행. Cargo/lock/의존성추가없음.

whole-fifth 실제 fmt/clippy/test/release 및root원source623/raw/GNU원본/artifact검토통과:1089passed/0failed/28ignored/43harness. CPU9,653,760B SHA294184aa5b0f6e328b31029fbf43aca24583d930407a27a061534823deebbdce이며직전fourth와크기동일. 실제GUIv7/새패키지용graph진행중. 설치보조C# genericDeserialize가entries를ArrayList로만드는차이를실제합성타입시험에서확인했으며원IDfailure를완료로표현하지않고fresh최종fixture후해당binding만교정한다.

v7 실제 toolmodal은정확DlpadToolExecutablePickerProxy/PID/STA/mainownerchain으로열렸다. constructor문제수정동작은관측됐으나observer의파일명입력 AutomationId1152 lookup에서실패했다. folderchoose/cancel2인수는통과했으며전체save/restart완료아님. 원실패/fixturecleanup/inputdesktopunchanged를보존하고bounded실제modal controltree·PNG로관측가정을축소검증한다.

v8실제modal PNG를root직접확인했고93행boundedtree에서toolfilename은AutomationId1148의writableValuePattern/nativeEdit임을확인했다. folder1152와다른실제Shell구조이며같은1148ComboBox는strictEdit검사로제외한다. observer종류별명시ID교정후freshGUI인수를실행한다. 제품source수정없이관측선택자만교정하며원실패보존한다. 최신ZIP16,451,372B SHA780e0e1f1beb1c560e9d5bdc8876bc8a8021747df9171cce856b71f9b7f6c123 boundedarchive독립검토통과, 실제설치실행중/현재사용자링크이전유지.

최신shell-hint-final 실제설치/재검증은원rawJob0/source623/33pins/5파일exactSHA/5journal독립검토통과했고root가installed.png/verified.png를직접확인했다. 보조JSON타입실험근거로DeserializeObject만교정한새nativeidentity probe는27originalHANDLE관측/등록payload5ID·recordID/현재journal5를검증했으며root원raw/source/원핀재검토통과. 현재FileID관측인수이며과거ownership/삭제권한/atomicnamespace증명아님.

v9 actual 세도구choose/cancel6과folderchoose/cancel/aftertools3의개별native/Shell/field원receipt가모두생성됐다. 그후phase보고서에서중복invokecontrol목록으로JavaScriptSerializer maxJsonLength64KiB를넘어시험observer가실패했다. 원실패보존하며완료보고하지않는다. 앱source수정없이원별도boundedinvokeJSON을SHA/bytes/path참조로묶는observer교정과5단계phase결합을준비한다. 제한확대/관측체크삭제없이재검증한다.

v10 실제GUI 전체5단계 save/reopen/use/reopen-managed/owner-close·closedDB5 integrity/exact정책·원pins/source623·원raw자연0/Job0가root독립검토통과했다. 세도구한글파일choose/cancel6, folderbefore/aftertools동일process3, ownerlivemodal종료1의모달10PNG·STA/proxy/fileID128/actualShell경로를확인했고root가직접UI/modal캡처를검토했다. 보고서64KiB상한은유지하며원phase/invokeJSON을SHA/bytes로결합했다. workerACK직접계측/물리IME/reader는미검증이며전체P0/정식릴리스완료아님. whole-fifth1089와동일CPU294184…최신ZIP780e…/실제설치5/현재nativeID27을통합해실행파일picker수직범위를완료한다.

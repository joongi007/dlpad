# S07: 네이티브 설치 명세 파일 선택

- 상태: DONE (파일 선택 기능의 Windows 개발 범위; 전체 P0 아님)
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — 프로젝트 게시·Cargo/shared interface·통합/최종 검증; metrics_recovery — /tmp 독립 picker 모듈 초안; diagnostic_cache — /tmp installer_main/catalog 연결 초안; soak_recovery_audit — /tmp private desktop observer/collector 초안. 같은 프로젝트 파일 동시 수정 없음.
- 선행 작업: 기존 설치 재검증 및 current 세 binary 개발 ZIP scoped DONE.

## 사용자 결과

설치 명세 입력 옆의 파일 선택 버튼으로 기존 로컬 파일 하나를 선택한다. 선택은 입력값만 바꾸며 설치·검사·제거는 사용자가 별도로 실행한다. 선택 취소는 기존 입력과 검증 상태를 보존한다.

## 계약과 범위

수정 후보는 crates/app/src/installer_file_picker.rs, installer_main.rs, installer_catalog.rs, crates/app/Cargo.toml 및 private native fixture/collector와 개발 안내다. root가 기존 windows-sys 0.61의 Com/Shell 선언 feature를 조정한다. 새 crate·schema·lock 변경은 제안하지 않는다. Common Item Dialog/IFileOpenDialog를 같은 GUI STA에서 소유하고 GetOpenFileName legacy cwd 동작에 의존하지 않는다. UI 코드에서 manifest 내용을 파싱하거나 source/bundle/journal을 읽지 않는다. 필터는 내용 신뢰가 아니다. 반환 경로는 UTF16 32767상한·로컬 절대 경로로 검증하며 worker 권한은 기존과 동일하다.

공유 인터페이스 후보: ManifestPicker::new(initial: Option<&Path>, title: &str, filter_name: &str) -> Result<Self, PickerError>; closer() -> PickerCloser; show(owner: HWND) -> Result<Option<PathBuf>, PickerError>; PickerCloser::cancel() -> Result<(), PickerError>. 원 COM object·apartment의 균형 있는 RAII와 같은 thread 전용 수명을 유지한다. ERROR_CANCELLED만 취소로 처리하고 setup/COM/invalid-path 실패는 성공으로 승격하지 않는다. FOS_FORCEFILESYSTEM/FILEMUSTEXIST/PATHMUSTEXIST/NOCHANGEDIR/DONTADDTORECENT, 단일 선택을 명시한다. 초기 값은 경로 힌트일 뿐 파일 권한이 아니다.

ID205 버튼은 idle·비확인·비종료 때만 사용할 수 있다. modal Show 동안 RefCell borrow를 보유하지 않는다. picker 활성 상태에서 worker/확인 요청을 차단하고 owner Close는 원 dialog cancel과 종료 의도 보관 후 Show 반환·COM 정리 이후 자연 종료한다. 성공 결과 적용 전에 같은 owner와 현재 상태/입력을 다시 확인한다. 자동 설치·기존 root 채택·재귀 삭제·global input/clipboard·visible desktop fallback은 없다.

## 인수 조건

private desktop에서 원 HWND/PID/creation·same dialog 소유권을 확인하고 Unicode 합성 파일 선택·취소·owner Close를 실제 시험한다. 취소는 네 입력/status/검증 eligibility를 보존하고 선택은101만 변경하며 no worker/root creation을 확인한다. 잘못된 로컬 경로/UNC/device/overlong 반환 및 오류는 기존 내용을 보존하고 중앙 오류문구를 표시한다. actual cwd/input desktop·원 ID/SHA와 자연 종료/Job0·desktop 복원을 확인한다. 기존 설치/재시작/확인취소/제거 회귀와 Windows fmt/Clippy/test/release를 최신 source에서 실행한다. 새 feature의 크기·런타임 비용을 기록한다. 물리 IME/reader·cleanOS·정식 installer와 전체P0 인수로 확대하지 않는다.

## 검증과 handoff

Git 상태는 2026-10-06에도 exit128 not-a-repository이며 .git을 보존한다. root가 이전 중단 뒤 실행 상태를 확인했고 기존 채집 프로세스는 종료됐다. 문서의 Proposed ADR0005를 유지한다. 프로젝트 코드 구현·빌드·native 선택 시험은 아직 하지 않았다. Microsoft Common Item Dialog 문서에서 STA 초기화·단일 filesystem 결과·cancel HRESULT·기존 옵션 보존을 확인했다. 범용 설치 패키지·업그레이드·서명·공개 배포는 별도 후속이다.

### root 통합

### 2026-10-06 첫 Windows 검증

실제 포맷 적용 exit0 이후 전체 검증의 fmt는 통과했다. Clippy는 installer_main의 raw pointer 동등 비교 한 곳에 ptr_eq 경고를 내어 exit101로 종료했다. 첫 실패 증거는 windows-msvc-native-manifest-picker-workspace-20261006-continue에 보존한다. root가 std::ptr::eq로 수정했으며 테스트·release는 첫 실행에서 미실행이다. 새 증거 폴더에서 전체 검증을 다시 수행한다.

원 main/catalog·draft SHA를 대조한 뒤 세 파일과 기존 windows-sys Com/Shell 선언 feature를 통합했다. UI v1은 Close/Release를 State borrow 중 호출하는 재진입 위험이 있어 통합하지 않고 v2로 수정했다. 모든 COM setup/Show/Close/final Release/CoUninitialize는 State borrow 밖에서 수행한다. 명세 버튼은 첫 입력 바로 다음 Tab 순서다. 원 dialog를 유지하는 closer Clone은 Rc 복제만 한다. 초기 폴더는 valid initial parent 또는 현재 exe parent이며 실패 시 Recent fallback을 하지 않는다. root는 shortcut 비참조 옵션과 Show 전후 실제 current_dir 불변 검사도 추가했다. 이는 독립 native 대상 CWD 값 채집의 완료를 의미하지 않으며 해당 observer는 미관측을 명시한다. 새 crate/schema/lock는 추가하지 않았다. Windows 빌드·OS 선택 시험은 아직 미완료다.

### 현재 전체 검증과 실제 GUI 실패 보존

retry whole은 실제 fmt/Clippy/test/release exit0, 850passed/0failed/13ignored, source593 exact map 전후·현재 일치, GNU/lock 불변을 root와 독립 담당이 검토했다. CPU80b657…/8,710,656B와 CLIbbcf66…/974,848B는 직전 인수본과 바이트 동일하며 설치 화면e9ec1a…/815,616B는15,872B 증가했다. COM/Shell 선언 feature만 추가했고 새 crate/lock/schema 변경이나 상주 thread는 없다. same STA modal 소유 수명·raw ABI를 cached Microsoft generated binding과 대조했다.

최초 choose GUI는 UIA AutomationId 중복 식별로 실패했고 typed selector 재시험은 UIA InvalidOperationException으로 실패했다. owner-close의 posted WM_CLOSE는 10초 내 종료를 확인하지 못했다. 세 실패는 실제 evidence와 소유 Job 강제 종료 사실을 보존하며 성공으로 쓰지 않는다. input desktop은 변경하지 않았다. 현재 UIA 호출 단계 계측과 별도 HWND 메시지 시험, modal owner Close 전달 방식을 확인한다. graph 최초 요청은 effective configuration 대신 원 plan을 넘겨 native 실행 전 거절됐고 원 whole effective configuration으로 실제3명령 graph258 채집을 완료했다. 사용자 시험용 ZIP은 생성·검토 중이며 아직 GUI 전체 인수를 주장하지 않는다.

### 종료 실패 분리와 선택 경로 canonicalization

UIA 단계 계측은 Open Invoke에서 InvalidOperationException을 관측했다. 별도 원 HWND 메시지 시험은 dialog를 닫고 실제101을 변경했지만 Shell이 D:\project를 D:\Project로 반환하여 case-sensitive fixture 인수가 실패했다. 원 파일 volume/ID/size와 정확한 Unicode basename 검증으로 observer 계약을 보완했다. 제품 경로를 소문자로 바꾸거나 case-insensitive 문자열만으로 원 파일 권한을 추정하지 않는다.

owner Close는 bounded SendMessageTimeout 전달8ms 이후에도 naturalexit 실패했다. 종료 의도→자체WM_APP17→COMClose로 분리한 새whole도850/0/13, 실제4명령·CPU/CLI바이트불변을root/독립검토했으나 동일ownerClose회귀는 실패했다. 이 실패를 보존하고 임시scalar WM_APP18 query로 handler단계/CloseHRESULT를 관측하는 진단 빌드를 준비한다. 진단은 최종후보 전 제거한다. 첫 previewZIP SHA551051…/13,059,848B/466entries의전체members·PE/원notice는root/독립검토했으나이후source변경전historical후보이며최종후보로게시하지않는다.

### 최종 소스 검증과 자연 종료

원COMClose의HRESULT0와rawShow미반환을실제scalar로분리했다. 원COM에서IUnknown_GetWindow로받은현재HWND를현재PID/STA/GW_OWNER와대조하고Close후원COM으로다시얻어같은창인지확인한뒤원창에만WM_CLOSE를게시한다. 이경로에서기존실패재현이naturalexit0/Job0로통과했다. HWND의영구identity/queue원자성을주장하지않는다. 공식근거는Microsoft IFileDialogEvents::OnFileOk·IUnknown_GetWindow이며private/native관측을별도기록한다. 자체Close의동기입력dispatch는재게시하고Close오류와게시실패는중앙catalog문구로안내한다.

임시WM_APP18/Atomic/phase계측은모두제거했다. 최종whole실제4명령exit0·850/0/13·source593전후/현재map과GNU/lock불변을root/독립검토했다. CPU/CLI불변, installer2af716…/817,152B. 최종owner-close native는성공했으나첫collector는owned_root누락KeyError였으며원실패보존후receipt를추가한재시험은REVIEW_REQUIRED까지완료했다. 선택/취소·root별도native자료·PNG·최종ZIP인수는이시점진행중이다.

### 최종 범위 인수

최종 source593의 Windows fmt/Clippy/test/release를 실제 실행했고 모두 exit0, 850 passed/0 failed/13 ignored다. 명세 선택·취소·owner Close 3사례와 재시작 확인취소·제거 2사례의 실제 private GUI 수집10실행, root 추가 native42명령 및 직접 PNG7장을 확인했다. whole 증거는 `windows-msvc-native-manifest-picker-final-workspace-20261006-continue`, 선택 root 증거는 `windows-msvc-native-manifest-picker-final-root-native-20261006-continue`, 재시작 root 증거는 `windows-msvc-native-manifest-picker-final-restart-root-native-20261006-continue`다. 실패한 이전 실행은 보존했다.

COM Close가 S_OK를 반환해도 private fixture에서 Show가 끝나지 않는 현상을 재현했다. 같은 COM object가 반환한 dialog HWND의 PID/thread/owner를 확인하고 Close 후 재조회하여 여전히 같은 창인 경우에만 WM_CLOSE를 게시한다. 자연 exit0/Job0을 실제 확인했다. 임시 phase/query 진단 코드는 최종 소스에서 제거했다. 기존 I/O worker/취소/확인 상태 계약은 유지한다. 새 crate/schema/lock 변경 없이 기존 windows-sys Com/Shell 선언 feature를 추가했다. installer817,152B는 이전799,744B 대비17,408B 증가했다. 별도 상주 worker는 없으며 새로운 installer 비용 측정 자체의 완료를 주장하지 않는다.

CPU SHA80b65756b8b5b0b9d84a4e1f250c5b371db15bafca695b8e26694966b369a01a, CLI SHAbbcf6632fefc57cd502e2c6bad25c681b99d009e328e480f832b5843769f42f9, installer SHA2af7165c976a90d4e693ac33ea70c1a0aeab6428b26f7ef719998645c341fe9a다. GNU/lock 불변, Proposed ADR0005 유지. 사용자 직접 UI/UX·물리IME/reader, 기본 경로·폴더 선택, upgrade/crash rollback·cleanOS·법적 전체 notice/서명·전체 P0는 후속이다.

### 최종 사용자 ZIP

최종 whole/graph를 입력으로 `python3 scripts/windows_msvc_candidate_package.py`를 실제 실행해 exit0을 확인했다. ZIP13,061,430B, SHAab920ff1ae595c5560985fc1c93fb9d315fdb39d2ebf517489f86f6559855d11, 466members다. root는 전체member SHA/CRC/크기/모드/고정timestamp와3binary PE/current source·whole/graph pin을 별도 검토했다. selected graph258의 누락은0이나 wholelock43누락·내장CRT/std/법적 전체coverage는 미완료다. 증거 `windows-msvc-native-manifest-picker-final-graph-20261006-continue/package-root-review.json`, [실행 안내](../development/windows-user-preview.md). 문서 검사의 첫 실행은 ZIP 생성 전 링크부재로 실패했고 생성 후 다시 검사한다.

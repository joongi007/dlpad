# S00: 진단 자료 경로의 파일 선택 대화상자

- 상태: in-progress
- 분류: UX
- 우선순위: P0
- 요구사항: R08, R13
- 담당/수정 범위: root — crates/app/src/accessible_diagnostics.rs, native picker와 worker 연결, catalog.rs; 공유 인터페이스는 root 조정. 구현 파일 목록은 후보 검토 뒤 확정한다.
- 선행 작업: 최신 설치·목록 native의 source 고정 검증 종료

## 사용자 결과

진단 저장 위치와 선택적 MPD 파일을 직접 긴 경로로 입력하지 않고 Windows 파일 대화상자에서 선택한다. 직접 입력도 유지한다.

## 계약과 범위

PRODUCT의 비CLI 작업 흐름과 DESIGN의 경로 선택·안정적인 입력 identity를 따른다. 실제 네 PNG 감사 /tmp/dlpad-diagnostic-current-visual-audit-20261012/REPORT.md에서 저장 경로의 텍스트 입력만 제공됨을 확인했다. 기존 설치용 picker는 독립 계약이며 무조건 재사용하지 않는다. UI thread 긴 I/O 금지, 소유 창·worker 취소·종료 ACK와 bounded completion을 유지한다. 대화상자는 draft만 변경하며 명시 저장/첨부가 기존 worker를 시작한다. 기존 파일 덮어쓰기 허용을 새 권한으로 해석하지 않으며 create-only 저장을 유지한다. Cargo/schema 추가는 root 검토한다.

## 인수 조건

저장 형식에 맞는 파일 선택과 MPD 선택, 취소·오류 시 기존 초안 보존, 닫기 중 선택 미적용·worker ACK, 새 파일 명시 저장·기존 파일 보존을 실패 경로와 실제 private GUI에서 검증한다. Light/Dark 작은 창 확대에서 경로·선택 버튼·오류·focus가 가려지지 않고 논리 ID를 유지한다. global input/visible desktop fallback은 없다.

## 검증과 handoff

현재는 감사 근거와 작업 범위 기록 단계다. 구현·native 실행·완료는 미주장이다. 현재1173 whole과 기존 설치 ZIP에 소급하지 않는다.

- 파일/인터페이스 분담: diagnostic_cache는 /tmp/dlpad-diagnostic-file-dialogs-20261012의 diagnostic_file_dialog.rs/diagnostic_file_worker.rs 후보만 담당한다. root는 diagnostic_path_picker.rs coordinator, accessible_diagnostics.rs 입력·busy/stop/poll, path_picker.rs owner 전달, accessible.rs 부모 연결, 양 entrypoint·catalog·문서를 담당한다. 같은 파일 동시 수정 없음. native Mode/Target은 ReportMarkdown/ReportJson/Mpd이며 root가 계약을 확정했다. 기존 snapshot 검토가 끝날 때까지 root 제품 수정은 보류하고 /tmp 통합 후보를 작성한다. 새 의존성/lock/schema 변경은 없다.

- /tmp 후보 작성 완료: diagnostic_cache의 native 두 파일은 shared Mode/AsyncFilePicker 계약을 제공한다. root의 coordinator·panel·parent 연결 후보는 /tmp/dlpad-diagnostic-{path,panel,parent}-picker-root.rs에 작성하고 pinned1.94 rustfmt로 문법/형식을 확인했다. queued selection은 draft만 변경하고 stop/cancel/oversized 결과는 기존 draft·report 보존을 검사하는 실패 경로 테스트 후보를 추가했다. 제품 통합·컴파일·실제 대화상자 인수는 아직 미실행이다.
- 잠재 비용: 대화상자 요청 시 일시적 thread/STA/proxy 각1개와 capacity1 completion을 추가하며 idle worker는 생성하지 않는다. 기존 Windows COM/Shell dependency를 사용하므로 Cargo.lock/schema 변경은 없다. 실제 PE/RAM/종료 지연은 실행 후 측정한다.

- root 통합 수행: diagnostic_path_picker.rs·diagnostic_file_dialog.rs·diagnostic_file_worker.rs 추가, accessible_diagnostics.rs에 경로 선택 버튼·초안 적용·busy/stop/cancel/poll 연결, accessible.rs 및 path_picker.rs에 원 Window owner 전달/호스트 닫기 연결, 양 접근성 entrypoint의 module 등록과 catalog10문구 추가. 선택은 저장/첨부를 자동 실행하지 않는다. 임시 완료·닫기 취소·과대 결과 초안 보존 회귀를 추가했다. pinned1.94 rustfmt 통과 후 실제 Windows 진단 cluster70228 실행 중이며 성공 여부는 미확정이다. 현재 README ZIP은 이 변경 전의1173 검증 후보로 유지한다.

- actual Windows 진단 cluster70228 exit0/source 보존: 양 UI harness에서 각각31개 관련 회귀 통과. 새 picker의 세 mode 실제 COM constructor(noShow), UTF16/local 결과 정책, 완료 mailbox, queued cancel, panel draft-only·stop ACK·과대 결과 보존과 기존 full input clip/포커스/preview/privacy/저장/MPD/최초 공개 회귀가 통과했다. 실제 dialog Show/selection/close는 이 테스트로 대체하지 않는다. 최신 전체82619 실행 중이며 native producer 후보는 diagnostic_cache가 /tmp에서 담당한다.

- 첫 전체82619는 fmt 통과 후 Clippy 두 경고로 exit1: 복제된 미사용 registered_proxy_identity_for_test와 poll의 중첩if. helper를 제거하고 poll조건을 동등한 let-chain으로 정리해 전체v2 47135를 실행 중이다. 원 실패 로그는 보존한다.
- root 추가 검토에서 renderer 오류 시 software_host::about_to_wait가 Candidate::poll_folder_picker만 호출함을 확인했다. 새 진단 picker completion도 이 경로에서 drain해야 하므로 해당 종료 경로 회귀와 수정이 필요하다. 일반 panel.poll 성공을 renderer 실패 종료의 인수로 대체하지 않는다. 현재 whole 완료 뒤 source 변경을 진행한다.

- 전체v2 47135 exit0: 43harness/1191 성공/0 실패/30 ignored, fmt·clippy·test·release 모두 통과. CPU9,762,304B SHA0121e60c236fb0d9c24e0216124bdf5568e2f4f36919b83106852421f8d2a888, 이전1173 후보 대비29,696B 증가. 이 snapshot에는 후속 renderer오류 회수 수정이 없으며 최신 전체 인수로 소급하지 않는다.
- renderer 회수 회귀 첫RED93572와 임시GREEN33105는 테스트가 bounded_font_tests에 삽입돼 Fixture/wait_finished 미해결 compile 실패였다. 결함 재현으로 계산하지 않는다. 기존 tests 모듈로 옮겨 원 미수정 poll 경로의 RED-v2 28117를 실행했다. 컴파일 실패/원본 로그는 보존한다.

- 실제 RED-v2 28117 exit101: 컴파일 후 원 Candidate renderer-error polling 경로가 diagnostic picker ACK를 drain하지 못한다는 assertion에서 실패했다. 원 panel/metrics는 별도 정리·ACK 후 실패를 반환했다. root는 rawclose 취소 뒤 Candidate::poll_folder_picker에서 panel.poll도 수행하도록 수정했다. GREEN-v2 10560 exit0/source 보존: 양 UI harness에서 queued cancel completion 회수·초안 보존·파일 미생성·원 panel/metrics ACK가 통과했다. 최신 전체v3를 실행하며 앞1191 검사를 최신 수정으로 소급하지 않는다.

- 최신전체v3 50034 exit0, 독립48288 exit0: 43harness/1193 성공/0 실패/30 ignored, fmt/clippy/test/release raw와 source/원 artifact 대조를 완료했다. CPU9,760,768B SHA a53c9ef3bc52b3761d12ee467523e5c0d45e1a9819d988d4f5da167545e1fe98. 최초1173 후보 대비28,160B 증가, 새dependency/lock/schema 변경 없음. 원 result/root-review는 windows-msvc-diagnostic-file-picker-whole-v3-20261012에 보존했다. 실제 Markdown 저장창 Light86255/Dark10541을 새 source/CPU로 실행 중이며 Json/MPD/cancel/close/modal pixels는 후속 실제 시험 범위다.

- 실제 Markdown GUI Light86255/Dark10541 둘 다 exit2: compile exit0 뒤 original app/worker proxy chain의 SaveDialog를 찾았으나 Exact native filename/save controls에서 실패했다. 원GetDlgItem1152/Action1 가정을 실제 소유 child HWND/controlID/class/TID/PID로 계측한다. 성공 인수 없음, 원 forced owned Job cleanup과 실패 로그를 보존한다. 프로젝트 runtime 결함인지 관측 selector/초기준비 문제인지 아직 미확정이다. diagnostic_cache가 별도 /tmp 후보를 준비한다.

- 원 대화상자 child 컨트롤128개/ID/class/PID/TID/text/enabled·UIA descriptor 및 원 PrintWindow PNG를 filename 조작 전에 기록하는 별도v2 observer를 실행했다(Light8601/Dark43529). 기존1152/Action1 요구는 유지하며 임의 fallback이나 제품 소스 변경은 없다. 실제 컨트롤 구조를 확인한 뒤 적절한 조작 경로를 선택한다. 최신1193 전체와 원CPU는 고정한다.

- v2 Light8601/Dark43529 exit2, root가각 dialog-before-filename-action.png와 실제child/UIA 구조를 직접 확인했다. filename은 FileNameControlHost ComboBox의 Edit1001이며1152가 없고Save1은 존재했다. ID1001에는주소Toolbar도있어 role/class/부모관계를함께검증해야한다. 기본diagnostic-report.md·Markdown필터·Save/Cancel 시각을 scoped확인했다. exact host→Edit1001/nativeparent/class/PID/TID/readback을검증하는v3 Light86396/Dark91656를 실행 중이다. 제품 source/Cargo 변경 없이 관측 selector만 실제 구조로 수정한다.

- v3 Light86396/Dark91656는 GUI 시작 전 C# compile CS1061로 실패했다: managed TreeWalker에 GetParentElement 메서드가 없다. 이 실패는 실제 dialog 조작 실패가 아니며 원 nativeguard/compile로그를 보존한다. 같은 직접 부모 대조를 제공하는 실제 managed API로 고쳐 별도v4를 준비한다. root product source/CPU/1193 전체에는 변화가 없다.

- v4는 공식 managed TreeWalker.GetParent와 기존 프로젝트 코드 근거로 메서드명1개만 교정했고, 동일한 parent equality/nativeclass/ID/PID/TID/readback/원guards를 유지한다. compile/gui/validation 실패 단계 보고를 분리했다. Light20837/Dark67071 실제 실행 중이다. 원v1~v3 실패는 보존하며 성공으로 바꾸지 않는다.

- v4 Light20837/Dark67071 실제GUI exit2: compile과 exact filename host/Edit1001/parent/class/PID/TID 및Unicode fresh readback은 통과했으나 modalreturn+draft eligibility 단계에서 실패했다. 원filename-semantic-route와failed Job cleanup은 보존했다. modal이 닫혔는지·실제export draft·버튼/status와after PNG를 v5에서 분리 관측해 action 미전달/경로 canonical spelling/제품 오류를 구별한다. Shell주소 D:\Project와원입력 D:\project의case차이는 가설일 뿐이며 대조를 단순caseblind로 완화하지 않는다.

- v5 Light82213/Dark88887 실행 중. 원BM_CLICK/15초/literal equality는 그대로 두고 modalalive/원ownerchain/실제exportdraft·버튼/status·app/modal afterPNG를 최종판정전에 수집한다. 조건을 먼저 완화하지 않고 action미전달과반환표기/완료오류를구별한다. 원v4는compile와filename입력/readback은성공,GUI완료는실패였음을유지한다.

- v5 실제 after-state 양테마: modal_alive=false, export/browse/save enabled=true, 선택만으로fileexists=false. rawselected D:\project와native반환draft D:\Project의표기차이가literal equality실패원인이다. root가양테마 finalappPNG를 직접확인해경로/선택/저장행가시성을scoped기록했다. 명시export/Job자연종료는아직실패이므로완료로주장하지않는다. exactleaf Ordinal+부모원readonlyHANDLE의volume/128fileID/nonreparse 대조로동일freshownedtarget을검증한후실제반환draft로후속검사를하는v6후보를위임했다. 전역Windowscase-insensitive가정이나제품입력변조는도입하지않는다.

- v6 Light45004/Dark90510는 input-copy triple tuple 오류로 native/gui미시작 exit2였다. root가 별도v7에서 두 helper를 각각정확(source,name)쌍으로분리하고pair형태를검사했다. 원v6결과보존.
- v7 Light26715/Dark73295 exit0, 독립84973/23123 exit0: 실제parent native128ID/volume/nonreparse/exactleaf·filenamehost/원PID/TID/Unicode readback·선택만파일미생성·명시Markdown저장·기존파일재저장거절/원bytes보존·privacy/Metadata7/readonlyDB/Job자연0·원source를검증했다. root가draft/save PNG4개를직접확인했고각evidence에independent/pixel리뷰를복사했다. 저장완료안내는button근처화면에안보여S00-diagnostic-save-feedback-20261012로후속기록했다. JSON/MPD/cancel/close/확대완료는남으며이작업을완료로바꾸지않는다.

- 추가agent후속요청은model capacity로종료해root가JSON producer를직접 /tmp/dlpad-diagnostic-json-picker-native-20261012에작성했다. 원MDv7불변·원owner/parentnativeID/readonly/Job/currentwhole/privacy 기준을유지하며JSON형식선택·.jsonleaf·직접JSONparse 검증으로분리했다. Python syntax와inputcopy tuple형태검사통과. actualJSON Light19994실행중이며MPD/cancel/close는아직남는다.

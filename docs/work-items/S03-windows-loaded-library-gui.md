# S03: Windows 합성 10k 라이브러리 실제 GUI 인수 준비

- 상태: in-progress (collector 초안, native 미실행)
- 분류: validation
- 우선순위: P0
- 요구사항: R03, R06, R11, R13
- 담당/수정 범위: metrics_recovery — `/tmp/dlpad-windows-library-loaded-probe.cs`, 이 문서만
- 선행 작업: root 최신 CPU immutable release/source freeze 종료, 기존 Windows privateDesktop UIA host

## 사용자 결과

실제 Windows960×640 네이티브 목록에서 합성 10,000개 파일을 읽고 다음128개 페이지, 수동 추가/삭제/이름 변경 반영, 잘못된 저장 루트의 원 목록 보존을 검증하는 초안이다. 아직 실제 GUI 인수가 아니며 Linux 기존 probe 성공을 Windows 인수로 확대하지 않는다.

## 현재 코드 계약

R03은 첫 시작/재진입/수동/파일 이벤트 조정과 외부 변경 반영 및 출처 불명 파일의 로컬 파일 표시를 요구한다. ARCHITECTURE는 접근 불가와 삭제를 구분하고 원 file ID 기반 rename 연결이 가능한 경우만 추정한다. `accessible_library.rs` worker가 reconcile+library_page(128)를 호출하고 command/results 채널 각1개 및 dirty watcher debounce로 UI thread 밖 I/O를 유지한다. UI는 show_rows 가상화로 visible row만 노드가 될 수 있으므로 UIA 노드 수를 DB 전체10k 또는 resident128의 증거로 쓰지 않는다. root canonical/data 겹침 거절과 location 실패는 원 DB/목록 유지가 필요하다. scan stop은 UI/Drop에서 join하지 않고 acknowledgment를 기다리는 기존 정책이다.

## collector 초안과 역할

기존 windows_runtime_tools_observer.cs의 원 process creation/HWND/Job/privateDesktop/UIA/bounded tree/PNG/원 window pointer 및 WM_CLOSE 패턴을 복사했다. 별도 새 CLI는 immutable CPU exe/expected SHA/fresh output이다. OS-temp 새 고유 root와 초기 SD protected SID 디렉터리를 생성하고 정확10k 3byte 합성 media 파일과 외부 sentinel을 create-new한다. 실제 media/Documents/account/clipboard/global input/visible fallback은 없다. root 이후 Python/SQL의 원 library.sqlite read-only 조회가 DB count/order/missing/fileidentity를 별도로 확인해야 한다. collector는 source-only assertion으로 이를 대체하지 않는다.

현재 draft는 실제 UI 파일 목록 선택→media-00000 actual label→다음128→media-00128 actual label과 이전row 부재→소유 파일 rename/delete/add→수동 새로고침→추가/rename actual row와 삭제row 부재를 요구한다. 정상 root 적용 후 data 겹침을 Edit에 입력하고 location failure 및 원 loaded row 보존을 요구한다. 단계별 UIA/PNG를 보존하고 sentinel을 확인한다. 새 source는 컴파일하지 않았다. 기존 helper의 UIA pattern 적용 가능성 및 action/scroll의 실제 지원은 첫 native에서 확인해야 한다.

## 아직 남은 acceptance

- 실제 DB10k 및128 페이지 경계/원 fileID rename 유지: root read-only SQL/DBsnapshot 필요.
- watcher-only 자동 변경 및 재진입: 수동 새로고침과 분리한 후속 단계 필요. 현재 false 표기.
- scan CloseWait: 실제 스캔 중임을 원 lock/worker 증거로 확인하고 WM_CLOSE→종료 대기→owned DB lock release→원 자연 종료를 검증해야 한다. 단순 파일10k 생성 후 즉시닫기를 scan-inflight라고 주장하지 않는다. 현재 false 표기.
- malformed root: data overlap은 현재 초안 범위다. 없는 root/regularfile/reparse/권한 실패와 원 DB bytes 보존은 별도합성 case 필요.
- 비용: loaded10k scan/idle CPU/RSS는 아직 미측정이다. empty-library idle 측정으로 대체하지 않는다.

## 검증과 handoff

/tmp source SHA256 `79d9ff98f71968465256220bfd71f93aa794f6c6ed550318cabe2111ae41b23a`.

실제 수행은 코드 읽기와 /tmp 초안 생성뿐이다. C# compile/native GUI/Cargo/프로젝트 Rust 및 기존 scripts 변경0. rootfeature39895 중에는 실행하지 않는다. root 첫 compile 및 private native 관측에서 actual loadedrows를 우선 확보하고 실패 raw를 보존한다. 전체 R03 완료나 scanclosewait 인수는 아직 주장하지 않는다.

## root 검토 교정 및 실행 runner staging

첫 초안은 기존 도구 workflow의 중간 Invoke를 기준으로 잘라 도구 loop가 남아 있었다. 이를 실패 검토로 기록하고 도구 탭/selector/activation/install 안내 검사 전체를 제거했다. 현재 실제 실행 흐름은 Activate→파일 목록만이다. initial source는 native 인수로 쓰지 않는다.

C# Main은 정확3인자 `immutableCPUexe expectedSHA freshoutput`이며 normalized absolute/reparse 검사 후 원 binary를 FileStream shareRead로 GUI 전체 동안 유지한다.256MiB 이하 원 SHA 전후를 비교하고 결과 JSON의 최상위 status로 판정한다. 원 HANDLE 자연 exit code0 및 Job0를 요구하고 강제 Job 정리는 FAIL을 유지한다. 신규 output 생성이 성공한 경우만 failure.json을 create-new 기록한다.

최종 C# staging SHA256 `4ed3c6ed000f011387500c5f0f540a3624a86516946a03f79ee45a2291eedf4f`. 별도 `/tmp/dlpad-windows-library-loaded-runner.py` SHA256 `dbfc4f299d08d82bf99f755286c3a4fe3e3d54352662467ca33ccefa10398968`는 root가 사전 컴파일한 collector를 기존 windows-cli-host(180초 deadline)로 실행하는 초안이다. Python CLI는 --project/--binary/--collector/--collector-source/--output/--sha를 받는다. expected frozen binary SHA는 root가 전달한다. source snapshot/observer source+exe 및 binary SHA를 전후 비교하며 inputs-before를 실행 전에 저장한다. 원 자체fixture의 expected10k names/각3byte SHA(추가파일만7,8,9)/sentinel을 대조하고 종료 후 원 DB를 read-only로 열어 present10k/변경 first129 행을 확인한다. fixture hashes와 DB review는 별도 원시 결과다. scan 중128 resident/full native renameID 유지 증거는 아직 별도 필요하다.

실제 수행: `python3 -m py_compile /tmp/dlpad-windows-library-loaded-runner.py` exit0. C# static 읽기에서 도구 실행 workflow 부재/3args 일치를 확인했으나 native compiler 검사는 미실행이다. log는 파일로 받으며 proactive bytes quota는 없고 host 시간 상한에 의존한다고 결과에 명시한다. rootS10suite6462/batch1720 중에는 native 실행하지 않았다. 프로젝트 Rust/Cargo/scripts 변경0.

### 탭 접근·환경·bounded host 교정

실제 accessible.rs853은 library.show를 호출하고 accessible_library.rs는 SAVE_ROOT Edit/APPLY_ROOT를 그 탭에 직접 제공한다. 다운로드 탭895는 SAVE_ROOT field만 있고 APPLY_ROOT 버튼이 없으므로 다운로드→Apply는 존재하지 않는 동작이다. source 근거에 따라 라이브러리 탭의 원 Edit/Apply를 유지하고 실제 enabled 여부를 요구한다.

runner는 기존 windows_current_workspace_probe의 configure/run을 그대로 호출하도록 교정했다. 원 host proactive8MiB log bound/시간deadline/owned control 취소/privateDesktop 표시/inputdesktop 확인/source snapshot을 재사용한다. 정확 native schema1/mode/status/원 exit0/자연Job0/desktop/input/loadedrows/원binarySHA를 타입까지 대조한다. 출력은 새 프로젝트 evidence 아래만, binary/collector는 프로젝트 안만 허용한다. fixture는 configure의 원 tools/tmp 바로 아래 dlpad-library-owned prefix로 제한하고10k entry/3bytepayload/별도sentinel 및 DB64MiB 이하 read-only 조회를 요구한다. source/observer/binary 전후 대조를 유지한다. 사용자 자료 위치나 기존 출력은 쓰지 않는다.

최종 staging C# SHA256 `b92d6b7cbd9b3c293838143b4a7b0c063be08d47485d49af0d8b10ba17133d4c`; runner SHA256 `f775d0c76e939a7bdcdda3cdbcb5b4eeb39e5121cd63e36942076457f50e44d2`. `python3 -m py_compile /tmp/dlpad-windows-library-loaded-runner.py` exit0. C# nativecompile/GUI는 미실행이며 rootS10suite/batch와 겹치지 않았다. 초안은 mutable root currentbinary를 임의로 선택하지 않으며 --sha에 root가 확인한 frozen exact값을 받아야 한다.

## 첫 실제 Windows 실행 실패와 단계 보존

root의 `windows-library-loaded-20261004-resume/gui`는 C# compile PASS 후 실제 page1/page2/manual-refresh PNG 및 UIA를 확보했지만 `Library root controls inaccessible`에서 실패했다. 원 Job empty/private desktop close는 확인되었으며 전체 인수는 FAIL을 유지한다. row 표시가 확인된 것만으로 worker scan 완료와 root edit 활성화를 같은 시점이라고 단정하지 않는다.

후속 /tmp C#은 exact 원 root Edit/Apply enabled를50ms 간격으로 최대10초 기다리고 원 HANDLE identity를 반복 확인한다. 기존 전체 UIA worker90초 상한을 유지하며 초과는 실패다. wait 이전/이후의 exact controls enabled JSON을 남긴다. 원 현재루트를 불필요하게 Apply하여 또scan하는 동작을 제거하고 invalid root Fill 이전 및 Apply 이전에만 bounded enabled wait를 수행한다. 원 scan 완료 시점이나 idle 비용 인수로 확대하지 않는다.

최종 staged C# SHA256 `70f559c9629b108960f9be4bd4fff8c7ae97a9df6fbcba565a440246fe1cfb82`. runner source는 직전 staging SHA 유지다. 새 C# compile/native 재실행은 아직 미실행이며 root가 수행한다. 프로젝트 published collector 수정 없음.

## loaded runner post-review 비용 교정

root library-ready native는 GUI 및 invalid-root PNG 인수를 통과했으나 최초 runner의 fixture 검토는10k파일마다 전체 ancestor 검사와 반복 SHA/stat를 수행하여 outer200초 TimeoutExpired로 실패했다. 원 GUI cleanup은 종료된 상태이며 이 후처리 실패를 native GUI 실패로 바꾸거나 성공으로 숨기지 않는다. root가 기존 nativeproof의 독립 post-review를 수행 중이며 GUI를 재실행할 필요는 없다.

/tmp runner는 media ancestors를 한 번만 확인하고 os.scandir1회로 최대10k entries를 제한한다. 각 entry는 no-follow type 검사 후 O_NOFOLLOW/CLOEXEC로 원 파일을 열어4byte만 읽는다. expected3byte exact 비교와 해당 bytes SHA를 한 번 계산하여 저장한다. 전체fixture review120초 deadline을 매entry에서 확인하며 native180초 budget은 그대로다. directory/file namespace 경합 전체 원자성을 주장하지 않는다. DB는 루트 column을 최다2개 읽어 원media canonical Windows 경로와 정규화 비교하고 정확한 원root parameter를 넣어 present count/first129를 조회한다. 다른 root 집합을 전체count에 섞지 않는다.

최종 runner SHA256 `41e2b58959736bf3b6737b0d2c0f82e47225e7fc1578f34bf44f33733ee948dc`. py_compile exit0만 실행했고 native 재실행/프로젝트 source 수정은0이다. 새 효율화 actual runtime과 독립postreview 성공은 root 결과를 기다린다.

root 독립 postreview는 os.scandir 단일 순회와 각3byte/최대4byte 읽기로18.73초에10,000개 원fixture 내용·이름·sentinel 및 read-only DB10,000행을 확인했다. 실제 invalid-root PNG에서 오류안내와 추가/rename 행 보존을 직접 확인했다. windows-library-ready-20261004-resume/root-review.json에 기록했다. 원native PASS와 wrapper postreview200초 timeout을 구분하고 기존 결과는 수정하지 않는다. 효율화 runner를 별도 통합했지만 아직 그 runner의 fresh end-to-end 인수는 아니다. watcher-only/native rename identity/loaded cost/scan closewait는 남는다.

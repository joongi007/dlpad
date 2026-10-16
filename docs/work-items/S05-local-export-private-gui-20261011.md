# S05: 구간 저장 실제 private GUI

## 2026-10-12 실제 진행 중 취소 보강

cancel native 수집은 REVIEW_REQUIRED로 종료했다. 원 receipt에서 running/cancel-invoked/completed-preservation-label=true와 success=false를 확인했다. root가 actual range0..10000, 원 raw Job natural lifecycle/currentsource/모든tool·fixture pins를 독립 대조하고 export-final.png를 직접 봤다. 완료 우선 경합이 아닌 실제 취소 결과 인수다. 긴 결과 경로는 현재 source가 truncate+hover 상세를 사용한다. 캡처만으로 overflow 결함을 확정하지 않으며 실제 geometry/hover 확인이 남는다. close-stop은 별도 output에서 실행 중이고 CloseWait/direct ACK 미완료를 취소 성공으로 대신하지 않는다.

root가 `/tmp/dlpad-local-export-inflight-current` 후보 diff와 기존 실제 FFmpeg/ffprobe pins 및 소유 fixture manifest를 확인했다. 최신 whole1101 source의 실제 합성4K30 10초 fixture(3,432,345B/SHAe778a4fbd983d8c9a8474d64194bb478260c27e7dd66a181ca9afccaf3eccbf4) 전체0..10000 범위로 cancel 시나리오를 실행한다. 기존 성공 시나리오1100..1800은 유지한다. 원 running 관찰/취소 요청/실패·보존 안내/success=false가 모두 인수 조건이며 completion-won race는 실패로 보존한다. 제품 지연·가짜 barrier·실제 사용자 영상은 사용하지 않는다. 원 namespace·파일SHA·private desktop·natural Job 종료 검증을 유지한다. 닫기/CloseWait는 별도 fresh 시나리오이고 아직 완료로 표시하지 않는다.

- 상태: in-progress
- 분류: test
- 우선순위: P0
- 요구사항: R04, R05, R08
- 담당/수정 범위: diagnostic_inputs /tmp 후보; root scripts/windows_msvc_local_export_collect.py, scripts/windows_local_export_private_probe.cs, scripts/test_local_export_collect.py 통합
- 선행 작업: S05-local-range-export와 현재 Windows 전체 검증

## 사용자 결과

파일 목록의 합성 영상 선택부터 범위 입력과 새 구간 저장까지 실제 native UI를 확인한다.

## 계약과 범위

사용자 데스크톱과 분리한 가상 화면만 사용한다. 원본 SHA와 원 프로세스 Job/자연 종료 guard를 보존한다. source/tool은 256 MiB 이하 스트리밍으로 검증한다. 실제 저장 output의 hardlink 정체성과 원본 보존, 성공 glyph PNG를 확인한다.

## 인수 조건

1100~1800ms 실제 UI 입력/readback, 성공 파일과 보존 stage 동일 NativeID, 길이·파란 프레임 독립 검증. 실제 running을 못 잡으면 취소·CloseWait 성공으로 보고하지 않는다.

## 검증과 handoff

하위 Python 7건 통과; root 독립 실행과 C# native 검증 진행 중이다. 물리 IME/reader와 사용자의 디자인 최종 평가는 대체하지 않는다.

2026-10-11: 실제 local-ready 화면에서 URL 입력은 숨겨지므로 수집기의 활성 작업 source gate를 다른 영상 열기 Button으로 교정했다. 창 높이 확대 없이 기본960x640을 유지한다. 기존 close scenario는 돌아가기 관측이며 in-flight 종료 ACK 완료로 주장하지 않는다.

실제 default-ui-final 시험은 local Prepared red frame·기본960x640·시작1100/끝1800 입력 각32px와 SHA readback을 확인했으나 저장 뒤 ElementNotAvailableException으로 FAIL이다. owned Job 강제 정리가 발생했으므로 성공/자연 종료/workerACK를 주장하지 않는다. provider 요소 수명과 재조회 문제를 조사 중이며 원 실패 JSON·PNG를 보존한다.

fresh-provider 시험은 이전 요소 수명 오류를 넘겼으나 PNG 수집 뒤 export가 이미 끝난 것을 running sample로 판단하여 navigation gate에서 FAIL했다. 수집기를 완료 경쟁에 맞춰 교정: enabled 관측 시 원 cancel action이 여전히 활성인지 다시 확인하고, 활성 중 gate 위반은 계속 거절한다. 이 관측은 제품 gate 성공/실패의 원자적 증명이 아니므로 in-flight 증거와 분리한다. terminal-race-aware 재실행 중.

terminal-race-aware 실제 GUI 성공: 원 source SHA 보존,1100..1800ms 입력,clip/stage 같은nativeID·1020B·동일SHA,worker ACK 뒤 원자연종료0/Job0. root final PNG 직접 확인. 별도 GUI-created clip metadata/decode는0.7초와blue RGB 통과. actual in-flight cancel/CloseWait 종료ACK는 별도 남는다.

추가실패경로: close-stop 수집기를 root 통합했다. 실제running관측·원Close확인title·중단후종료1회·CloseWaitlabel·natural0/Job0와원본보존을모두요구한다. directworkerACK관측과naturalexit에서의productiongate추론을구분하며라벨을못보면FAIL한다. collectorguard10회귀PASS. 4K60fps2초합성source≤1MiB를실제pinnedFFmpeg로준비중이며 실제취소/CloseWait인수는아직남는다.

실제4K fixture준비:60fps/q3는1.34MB,60fps/q8/GOP120은13MB,30fps/q8/GOP120은6.5MB로collector1MiB상한을넘겼다. 이전산출물과원natural0증거는보존. 기본GOP/q3/30fps/2초는755020B와SHA b12b1afc43f038f78f69b2f51bf05d70a2247739d2ff30d70f9dbd22cd64c7bf로상한내생성통과. 파일명4k60은실험명잔재이며실제argv/rate는30fps이다. 추가크기상한이나제품codec변경없음. 실제GUI취소시험진행중.

actual-4k-cancel은실제running중cancelinvoke뒤완료가경쟁에서이겨성공clip/stage가보존되고natural0/Job0였다. 수집기의REVIEW_REQUIRED를취소완료로승격하지않는다. root범위는완료/취소경쟁과원본보존이며completed cancellation=false이다.

actual-4k-close-stop은실제닫기title·STOP_EXITinvoke뒤privateprovider폐기중 E_UNEXPECTED로FAIL이었다. 원process/job는자연0·Job0이며강제cleanup은없었다. collector예외범위를E_UNEXPECTED+원process종료확인때로만좁혀providerdestruction경쟁을처리하되CloseWait라벨관측필수는유지한다. 현재fail을인수성공으로승격하지않는다.

close-destruction 재실행은STOP_EXIT뒤HWND폐기가processsignal보다먼저일어나Owned HWND/process identity Win32 1400에서FAIL이다. 원Job0/자연종료/desktop보존은확인됐으나CloseWait라벨·직접ACK성공은보고하지않는다. 원processHANDLE을기다리는observer수명분리후보를검토한다. 반복실패근거는보존한다.

observer 종료계약교정: STOP_EXIT뒤원HWND가폐기되면UIA재조회중단,원processHANDLE/GetProcessTimes생성identity를검증하며120초boundedwait한다. E_UNEXPECTED/ElementNotAvailable도원HWND부재또는원processsignal확인때만처리. CloseWaitlabel 실제관측필수와직접ACKfalse는유지한다. 새native검증예정.

original-handle-close 원native는STOP_EXIT·natural0/Job0·원본보존을통과했으나짧은완료경쟁으로CloseWait라벨을실제관측못해Python인수FAIL을유지했다. root한정범위는STOP_EXIT정상종료이며completed cancellation/directACK/CloseWaitlabel을성공으로쓰지않는다.


### 진단 viewport 최종CPU의 실제 저장 재검증

`windows-msvc-diagnostic-final-local-export-ui-20261011` 실제privateGUI1100..1800ms구간 저장PASS. root는원rawJob0/자연종료/전체pin/source보존·clip/stage1,020B SHA24584e7dd7e2eb99c71567b48497a2740e20a32dfd1524b181ea736c2f4de0e8·samefile·finalPNG를검토했다. `windows-msvc-diagnostic-final-local-independent-decode-20261011`에서실제저장clip의0.7초duration/bluepixel을독립ffprobe/FFmpeg로확인했다. 물리pointer/IME와inflightcancel완료/CloseWaitwaitingframe는이시험의인수범위가아니다.

현latestwhole-fifth1089/CPU294184…에서기존성공GUI증거와현재engine/UI/worker공통source6개SHA동일을확인했다. 남은인수는완료전취소·CloseWait이며기존race를재시험성공으로합산하지않는다. root가실제10초4K30합성source및긴선택구간실험을준비한다. 새실패시나리오시험입력만16MiBbounded로허용해실제encoding관측시간을확보하며기존successfixture1MiB/product구간10분/output256MiB/threads1/시간상한은유지한다. observer/collector후보는diagnostic_inputs /tmp전용, root가actualtool/pins/원Job/인수검토한다. 현재CI동결중제품source수정없음.

## 2026-10-12 실제 저장 중 종료 관측 실패 보존

`current-local-export-inflight-close-stop-20261012`의 실제 0–10000ms 합성 fixture에서 실행 중 Stop/Exit와 원 프로세스 자연 종료(0)는 관측했다. 그러나 `gui/local-export-result.json`의 `closewait_label_observed=false`, `closewait_inflight_verified=false`여서 Python 인수 검사는 실패했다. 직접 worker ACK는 관측하지 않았고 제품의 자연 종료 gate에서만 추론했다. 종료 대기 UI 인수 완료로 처리하지 않는다. 강제 대기·가짜 barrier를 추가하지 않고 실제 대기 안내 관측과 종료 안전성의 요구 범위를 대조한다.

## 2026-10-12 미관측 원인과 다음 직접 관측 계약

주 담당이 `local_export_ui.rs`의 poll/show_status와 worker 원 stopped ACK를 읽어 진단을 대조했다. busy인 동안 저장 중, busy가 끝나고 worker가 남아 있는 동안 종료 확인을 표시한다. completion과 stopped ACK를 같은 poll에서 소비하면 worker가 제거되어 종료 확인 문구가 한 프레임도 그려지지 않을 수 있다. R08은 특정 안내의 최소 노출 시간을 규정하지 않는다. 현재 자연 종료를 직접 ACK 증거로 승격하지 않으며 기존 label 관측 실패도 유지한다.

후속 담당 diagnostic_cache는 /tmp에만 기본 OFF·명시 opt-in의 bounded numeric lifecycle 계측 후보를 준비한다. 담당 후보 파일은 local_export_worker.rs/local_export_ui.rs/accessible.rs, 주 담당이 통합한다. 원 completion 소비·RAII ACK·worker 제거 조건·Close 전이의 직접 순서를 관측하며 URL/경로/비밀 출력·minimum wait·barrier·원 상태 순서 변경을 도입하지 않는다. 인수는 원 process identity, 원 정리 ACK와 창 종료 순서, 제한된 로그 및 opt-out 비용을 대조하는 것이며 label 캡처 실패를 자동 성공으로 바꾸지 않는다.

## 2026-10-12 계측 후보 통합

주 담당이 원 3파일 SHA와 패치를 대조하고 Windows 전용 모듈 호출에 cfg(windows)를 추가한 뒤 bounded numeric phase 계측을 통합했다. first-paint-fixture와 정확한 DLPAD_TEST_EXPORT_ACK_TRACE=1에서만 프로세스 전체 128행을 출력하고 일반 빌드에서는 no-op이다. 원 release stopped store 이후 phase4는 직접 worker ACK이며 seq는 원자 번호 할당 순서일 뿐 stderr 전체 사건 순서가 아니다. 상태 전이·대기·barrier·의존성·Cargo.lock 변경은 없다. 새로운 소스 전체 검사와 명시 feature 실제 실행은 아직 남아 있으며 과거 1,101/설치 ZIP 근거를 계측 소스에 소급하지 않는다.

별도 feature build windows-msvc-export-ack-feature-build-20261012의 실제 네 검사와 원 source/artifact/pins를 주 담당이 독립 검토했다. cpu,first-paint-fixture 1,121 passed/0 failed/28 ignored, immutable CPU9,700,352B/SHA850134b7822033e10d8aa782fff974872fd88b1ab09b1885067a4ee3963545cc다. 기본 CPU·CLI·GNU 원본·lock 등11핀을 보존했다. 계측 빌드를 일반 릴리스 비용/GUI 성공으로 소급하지 않는다. current-local-export-direct-ack-close-stop-20261012에서 원 0–10000ms 합성 fixture와 도구 SHA를 고정해 실제 직접 ACK 수집을 실행했다. 원 CloseWait label guard는 유지하고 직접 계측은 별도 진단 receipt로 검토한다. 아직 native terminal 결과가 없다.

## 2026-10-12 실제 직접 ACK 검토 결과

current-local-export-direct-ack-close-stop-20261012는 원 CloseWait label guard에서 FAIL로 종료했다. 별도 계측은 원 stderr360B/8행·stdout0B/natural EOF이며 phase1(busy1/stopped0)→phase2(cleanup_uncertain0)→phase4(원 stopped store 뒤 직접 ACK)→phase3(stopped1인 completion 소비)→phase5(stopped&&!busy 제거 gate)→phase6(원 전체 close gate) 기록을 관측했다. 주 담당이 원 compile/native 로그의 Job guard, 현재 source,39원본핀과 bounded stream SHA,8개 고유 seq 및 단계값을 독립 대조했다. root-review.json은 PASS_SCOPED_DIRECT_WORKER_ACK_AND_CLOSE_GATES다. stderr의 스레드 간 전체 사건 순서·닫기 뒤 publication·기본 release의 대기 문구 표시를 증명하지 않으며 원 UI FAIL은 유지한다. 빠른 정리 뒤 안내가 렌더링되지 않는 경우와 ACK 없는 종료를 구분할 직접 증거를 얻었다.

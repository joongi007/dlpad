# S11: 설치 완료 후 앱 실행 안내와 안전한 실행

- 상태: completed
- 분류: feature
- 우선순위: P1
- 요구사항: R08, R11
- 담당/수정 범위: 주 담당 통합; installer_batch는 /tmp candidate 전용. crates/app/src/installer_main.rs, installer_catalog.rs, 신규 installer_launch.rs 및 ADR/CONTEXT/본 기록.
- 선행 작업: 등록 설치와 typed launch_explicit_selection 검증, 최신 v9 설치 검수 후 source 동결 해제

## 사용자 결과

설치가 끝나면 설치된 앱 위치와 실행 방법을 바로 알 수 있고, 안전한 실행 버튼으로 앱을 시작한다. 설치 창을 닫아도 실행한 앱이 함께 종료되지 않아야 한다.

## 계약과 범위

현재 UI는 설치·중지·재검증·제거만 제공한다. typed 실행 API는 실행 파일과 namespace identity/hash를 확인하고 suspended child를 소유 Job에 연결한다. Job/OwnedSelectedProcess 해제 시 child가 종료되므로 무소유 spawn이나 detach를 도입하지 않는다. 창 종료와 supervisor 종료를 구분하는 bounded controller 및 오류/취소 경로가 선행되어야 한다. 공유 schema/Cargo 변경은 주 담당 승인·통합 범위이다. Default desktop 자동 실행·사용자 데이터 사용·ShellExecute fallback은 제외한다.

## 인수 조건

정확히 등록되고 재검증된 앱만 실행한다. 준비되지 않은 설치, 바뀐 실행 파일, 중복 클릭, launch 실패는 안내하고 child를 남기지 않는다. 실행 성공 후 설치 창을 닫아도 owned supervisor가 자연 종료까지 child 소유권을 유지한다. 오류·취소·WM_CLOSE 의미를 회귀 검증한다. 격리 화면과 합성 데이터로 실제 실행을 확인한다.

## 검증과 handoff

2026-10-13 설치 화면 캡처와 코드 감사에서 실행 affordance 부재 및 수명 의존성을 확인했다. /tmp/dlpad-installer-launch-affordance-candidate-20261013 에 감사와 수명 대안이 있다. 구현·native 인수는 아직 미완료이며 실행 버튼을 완료로 보고하지 않는다. 기존 v9 소스·설치 검수와 충돌하지 않도록 candidate 작업만 병렬 진행한다.

후보controller/main/catalog 구현을Root와windows_auto가독립검토했다. 전체7test는미실행이다. confirmed launchfailure outcome을main에서버리는문제를발견해fresh v2 후보에서원outcome/exitcode보존·새실행명시초기화·close최종판정을수정중이다. 현재v10에는해당기능이없고 source채택은검토후일괄3파일로한다. [Proposed 수명 결정](../adr/0009-installer-launch-supervision.md).

Root가fresh v2 controller/main/catalog 3파일을읽고 sourcebase2 SHA와11개후보test·원source/3binary를확인한후일괄채택했다. 이전v10 source map655 및바뀌는원2파일/3binaries는 /tmp/dlpad-ui-v10-before-installer-launch-20261013에보존했다. 새source656이며workspace/lock/dependency 변화없다. Windows windows-msvc-installer-launch-whole-20261013 fmt/clippy/test/release를실제로시작했고인수는결과및native시험후확정한다. catalog는실행이전·실행시도후·원nonzero 종료코드를구별하는bounded문구를제공하며main이모든failedCompletion을cap1마지막receipt로보존한다. explicitacceptedretry만confirmedreceipt를clear하고unknowncapability는유지한다.

통합Windows fmt/clippy/test/release 실제PASS:1,295/0/30,11신규supervisor/실제mainreceipt정책 회귀 포함. Root가43harness/원4raw/source656hashmap/3artifact/원GNU·lock 대조PASS를확인했다. CPU0093b3b7…9,836,544B와CLIbb5976d8…978,432B는v10과정확히같다. 설치실행파일0a26effb0dcff63d2862c1eb19f9232c883c33ccf25cfe9377036b50e01d670f/4,398,592B로48,128B 증가(약1.1%)이며새의존성·lock변화없다. 실행요청때만단일thread/cap1 channel/기존100ms kernel wait를사용한다. runtime메모리·CPU추가비용은미측정이다. [원전체검사독립검토](../validation/evidence/windows-msvc-installer-launch-whole-20261013/root-review.json). 실제새v11설치→앱실행수명 검수는패키지완성후진행한다.

첫v11 native lifetime시험은201disabled/212enabled실제PNG와클릭후child원Job/identity까지확인했으나 모든visible HWND를곧바로앱class/title로가정한observer에서실패했다. 원windows-msvc-ui-cross-surface-v11-20261013-popup-settled FAIL/원C#·로그·강제ownedJob정리를보존한다. 실제창inventory가없어앱또는시험원인으로확정하지않는다. v2 helper는원PID/creation/Job/image를유지한채64HWND·변경128행/64KiB inventory와own오류dialog readonly캡처를추가하고정확appclass/title의유일한창만선택한다. 동일10초상한및other-windowaction0이다. 제품source/시간상한/guards를바꾸지않고별도freshoutput으로실행한다.

v2 helper는새EnumChildWindows 선언과기존동일선언이중복되어C# CS0111 compileFAIL이었다. 앱/native실행은일어나지않았고원compilelog/helper/result를보존했다. v3는추가동일선언한줄만제거하여원선언을재사용했고명칭60개추가중복이없음을정적으로확인했다. 시간/metadata관측/owner/정확창매칭guard변경없이freshv3 output으로다시실행한다. 제품source/whole1295는변경하지않았다.

v3 실제inventory는원앱PID의10 HWND를기록했고정확WindowClass/dlpad1개와visible UAC_InputIndicatorOverlayWnd(0rect), Winit Thread Event Target(16rect)을확인했다. v1은이보조창을앱창으로잘못단정한시험문제였다. v3는실제앱UI와설치창제거/원supervisor생존, 앱닫기/양쪽exit0 단계이후theme.sqlite필수기대에실패했다. source accessible_settings.rs525–546의실제회귀는legacy theme.json startup이readonly이며DB를생성하지않음을보장한다. v4는해당계약에맞춰DB실제존재를정직하게기록하고있으면boundedreadonly로읽는다. legacytheme.json의원7바이트/SHA/nativeID는유지하며실제exitcode를DB관측이전즉시기록한다. 원v1/v2/v3전체FAIL 및원source/log/시간/owner/payloadguards를보존했다. 제품코드를변경하지않고새scopev4실행중이다.


## 실제 설치·실행 인수

v11의 Windows 격리 설치→재검증→앱 실행→설치 HWND 종료/감독 프로세스 생존→앱 닫기→양쪽 자연 exit0·Job0 시험이 통과했다. Root는 원로그·656개 소스·원본 7파일·5개 설치 파일·journal·PID/creation/native ID/이미지 SHA를 독립 대조했다. 별도의 27개 canonical ID 관측도 통과했다. legacy theme.json은 원 7바이트를 보존했고 시작만으로 theme/jobs SQLite를 만들지 않는 기존 계약을 확인했다. [실행 수명 독립 검토](../validation/evidence/windows-msvc-ui-cross-surface-v11-20261013-popup-settled-v4/root-review.json), [27 ID 검토](../validation/evidence/windows-msvc-ui-cross-surface-v11-20261013-installed-independent-v4/root-review.json).

별도 소유 fixture의 설치된 exe 첫 바이트만 XOR1로 바꾼 negative-v2 시험도 통과했다. 실행을 거부하고 설치/검증 버튼을 활성화하며 실행 버튼을 비활성화했다. 원 Job의 전체 생성 프로세스 수는 전후 및 종료 시 1로 자식을 생성하지 않았고, 마지막 실패 결과를 보존해 설치기는 자연 exit1이었다. Root가 전체 파일을 64KiB씩 비교해 첫 바이트만 다름을 확인했다. 원본 7파일·나머지 설치 4파일·journal 5기록을 보존했다. 오류 화면을 직접 확인했다. [변조 거부 검토](../validation/evidence/windows-msvc-ui-cross-surface-v11-20261013-launch-tamper-negative-v2/root-review.json).

실제 실행 성공과 변조 거부 범위를 인수했다. 원 v1/v2/v3 observer 실패와 모든 원로그는 유지했다. 늦은 실패 오류창 재표시, 감독 프로세스의 실제 idle 비용, 전체 GUI/물리 입력 및 정식 배포는 별도 인수다. 후속 UI 변경은 이 v11 시험을 새 소스 성공으로 소급하지 않는다.

## v13 완료 안내 통합 뒤 실행·실패 재인수

완료 안내를 통합한 최신1297/0/30·source656 v13에서 실제 설치→재검증→실행 버튼→앱 private HWND/원PID·creation·설치 image SHA/nativeID/원Job→설치 HWND 제거·감독 프로세스 생존→앱 닫기→앱과 감독 프로세스 자연exit0·Job0가 다시 통과했다. 원7파일·설치5파일·journal을 보존했고 별도 현재27개 canonical nativeID 대조도 통과했다. [현재 정상 수명](../validation/evidence/windows-msvc-ui-cross-surface-v13-20261013-completion-happy-v3/root-review.json), [27 ID](../validation/evidence/windows-msvc-ui-cross-surface-v13-20261013-completion-independent/root-review.json).

별도 v13 fixture의 설치 exe 첫 바이트만77→76으로 변조한 뒤 Run을 거부했다. 생성된 프로세스는 전후·종료 시 원installer1개였고 transient child도 생성되지 않았다. Run 비활성화·설치/검증 활성화·준비 안내 복귀 및 실패 receipt 유지로 자연exit1을 관측했다. 전체 파일을64KiB씩 비교해 첫 바이트 외 원본이 같고, 변조된 파일 ID/길이/SHA를 Run이 다시 바꾸거나 복원하지 않았음을 확인했다. 나머지4payload·원7source·journal5는 보존했다. [현재 변조 거부](../validation/evidence/windows-msvc-ui-cross-surface-v13-20261013-completion-negative-v3/root-review.json).

앱 데이터·영상 경로는 fixture 내부의 명시 child 환경으로 설정했고 global 환경·사용자 Documents·계정·실영상은 사용하지 않았다. legacy theme.json 원7바이트를 보존했으며 읽기 전용 초기 실행에서 theme/jobs SQLite가 없는 상태도 정상 계약으로 기록했다. 시작 ACK는 GUI-ready handshake가 아니고 설치 WINDOW 종료와 PROCESS/Job 소유권 종료는 다르다. 원v11 observer 실패와 v13 v1/v2 compile/GDI 실패는 보존한다. 지연된 실패의 visible 오류창 재표시·실제 감독 idle 비용·물리입력·전체 사용자 UI평가·서명/정식릴리스는 별도 미완료다.

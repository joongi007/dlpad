# S04: Windows 우선 P0 잔여 게이트 감사

- 상태: done (읽기 감사 완료; P0 미완료)
- 분류: audit
- 우선순위: P0
- 요구사항: R01–R08, R10, R11, R13; R09/P2·R12/P1은 제외
- 담당/수정 범위: 최초 감사 diagnostic_cache; 이번 근거 갱신 metrics_recovery, 이 문서만
- 선행 작업: Windows-first 범위 계약, 최신 paired/recovery/app-worker 실제 인수


## 이번 갱신의 범위와 인수 조건

2026-10-04 최신 [P0-status](../validation/P0-status.md), [앱 시작 통합](S04-managed-startup-app-integration.md), [GUI 관측 계약](S04-managed-recovery-gui-observer.md)과 아래 실제 JSON/독립 검토를 읽어 완료와 잔여 범위를 갱신했다. 수정 파일은 이 감사 문서 하나뿐이다. Rust/Cargo/schema/다른 문서/프로세스/GUI를 변경하거나 실행하지 않는다. completed/pending/malformed 실제 시작 GUI를 다시 미구현으로 쓰지 않고, live Busy와 CloseWait의 실제 GUI 경합은 호출 단위 회귀로 대체하지 않는다. 기존 요구 범위·Windows-first·S10 최초 릴리스 포함을 유지하며 전체 P0 완료를 주장하지 않는 것이 이번 인수 조건이다.

## 현재 실제 완료 범위

2026-10-04 README→PRODUCT→CONTEXT→REQUIREMENTS/ROADMAP→S00/S10/P0-status 및 최신 작업·원 증거 참조를 읽었다. P0-status의 옛 미연결 절보다 마지막 통합 결과를 우선한다. `windows-paired-parent-crash-sentinel-retry/root-review.json`은 원 부모 HANDLE 종료, managed3/cookie2 receipt, 독립 실제 복구·재사용·DB Interrupted·원 Job Active0를 인수했다. `windows-managed-startup-app-workspace`는 Windows GNU CPU 전체4검증 및 startup/retry/CloseWait 호출5회귀를 통과했다. 실제 GUI를 대신하지 않는다.

Windows 실제 startup GUI 세 사례는 더 이상 남은 첫 구현 게이트가 아니다.

- completed: [실제 GUI JSON](../validation/evidence/mg-gui2/g/native-result.json)과 [root 독립 검토](../validation/evidence/mg-gui2/root-review.json)는 원 paired 종료 fixture를 새 앱이 회수하고 원 관리 기록·cookie owner를 제거한 뒤 실제 '비정상 중단됨' 이력과 DB Interrupted를 확인했다. 원 source/foreign SHA, 원 Job 자연 Active0, private desktop 종료 및 input desktop 유지도 PASS다. binary는 `709de3f3213e00b6041fda4a2b4e416f070519ca66327d9b75e9cf8073feec92` 고정 scope다. 첫 .NET long-path collector 실패를 삭제하지 않고 짧은 새 합성 root에서 재검증했다.
- pending/malformed: [원 독립 검토](../validation/evidence/windows-managed-recovery-gui-prepare-first/pending-malformed-root-review.json)와 [malformed JSON](../validation/evidence/windows-managed-recovery-gui-prepare-first/gui-malformed/native-result.json)은 실제 재확인, 원 보호 파일 SHA, 손상 시 시작 차단과 자연 Job 종료를 확인했다. 이때 pending은 pristine 종료 미확인 기록이며 **실제 살아 있는 helper/journal Busy 경합 인수가 아니다**.
- 최신 pending 첫 화면: [GUI JSON](../validation/evidence/windows-managed-recovery-gui-prepare-first/gui-pending-visible/native-result.json)과 [독립 PNG/source 검토](../validation/evidence/windows-managed-recovery-gui-prepare-first/gui-pending-visible/root-review.json)는 binary `fa909fbb2b94b53c91ced0bbf8b42c99f949749e6d2bee490fb10f15d93ab64d`에서 첫960×640 viewport의 중앙 관리 보류 안내, retry, 원 SHA와 자연 종료를 인수했다. 이전 completed/malformed까지 최신 binary로 재실행했다고 확대하지 않는다.

[Windows 관리 runtime GUI](S04-windows-runtime-tools-gui.md)의 3tool 전환·runtime 설치 disabled·wrong-version 거절·Use→Deno/FFmpeg 필드 literal 전달은 고정 실행 파일의 scoped 성공이다. 당시 aggregate source set 변경 실패는 보존하고 이 UI 증거를 전체 최신 소스 인수로 승격하지 않는다. 자동 runtime 공급 설치와 live 실행 수명은 별도다.

8시간 headless 합성 시험은 이미 종료했다: `soak-terminal-independent-20261004/root-review.json`의 약28802/28804초, exit0·소유 PID 부재·SQLite/artifact 검증을 남은 대기로 다시 쓰지 않는다. GUI 장시간·실제 사이트·최신 Windows 제품 연속 실행은 별도다.

## 남은 게이트: 안전성이 높은 순서

| 우선 | 요구/선행 | 정확한 잔여 인수 |
|---|---|---|
| 1 | R08/R05, 실제 시작 GUI 세 사례 완료 뒤 최신 immutable binary | **live Busy·CloseWait 실제 GUI 경합**: 원 helper가 살아 있어 journal lock Busy인 시작에서 원 파일·실행을 보존하고 pending 안내; retry 연속 클릭/다른 worker/CloseWait와 충돌할 때 추가 worker·도구 spawn0, 외부 경로 허용 정책과 종료 지연 안내 검증. pristine pending/malformed retry/completed 회수는 위 실제 PASS를 유지하며 새 남은 작업으로 중복 기록하지 않음. 호출 회귀5개를 실제 live 클릭·화면 경합으로 확대하지 않음. |
| 2 | R08/R05, 위 GUI 정상 시작 | GUI 관리 도구 사용 중 취소·Pause/재개·StopExit·부모 crash→원 Job terminal/receipt→다음 시작, 실행 중 managed 선택/제거 거절, 외부 도구·source·미디어 보존. 현재 paired 실제 CLI crash 성공을 전체 GUI crash로 확대하지 않음. |
| 3 | R05, 정확한 Windows 공급 package와 통지 문서 | Windows FFmpeg/Deno **공급 채택·license/source/notice**의 남은 full source/transitive mapping과 최소 OS/cost를 정리. [Windows FFmpeg 보완](S04-windows-ffmpeg-supply-adoption.md)의 실제 configure61/recipe60 대응, exact mingw 전체 source2개와 기존 notice73 SHA 확보는 완료 근거로 유지하되 전체 enabled/transitive source·compiler image provenance·notice bundle 완료로 확대하지 않음. 공식 mapped ZIP 설치/정확 version 선택은 이미 scoped PASS. runtime 자동 설치 버튼은 공급 미채택으로 disabled이므로 구현 완료로 보고하지 않음. 외부 FFmpeg 선택 경로는 별도 허용된 P0 경로다. |
| 4 | R04/R08, 안정적인 reader/recovery 및 안전한 시험 환경 | 실제 YouTube·치지직 VOD/장시간 작업, 만료 URL·연결 단절·디스크 부족·절전/복귀·재부팅 실패 인수. 합성 fixture와 실제 사이트의 차이를 기록하며 실제 계정 비밀의 외부 전송 등 별도 승인 범위는 독립 보류한다. |
| 5 | S00/R06/R08, 최종 Windows 후보/격리 입력 환경 | 실제 한글 TSF/IME 조합·commit·cancel·선택, 실제 Windows reader/keyboard-only 주요 화면/상태 안내, 200%·focus·System OS 변경/첫 observed painted frame, idle/RSS/scan/background/GUI 장시간 비용. ASCII/WM_CHAR/UIA tree 및 Linux IME/Orca는 Windows 실제 입력·reader가 아니다. 최종 스택 채택 근거도 이 범위와 맞춰야 한다. |
| 6 | S10/R13 최초 릴리스 필수, 위 입력/reader 환경 | 현재 보고서 readonly/정제/정상960×640 순차 가시성/export/sentinel 덮어쓰기 거절은 scoped PASS. **Windows 실제 제품 clipboard 복사**와 reader/로그 편집·제외·첨부·MD/JSON 전체 흐름이 남는다. named private WindowStation probe는 ERROR5로 실패했고 사용자 clipboard를 읽거나 fallback하지 않았다. |
| 7 | R01/R02/R03/R07, 최종 동일 binary | Windows 다운로드 B/s·ETA/처리배속/볼륨과 미지원 counter `측정 불가` 구분; Documents redirect/한글·OneDrive·권한거절/fallback; loaded library 외부 변화·pagination/scan CloseWait; options·snapshot 재개·실행계획·도움말 keyboard 회귀를 최종 후보에서 연결 검증. 각 기존 scoped PASS를 누락 기능으로 단정하지 않되 전체 최종 화면 증거는 필요. |
| 8 | R10/R11/S07, 기능/실패/GUI 게이트 완료 | 현재 Git invalid/remote·실제 CI runner 없음. 기존 .git 보존하며 정상 Git 반영·실제 Windows CI/release 재현→패키지 설치/업데이트/제거·최소 OS·서명/배포 게이트를 분리한다. 현재 Windows GNU 실증은 실제 PASS, **MSVC는 미검증**이며 기존 CICD의 MSVC 제안과 배포 target 결정을 조정해야 한다. MSVC 자체를 새 요구로 임의 추가하지 않는다. 공개 배포/유료 서명은 승인 범위. |

## 인수·후속 선택

가장 작은 다음 수직 기능은 1번 live Busy·CloseWait의 실제 GUI 경합 인수다. 완료된 기본 GUI 시작 회수를 다시 시작하지 않는다. 실패 원시/원 source SHA/immutable binary SHA·원 Job cleanup·source/sentinel 보존을 함께 제출하고 주 담당이 PNG·JSON을 독립 확인한다. 이후 2번 GUI 실행 수명과 6번 S10 clipboard의 안전한 전용 Windows 환경을 병렬화할 수 있다. 실제 reader/IME/clipboard 환경 미확보는 별도 환경 게이트이며 제품 기능 계획으로 성공을 대체하지 않는다.

Linux/macOS·3OS 출시 인수는 사용자 Windows-first 지시로 후속이며 Windows P0 차단 목록에 넣지 않는다. P1 known issue/config 및 P2 업로드/AI, 후속 updater는 P0를 대신하지 않는다. 자동 cache=0 정책은 원 attachment/export 보존과 이미 분리했으므로 과거 S10 cache 만료 미구현 문구를 현재 요구로 재생성하지 않는다.

## 검증과 handoff

최초 감사 및 이번 갱신은 읽기 `rg`/`sed`와 원 JSON·문서 참조 대조만 수행했다. 이번 갱신에서 실제 GUI JSON의 세 상태·binary SHA·retry·자연 Job 종료와 root 독립 검토를 구분했다. 이 감사 문서 외 코드·Cargo·다른 docs·프로세스·GUI 실행 변경 없음. 본 문서는 최신 실제 API/crash/app-worker 완료와 남은 GUI·배포 증거의 경계를 요약한 감사이며 신규 실행 PASS가 아니다.

이번 파일의 로컬 링크11개 존재를 읽기 Python 검사로 대조했고 missing0이었다. 전체 문서 checker/새 native 실행은 이번 갱신에서 수행하지 않았다.

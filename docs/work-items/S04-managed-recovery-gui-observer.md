# S04: Windows 관리 실행 시작 복구 GUI 관측기

- 상태: in-progress
- 분류: test
- 우선순위: P0
- 요구사항: R05, R08, R13
- 담당: metrics_recovery
- 수정 파일: scripts/windows_managed_recovery_gui_observer.cs, 이 문서
- 선행 작업: root의 실제 completed/pending/malformed 합성 fixture 및 최신 CPU 바이너리

## 사용자 결과와 계약

실제 네이티브 시작 복구 worker를 private desktop에서 관찰한다. CLI는 CPU 실행 파일 절대 경로, 원 ownedroot, 새 output 세 인자다. DLPAD_TEST_DATA=root/data, DLPAD_LIBRARY_ROOT=root/media, --fixture를 사용한다. root/gui-recovery-case.json은 schema1, case completed|pending|malformed, protected_files 절대 경로 배열, removed_files 절대 경로 배열이다. 모든 시험 파일은 root 하위이며 reparse 경로를 거절한다. 원 보호 파일의 SHA는 앱 실행 전에 수집하고 worker 및 retry 이후 재검증한다. 최대 32개/파일당256MiB로 제한한다.

completed는 실제 중앙 '이전 작업 상태 재확인', 관리 pending 안내 없음, 원 marker/journal/receipt 및 지정 cookieowner 부재, '작업 이력'의 실제 '비정상 중단됨'을 검사한다. DB 원 Interrupted 상태는 root가 별도로 SQL 검증한다. cookie foreign sentinel 때문에 나올 수 있는 cookie 경고 부재는 요구하지 않는다.

pending은 관리 실행 미확인 안내와 원 pending 파일 bytes 보존, malformed는 안전 확인 실패 안내와 다운로드 시작 disabled를 검사한다. 둘 다 실제 '이전 작업 다시 확인' InvokePattern 후 같은 상태와 원 보호 파일 SHA를 확인한다. 다운로드를 시작하지 않는다.

## 격리와 상한

기존 Win32 private desktop/UIAutomation 계약을 재사용한다. suspended 원 Process를 소유 Job에 먼저 연결한다. 원 HANDLE FILETIME/HWND/PID 대조 후에만 입력과 WM_CLOSE를 전달한다. 사용자 desktop/clipboard, shell, 추정 PID 종료, 가시 desktop fallback을 사용하지 않는다. UIA subtree 256개, evidence JSON64KiB, 창 탐색15초, 복구 확인20초, retry 확인20초, UIA worker90초, 원 WM_CLOSE5초다. UIA Invoke 호출 자체가 hard realtime 보장은 아니며 worker deadline 실패는 FAIL이다.

성공에는 원 Job 자연 ActiveProcesses0, private desktop close, input desktop 전후 동일이 필요하다. 실패 cleanup TerminateJob으로 성공을 만들지 않는다. 상태 labels/bounds와 PNG를 원 output에 남긴다. PNG glyph 가시성은 root가 실제 파일 SHA와 함께 독립 확인한다.

## 검증과 인계

현재 C# 구현 단계이며 native 컴파일/GUI 실행은 아직 미실행이다. Cargo/제품 Rust/기존 관측기는 변경하지 않는다. 실제 root fixture 세 사례와 바이너리로 주 담당이 컴파일 및 GUI 실행한다. 준비본을 시작 복구 인수 완료로 기록하지 않는다.

주 담당 실제 Windows 인수: 첫 준비 수집에서 fixture·두 C# observer 컴파일 및 원 부모 종료 후 receipt 보존 PASS. 실제 GUI pending/malformed의 재확인·원 보호 SHA·malformed 시작 차단·원 Job 자연 종료/private desktop 종료/input desktop 유지 PASS, pending-malformed-root-review.json으로 독립 대조했다. completed 첫 수집은 .NET4 경로 길이 제한으로 실패하여 원 실패를 보존했다. 짧은 fresh `mg-gui2` 합성 경로에서 새 부모 종료 기록으로 completed GUI 회수/실제 비정상 중단 이력·원 보호 SHA·소유 기록 부재·DB Interrupted를 통과했고 root-review.json으로 독립 대조했다. 당시 고정 앱 SHA709de3이며 이후 pending 안내를 항상 보이는 status에 보강했다. 최신 Windows 전체 fmt/clippy/test/release PASS, 최신 pending 화면 재검증은 진행 중이다. 실제사이트/IME/절전/전체 P0 완료로 확대하지 않는다.

최신 pending-visible 실제 GUI도 PASS. 주 담당 startup.png를 직접 읽어 첫 viewport의 status에 중앙 관리 보류 안내가 보이는 것을 확인했다. 재확인·원 파일 SHA·자연 Job 종료 및 최신 고정 binary/current source SHA도 독립 대조했다. 최신 Windows 전체 네 검증과 문서 612개 검사 PASS, 전체 P0는 미완료다.

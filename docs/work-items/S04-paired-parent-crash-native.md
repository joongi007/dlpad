# S04: Windows paired 원 부모 종료 관측기 준비

- 상태: in-progress
- 분류: test
- 우선순위: P0
- 요구사항: R05, R08, R13
- 담당: metrics_recovery
- 수정 파일: scripts/windows_paired_parent_crash_observer.cs, 이 문서
- 의존성: 주 담당 및 soak_recovery_audit의 실제 fixture와 public 복구 API

## 현재 계약

관측기 CLI는 fixture 실행 파일 절대 경로와 새 output 디렉터리다. private desktop과 원 소유 Job을 만들고 suspended child를 Job에 연결한 뒤 Resume한다. HANDLE_LIST는 stdin NUL/stdout writer/stderr writer 세 개다. 제품 helper HANDLE 노출, PID 추정 종료, 사용자 desktop fallback은 없다.

관측기는 output/dlpad-paired-parent-crash-<GuidN> wrapper만 만든다. 그 아래 owned-fixture 생성은 producer가 담당한다. CLI는 paired-parent <owned-root>, paired-inspect <owned-root>다. fixture가 root basename과 wrapper prefix를 검사한다.

parent ready는 schema=1, phase=paired-tool-started, parent_creation JSON 한 줄이다. 실제 합성 download tool이 cookie를 읽고 Resume 뒤 ready capability를 동기화한 것을 parent가 확인한 다음 전달한다. 관측기는 원 Process HANDLE의 GetProcessTimes를 대조한 뒤 그 HANDLE만 TerminateProcess73하고 원 exit73을 확인한다.

fresh inspector는 원 journal anchors와 실제 receipt를 검사하고 public 복구를 유한 반복한다. schema=1, phase=paired-recovery-evidence 및 both_receipts_valid/source_foreign_preserved/managed_recovered/cookie_recovered 모두 true가 인수 조건이다. JSON 자체는 제품 삭제 authority가 아니다.

## 상한과 실패 보존

parent ready/inspector JSON 각각 20초, 원 process 종료 wait 각각 5초다. PeekNamedPipe 가용 바이트만 읽고 stdout/stderr 각각 4096바이트 상한을 적용한다. malformed schema, creation 불일치, 추가 stdout, EOF/deadline는 실패다.

성공 종료 전 원 observer Job ActiveProcesses=0을 최대 10초 확인한다. 자연 empty 실패는 FAIL이며 이후 소유 Job 실패 정리로 성공 판정을 바꾸지 않는다. 후속 최대 5초 empty 확인, private desktop close, input desktop 전후 동일성을 기록한다. helper 개별 HANDLE wait는 주장하지 않는다. 두 실제 receipt와 전체 원 Job 자연 empty가 종료 증거다.

실패 raw와 합성 fixture를 보존한다. 실제 계정/영상/Documents/clipboard/시스템 설정은 사용하지 않는다.

## 실제 검증 범위

이전 helper HANDLE 전달 계약은 주 담당 windows-paired-parent-observer-compile/compile.json에서 csc exit0이었다. public Repository가 helper HANDLE을 노출하지 않아 현재 계약으로 변경했다. 그 이전 결과를 현재 소스 컴파일 성공으로 확대하지 않는다. 현재 개정 소스의 컴파일/native 실행은 미실행이며 주 담당이 수행한다. 준비본만으로 실제 부모 종료와 두 복구를 완료 처리하지 않는다.

주 담당 실제 인수: `windows-managed-startup-recovery-workspace-first` Windows GNU CPU fmt/clippy/test/release PASS, 신규 보존 회귀 8개 실제 통과. `windows-paired-parent-crash-sentinel-retry`는 공개 Repository 실행 중 원 부모 HANDLE TerminateProcess73, 실제 두 도메인 receipt·원 관리/쿠키 복구·재사용·DB Interrupted·원 Job 자연 Active0·private desktop 종료/input desktop 불변을 통과했다. root-review.json에서 원 receipt 5개 SHA/256바이트, 원 cookie/tool/sentinel 보존, DB Interrupted/artifact0·인증 비밀 부재, 소유 journal/pending/owner 실제 부재 및 당시 source SHA를 독립 대조했다. 초기 fixture의 foreign sentinel rejected0 기대 오류는 기존 API 계약의 scanned2/recovered1/rejected1로 고쳤으며 두 원 실패를 보존했다. 관측기의 빠른 종료 뒤 미수신 pipe 출력을 먼저 읽도록 bounded drain을 보강했다. 실제 GUI 전체 crash·sleep/reboot와 전체 P0 인수는 별도다.

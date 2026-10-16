# S03: Windows 시작 전 취소와 실행 deadline

- 상태: done
- 분류: fix
- 우선순위: P0
- 요구사항: R08, R04, R11
- 담당/수정 범위: root — crates/engine/src/process/windows.rs, 통합 동결 후 process.rs 호출 연결, 이 문서
- 선행 작업: S03-startup-cancel-deadline-review, metrics 담당 공통 ExecutionControl 계약

## 사용자 결과

취소되었거나 전체 실행 시간이 지난 요청은 suspended Windows child를 실행하지 않고 소유 Job 종료 확인 후 반환한다. 미확인 정리는 쿠키 보존 오류로 유지한다.

## 계약과 범위

기존 atomic Job assignment와 owned process/thread handle을 유지한다. CreateProcess 전 및 ResumeThread 직전 공통 checkpoint를 사용한다. suspended child 중단 시 Job signal, bounded Child wait, Job ActiveProcesses=0을 모두 확인한 경우만 Cancelled/TimedOut을 반환한다. 실패는 typed cleanup error다. Cargo/schema 변경 없음. process.rs는 metrics 수정 동결 이후 root가 통합하며 같은 파일을 동시에 수정하지 않는다.

## 인수 조건

실제 Windows native current-test-executable의 suspended child에 취소/만료 checkpoint를 적용하여 user code 출력 없이 Child 회수 및 Job terminal 확인을 시험한다. 정상 다운로드/취소/출력 pipe/쿠키 합성 시험을 함께 회귀한다.

## 검증과 handoff

현재 source 읽기와 계약 기록만 수행했다. 구현·native 시험은 아직 미완료다. Windows GNU 증거를 MSVC/macOS 또는 실제 GUI 인수로 확대하지 않는다.

## root 통합 검증

Windows GNU CPU workspace test와 all-targets Clippy exit0. root가 before/after/current 입력 SHA 및 신규 실패 시험 PASS를 독립 대조했다. 원시 증거는 `docs/validation/evidence/startup-checkpoint-windows-retry/`이다. 최초 root test의 DuplicateHandle namespace 컴파일 실패는 별도 first evidence에 보존하고 수정했다. 실제 GUI·MSVC·macOS 검증은 포함하지 않는다.

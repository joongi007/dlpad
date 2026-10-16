# S00: Windows 격리 입력 이벤트 원인 분리

- 상태: done
- 분류: diagnostic
- 우선순위: P0
- 요구사항: R08, R11, S00
- 담당/수정 범위: 주 담당 app/main.rs·fixture_trace.rs; guardian_fault Windows harness·전용 증거
- 선행 작업: Windows privateDesktop에서 typed/click 입력 미반영 재현

## 사용자 결과

사용자 화면을 건드리지 않고 시험 창에 전달된 native 이벤트와 widget capture 여부를 구분한다.

## 계약과 범위

--fixture와 --fixture-events를 함께 지정하고 InstanceGuard가 확인한 data root가 있을 때만 create-new fixture-events.jsonl을 만든다. UI는 고정 event kind와 captured bool만 capacity64 try_send하며 writer는 별도 thread로 최대64행 저장한다. 키값·text·좌표·URL·비밀을 기록하지 않는다. 일반 제품 실행/기존 trace파일은 변경하지 않는다. 기존 GUI 증거는 각각 기존binary hash 범위로 유지한다.

## 인수 조건

제한 없는 입력 로그 없음, 기존 파일 보존, explicitfixture없으면 비활성화, 모든 I/O는writer. 독립 소유 privateDesktop에서 실제 태그 순서를 관측하고 원인 가설을 검증한다. visibledesktop/globalinput fallback 금지. 공유Cargo/schema변경 없음.

## 검증과 handoff

Linux fmt/clippy/workspace190passed5ignored/release 및Windowsnativebuild exit0. 64행/3460..3490B/고정세필드 anonymous태그를실제privateDesktop에서독립검토했다. baseline CursorLeft가click전에발생하고burst click은그전에Captured됨을확인해harness만수정했다. 기존tracefile보존시험도pass이며일반제품에는trace없다. [실제GUI증거](../validation/windows-background-gui.md). UIstate약32B와explicitfixture에만writer1thread/capacity64(약1.5KiB)/출력8KiB미만비용이다. 후속candidate Cargo변경은별도작업이며trace자체새의존성없다. 태그 관측은 한글IME/실제reader 또는 active 숨김·복귀 성공을 대체하지 않는다.

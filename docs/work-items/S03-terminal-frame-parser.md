# S03: 소유 child 회수 후 종료 프레임 검증

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: stress_recovery — 신규 crates/engine/src/process/terminal.rs 및 본 문서만
- 선행 작업: root의 process module 연결 및 OwnershipCleanupError::TerminalUnconfirmed 추가; caller child 회수/소켓 직렬화

## 사용자 결과

외부 도구 종료의 성공 exit만으로 쿠키 복사본을 지우지 않고 소유 프로세스 정리 프레임까지 확인한다. 확인되지 않으면 고정 typed 오류로 복사본 정리 보류 경로를 선택할 수 있다. 이 파서는 process/쿠키/UI 연결 전체 완료를 뜻하지 않는다.

## 계약과 범위

pub(super) confirm(&mut UnixStream, expects_service:bool)->io::Result<()>. 직접 실행은 정확 GC와 EOF, service는 정확 GC→LC와 EOF만 허용한다. caller가 소유 child를 reap한 뒤 호출하며 해당 socket read/write 사용을 직렬화한다. 실제 socket은 blocking 모드 계약이다. read_timeout으로 최대20ms씩 기다리며 절대1초 상한, buffer5bytes, 새 thread/queue/의존성 없음. read queued bytes를 먼저 처리하고 EOF에서 전체 일치만 성공한다. unknown/GU/LU/중복/역순/추가바이트/잘림/EOF없음은 io::ErrorKind::Other의 OwnershipCleanupError::TerminalUnconfirmed로 반환한다. 원문 frame/OS error는 공개하지 않는다.

호출 전 read_timeout을 보관하고 성공·실패 모두 복원한다. O_NONBLOCK을 변경하지 않는다. 정상 frame 후 EOF까지 기다려 늦게 큐에 들어온 추가 바이트도 거절한다. writer가 남아 EOF가 오지 않으면 1초 뒤 미확인이다. service manager endpoint 복제 종료는 root 실제 backend 인수 사항이다. 스케줄러/커널 syscall 지연으로 wall-clock이1초보다 조금 길어질 수 있으며 개별 OS호출의 강제중단 보장은 아니다.

## 인수 조건

UnixStream fixture6개: direct/service exact EOF, 분할4bytes+기존timeout 복원, 기형/ASCII spoof/잘림/중복/역순/초과, direct/service 구분, 정상prefix지만writer생존 시 약1초 실패, 분할정상prefix 이후 추가byte 거절. typed error downcast 및 timeout복원 확인. 실제 guardian/launcher의 frame 제공과 reaped-child 연결은 root 후속 통합 검증으로 구분한다.

## 검증과 handoff

단독 rustfmt 및 문서 검사를 수행한다. Cargo/module/Ownership/shared source는 변경하지 않는다. 새 unit 시험은 root module 연결 후 실제 Cargo 검증 대기이며 PASS를 주장하지 않는다. nonUnix 지원은 이 UnixStream parser 시험 범위에 포함되지 않는다.

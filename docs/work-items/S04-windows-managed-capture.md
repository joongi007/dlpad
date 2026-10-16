# S04: Windows managed authenticated capture 연결

- 상태: implemented; integration-validation-pending
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: soak_recovery_audit; 신규 process/windows_managed_capture.rs와 본 문서만
- 선행 작업: managed journal/config/helper/runtime/protocol; root module 및 public caller 연결

## 사용자 결과

관리 도구 호출은 durable stage Intent/Bound와 helper 실제 native terminal receipt를 통해 capture하며 실패 시 pending을 보존한다.

## 계약과 범위

fresh BCrypt nonce→journal.begin→config/launch_managed→실제 helper creation Prepared→bind/release→Ack/Begin→Terminal/EOF/exitcode 일치→reacquire/resolve. 별도 managed Frame; cookie 타입0. 원 receipt writer는 launch 시도 직후 drop. 기존 bounded drain/metadata 정책과 fresh cancellation cleanup budget 사용. Cargo/기존 source/shared schema 변경 제외.

## 인수 조건

runtime error는 typed TerminalUnconfirmed로 남고 receipt/pending을 위조하지 않는다. stdout/stderr bounded drain와 EOF 실패는 정상 결과로 위장하지 않는다. 주 담당 실제 native caller/회귀 및 compiler 검증이 필요하며 새 소스만으로 완료 주장하지 않는다.

## 검증과 handoff

신규 source freeze SHA256 `933e79cfc822c891cdba0f5152db74fa3f033c7e5cd368a673d4cc51ec08f8b4`. explicit8-argument wrapper 계약 때문에 함수 하나에만 too_many_arguments allowance를 명시했다. `crate::tools::ManagedJournal` 재노출과 `Launch::send_managed_frame`은 root 연결이 필요하다. 기존 guarded helper executable registration을 사용한다. 원 writer는 launch 성공·오류와 무관하게 즉시 drop하며 native pipe 준비 오류도 typed TerminalUnconfirmed로 매핑한다. terminal 수신 뒤에는 늦은 Cancel/TimedOut을 보내지 않는다. 실제 Terminal→events EOF→원 helper bounded wait/exitcode 일치→journal reacquire/원 receipt resolve 이후에만 정상 capture 결과로 돌아간다. 오류는 fresh 2초 cleanup budget으로 원 helper를 기다리고 원 child RAII를 drop하되 journal pending을 해제하지 않는다. managed capture 로그는 기존 bounded output 정책이며 인증 omission은 실제 download adapter 책임으로 여기에 새 raw-cookie 정책을 넣지 않는다.

전용 native fixture는 실제 managed stage VersionProbe→Metadata→Download→DirectProcessing 순차 실행과 동일8-index 재사용 거절, 실제 cancel/timeout/outputlimit/guardian kill 후 pending 및 원 receipt, foreground crash/root preserve, stdout/stderr EOF를 검증해야 한다. 기존 cookie 테스트 include를 복제하지 않는다. 원소유 root/marker/lock/8 receipt 생성은 기존 managed native fixture 계약을 사용하고 사용자 자료는 제외한다. 최소 정상 fixture는 실제 managed stage0/1 두 호출→각 receipt256/record Resolved→stderr/stdout/exit42 원결과→cleanup 완료·foreign sentinel 보존을 확인한다. 이후 실제 취소 및 timeout은 정상 capture outcome 또는 strict unconfirmed를 구분하고 receipt 없는 pending을 성공으로 처리하지 않는다.

Cargo/rustfmt/native 실행 및 다른 Rust 파일 수정 없음. root의 compile/clippy/caller/helper integration과 실제 native 회귀 실행 전이므로 **제품 연결/검증 완료 아님**.

관리 전용 실제 helper 인수 통과: windows-managed-capture-exact-marker-retry에서 metadata JSON, 성공, 도구 실제 exit 7, 실행 취소와 원 journal/receipt 정리 후 재사용을 모두 확인했다. 주 담당이 acceptance.json, 원 로그, 현재 소스 SHA와 고정 예제 binary SHA 9e06c75cdb675774bbb458cd111abed9fd52b9b4f88aa750adb26837cd64e7c6를 독립 대조했다. 실제 관리 Job/helper 경로의 합성 인수이며 cookie 결합과 제품 DownloadSession 전체 전달 및 startup 자동 회수는 미검증이다. spawn marker 수정 후 전체 workspace 회귀는 별도 진행 중이다.

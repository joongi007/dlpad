# S03: Windows guardian의 원 journal·receipt capability 경계

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — cookies/windows/guardian.rs, cookies/windows.rs와 cookies.rs 연결, 이 문서
- 선행 작업: 원 journal 잠금 인계, PreparedChild, Windows records

## 사용자 결과

독립 helper가 앱이 저장한 정확한 Bound를 원래 native 파일들과 직접 잠금으로 검증할 때만 실행 ACK 및 원 receipt 기록 권한을 확보한다.

## 계약과 범위

원 Intent record에서 원 suspended process 생성 identity를 결합한 예상 Bound를 계산해 actual journal과 일치하는지 확인한다. 원 cache/owner/copy/journal/receipt FileId·SID·ACL·regular/nonreparse를 검증하며 receipt writer는 전달받은 원 handle을 사용한다. 출력/실패는 고정 문구이며 credential/path를 노출하지 않는다. 새 Cargo/schema 없음. guardian completion 호출은 원 Job 종료 확인 뒤에만 연결할 후속 helper의 책임이다.

## 인수 조건

Busy/foreign creation/record mismatch/receipt 교체/미완료 bytes에는 ACK·receipt 기록을 하지 않는다. 실제 original Bound lock 인계와 receipt sync를 native synthetic fixture로 검증한다. facade 구현만으로 public recovery 완료로 표시하지 않는다.

## 검증과 handoff

실제 helper/public startup 연결과 native fixtures는 후속 검증이다. Linux/Windows 전체 입력 검증은 source 안정화 뒤 수행한다.

## 구현과 실제 native 검증

GuardianJournal::acquire가 원 Intent에서 실제 생성값에 결합한 예상 Bound와 잠금 획득 후 actual journal bytes를 비교한다. 전달받은 원 writer FileId/ACL, 원 cache/owner/copy의 identity를 확인한다. complete는 원 locked journal과 pendingBound, 빈 원 receipt를 다시 확인해 정확256byte를 sync하며 중복/부분write 재시도하지 않는다. 이 메서드는 원 Jobempty를 확인한 후에만 호출할 helper runtime 계약이며 런타임은 아직 미연결이다.

Windows 격리 CLI의 `cargo test -p dlpad-engine --lib --locked cookies::windows::guardian::tests -- --nocapture`: actualexit0/sourceunchanged/inputdesktopunchanged, 정확1test PASS. Busy acquire/foreign creation 거절에서 원receipt0bytes, exactBound acquire, 앱재획득Busy, 원receipt sync·중복complete거절·앱reacquire/resolve를 확인했다. fixture에서는 Job을 실행하지 않아 앱crash/도구전체종료를 입증하지 않는다.

windows-journal-handoff-native-workspace-first 및 linux-workspace-first에서 전체 통합 검증이 진행 중이다. terminal 전에는 완료로 표시하지 않는다. publicstartup scanner/immutable marker/실제 helpercontrolEOF 인수는 계속 미완료다.

Facade 포함 handoff snapshot의 Windows 전체4명령 및 Linux 기본/CPU 필수7명령+version2가 모두exit0/PASS로 terminal 종료됐다. root가 당시 현재입력SHA를 독립 대조했다. 후속 runtime 변경은 이 범위에 포함되지 않는다.

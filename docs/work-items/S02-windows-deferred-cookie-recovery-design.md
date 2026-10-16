# S02: Windows 보류 쿠키 회수 설계

- 상태: proposed
- 분류: review
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: metrics_recovery — 이 문서만; 제품 코드와 공유 schema/Cargo는 읽기 전용
- 선행 작업: Windows private copy 및 Job terminal 구현, root 공유 계약 조정, 독립 Windows guardian 실제 검증

## 사용자 결과

현재 Windows 정상 세션은 임시 Netscape 사본을 정리하지만 recover_sessions는 Unavailable이다. 종료 미확인 사본의 다음 시작 회수는 미구현이다. INTEGRATIONS의 작업 종료·다음 시작 정리는 명시된 목표이므로 P0 잔여 게이트이며 안전한 미지원 안내가 요구사항 완료를 뜻하지 않는다. ADR 0003의 Proposed 및 최종 OS 검증 경계는 유지한다.

## 읽기 근거와 현재 경계

AGENTS→README→PRODUCT→CONTEXT→관련 요구사항→ADR 0003 및 ARCHITECTURE/INTEGRATIONS를 읽었다. cookies/windows.rs의 OwnedCopy는 현재 SID의 보호 DACL, no-reparse 경로 핀, 원래 native identity, 파일 종류와 정확한 entries를 검증한다. cleanup은 DELETE 핸들 identity를 재검증하여 SetFileInformationByHandle로 삭제한다. defer_cleanup은 핸들을 닫고 사본을 남기지만 durable 회수 marker를 만들지 않는다. 원본은 수정하지 않는다.

process/windows.rs는 비상속 unnamed Job의 KILL_ON_JOB_CLOSE, suspended CreateProcess→Job 할당→Resume, 원래 process handle, 유한 wait와 ActiveProcesses=0 확인을 구현한다. 독립 guardian 및 다음 시작까지 전달되는 durable receipt는 없다. 핸들은 프로세스 종료로 소실된다. PID·생성 시간·Job 이름 재조회로 원래 권한을 복원하지 않는다.

## 최소 수직 계약

1. Windows 전용 journal: 현재 SID/보호 DACL, volume+128-bit FileId, 디렉터리 핀, regular link1 및 정확한 allowlist를 I/O·삭제 전에 재검증한다. 원래 record의 nonce/digest/anchor를 고정하고 현재 관측한 FileId로 원본 증명을 대체하지 않는다. LockFileEx 비차단 배타 잠금의 실제 수명을 검증한다. 최대128세션·2reader·4096B record·256B receipt는 root schema 승인 뒤 적용한다. unknown entry/partial record/교체/ACL 변경/lock busy는 보존한다.
2. 독립 guardian gate: 선택적인 execution observer를 사용하여 ProcessSpec의 필수 필드를 늘리지 않는다. 제안 API는 begin_windows_reader(slot), 단일 사용 launch capability, bind_windows_identity(identity), resolve_windows_receipt()이며 root가 타입을 확정한다. 앱이 Intent와 create-new receipt를 flush한 뒤 제한된 write-only receipt handle 및 같은 잠금 권한을 guardian으로 전달한다. guardian이 원래 Job/process handle을 보유하고 reader를 suspended 생성·할당한다. 앱의 정확한 identity durable Bound commit과 Begin 승인 이전에는 Resume를 금지한다. STARTUPINFOEX handle allowlist로 도구에 관리 핸들을 넘기지 않는다.
3. 같은 부팅 회수: guardian이 원래 tool 종료와 같은 Job ActiveProcesses=0을 확인하고 추가 할당 권한을 닫은 뒤 receipt를 쓰고 FlushFileBuffers한다. 도구 exit126와 정리 확인은 별개다. 앱 소실은 owned control EOF로 Job 종료를 유도한다. guardian도 죽으면 마지막 Job handle close가 종료를 유도하지만 durable 완료를 추정하지 않는다. 다른 handle 복제본의 잔존을 실제 시험한다. 다음 시작은 원래 anchor/ACL/잠금/record/전체 receipt identity가 일치해야 Resolved 및 정리를 commit한다. Intent/Bound에 receipt가 없으면 보존한다. PID 부재나 named Job만으로 삭제하지 않는다.
4. 이전 부팅 회수: 검증된 Windows boot identity가 없다. uptime·wallclock 차이·로그온·Linux UUID·추측한 내부 NT 구조체로 삭제 권한을 만들지 않는다. Fast Startup/hibernate/suspend/reboot 의미와 지원 API를 검증할 때까지 receipt 없는 회수는 Unavailable/보존이다. 향후 별도 boot authority는 원래 boot/digest/anchor에 묶고 각 unlink 직전에 재검증한다. Unix 조건을 완화하지 않는다.

Empty→Intent→Bound→Resolved→정리 순서다. 슬롯 재사용/동시 reader는 거절한다. durable commit 전에 메모리 pending을 지우지 않는다. 삭제 부분 실패는 원자성을 주장하지 않고 잔여물을 보존한다.

## 인수 조건

- Intent 전후, Bind 전후, Resume 전후 앱 crash의 tool 실행 여부와 원래 기록을 각각 확인한다.
- 정상/exit126/취소/timeout/grandchild는 원래 Job empty와 실제 receipt를 검증한다.
- 앱+guardian 동시 종료, partial receipt, lock 복제, handle 누출은 receipt 없는 삭제를 거절한다.
- FileId/ACL/reparse/hardlink/nonce/digest 교체, foreign entry, malformed/oversize는 원본과 외부 sentinel을 보존한다.
- 검증된 boot identity 없는 이전 부팅 fixture는 보존한다. 합성 record 변경을 실제 reboot로 보고하지 않는다.

## 파일 분담과 비용

신규 Windows journal/receipt 모듈 및 guardian fixture를 독립 배정할 수 있다. root가 cookies.rs, process.rs/windows.rs gate 연결, shared schema/catalog/Cargo를 조정한다. 동일 파일 병렬 수정은 금지한다. 새 의존성 없이 우선 검토하고 guardian 추가 프로세스의 크기·idle CPU/RSS·handle 수·flush 비용을 실제 측정한다. bounded native I/O가 hard real-time을 보장하지 않는다.

## 검증과 handoff

이번 작업은 코드 읽기와 설계 문서만 수행했다. 기존 Windows synthetic private-temp ACL/FileId/Job fixture는 신규 구현 시험의 기반이며 guardian durable recovery 성공 증거가 아니다. Cargo/native/GUI/실제 계정/사용자 Documents 시험은 수행하지 않았다. `python3 scripts/check_docs.py` 실제 exit0, 464개 문서 PASS. 외부 URL과 heading anchor는 검사 범위 밖이다.

현재 권한으로 임시 파일, 소유 process crash/EOF, private desktop 및 bounded native fixture를 시험할 수 있다. 시스템 reboot/WSL 종료·관리자 보안 변경·실제 계정 인증은 필요하지 않으며 별도 승인 없이 수행하지 않는다. 실제 부팅 전환 시험이 필요하면 해당 단계만 보류하고 독립 synthetic 회귀를 계속한다.

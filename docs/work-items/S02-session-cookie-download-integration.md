# S02: 세션 쿠키를 다운로드와 native 입력에 연결

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R07, R08, R13
- 담당/수정 범위: root — 공유lib export·download.rs·native UI/catalog·통합시험; cookies.rs와importer시험은stress_recovery독점
- 선행 작업: session-cookie-import 기반의컴파일/실패경로인수, guardian최신전체검증

## 사용자 결과

사용자가선택한Netscape파일을세션전용복사로다운로드에적용하고완료/실패/취소뒤소유복사만정리한다. 실제계정/쿠키/사이트전송은승인없이시험하지않으며합성localhost/fake-tool만사용한다.

## 계약과 범위

INTEGRATIONS의세션쿠키·비밀배제계약. DownloadOptions의persistedJSON/argvsnapshot에는authfile/content를넣지않고실행세션capability를별도전달한다. tool버전조회에는cookieargv를넣지않고metadata/download실행에만붙인다. lease수명은두실행과취소ack를포함한다. 보고서/미리보기/로그/DB에credential값과privatepath를노출하지않는다. 재개시명시적으로새session을선택하며평문auth를자동복원하지않는다. browserprofile복호화와OS별credentialstore/ACL은독립후속게이트다. 새schema/Cargo변경은root만조정한다.

## 인수 조건

합성fixture로metadata/download에같은cookiecapability전달·literal값보존·DB/snapshot/report비밀배제·세션 종료/취소/도구오류정리·source/sentinel보존을확인한다. 실제nativeGUI입력/오류/세션취소와nonUnixUnavailable를OS실증과구분한다. 일반unauthenticated다운로드/재개/구간/자막회귀를유지한다.

## 검증과 handoff

공유interface준비기록이며argv/UI연결은아직미구현이다. importer의실제검증후인수에따라구체화한다.

## 2026-10-04 실행 인터페이스 분담

root는 native UI/catalog와 최종 통합을 담당한다. metrics_recovery는 download.rs와 S02-cookie-execution-adapter 작업 기록만 담당하고, stress_recovery는 cookies.rs의 실행 직전 소유권 확인 및 importer 시험만 담당한다. 같은 파일을 동시에 수정하지 않는다. 기존 실행 API를 유지하면서 세션 capability를 별도 전달하며 DB/schema/Cargo 변경은 하지 않는다.

인증 실행에서 도구가 임의의 비밀을 stdout/stderr에 출력할 수 있으므로 알려진 문자열 치환만으로 비밀 배제를 증명하지 않는다. 내부 artifact framing 처리 뒤 외부에 반환하는 인증 실행 raw 로그는 생략하고 생략 사실을 명시한다. 자유로운 metadata 문자열의 반환도 별도로 제한한다. 인증 작업의 진단 정보 감소를 기록하고, outcome/exit code·취소 및 검증한 artifact 증거는 유지한다. UI 연결·합성 실패/취소 인수·전체 회귀가 끝나기 전에는 완료로 표시하지 않는다.

## native 후보 연결 구현 (검증 전)

root가 accessible.rs와 catalog.rs에 이번 요청의 쿠키 경로 입력, worker로의 일회성 전달, Unix0700 cache 생성, session import와 DownloadSession 전달, 종료 뒤 명시적 cleanup을 연결했다. 입력은 worker 시작 시 비우며 설정/옵션 snapshot에 저장하지 않는다. nonUnix는 고정 미지원 오류로 중단한다. cleanup 소유권 변경은 기존 job 결과를 변경하지 않고 상태에 정리 보류 안내를 추가한다. root 담당 두 파일 rustfmt는 exit0이다. 공유 API 통합 컴파일과 실제 GUI/실패 경로는 아직 실행 전이다.

통합 검토에서 process adapter가 child cleanup deferred/unconfirmed로 반환하는 특수 경로를 발견했다. 현재 lease Drop과 UI 명시 cleanup만으로 이 경우 모든 child 종료 뒤 정리를 보장할 수 없다. 해당 오류의 타입/소유권 전달 및 startup 회수 계약을 보강하기 전에는 이 수직 기능을 완료하거나 첫 릴리스 인증 기능으로 주장하지 않는다. 정상 실행 경로의 시험과 독립 구현은 계속한다.

## root 실제 통합 재검증

최초 download 시험은4PASS/1FAIL이었다. 정리되거나 교체된 lease가 version 실행 전 차단되지 않는 제품 결함을 수정했고 시험을 완화하지 않았다. 수정 후 `cargo test -p dlpad-engine --test session_cookies --test session_cookie_download --locked` exit0: download5PASS/0FAIL, importer14PASS/0FAIL. [원시 결과](../validation/evidence/session-cookie-integration-first-retry/0.log)와 [명령 결과](../validation/evidence/session-cookie-integration-first-retry/result.json)를 보존했다.

CPU 기능 포함 app all-target clippy의 최초실행은UI의collapsible_if로exit101이었다. root가조건식을수정했고같은검사를재실행중이다. 최초실패로그를보존하며재실행성공과혼동하지않는다. 실제nativeGUI·다른OS와미확인process종료시session정리계약은남는다.

같은 검사 재실행 exit0. [재실행 결과](../validation/evidence/session-cookie-integration-first-retry/ui-clippy-retry.json). CPU/accessibility 후보를 포함한native app 컴파일이며실제GUI인수는아니다.

## 보류 처리 포함 app 재검증

`cargo clippy -p dlpad --all-targets --features cpu --locked -- -D warnings` exit0. [원시 로그](../validation/evidence/session-cookie-deferred-integration/clippy.log). 이후 일회성 쿠키 선택을 worker로 옮긴 뒤 입력을 비우고 malformed source로 job을 만들지 않으며 원본과 보고서 비밀배제를 확인하는 CPU app 회귀를 추가했다. `cargo test -p dlpad --bin dlpad-cpu --features cpu --locked cookie_selection_is_consumed` actual1PASS/0FAIL/exit0. [결과](../validation/evidence/session-cookie-deferred-integration/ui-selection-unit.json). 이는UI상태/worker단위인수이며렌더링·실제GUI는대체하지않는다.

## 시작 복구 gate 연결 (공유 API 구현 대기)

root native worker는Linux의sessioncache가있는경우recover_sessions를긴I/Oworker에서수행한뒤jobjournal복구를진행한다. 확인실패는기존readygate를닫고새다운로드를차단한다. deferred/rejected/truncated항목은보존하고native입력옆고정안내를유지한다. 경로·record·쿠키값은UI에노출하지않는다. cache가없으면기존시작복구동작이다. 공유recoveryAPI구현중이므로컴파일/GUI인수는아직미검증이다.

## 복구 API·실제 fault 통합 첫 검증

[5명령 원시 결과](../validation/evidence/session-cookie-recovery-first/result.json) 모두exit0: importer17PASS, cookies기록/secretunit5PASS, 외부cache로향하는symlink의native시작gate1PASS, 인증download7PASS, CPU포함workspacealltargetclippy성공. 실제guardianSIGKILL인증회귀는TerminalUnconfirmed→SessionCleanupDeferred→복사본/기록보존·FD종료·원본/sentinel/DB비밀배제를확인했다.

기록은Linux에서노출전fsync한privateboundedJSON/nonce/identity와kernelbootUUID를사용하며128개capacity/scan,4KiBrecord/512Bmarker상한이다. 이전boot정리는private합성boot경계unit시험이며실제WSL/reboot시험을수행했다고표현하지않는다. 같은boot의미확인복사본은보존한다. 최신전체workspace/release/Windows/GUI와같은boot종료receipt회수는별도남은인수다.

## 29ce8c 시작 복구 GUI 독립 검토

agent가최신29ce8c의인증GUI3flow·unsafecachegate·sameboot보존case를실행했다. root는sameboot의recovery/marker before-after원문동일·현재kernelbootUUID/nonce/recordidentity바인딩·실제DB무결성/exactDownloaded1attempt·artifactbytes를독립대조했다. [검토](../validation/evidence/session-cookie-recovery-native-gui-same-boot-raw/root-review.json). 이는oldcopy보존+freshcopy다운로드인수이고samebootreader종료후자동삭제인수가아니다.

후속postreadyI/O오류분류와panicreaderguard변경은해당29ce8c실행파일에포함되지않는다. 새readerguard단위/통합및전체회귀는별도로진행한다.

## 29ce8c 성공·취소 GUI root 원시 검증

[독립 대조](../validation/evidence/session-cookie-recovery-native-gui/root-review.json)에서실제SQLite무결성/정확job·snapshot·options/1attempt,version쿠키없음,metadata·download동일literalcopy97B/SHA,success39Bartifact실제hash,cancelartifact0,worker·guardian호스트PID/cgroup부재를확인했다. 원본/sentinel은시험종료시임시폴더정리되어기록된전후hash일치만검토한다. 최신reader/startup/Child수명수정과는별도snapshot이다.

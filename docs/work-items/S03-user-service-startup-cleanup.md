# S03: 사용자 서비스 시작 실패 정리와 오류 전달

- 상태: done
- 분류: bug
- 우선순위: P0
- 요구사항: R04, R08, R10
- 담당/수정 범위: root guardian.rs/process.rs·통합·본기록; metrics_recovery user_service.rs·debug fault hooks; soak_recovery_audit 전용실제실패harness/driver/증거
- 선행 작업: native13case와등록앱defaultGUI3case

## 사용자 결과

보호서비스시작중오류를고정오류로전달하고, unit식별을확보하지못해도이미시작한소유guardian/tool의정리를즉시요청한다.

## 계약과 범위

ADR0002/ARCHITECTURE프로세스계약. launcher는CLOEXEC controlalias를보유하고run오류에서bounded EP frame을보낸뒤shutdown(Both). 같은socketendpoint의서비스복제에도EOF를전달하므로미확인unit/PID를검색/kill하지않는다. 앱은R/EP만해석하며환경/argv/DBus원문을오류프로토콜에넣지않는다. 새Cargo/schema없음. 실패hook은debugonly/release무시. 다른agentGUI/비용은기존immutableea7595로분리.

## 인수 조건

실제manager누락에서EP 고정오류·tool미실행. Start성공뒤/GetUnit뒤의실제실패주입에서소유guardian/tool자손·cgroup/unit회수·sentinel보존·FD/pipe종료검증. 정상실행/cancel/timeout과기존R회귀. launcher대기의시간상한은별도경계이며이번socket정리로전체S03완료를주장하지않는다.

## 검증과 handoff

구현전기록. 새source/fullrelease/Windows와실제fault 결과는아직미검증이다.

## 실제 실패 경로 인수

controlsockethandoff/frame6unit PASS·enginealltargetclippy PASS·actualdebug3fault+missingmanager PASS. root가 [오류/exit126/FD4→4/16PID/3cgroup](../validation/evidence/user-service-startup-fault-20261003/root-review.json)을직접대조했다. R-before-fault는ExecutionExited126이며다운로드성공이아니다. missingmanager는Other 고정오류/tool미실행이다. [원시자료](../validation/evidence/user-service-startup-fault-20261003/result.json). alias변경후strict13실제회귀도PASS. 최신Linux전체7명령과WindowsCLI3명령은진행중이므로완료상태는유지하지않는다.

최신Linux전체default213/CPU352PASS/각7ignored와fmt·default/CPUclippy는exit0이다. release2개는실행중이다. 실제WindowsGNUCLI clippy/test255PASS/1ignored/release도exit0·desktop불변·입력261파일SHA현재일치(root재대조)를확인했다. [Windowsraw](../validation/evidence/user-service-fault-final/windows/result.json). MSVC/macOS/WindowsGUI는미검증이다.

## scoped 인수 완료

후속7개Linux명령/Windows3명령전부exit0. releasea67d7224의actualGUI3flow에서도debugfaultenv실제존재/무시를확인했고root가DB·소유PID/cgroup을 [독립대조](../validation/evidence/user-service-app-cleanup-20261003/root-review.json)했다. 이done은소켓시작실패정리/frame기능범위이며별도guardian시작대기상한·다른OS·S03/P0전체미완료는유지한다.

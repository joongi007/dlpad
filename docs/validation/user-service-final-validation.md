# Linux 기본 사용자 서비스 통합 검증

현재Linux통합소스의fmt·default/CPUclippy·default/CPUtest·default/CPUrelease 총7명령모두exit0. 기본210PASS/CPU349PASS/각7ignored이며ignored항목은별도외부도구·사용자manager실증으로구분한다. [명령/경과시간/실행파일SHA](evidence/user-service-final/workspace-result.json)와각rawlog를보존했다.

CPU실행파일12438904B/SHA `ea75955c3335612097b39cf7d9587c3fd1dc13f07ca3bc6ac854b93de5e596cc`. [단독immutable앱](evidence/user-service-final/immutable-app.json)으로기본backend/동일앱guardian·숨김/복귀/종료실제GUI와비용표본을분리검증한다. 이snapshot의 [실제defaultGUI3flow](user-service-app-default.md)는PASS이며root가 [DB·PID·cgroup·동일앱SHA](evidence/user-service-app-default-20261003/root-review.json)를독립대조했다. 이후startupcleanup소스변경/비용표본은별도미완료다.

이전명시backend의strict13case는root가 [53PID/10cgroup원시증거를독립대조](evidence/user-service-engine-full-20261003/root-review.json)했다. 설정scroll의별도1fe649snapshot은 [6case실제휠/입력과18PID종료대조](evidence/cpu-scaled-native/root-review.json)를완료했다. 해당snapshot을이후backend모든UI검증으로확대하지않는다.

정상Git저장소가없음을재확인했고기존.git은보존했다. Windows최신소스/MSVC/macOS/실제사이트/절전/설치·배포및실제8hterminal게이트는이Linux검증으로완료하지않는다. [통합작업](../work-items/S03-user-service-guardian-integration.md)은계속진행중이다.

## a67d7224 후속 source 검증

시작실패alias/EPframe통합후 [Linux전체7명령](evidence/user-service-fault-final/workspace-result.json) exit0, 기본213/CPU352PASS/각7ignored. [실제WindowsGNU3명령](evidence/user-service-fault-final/windows/result.json) exit0/255PASS/1ignored, 입력261파일SHA현재일치·desktop불변을root가재확인했다. CPU12439160B/SHA `a67d7224f3b706547d903fc0058d2a6200efd98609ff906243a2f6a19d21c1a7`. [latest실제GUI3flow](evidence/user-service-app-cleanup-20261003/root-review.json)는실제앱/guardian의debugfaultenv존재와release무시·동일앱guardian·DB/종료정리를확인했다. 이후 [guardian시작대기상한](../work-items/S03-guardian-startup-wait.md)은별도구현중이므로이snapshot으로후속소스검증을주장하지않는다.

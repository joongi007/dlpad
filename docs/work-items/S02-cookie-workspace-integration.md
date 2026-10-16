# S02: 세션 쿠키 변경 전체 통합 검증

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: root — 전체 검증 및 이 기록/전용 evidence; 제품 소스는 검증 동안 동결
- 선행 작업: cookie importer/실행/UI/정리 미확인 타입 및 소유FD보류 연결

## 사용자 결과

합성 쿠키 인수와 기존 다운로드·복구 기능의 회귀를 같은 Linux source snapshot에서 확인한다.

## 계약과 범위

fmt/default+CPU clippy/test/release 7개 명령과 입력 source SHA를 보존한다. 실제GUI는 별도 immutable release에서 수행한다. 다른OS/실제계정/terminalcleanup 전체보장과 첫릴리스완료로 확대하지 않는다.

## 인수 조건

모든 명령 exit0, source 시작끝 SHA 일치, 성공/실패/취소/재개/비밀배제 및 소유권 검사. 실패원시자료는 보존하고 원인을 수정한 뒤 새근거로 재실행한다.

## 검증과 handoff

실행 준비. 장시간 시험과 기존 evidence는 수정/재시작하지 않는다.

## 첫 전체 검증 실행

source-start.json을보존하고fmt/defaultclippy실제exit0을확인했다. default workspace test가진행중이며이후CPUclippy/test와두release빌드를순차수행한다. [누적 명령 결과](../validation/evidence/session-cookie-workspace/workspace-result.json). terminalcleanup프로토콜검토는문서작업으로분리하며이snapshot의Rust소스를동시에수정하지않는다. 기존5개에추가된malformedduration회귀도본workspace시험에포함한다.

## 전체7명령 완료

fmt/defaultclippy/defaulttest/CPUclippy/CPUtest/defaultrelease/CPUrelease 모두exit0. 기본246PASS/0FAIL/7ignored, CPU387PASS/0FAIL/7ignored. source 시작끝 SHA 전체 일치. [시험집계](../validation/evidence/session-cookie-workspace/test-counts.json). CPU 실행파일12,475,256B/SHA5cb52a14이며격리GUI용immutable복사본을만들었다. [실행파일 근거](../validation/evidence/session-cookie-workspace/immutable-app.json). 같은소스의실제GUI/다른OS검증은이전체검증결과와구분한다.

## 복구·terminal 결합 최종 snapshot 검증 시작

이전5cb52snapshot의전체통과/GUI원시근거는보존한다. durable복구기록과terminal확인을포함한현재소스의fmt/default+CPUclippy/test/release7명령을새evidence/session-cookie-recovery-workspace에실행한다. Rust/Cargo입력SHA시작끝동결을검사한다. 같은boot회수계약은독립문서검토이며이빌드범위완료로대신하지않는다.

## Windows GNU 별도 검증 시작

동결한현재소스의WindowsnativeGNUclippy/test/release를별도새evidence/session-cookie-recovery-windows에실행한다. Linux빌드와별도target/toolchain이며시작끝Rust입력SHA와inputdesktop불변을검사한다. Windows쿠키ACL/GUI/IME/MSVC를검증했다고확대하지않는다.

## 현재 검증 진행

Linuxfmt/defaultclippy/defaulttest/CPUclippy/CPUtest exit0,두release진행중. WindowsnativeGNUclippy/test와Rust입력SHA·inputdesktop불변을확인했고release진행중이다. 완료하기전release성공/OSGUI성공으로표시하지않는다.

## 복구·terminal 결합 전체 검증 완료

Linux전체7명령exit0,기본262PASS/0FAIL/8ignored,CPU405PASS/0FAIL/8ignored. Rust입력시작끝SHA일치. 최신CPU29ce8c/12,512,376B를별도immutable로보존했다. [전체 결과](../validation/evidence/session-cookie-recovery-workspace/workspace-result.json), [실행 파일](../validation/evidence/session-cookie-recovery-workspace/immutable-app.json).

WindowsGNUactualnativeclippy/test/release3명령exit0,262PASS/0FAIL/1ignored. 세입력snapshot현재파일SHA일치와inputdesktop불변근거를대조했다. [root 검토](../validation/evidence/session-cookie-recovery-windows/root-review.json). Windowsexe7,256,064B/SHA560e063c이다. Windows쿠키ACL/GUI/IME/MSVC/macOS완료를뜻하지않는다. 새29ce8c actualGUI는기존5cb52근거와별도로수행한다.

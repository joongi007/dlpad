# S04: 관리 실행 공유 잠금 기반

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R08
- 담당/수정 범위: root — tools/execution_lease.rs, tools.rs
- 선행 작업: 동일 root install/mutation lock 및 native 원파일 확인

## 사용자 결과

실행 소유자가 보유하는 공유 잠금과 관리 변경의 배타 잠금이 충돌하도록 API 기반을 마련한다. 다운로드 worker/guardian 수명 연결은 별도 작업이며 이 기반만으로 사용자 실행 보호 완료를 주장하지 않는다.

## 계약과 범위

기존 .install-transfer-lock을 shared로 획득하고 원root/lockfile identity를 확인한다. ExecutionLease는Drop시FD닫기만 사용하며 공유 open-file-description 전체를unlock하지 않는다. 여러 실행lease 동시허용, lease중기존mutation배타잠금거절, 마지막lease종료뒤mutation허용을검증한다. 새의존성/schema/상주worker 없음.

## 인수 조건

공유lease둘/배타경합/해제, foreignlockfile/root교체·symlink에대한거절과외부sentinel보존. 정상in-process guard는실제child실행/appcrash/TerminalUnconfirmed보호로확대하지않는다.

## 검증과 handoff

구현 후 실제 scoped시험과Clippy결과를기록한다. guardian의독립descriptor전달또는durablepending회수계약은 S04-managed-execution-lifetime.md에서다룬다.

### 실제 scoped 검증

`cargo test -p dlpad-engine --lib tools::execution_lease --locked` Linux2PASS: 두공유lease의마지막release전배타mutation거절,exclusive보유중새reader거절,symlinklock이외부원bytes를보존하며UnsafePath반환. 다운로드/guardian연결은아직없고TerminalUnconfirmed/crashsafe실행보호완료가아니다. enginealltargetClippy는실행중이다.

`cargo clippy -p dlpad-engine --all-targets --locked -- -D warnings` exit0. Windows/macOS 실행잠금 native 인수는 별도다.

## 독립 descriptor 종료 회귀

원 shared file descriptor를 duplicate한뒤 parent ExecutionLease를drop해도readeralias가존재하는동안exclusive mutation을거절해야한다. Unixnativealias시험을추가했고WindowsLockFileEx수명의동일의미를가정하지않는다. process/guardian explicitcapability전달·servicekeeper·Windowsnativehandoff는후속계약그대로유지한다.

Linuxdescriptoralias추가후3시험PASS. CONTEXT와ADR-0003에용어·close-only근거및미완료수명범위를반영했다.

## 독립 프로세스 capability 실험

소유Python합성reader에명시FD를상속하고실제ready파일을확인한뒤parentlease를닫는다. 별도processreader가원FD를보유하는동안mutation거절,명시finish/actualexit후허용을검사한다. 소유ChildDropkill/wait와5초deadline으로실패경로도정리한다. 이실험은기존제품guardian/service와appSIGKILL수직시험을대체하지않는다.

Linux독립reader상속추가후4회귀PASS. originalcapabilityparentdrop뒤actualchildreader가보유한FD때문에mutation거절,ready→finish→actualexit후허용을확인했다. 실제서비스guardian제품연결및GU/앱crash수직인수는후속이다.

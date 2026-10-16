# S03: Windows helper launch capability

- 상태: drafted; root 게시·통합 검증 대기
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R02
- 담당/수정 범위: soak_recovery_audit — /tmp/dlpad-windows-helper-launch-draft.rs, 이 문서
- 선행 작업: root GuardianConfig 및 spawn_independent_helper API

## 계약과 인수

private anonymous pipe 4096-byte 四組와 original receipt의 새 inheritable duplicate만 helper에 전달한다. child HANDLE_LIST는 control read/events write/receipt/stdout write/stderr write 정확 다섯 개이다. helper는 tool Job에 넣지 않는다. launch 이전 config를 검증·serialize하며 Launch가 length-prefixed bounded bytes를 반환한다. launch 자체는 pipe에 config를 blocking write하지 않는다. caller의 cancel/deadline-aware config 전송은 후속 통합이다. parent/module/Cargo/기존 Rust는 root 담당이며 게시 GO 전 tmp만 작성한다.

## 검증과 handoff

원 receipt inheritance flag는 변경하지 않는다. 프로세스는 root spawn API의 original handle RAII Child로 소유한다. native inheritance/EOF/동시실패/config 전송 timeout 검증은 root 후속이고 이 초안 자체로 public gate 완료를 주장하지 않는다.

tmp SHA256 `4504769d736ce7734c6e42861a774ff9f494ec37fcc02b5affd13e5d11e7fa44`, rustfmt 실제 exit 0. Cargo·native 실행과 project Rust 게시는 하지 않았다. 내부 marker 초안은 `--dlpad-internal-windows-cookie-guardian`이며 root entry와 일치시켜야 한다.

256 KiB config는 4 KiB pipe보다 크다. worker thread에서 수행해도 무기한 write_all은 cancel/deadline 보장이 아니다. Launch는 bounded bytes만 반환하고 호출자가 전송을 책임진다. root 후속은 owned writer의 interruptible/overlapped I/O 또는 bounded 작업자의 원 handle 기반 취소 및 종료확인을 마련해야 한다. private file capability로 바꾸면 정확 다섯 HANDLE 계약과 config provenance를 다시 합의해야 하므로 이 초안에서 도입하지 않는다. helper 원 process creation identity getter와 native config deadline 실패 회귀 역시 후속이다.

## Root 통합과 실제 독립 helper

root가 windows_helper_launch.rs를 게시하고 windows.rs의 independent-helper 생성 API를 추가했다. Attributes는 helper만Job목록없이명시HANDLE_LIST를사용하고 기존도구는원자적Job할당을유지한다. helperstdin/stdout/stderr도5caps안의원handle에고정한다. 원helper process handle의 GetProcessTimes를 Preparedframe과비교한다.

앱소유controlwriter에PIPE_NOWAIT를설정하고4096byte이하쓰기·0bytes일때10ms대기·원child종료/취소/absolute deadline확인을연결했다. config는전송시한번만소비하며성공전에Begin을허용하지않는다. 무기한write_all이나새writerthread를도입하지않았다. [Microsoft API](https://learn.microsoft.com/en-us/windows/win32/api/namedpipeapi/nf-namedpipeapi-setnamedpipehandlestate)는익명pipehandle의waitmode설정을설명한다. overlapped I/O를구현했다고표현하지않는다.

실제nowait-native-third는4096bytes쓰기9us,fullpipe쓰기0bytes/1us,drain후128bytes전송,readerclose뒤쓰기실패를exit0로확인했다. native-first는링커gcc라이브러리부재exit1,second는64KiB쓰기에서부분전송을가정한assertion실패exit101이다. 반환값은출력되지않아0/전체전송중어느것인지확정하지않는다. 두실패로그를보존하고검증된self-contained링크·4KiBchunk를사용했다.

windows-independent-helper-native-first: enginealltargetClippy exit0,최신guardbinary빌드exit0,정확1native독립helper시험PASS. originalcreation일치/정상nativeexit0/실제256receipt/원journal재획득·resolve를root가원시로그로확인했다. binarySHA/크기는helper-artifact.json에고정했다. sourceunchanged/inputdesktopunchanged=true. 앱실제강제종료/전송중취소/largeconfig마지막기한/managementhandle비상속전체인수/publicstartup은미완료다. 이전runtime전체Windows4및Linux7+2검증은terminalPASS이며새launcher변경이전snapshot이다.

# S03: Windows headless guardian entry

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — process/windows_guardian_entry.rs, process.rs 연결, 이 문서
- 선행 작업: 원Job runtime, boundedconfig, fixedframes, journalhandoff

## 사용자 결과

동일 네이티브 앱의 내부 명령이 GUI/DB 초기화 전에 독립 guardian을 실행한다. cookie path/argv/config는 private pipe 안에서만 전달한다.

## 계약과 범위

내부 marker와 5개의 원 inherited handle(읽기config/control, 쓰기events, 원receipt writer, toolstdout/stderr)을 argv로 전달한다. 숫자만 들어있는 argv는 삭제·PID 권한이 아니다. 서로 다른 원handle/type/inheritflag 검증 후 inheritflag를 해제하고 RAII로 소유한다. config는 길이prefix+256KiB상한, PeekNamedPipe와10초초기수신기한, fixed오류만 사용한다. standaloneparentlauncher와 cookie 제품 wiring은 후속이며 공개회수완료로표시하지않는다.

## 인수 조건

invalidmarker는일반앱기동을유지하고 invalidarity/handle/type/config/EOF/deadline은 tool0/fixedfailure이다. 실제 helper프로세스와 원Job 정리·receipt·현재nativeexit를 fixture로 확인해야 한다.

## 검증과 handoff

GUI·사용자desktop·실계정·설정변경없음. implementation/native 인수는실제명령과 결과를추가한다.

내부 marker/5nativecapability검증 및 boundedconfig수신을 구현해 process::guardian_entry의 Windows분기에연결했다. native engine all-target Clippy exit0를확인했다. 실제 독립 helperentry/invalidargs/configEOF/handle inheritance 인수는미완료며Requestthread5case로대체하지않는다.

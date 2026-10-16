# S02: Windows 인증 다운로드 gate 수직 회귀

- 상태: in-progress
- 분류: test
- 우선순위: P0
- 요구사항: R03, R04, R08, R11
- 담당/수정 범위: metrics_recovery — 신규 crates/engine/tests/windows_cookie_download_gate.rs 및 이 작업 문서; 소스 동결 동안 /tmp/windows_cookie_download_gate.rs 초안만 작성
- 선행 작업: root Windows 다운로드 gated adapter; 최신 Windows dlpad-process-guard.exe 및 session_cookie_fixture_tool.exe 빌드

## 사용자 결과

실제 Windows .exe 합성 도구로 인증 다운로드 성공, metadata 실패7, download 실패9, 실제 download 시작 이후 취소를 검증한다. 각 경우 정상 terminal receipt와 원래 source 보존을 함께 요구한다.

## 계약과 범위

기존 Linux cookie_download_gate 수직 시험을 참고하고 기존 session_cookie_fixture_tool 예제를 변경 없이 재사용한다. root가 새 예제를 빌드하고 DLPAD_WINDOWS_COOKIE_FIXTURE_EXE 절대경로를 제공해야 한다. 합성 temp 루트 이름과 capability 파일이 예제의 엄격한 시험 제한에 일치한다. 사용자 Documents, 실제 계정, GUI, Cargo 설정은 사용하지 않는다.

## 인수 조건

version에는 cookie argv가 없어야 하고 metadata/download에만 원 private path 및 합성 내용 SHA가 전달된다. 반환 성공·실패 로그 및 저장 DB에 cookie secret/path가 없어야 한다. 성공 artifact 바이트, 실패 stage/exitcode, 취소 outcome을 확인한다. 원 journal slot0 Resolved+receipt256, metadata 실패 시 slot1 Empty+receipt0, 나머지 slot1 Resolved+receipt256을 실제 파일로 확인한 뒤 lease cleanup으로 private copy가 제거되고 source/sentinel이 보존되어야 한다. actual worker-ready를 보고 취소하며 sleep만으로 시점을 추정하지 않는다. 각 worker subprocess는30초 유한 상한과 소유 child 종료/reap를 가진다.

## 검증과 handoff

현재 /tmp 초안만 작성했다. 프로젝트 Rust는 root sourcefreeze 때문에 게시하지 않았다. Cargo/native 실행 및 Windows 성공 주장은 없다. 최신 helper와 example의 명시 빌드 및 hidden CLI에서 native_authenticated_ 4개 ignored 시험 실행이 root 선행 검증이다. 실제 journal state의 읽기는 인수 관찰일 뿐 새 회수 권한을 부여하지 않는다.

## Root 실제 제품 native 검증

first4case실패: 신규test cache명과기존fixture private-cache allowlist 불일치(metadata64); second4case실패: 실제실행결과 assertions 통과후 locked reader-state.json 독립fsread가WindowsERROR_LOCK_VIOLATION33. 두 실패자료 유지. root가 fixture경로를맞추고 원lease.defer_cleanup 뒤 독립journal/receipt관측→publicrecover실제원사본회수로 시험계약을수정했다. third Windows DownloadRepository 실제success/metadatafailure/downloadfailure/cancel4PASS/0failed/1filtered, exit0/sourceunchanged/inputdesktopunchanged. root raw log/JSON직접확인. 실제slot0/1Resolved 및receipt256(metadatafailure slot1Empty/receipt0), 출력/DBsecret·privatepath생략, artifact/source/foreign보존/publicrecover1 확인. currentnativehelper와고정example는first에서명시buildexit0. 단독lease.cleanup을4case에서실행했다고주장하지않는다(publicrecover가원cleanup을검증). 변경후전체workspace/GUI/3OS인수남음.

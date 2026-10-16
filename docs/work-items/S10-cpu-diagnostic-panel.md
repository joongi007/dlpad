# S10: CPU 진단 panel

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R13, R08, S10 (최초 릴리스 P0)
- 담당/수정 범위: library_identity — 신규 crates/app/src/accessible_diagnostics.rs 및 전용 docs
- 선행 작업: 기존 DiagnosticReport/reports API, root후속 module/Candidate 연결

## 사용자 결과

진단 Markdown/JSON 검토·로그수정/제외·MPD구조첨부/제외·명시복사·새파일export를CPU화면에서제공한다.

## 계약과 범위

DiagnosticPanel new/show/poll/busy/request_stop/stopped. 파일I/O singleworker/channel1, preview2MiB/input32KiB/MPD1MiB. 보고서원본이동/동작중변경disable. 외부전송/cache/기존파일overwrite/자동삭제없음. 기존catalog재사용,새문구root요청. 제품기존파일/Cargo/schema수정금지.

## 인수 조건

private합성temp의exportexistingpreserve/MPDoversize/수정재sanitization/workerpanicstopack/상한/stop회귀. UIthread I/O/unfinishedjoin없음. root연결후lint/test/privateGUI별도.

## 검증과 handoff

신규panel/rootCandidate/entries/catalog연결및ownfmt완료. cachedpreview·report이동·singleworker·실패회귀7개실제실행통과. rootLinuxCPUentry27tests/Clippy/release37.35초 및후속전체fmt/default193/CPU242/fullrelease통과. 생산CPU SHA8fb4a67c27ff89fd883ff6fddaa9136a5fc36c62d74c049665543a513ab06986는fixturefix전후동일.

WindowsGNUCPUClippy/test/release exit0/167passed1ignored이며진단7개회귀는accessible/CPU양binary에서통과했다. raw입력237개/runner/lock/binarySHA와최초fixtureAlreadyExists실패는각전용prefix에보존했다: docs/validation/windows-cpu-diagnostics-fixturefix.md.

현재in-progress는진단전체actualGUI인수가진행중이기때문이다. 모듈7회귀성공은실제previewformat/clipboard/logeditomit/localMPD/export/keyboardreader/3OSGUI성공을대신하지않는다. P0S10이최초릴리스필수인것은유지한다.

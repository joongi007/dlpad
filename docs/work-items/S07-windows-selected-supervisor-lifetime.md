# S07: 명시 선택 실행의 감독 수명

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — 공개 API·CLI/공유 계약·통합·native 검증; diagnostic_cache — /tmp 설계/patch 초안
- 선행 작업: 원 active selection/launch capabilities와 MSVC 기본 CLI 실제 실행·취소·기한 인수. 원 MSVC artifact 검증을 마친 뒤 Rust를 통합했다. 성능의 간헐적 초과 조사는 별도 작업에서 계속한다.

## 사용자 결과

명시 선택 앱을 개발 시험의 최대600초 수명과 분리해 원 감독 프로세스가 정상 종료까지 보호할 수 있게 한다.

## 계약과 범위

원 등록/receipt/active 세 경로와 startup 검사 기한, suspended child·Job-before-resume·원 image identity·cancel/natural/cleanup 계약을 유지한다. deadline 없는 감독 wait API를 명시적으로 구분하며 분리된 child detach·경로 자동 탐색·삭제/게시 권한은 도입하지 않는다. 새schema/dependency/lock은 필요하지 않다. 담당 파일은 crates/engine/src/app_install/active_launch.rs 및 crates/engine/src/bin/dlpad-local-install.rs이고 root가 공개 API와 CLI를 통합한다. 정식 GUI bootstrap/설치·upgrade/rollback 완료로 확대하지 않는다.

## 인수 조건

기존 launch-selected 기한 동작과 취소 실패 경로가 유지되고, 새 감독 수명에서 startup 기한이 실행 후 수명으로 잘못 적용되지 않으며 자연 종료/취소/cleanup 미확인 실패를 typed 결과로 구분한다. Windows 원 Job/private GUI/native image 보존을 실제 인수한다. 계획/초안만으로 완료하지 않는다.

## 검증과 handoff

Rust 구현을 통합했다. 실제 GUI 수명 인수는 진행 중이며 전체 P0 완료가 아니다.

root는 원source SHA와 초안을 대조하고 wait_cancelled(None/100ms)와 run-selected CLI를 통합했다. 기존 bounded wait(Some/20ms), 원 startup 검사 기한·Job/creation/cancel/cleanup 정책과 stdout capabilities 한계를 유지한다. runtime_deadline_seconds=null과 startup_timeout_seconds를 명시하며 새 시작 기한 flag가 다른 명령에 섞이면 거절한다. default wait 정책이나 Cargo/lock/dependency/schema를 바꾸지 않았다. 두 parser 실패 회귀를 추가했으며 native format·전체 검사·실제 수명 인수는 다음 단계다.

2026-10-06 corrected 전체 MSVC fmt/clippy/test/release 실제 exit0, 760passed/0failed/13ignored·설치89를 root와 독립 담당이 확인했다. 첫 clippy collapsible_if 실패를 보존하고 수정 후 재검사했다. 새 observer/collector 담당 파일은 scripts/windows_run_selected_cli_probe.cs와 scripts/windows_msvc_run_selected_cli_collect.py이며 diagnostic_cache의 /tmp 초안을 root가 통합한다. 각각 시작2초 기한을 넘긴4초 실행 유지·자연 종료·취소와 파싱8실패·Natural17·다른묶음/foreign거절·원nativeID10 보존을 실제 확인해야 한다. GUI는 한 번에 하나만 실행하고 문서/독립 검토는 병렬로 수행한다.

새 run-selected 실제 수집은 windows-msvc-run-selected-cli-real-20261006-continue에 보존했다. 원17명령/14사례와 GUI3개의 시작2초 뒤4024/4012/4006ms 유지, Natural0 두 번/Cancelled 한 번, 파싱8거절/Natural17/다른묶음/foreign거절·nativeID10·원파일·readonly SQLite Light/integrity·private PNG를 root/독립 검토했다. 원 REVIEW_REQUIRED·예상 CLI1의 passedfalse를 보존하며 supplemental 검토로 범위 인수한다. 현재 기존 launch-selected의 새 바이너리 회귀를 실행 중이다. 부모 종료 후 자식만 남은 분기는 S07-windows-selected-descendant-lifetime의 후속 실제 시험이며 정식 bootstrap/installer·전체 P0는 미완료다.

2026-10-06 범위 완료: 새 바이너리의 기존 bounded launch-selected 실제15명령/13사례·GUI4(Natural/재시작/Cancelled/TimedOut)·nativeID10·원파일/DB도 root/독립 검토했다. 기존 기한 정책을 보존했다. 원 corrected whole와 새 run-selected/legacy raw 및 supplemental 검토는 docs/validation/evidence 아래 각 고정 snapshot 폴더에 보존한다. 후속 descendant fixture는 새 source를 만들므로 이 인수를 그 snapshot에 소급하지 않는다.

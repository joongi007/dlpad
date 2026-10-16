# S02: CPU 화면 설정 통합

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R06, R08
- 담당/수정 범위: 주 담당; accessible.rs, accessible_main.rs, accessible_cpu_main.rs, software_host.rs, catalog.rs, 본 문서. accessible_settings.rs는 library_identity; 새 GUI harness는 diagnostic_cache.
- 선행 작업: S02-theme-readonly-store, S02-cpu-appearance-settings

## 사용자 결과

설정에서 System/Light/Dark와 75–200% 배율을 선택한다. 저장 테마는 창 생성 전에 적용하며 저장 중 종료는 마지막 선택 처리 뒤 종료한다.

## 계약과 범위

기존 theme.sqlite/theme key와 enum JSON 유지. 배율은 기존 정책대로 session-only. 긴 저장 IO는 worker에서 실행. 공유 schema/Cargo 변경 없음.

## 인수 조건

공통 Rust 검증, 설정 실패·종료 drain 시험, 격리 실제 GUI 테마/배율/재실행 시험과 비용 확인. OS 미검증은 구분한다.

## 검증과 handoff

공유Store 7시험 PASS(5.15초); CPU fmt/clippy exit0(17.08초), entry 38시험 PASS(0.24초), release exit0(40.43초). CPU SHA256 `52b55a78b4613bf2b0a44666959e1e0976527e055843cfcc575565a248758dec`, 10488728B(이전 S10 대비 +51584B). 의존성/schema/lockfile 변경 없음. 설정 worker1thread, command/result 각capacity1와 최신요청1개가 추가되며 idle I/O 없음. startup SQLite busy 최대5초는 별도 첫표시 측정 게이트다.

공통 `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`, `cargo build --workspace --release --locked` 전부 exit0; 기본197 passed/0 failed/5 ignored, CPU workspace268 passed/0 failed/5 ignored. CPU workspace시험은 `--features cpu` 추가 실행. default release38.23초, docs223 PASS. Windows GNU CPU clippy/tests/release exit0:188 passed/0 failed/1 ignored. root가 Windows238개현재입력·3명령before/after·최종exehash를 독립 대조했다. [Windows 증거](../validation/windows-cpu-appearance/result.json). Windows GUI·최종symlink거부 및 macOS는 미검증.

초기 Linux GUI 수집기는 동일명 static label을 잘못 선택했다. 당시 재검증 중 상태를 유지했고 원 실패를 보존했다. 아래 최종 인수에서 실제 combo role과 새 source의 GUI를 검증했다.

## 최종 bounded Linux 인수

[테마별32px수정](S02-theme-control-sizing.md)까지최종f052fa5b...바이너리로구현·통합·실제GUI검증했다. root가styles32 raw의3개테마 저장/restart/ComboBox높이32·팝업닫힘·200%entry abc/양방향포커스·DB잠금보유생존/해제후175ms exit0/최종Light값을직접확인했다. [실제 GUI/IME 기록](../validation/cpu-settings-validation.md) 및PNG를대조했으며IME자동collector원exit1을PASS로바꾸지않았다. 최종CPU workspace268/0/5,WindowsGNU188/0/1와238입력root독립대조PASS. baselineRSS50820KiB/idleCPU0.10%는10초표본이고peak/장시간비용이아니다. source새의존성/스키마변경없음.

이설정수직기능의boundedLinux인수를완료한다. System OS 변경/WindowsmacOSGUI/전체reader/busyDB첫표시,TextEdit21px→32px와전체P0기능이식은남는다.

# S04: installer 크래시 staging 증거 회복과 용량 제한

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R08
- 담당/수정 범위: diagnostic_cache agent; crates/engine/src/tools.rs 및 tools 하위 모듈/전용 tests, docs/work-items/S04-staging-recovery.md. docs/validation/managed-tools-validation.md(주 담당 승인). staging_schema.sql은 주 담당 작성/소유.
- 선행 작업: 직접 executable download/install 및 bounded 관리 inventory

## 사용자 결과

installer가 비정상 종료되어도 소유 증거가 충분한 종료된 staging만 정리한다. 증명되지 않은 파일은 남기고 용량 제한으로 추가 설치를 막아 사용자 자료를 보호한다.

## 계약과 범위

새 API/schema는 주 담당과 먼저 합의한다. 소유 root와 persisted directory/file identity·정확 bytes/hash, Linux owner PID/startticks/boot identity의 읽기 대조, strict known files가 cleanup 조건이다. 파일명이나 PID 숫자만으로 삭제하지 않는다. 원본·외부 도구·영상·진단 export를 cache로 취급하지 않는다. 무증거·live/unknown owner·변조·symlink·외부 파일·회복 실패는 보존한다. 비 Linux owner identity/비 Unix file identity 증거가 없으면 보존 gate를 유지한다.

## 인수 조건

실제 tempfile process SIGKILL fixture의 종료 staging 정리, live owner 보존, identity/hash 변화·foreign file·symlink·무증거·실패 경로 보존과 bounded staging/entry capacity를 검증한다. 기존 public download/list/select/active 계약은 유지한다.

## 검증과 handoff

공유 계약과 schema를 주 담당과 합의하여 구현했다. `ToolManager::recover_staging()`은 removed/preserved/bytes를 반환하며 새 직접 설치는 회복 후 예약한다. dedicated activation DB의 install_staging evidence journal을 사용하고 library schema를 변경하지 않았다. 최대8 stage/합계512MiB/2048 filesystem entry, 새 executable 예상 bytes+64KiB manifest 예약을 적용한다. transfer lock으로 동시 설치 예약을 직렬화한다. 파일 내용 증거는 1MiB마다 sync/기록하며 마지막 checkpoint 밖 partial bytes는 추측 삭제하지 않는다.

실행: `cargo test -p dlpad-engine --lib tools::staging_recovery::tests --locked` 9/9 통과(0.22초); `cargo test -p dlpad-engine --lib tools:: --locked` 27 통과/공개 net1 기본 ignored(1.66초); `cargo clippy -p dlpad-engine --all-targets --locked -- -D warnings` 통과(8.49초); 네 tools Rust 파일 rustfmt check 통과. 공개 net 시험을 명시 실행하여 공식 yt-dlp pin40,446,224 byte HTTPS 다운로드→checkpoint→version→publication 임시설치1/1 통과(6.48초).

초기 전체 fixture 재시험1회에서 HashMismatch 기대가 실패했다. 일반 hash failure fixture의 시험 전용 deadline을1초→5초로 바꾸고 실제 오류를 출력하는 assert를 추가한 뒤 exact와 전체 재시험이 통과했다. 최초 실패의 실제 반환값은 기록되지 않아 원인을 확정하지 않는다. 제품120초 deadline과 의도적 timeout fixture 상한은 변경하지 않았다.

실제 owned fixture process SIGKILL 전 live 보존/후 cleanup, PID 재사용·boot 변화, 동일 bytes의 inode 변경, hash 변경, foreign file, symlink, 무증거, nested unknown, 합계 용량과 개수 초과를 검증했다. UID1000에서 chmod0500의 실제 cleanup PermissionDenied 및 SQLite delete trigger 실패 때 기존 active/external bytes 보존을 확인했다. 성공한 activation 이후 journal 정리 실패를 설치 실패로 반환하지 않는다.

Linux/WSL에서만 owner PID/startticks/boot 증거 회복을 실증했다. Windows/macOS는 owner/handle 증거가 부족한 잔여물을 보존하고 용량 제한으로 새 설치를 막는다. native Windows 최종 snapshot 시험은 별도 담당이다. std raw path 검사/unlink 사이 악의적인 동일 사용자 race를 완전히 차단하지 못하며 handle-relative cleanup은 잔여 gate다. final publication 도중 incomplete destination directory crash는 staging 회복 범위 밖 후속 gate다. 따라서 전체 installer crash/3OS/P0 출시 완료로 해석하지 않는다. Git/.git 및 외부 설치·원본·사용자 export 보존, Cargo 변경 없음.

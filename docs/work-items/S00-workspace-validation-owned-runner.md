# S00: 소유 process group으로 고정 workspace 검증 수집

- 상태: implemented (정상 실제 workspace PASS; 취소/timeout 후속)
- 분류: validation-tool
- 우선순위: P0
- 요구사항: R08, R11, R13
- 담당/수정 범위: stress_recovery; 신규 scripts/rust_workspace_validation.py 및 이 문서
- 선행 작업: 기존 wsl_interruption_snapshot.py --launch boot/PID/start 기록 계약 읽기

## 사용자 결과와 계약

고정 기본fmt/clippy/test/release와CPUclippy/test/release7개를 serial로 수행하며fresh evidence/각8MiB stream/각최대30분/durable runner-command boot/PID/start를남긴다. Cargo실행은root전용이다. 임의명령/shell/ignored nativefilter를받지않는다. tool은root환경PATH만사용하며사용자home경로를하드코딩하지않는다. Rust/Cargo/Windowscollector/기존script 수정없음.

## 인수 조건

작업 실행 전 fsync launch 기록과 고정 Python gate→exec로command PID/start를 유지한다. source before/after/final/lock/toolchain/vendorSHA와 exacttoolversion, fixed env안내, actualargv/exit/rawlog. 소유processgroup+identity에만cancel/reap하며boot/PID가달라지면signal하지않는다. source변경/outputoverflow/timeout/cleanup미확인은PASS아님. 일반 test가ignored native 인수를완료했다고표현하지않는다.

## 검증과 handoff

새 CLI/parser/Python구문만 검증하며Cargo/toolversion subprocess 실행은root후속이다. 기존WSLsnapshot은archive_owner등3role만읽으므로collector runner를archive_owner(실제evidence작성owner)로같이기록하며고유role을명시한다. 각command identity는해당launch.json에별도command필드이며현재snapshot이자동관측한다고주장하지않는다.

## 구현과 실제 검증

고정7명령은 기본 cargo fmt --all -- --check, clippy --workspace --all-targets --locked -- -D warnings, test --workspace --locked, build --workspace --release --locked 및 CPU clippy/test/release에 --features cpu를 추가한 것이다. 정확 version은 fixed cargo/rustc --version --verbose 두 probe로 별도 수집하며7check 숫자에 더하지 않는다. CLI는freshabsoluteevidence/execute/seconds만 받고 임의shell/argv/nativefilter를받지않는다. root PATH로 resolved cargo/rustc를 사용한다.

출력directory는 기존 canonical project evidence 안의 fresh leaf이며 symlink/dotdot/existing을 거절한다. runner launch.json과 각command-launch.json은 create-exclusive0600/no-follow, file+parentdirectory fsync로 durable 작성한다. command는 새 session의 fixed Pythonworker가 readonly gate에서 기다리고, 부모가 원 boot/PID/start/session/group/argv/toolSHA를 저장한 다음 G byte+EOF를 허용해 os.execv한다. exec 후 같은 PID/start를 유지하여 작업전 command identity가 보존된다. launch state는현재liveness가아니며완료result를별도로써야한다.

각 로그는8MiB/64KiB streaming. command budget≤1800초/toolversion10초, cancellation은별도유한2초TERM grace+8초reap확인이다. SIGTERM/SIGINT는flag만설정해owned cancellation으로처리한다. leader를reap하지않은상태에서 boot/동일PIDstart/group/session을검증하여본인group만TERM/KILL한다. PID변경이면signal0. /proc readonly group inventory는최대4096항목으로bounded하며초과/남은descendants는미확인failure다. Popen-ownedChild는signal권한과독립으로reap를시도한다. detached/systemd manager소유별도group의후손을이runner가소유했다고가정하거나임의signal하지않는다. 이한계때문에workspacePASS가actualnativecleanup인수를대체하지않는다.

source manifest는Cargo.toml/lock/toolchain/cratesRust/manifest/vendor전체파일 및현재runner자기파일을포함한다. 다른Python·문서·Windowscollector는Rustcompile입력이아니므로watch에서제외한다. 새Rusttest파일은포함하며source변경은PASS아님. toolversionraw와resolvedexecutableSHA를각launch에남긴다. 일반test의ignored native시험은완료false를명시한다. 환경값전체는출력하지않고root제공CARGO_TARGET_DIR/RUSTFLAGS/CARGO_BUILD_JOBS/CARGO_NET_OFFLINE만선택기록하며사용자homehardcode는없다.

실제 `python3 -m py_compile scripts/rust_workspace_validation.py`, `python3 scripts/rust_workspace_validation.py --help` 및직접import후고정7argv/locked/noignored/invalidpath순수parser확인PASS. Cargo/toolversion/실제childsubprocess는0회다. source SHA256 `f4fb20ebb7b4e52d43184378f4f13ca4645ae48f133b20a5cb0b0ca62d55e2bd` freeze. 다음cycle에서 root `python3 scripts/rust_workspace_validation.py /mnt/d/project/dlpad/docs/validation/evidence/<freshprefix> --execute --seconds1800`을실행한다. 현재수동rootcycle와병렬Cargo를실행하지않았다. 실제timeout/outputoverflow/취소실험은아직미검증이며첫실행원시로한계를검토해야한다.

## durable directory와 startup race 보강

fresh evidence mkdir를 canonical project parent FD에 no-follow component walk로 anchored 생성하고 부모directory fsync를 수행한다. manifest file 생성도 pinned parent dirFD에 create-exclusive/no-follow로 수행한다. source hashing은 no-follow 모든directory/file, regularfile검증 및 hash전후 dev/inode/size/mtime/ctime 비교를 적용한다. crates/vendor symlink는외부contents를읽지않고거절하며FIFO는nonblockopen후regular검증으로거절한다. tool executable은root PATH 허용된해결결과이므로source의project-only검증과별도다.

구체 startup 결함은 Popen 성공 뒤 identity읽기실패로owned=None이면기존except가cleanup을건너뛰어Childwait를하지않는것이었다. now 미개방execgate를먼저EOF로닫고Popen-ownedChild만8초boundedwait하며추정PIDsignal0이다. identity불일치group을terminal확정으로표시하지않는다. rawlog의with블록finally에서flush/fsync를수행하여timeout/취소/outputoverflow 예외경로도원시를durable로남긴다. 작업/제품timeout은변경하지않았다.

실제 pure/tmpfixture로 mkdir부모fsync nativeinode 호출, create-only중복거절 및원문보존, regularsourcehash/symlink/FIFO/parent-symlink/역외출력거절을확인했다. Cargo/subprocess 실행없는순수검증PASS.

첫 actualPythonidentityinjection은agent PATH의cargo 미발견으로Popen전ValueError에그쳤다. `evidence/rust-workspace-runner-startup-read-failure-first/launch.json`을보존하며이를childcleanupPASS로세지않는다. 새second는parent commandresolve만syntheticPython --version으로testpatch하되이를실행하지않고실제동결runner의Pythonworker를spawn했다. identity-read OSError주입뒤gateEOF로worker가125종료, Child697525실제reap와/proc부재, signal0/gateG0/Cargoexec0/sourceunchangedtrue를확인했다. actual-child와proof.json에원boot/PIDstart/argv및testpatch범위를기록했다. 이실험은identity획득전오류의실제Pythoncleanup증거이며Cargo동작/일반취소/timeout완료를대체하지않는다.

최종source SHA256 `3839ea7e378e6de800ed9964e18fbf457c5cd45ef7bccb8a8ef6c31d107e86eb`, py_compilePASS. root가이SHA를읽고sourcefreeze를승인했다. root 실제timeout/normalrunner collection 종료까지추가source수정없음.

## Root live 실행 및 WSL 호환 대조

동결 source3839ea7e378e6de800ed9964e18fbf457c5cd45ef7bccb8a8ef6c31d107e86eb를 실제 프로젝트 Rust1.94 환경에서 `--execute --seconds1800`으로 실행했다. evidence workspace-owned-runner-native-first. root는 별도 wsl_interruption_snapshot --launch로 workspace-owned-runner-live-snapshot-first를 create-only 생성해 sameboot/archive_owner PID697530 start4774811 identity-matches-live를 실제 대조했다. 이는 당시 live 관찰이며 향후 연속성 보장이 아니다. 현재 terminal 결과는 같은evidence result.json의 실제완료를 별도확인해야 한다.

Root는 identity failure second의 gate_granted=false/Cargoexec=false/signals=[]/exit125/reaped/procabsent를 원시proof와 fmt-result로 대조했다. 이는 synthetic injection의 실패 경로 확인이며 정상Cargo 전체인수를 대체하지 않는다.

Root terminal 확인: 실제 session exit0, workspace-owned-runner-native-first/result.json state=completed/workspace_checks_complete=true. 고정9operation 각각 exit0/PASS(7workspace+2toolversion). Root가 source_final의 모든현재SHA 일치도 독립 확인했다. 실제 취소/timeout/outputoverflow 시험과 WSL 중단은 아직 미검증이다. 단위시험에서 ignored native 인수는 완료하지 않았다.

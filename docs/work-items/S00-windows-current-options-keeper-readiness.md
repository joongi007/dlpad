# S00: 현재 options·keeper 소스 Windows GNU 검증 준비

- 상태: ready (hidden CLI capability 확인; 실제 Cargo 검증 대기)
- 분류: native-readiness
- 우선순위: P0
- 요구사항: R01, R04, R08, R11, R13
- 담당/수정 범위: stress_recovery; 이 문서 및 신규 scripts/windows_current_workspace_probe.py, 전용 fresh evidence
- 선행 작업: root 현재 Linux 전체7checks/native 통과, 기존 project-local Windows toolchain 및 private CLI host

## 사용자 결과와 계약

현재 source snapshot을 Windows GNU CPU workspace fmt/clippy/test/release로 검증할 명령·환경·bounded collector를 준비한다. Cargo 실제 실행은 root 승인 전 수행하지 않는다. hidden CLI capability는 fixed rustc version만 허용한다. 기존 사용자 data/desktop/clipboard/서비스/보안/.git와 기존 evidence를 변경하지 않는다. Rust/Cargo/schema/기존host·script 수정 없음.

## 인수 조건

exact source/vendor/helper/tool SHA before/after, owned host PID/start 및 native child host manifest, operation exit과 bounded raw log/timeout, private-desktop=true 및 input-desktop-unchanged=true를 구분한다. source변경/출력상한/hostfailure는PASS아님. Windows GNU CLI 결과를 GUI/IME/MSVC/macOS 또는 Linux-only keeper runtime 실제 증거로 확대하지 않는다.

## 검증과 handoff

준비 중. 기존 windows_native_probe는 logs fixedname overwrite 및 unbounded readline 출력이라 그대로 새 final evidence collector로 사용하지 않는다. 기존 windows-cli-host의 suspended creation→ownJob assign→resume/privateDesktop/noWindow/ownedstdin cancel/kill-on-close 정책을 읽기 재사용한다. executable/source manifest와 actual hidden capability는 후속 기록한다.

## 기존 환경 읽기와 신규 collector

기존 scripts/windows_native_probe.py, windows_winit_revalidate.py, windows_cli_host.cs 및 windows_environment.ps1과 Windows cookie UI 작업/evidence를 읽었다. project-local rust/bin, cargo-home, llvm-mingw, target, tmp와 host.exe를 그대로 사용한다. host는 CreateDesktop random private/CREATE_NO_WINDOW+suspended/Job assign 후 resume, own stdin cancel 및 kill-on-close, input desktop 전후 비교를 제공한다. visible fallback이나 user desktop input을 추가하지 않았다.

신규 collector는 fixed Rust1.94 도구 및 host/source SHA, Cargo/toolchain/crates 전체Rust·manifest/vendor모든파일 SHA를 기록한다. 각 operation의 before/after 및 전체 baseline 일치가 필요하다. 로그는 fresh create-only8MiB, 64KiB chunk streaming, stdout별도 무제한 readline 없음. host timeout 기본1800초/최대7200초, outer deadline host+15초, cancellation은 pinned native Job을 가진 host에 stdin C를 전달하며 guessed WindowsPID kill은 없다. collector취소 뒤8초 안에 host reap를 확인하지 못하면 terminal 미확인으로 기록하고 PASS하지 않는다.

own interop host /proc PID/start를 기록하고 native childPID는 host의 owned Job 생성 원시로만 보고한다. native CreationTime을 직접 수집했다고 주장하지 않는다. output 상한/source변경/unknown child/desktop marker missing은 failure다. WindowsGNU tool hashes와 CPUrelease artifact SHA/bytes를 기록하고 GUI/IME/MSVC/macOS/Linux keeper actualruntime 미검증을 명시한다.

실제 준비 명령 `python3 -m py_compile scripts/windows_current_workspace_probe.py` exit0. collector SHA256 `3630d6a783566c520f8792a3fdb10ec4976a00bf695d1f37565bd5d4d6dec17c`.

## 실제 hidden capability

`python3 scripts/windows_current_workspace_probe.py /mnt/d/project/dlpad/docs/validation/evidence/windows-current-options-keeper-readiness-first --capability` 실제exit0. fixed rustc --version --verbose만 실행했으며 Cargo명령은0회다. rustc1.94.0/hostx86_64-pc-windows-gnu/LLVM21.1.8, 3.10초/로그277bytes. host원시 private-desktop=true와 input-desktop-unchanged=True 확인. interop hostPID667876/start4582766, nativechildPID30584는 소유 host의 Job에 묶여 생성됐다. source343 before/after/finalSHA일치. plan.json/result.json/capability.json/log는fresh evidence에 보존했다.

## root 후속 실제 명령

승인 후 `python3 scripts/windows_current_workspace_probe.py /mnt/d/project/dlpad/docs/validation/evidence/<fresh-prefix> --execute --seconds 1800`을 실행한다. fmt는 cargo-fmt.exe fmt --all -- --check; clippy는 cargo-clippy.exe clippy --workspace --all-targets --locked --features cpu -- -D warnings; test는 cargo.exe test --workspace --locked --features cpu; release는 cargo.exe build --workspace --release --locked --features cpu. hiddenhost가 각argv를 shell없이 실행한다. Cargo/Rustfmt workspace 수정 없이check만 수행하고 모든 source freeze가 필요하다. 정확 Windowsargv와 project-local 환경은plan.json에 있다. 현재상태는 준비/capability확인뿐이며 네 Cargo 결과를 완료로 기록하지 않는다.

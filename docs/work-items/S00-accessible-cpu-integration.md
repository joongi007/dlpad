# S00: 동일 다운로드 UI의 CPU renderer 통합

- 상태: in-progress
- 분류: experiment
- 우선순위: P0
- 요구사항: R08, R13, S00
- 담당/수정 범위: 주 담당 Cargo.toml/Cargo.lock/accessible.rs/accessible_cpu_main.rs/통합문서; library_identity software_host.rs 전용
- 선행 작업: accessible 다운로드/IME/무음reader 실제검증; Glow대기RSS예산3조건FAIL

## 사용자 결과

동일 egui 다운로드·중단·진단 UI와 AT-SPI/IME를 유지하며 GL/Mesa 없이 CPU pixel renderer의 실제 비용을 비교한다.

## 계약과 범위

프로덕션최종stack채택이아닌 optional CPU후보. 기존Iced/Glowbinary제거없음. root는 Candidate new/show_ui·font/configure_context를공유하고 process/guardian/instance entry를유지한다. rendereronly egui_software_backend 정확0.0.2 std만사용하며 해당crate편의winit host는AccessKit누락으로사용하지않는다. 직접egui_winit0.33.3/기존winit0.30.13 vendor/softbuffer0.4.8를연결해IME/AccessKit/Close취소및repaintdeadline을보존한다. 새renderercrate484125B와constify3843B/strength_reduce17859B, MIT ORApache2, runtimeGL/JIT기동제거여부실측. lock변경은root가existing packageversion보존을대조한다.

## 인수 조건

fmt/clippy/tests/release, 동일success/failure/pause/closeGUI, 한글IME/privateAT-SPI/실제무음Orca, 10초RSS/CPU측정. 메모리80MiB예산을완화하거나기능누락으로통과시키지않는다. 0크기resize/과대pixel/texturebudget/작업중Closefailure는safe검사. 실제3OS와전체UIparity·firstframe·목록10000·8시간은별도게이트다.

## 검증과 handoff

정확pin과공유UI추출후host소스통합·빌드직렬진행. source/build완료전GUI신호금지. 기존Glowsnapshot증거는보존하며새binary는새hash로검증한다.

## 공유 display helper와 실측·실제 GUI

주 담당은 scripts/native_probe.py의 owned Xvfb helper에 optional screen=(width,height)을추가했다. 기본1280×900은유지하며integer/4096×2160/총8Mi pixels상한을준수한다. py_compile 및기존6개GUI안전검사PASS. diagnostic_cache는별도first-paint/200% script만소유한다.

초기LinuxCPU SHA28b412...은10tests/clippy/release, actual GUI4case/한글/무음Orca/대기RSS50592KiB(49.4MiB)/idle0%10초표본PASS. 주 담당은SQLite4개/artifact/report/Paused·완료PNG를직접대조했다. Windowsdefault116/CPU131tests와nativeclippy/release도PASS이며주담당이235source입력과runner/binaryhash를독립대조했다. Linuxdefault191/CPUworkspace206tests와필수4개common검사PASS(visibility수정이전snapshot).

이후WindowsactualCPUGUI에서AccessKit-before-show panic이발견됐다. hostowner는window.with_visible(false)로생성하고surface/input/AccessKit를준비해View를보관한뒤set_visible(true)로수정했다. CLI/단위검사와actualGUI인수차이를증거로유지하고새binary에서Linux/WindowsactualGUI재검증전이므로현재작업in-progress를유지한다. renderer비용변경을추정으로통과시키지않는다.

## hidden-init 수정의 현재 통합 검증 (2026-10-03)

주 담당이 `cargo fmt --all -- --check`, `cargo clippy --workspace --features dlpad/cpu --all-targets --locked -- -D warnings`, `cargo test -p dlpad --bin dlpad-cpu --features cpu --locked`, `cargo build -p dlpad --bin dlpad-cpu --features cpu --release --locked`를 실행했다. 모두 exit0, CPU entry 10 passed/0 failed/0 ignored. release 35.01초, 새 Linux binary SHA256 `1d4757de07cb04d8cfcc08d1f039d7f3521b38de369fb1c4fc2b3cd8762b15b5`, 10172280B. Windows hidden-init native clippy/test/build의 현재 입력 235개 SHA를 주 담당이 독립 대조해 불일치 0개를 확인했다. [Windows CLI 증거](../validation/windows-cpu-hidden-init/result.json). 실제 GUI 인수는 별도 담당이 이어 검증한다.

새 Linux binary의 owned Xvfb/private D-Bus 성능 시험은 exit0, 10.099초 idle CPU 0%, RSS 최대50624KiB(49.4MiB), 제안80MiB 예산 이내다. [새 원시 증거](../validation/accessible-performance/cpu-hidden-init/result.json). 종료는 harness SIGTERM 정리이며 정상 UI 종료 증거와 구분한다. 실제 media·목록10000·한글/reader/200% 및 Windows GUI 완료를 이 비용 시험으로 주장하지 않는다.

같은 hidden-init 소스의 `cargo test --workspace --features dlpad/cpu --locked` 전체 통합 검사는206 passed/0 failed/5 ignored, exit0였다. 주 담당이 실제 stdout 요약을 집계하고 새 idle.png의 한국어 화면을 직접 확인했다. 문서 검사187개 통과. ignored 시험과 OS별 실제 GUI 게이트는 이 결과에 포함되지 않는다.

## 새 Linux GUI의 주 담당 독립 대조

주 담당은 hidden-init-native의 source-before/after에 sha256/bytes 필드가 동일한지 확인하고 실제4파일을64KiB단위로 다시 해시해 일치를 확인했다. 최초 객체전체 equality 비교는 after에 추가된 matches_before 필드 때문에 실패했으며 비교필드를 명시해 재검증했다. raw Orca log에서 object:text-changed:insert·object:state-changed:focused·locusOfFocus를 직접 확인하고 preedit.png에서 한[글] 조합 표시를 검토했다. raw IME collector의 pixel proxy false/exit1은 보존하며 review.json의 시각 판정과 구분한다. GUI helper6개 안전 회귀는 통과했다. [전용 검증](../validation/cpu-hidden-init-native-validation.md).

Windows actual Cancel 결과와 cancelled.png도 주 담당이 읽어 owned worker 시작시각/Job 소속·취소 후 정상앱상태·worker 정리·사용자 input desktop 불변 및 primaryexit0를 직접 확인했다. 최종 StopExit 및 안정성 수집이 끝난 뒤 해당 전용 문서를 연결한다. 이 후보의 background restore는 아직 구현되지 않았다.

Windows hidden-init 실제startup/render/합성worker/취소/StopExit는 [전용 결과](../validation/windows-cpu-native-validation.md)로완료됐다. 주 담당은inspect/cancel/stop3개원시JSON의primaryexit0·ownedWorker·inputdesktop불변을확인했고SQLite Cancelled/Attempt1/artifact0/integrityok를직접확인했다. SQLite초기검토의상대as_uri와잘못된state-column가정오류를고쳐실제schema job_json을파싱했다. 이후librarypanel/cancel소스변경은이Windows GUI snapshot범위밖이고새검증으로추적한다.

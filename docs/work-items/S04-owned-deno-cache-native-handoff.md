# S04: Windows 원 Deno cache 잠금 인계 구현과 실제 검증

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R05, R08, R11, R12
- 담당/수정 범위: root — shared CacheBinding schema/lib 선언 및 통합·전체 native 검증. metrics_recovery — cache 구현 /tmp staging만. diagnostic_cache — 별도 /tmp 실패회귀 staging만.
- 선행 작업: Windows 관리 Deno 분류·소유 cache 기반 통합, 기존 Prepared/Ack/Begin guardian gate

## 사용자 결과

관리 Deno만 원 앱 소유 cache 환경을 사용하며, 앱이 종료되어도 실행 중 원 cache를 다른 reader가 재사용하지 않는다. 외부 Deno cache에는 권한이나 삭제를 주장하지 않는다.

## 계약과 범위

[수명 제안 검토](S04-owned-deno-cache-helper-contract.md)의 별도 원 cache.lock capability + typed attachment + explicit unlock/helper-own-lock Ack 방식을 통합 방향으로 채택한다. DuplicateHandle만으로 잠금 이전을 주장하지 않는다. CacheBinding version1은 root UTF16 path와 원 root/runtime/cache/owner/lock Windows native IDs 및 원 bounded owner bytes SHA를 전달하며 distinct IDs/same-volume/no-zero/strict JSON/path를 검사한다. 이 형식 자체는 소유권 증명이 아니다. helper는 app이 확인한 expected IDs와 원 inherited lock을 대조해 no-create로 pins를 열고 자체 lock을 획득한다. unknown/partial 기록은 보존하며 경로에서 새 권한을 채택하지 않는다.

cache 구현 API는 binding(), lock_file(), release_for_guardian(), reacquire(), from_inherited(binding, lock)이다. 원 binding은 app lock 보유 중만 생성한다. from_inherited는 원 expected identities/owner SHA/native type·ACL·containment를 검증하되 아직 잠금을 획득했다고 표현하지 않는다. explicit reacquire 뒤에만 실행한다. 보유 여부를 상태로 구분하며 실패는 원 bytes를 유지한다. 실제 gate/launcher/config/download 연결은 root 통합 범위이며 source freeze 중 프로젝트 Rust를 agent가 수정하지 않는다. 기존 dependencies/schema DB/Cargo.lock은 변경하지 않는다.

## 인수 조건

실제 Windows에서 원 app lock Busy → explicit release → helper own lock → 원 app HANDLE close 뒤 helper-live Busy → helper terminal/drop 뒤 재획득을 관측한다. binding 변조/root·owner·lock ID mismatch/owner digest/wrong path·ACL·hardlink/unknown은 원 bytes 보존과 함께 거절한다. getter나 wire pure roundtrip만으로 actual lifetime을 증명하지 않는다. 모든 config/capability/gate 완료 전 기존 production Some 사전 거절을 유지한다. 실제 Deno 생성 payload ACL와 사후32MiB 예산 한계는 별도 인수다.

## 검증과 handoff

구현 및 실제 검증은 진행 중이다. 이전 cache 기반9회귀 성공은 새 handoff 수명이나 생산 다운로드 성공 근거가 아니다. 구체 실행 명령·실패/성공 결과·SHA·남은 gate는 root 통합 후 갱신한다.

## 주 담당 통합·실제 Windows 검증

root가 원 staging SHA/manifest를 대조한 뒤 원 cache wrapper·typed attachment·wire·launcher·3 entry/guardian gate를 통합했다. optional cache capability는 기존 cookie/managed5개 뒤에 추가하여6개, paired6개 뒤에 추가하여7개다. root는 공유 spawn의 stdout/stderr index를 원 base layout으로 고정했다. entry는 원 IDs·owner SHA/ACL을 no-create로 대조하고, 원 tool은 suspended Prepared→app explicit unlock/Ack→helper own lock+원 journal 획득→helper Ack→Begin/원 cache 재검증→Resume 순서를 따른다. terminal/receipt 수명까지 cache pins를 유지하고 사후 inventory 실패/예산 초과는 helper exit126으로 전달한다. 새로운 현재 경로를 expected로 채택하거나 cache를 자동 삭제하지 않는다.

[최초 native 컴파일](../validation/evidence/windows-deno-cache-native-handoff-first/result.json)은 ignored reparse fixture의 CreateSymbolicLinkW bool 반환형 불일치 때문에 cache 테스트 compile exit101이다. root가 현재 binding에 맞춰 수정했고 원 실패를 보존했다. [수정 후 실제 native](../validation/evidence/windows-deno-cache-native-handoff-corrected/result.json)에서 guard build와 cache11PASS/1ignored, 실제 독립 cookie helper Some1PASS를 확인했다. AppHeld Busy→explicit unlock→HelperAck 뒤 원 app HANDLE close→helper-live Busy→actual Terminal/원 receipt/자연 helper exit→재획득과 owner bytes 보존, 실제 child의 두 환경값을 확인했다. 실제 부모 프로세스 kill은 이 시험에 포함되지 않는다.

[전체 Windows workspace](../validation/evidence/windows-deno-cache-handoff-workspace-first/result.json)의 fmt/Clippy/test/release 모두 exit0이며 engine library304PASS/0failed/3ignored다. [root 독립 대조](../validation/evidence/windows-deno-cache-handoff-workspace-first/root-review.json)는 현재 source SHA/전후snapshot/CPU release artifact 일치를 확인했다. 실제 native 명령은 `cargo build -p dlpad-engine --bin dlpad-process-guard --locked`, `cargo test -p dlpad-engine --lib --locked windows_owned_deno_cache -- --nocapture`, `cargo test -p dlpad-engine --lib --locked actual_cache_handoff_helper_lock_survives_original_app_handles_close -- --nocapture`; 전체명령은 `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked --features cpu -- -D warnings`, `cargo test --workspace --locked --features cpu`, `cargo build --workspace --release --locked --features cpu`다.

[별도 실제 reparse 시험](../validation/evidence/windows-deno-cache-reparse-capability-first/result.json)은 `--ignored` 명시 실행 exit101이다. 이 환경에서 synthetic CreateSymbolicLinkW 생성이 실패해 capability 부족으로 기록하며 PASS로 바꾸지 않는다. 관리자/DeveloperMode/보안 설정 변경은 없고 junction 기반 인수 대안을 별도 검토한다. 일반 suite의 ignored나 type guard만으로 actual reparse 인수를 완료하지 않는다.

남은 게이트는 정확 관리 Deno만 사용하는 생산 factory/capture/download 연결, 실제 managed·paired Some와 원 app processkill/경쟁gap/실패/cancel 및 사후 비용 UI, Deno 실제 생성 payload ACL/EJS/공급notice/source다. 현재 생산 Some 사전 거절을 유지하며 전체 P0 완료를 주장하지 않는다. Linux/macOS 신규 인수는 후속이다.

## 실제 junction 인수 후속

[실제 native junction 시험](../validation/evidence/windows-deno-cache-junction-native-first/result.json)은 explicit `cargo test -p dlpad-engine --lib --locked inventory_native_owned_junction_is_rejected_and_target_preserved -- --ignored --nocapture` 실행 exit0이다. 원 owned empty-directory HANDLE에 native mount-point tag/bytes를 설정하고 GET readback으로 대조했다. cache inventory는 Changed로 거절했고 원 owner bytes/외부 synthetic target bytes 및 두 원 IDs를 유지했다. 원 junction만 no-follow HANDLE/ID 대조 뒤 제거했고 target을 보존했다. 관리자권한·DeveloperMode·보안설정·의존성/feature 추가는 없다. 이 실제 성공은 junction mount-point 범위이며 CreateSymbolicLinkW의 앞선 capability 실패를 PASS로 바꾸지 않는다. normal suite에서는 environment-specific capability 시험을 ignored로 두고 실제 명시 실행 결과를 별도로 유지한다.

## junction 추가 뒤 최종 통합 snapshot

[최신 Windows 전체 네 명령](../validation/evidence/windows-deno-cache-handoff-junction-workspace/result.json)은 모두 exit0이며 engine304PASS/0failed/4ignored다. [root 독립 검토](../validation/evidence/windows-deno-cache-handoff-junction-workspace/root-review.json)는 최신 전체 source SHA/CPU release artifact와 현재 workspace의 실제 helper HANDLE-close 시험 및 명시 junction 시험 성공을 대조했다. 네 ignored 항목은 성공으로 세지 않으며 junction의 별도 명시 실행1PASS를 구분한다. 생산 factory/download 연결·실제 appkill·managed/pairedSome 인수는 계속 남는다.

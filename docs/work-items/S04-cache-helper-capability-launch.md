# S04: 관리 Deno cache 원 handle helper 전달

- 상태: implementation-ready (임시 사본; native 인수 대기)
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: soak_recovery_audit; 본 문서와 `/tmp/dlpad-cache-helper-launch/`의 launcher 및 3 entry 수정 사본만
- 선행 작업: typed cache binding/OwnedDenoCache inherited validation 및 guardian Request cache 계약(root)

## 사용자 결과

선택된 관리 runtime cache의 원 handle만 helper에 전달하며 binding/handle 불일치는 도구 실행 전에 거절한다.

## 계약과 범위

기존 launch API는 None wrapper를 유지한다. optional cache disk handle은 기존 원 capability 배열 끝에만 추가하며 exact inheritance/type/dedup 검증을 유지한다. entry는 5/6 또는 paired 6/7 handle만 허용하고 typed binding과 actual cache cap를 함께 검증한다. cache 생성·락 재획득·guardian 실행은 root 담당이다. 프로젝트 Rust/Cargo/native 실행 및 shared API 수정은 제외한다.

## 인수 조건

None/None 기존 동작, Some/Some inherited actual validation, 모든 불일치 실패. stdout/control/events/receipt 원 cap 계약 유지. 4 원 source SHA 및 수정 SHA 기록; actual native 전달 검증 전 완료 주장 금지.

## 검증과 handoff

임시 사본만 rustfmt하고 manifest와 API를 root에 전달한다. 실제 compile/native 결과는 통합 담당이 기록한다.

`/tmp/dlpad-cache-helper-launch/`에 4 수정 사본과 `manifest.json`, 4 diff를 준비했다. 원본 4 SHA는 모두 변하지 않았다. `rustfmt --edition 2024 /tmp/dlpad-cache-helper-launch/*.rs` exit0; Cargo/native/제품 게시 없음. 기존 wrapper는 None을 전달하고 새 with-cache wrapper는 binding/handle presence 불일치를 pipes 생성 전 거절한다. entry는 optional 마지막 disk cap를 먼저 분리한 뒤 기존 stdio/receipt pop 순서를 유지하고 `from_inherited(binding, File)` 검증 후 Request에 owned cache를 전달한다.

통합 담당에게 발견한 공유 선행 조건을 전달했다: `windows.rs::spawn_independent_helper`의 기존 exact5/6 count를 optional6/7로 확장하고, stdio를 배열 마지막 두 항목이 아닌 원 cookie/managed index3·4 및 paired index4·5로 설정해야 한다. cache가 마지막일 때 기존 stdio 선택은 잘못된 handle을 가리킨다. 해당 공유 파일은 수정하지 않았다. guardian Request/cache Ackgate 및 typed binding 구현 연결, 실제 NONE/Some 전달·실패/native 회귀는 root 담당으로 남는다.

## 주 담당 통합·실제 Windows 검증

root가 원 staging SHA/manifest를 대조한 뒤 원 cache wrapper·typed attachment·wire·launcher·3 entry/guardian gate를 통합했다. optional cache capability는 기존 cookie/managed5개 뒤에 추가하여6개, paired6개 뒤에 추가하여7개다. root는 공유 spawn의 stdout/stderr index를 원 base layout으로 고정했다. entry는 원 IDs·owner SHA/ACL을 no-create로 대조하고, 원 tool은 suspended Prepared→app explicit unlock/Ack→helper own lock+원 journal 획득→helper Ack→Begin/원 cache 재검증→Resume 순서를 따른다. terminal/receipt 수명까지 cache pins를 유지하고 사후 inventory 실패/예산 초과는 helper exit126으로 전달한다. 새로운 현재 경로를 expected로 채택하거나 cache를 자동 삭제하지 않는다.

[최초 native 컴파일](../validation/evidence/windows-deno-cache-native-handoff-first/result.json)은 ignored reparse fixture의 CreateSymbolicLinkW bool 반환형 불일치 때문에 cache 테스트 compile exit101이다. root가 현재 binding에 맞춰 수정했고 원 실패를 보존했다. [수정 후 실제 native](../validation/evidence/windows-deno-cache-native-handoff-corrected/result.json)에서 guard build와 cache11PASS/1ignored, 실제 독립 cookie helper Some1PASS를 확인했다. AppHeld Busy→explicit unlock→HelperAck 뒤 원 app HANDLE close→helper-live Busy→actual Terminal/원 receipt/자연 helper exit→재획득과 owner bytes 보존, 실제 child의 두 환경값을 확인했다. 실제 부모 프로세스 kill은 이 시험에 포함되지 않는다.

[전체 Windows workspace](../validation/evidence/windows-deno-cache-handoff-workspace-first/result.json)의 fmt/Clippy/test/release 모두 exit0이며 engine library304PASS/0failed/3ignored다. [root 독립 대조](../validation/evidence/windows-deno-cache-handoff-workspace-first/root-review.json)는 현재 source SHA/전후snapshot/CPU release artifact 일치를 확인했다. 실제 native 명령은 `cargo build -p dlpad-engine --bin dlpad-process-guard --locked`, `cargo test -p dlpad-engine --lib --locked windows_owned_deno_cache -- --nocapture`, `cargo test -p dlpad-engine --lib --locked actual_cache_handoff_helper_lock_survives_original_app_handles_close -- --nocapture`; 전체명령은 `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked --features cpu -- -D warnings`, `cargo test --workspace --locked --features cpu`, `cargo build --workspace --release --locked --features cpu`다.

[별도 실제 reparse 시험](../validation/evidence/windows-deno-cache-reparse-capability-first/result.json)은 `--ignored` 명시 실행 exit101이다. 이 환경에서 synthetic CreateSymbolicLinkW 생성이 실패해 capability 부족으로 기록하며 PASS로 바꾸지 않는다. 관리자/DeveloperMode/보안 설정 변경은 없고 junction 기반 인수 대안을 별도 검토한다. 일반 suite의 ignored나 type guard만으로 actual reparse 인수를 완료하지 않는다.

남은 게이트는 정확 관리 Deno만 사용하는 생산 factory/capture/download 연결, 실제 managed·paired Some와 원 app processkill/경쟁gap/실패/cancel 및 사후 비용 UI, Deno 실제 생성 payload ACL/EJS/공급notice/source다. 현재 생산 Some 사전 거절을 유지하며 전체 P0 완료를 주장하지 않는다. Linux/macOS 신규 인수는 후속이다.

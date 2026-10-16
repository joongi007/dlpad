# S04: 소유 Deno cache native 실패 회귀 staging

- 상태: in-progress
- 분류: safety
- 우선순위: P0
- 요구사항: R05, R08, R13
- 담당/수정 범위: diagnostic_cache; /tmp/dlpad-owned-deno-cache-failure-tests.rs 및 본 문서만
- 선행 작업: 실제 Windows cache 기반4tests/분류5tests 및 전체fmt/clippy/test/release PASS

## 사용자 결과

원 cache/owner 권한 확대와 변경·bounded inventory에서 unknown/외부 자료를 보존하는 native 실패 인수를 추가 준비한다.

## 계약과 범위

기존 module tests의 root()/native helpers를 재사용하여 붙일 staging Rust다. 현재 SID+Everyone read ACE의 보호 DACL로 이번 원 소유 임시 cache/owner만 변경한다. 관리자권한/기존 사용자 설정/외부 cache 변경 없음. 원 retained handle의 no-delete sharing이 rename을 거절하는지 native ID 및 원 bytes로 검사한다. ownerJSON oversize/unknownfield, 정확4097 entry의 bounded 실패를 준비한다. 프로젝트 Rust/Cargo/공유 파일 게시 및 native 실행 없음.

## 인수 조건

ACL 변경은 actual SetSecurityInfo success 뒤 verify/reopen이 거절되고 원 bytes/sentinel 보존. held owner/cache/runtime rename은 실제 실패하고 original IDs/bytes 유지. owner4097B 또는 unknown은 자동 수정/삭제 없이 보존. inventory4097 entry는 Bounds와 원 파일 count/bytes 불변이다.

reparse 시험은 default ignored로 명시한다. 기존 private host에서 비관리자 CreateSymbolicLinkW(directory|allow-unprivileged) capability가 가능한 경우만 별도 --ignored 실제 인수를 수행한다. 생성 실패는 실제 실패이며 PASS/성공 skip으로 처리하지 않는다. developer mode/보안설정/admin을 자동 변경하지 않는다. capability 없으면 reparse 인수 미검증으로 남긴다.

## 검증과 handoff

/tmp staging 개별 rustfmt exit0. 실제 Windows 컴파일/실행은 root가 source freeze 후 수행하며 ignored capability 인수는 별도다. 기본4개 regression과 explicit1개 reparse acceptance를 제공한다. staging SHA는 READY 메시지에 기록한다.

## Junction 원 HANDLE 후속 인수 초안

원 explicit CreateSymbolicLinkW 시험은 capability unavailable로 exit101이며 `docs/evidence/windows-deno-cache-reparse-capability-first/actual-reparse.log`를 보존한다. 후속 담당 파일은 `/tmp/dlpad-cache-junction-regression.rs`와 본 문서만이다. 프로젝트 Rust/Cargo 게시와 native 실행은 root 담당이다.

동일 owned TempDir 안의 cache 밖 합성 target/sentinel과 새 빈 directory를 Native CreateDirectoryW로 생성한다. 원 no-delete-share directory HANDLE/native ID/containment를 검사하고 그 원 HANDLE에만 FSCTL_SET_REPARSE_POINT MOUNT_POINT를 적용한다. bounded 16KiB Microsoft MountPointReparseBuffer의 실제 GET readback/tag/full bytes와 inventory Changed, 원 owner/target bytes 보존을 인수한다. 정리는 no-follow DELETE HANDLE을 재open하고 원 ID와 대조한 뒤 FileDispositionInfo로 원 junction만 삭제한다. 교체 ID는 삭제하지 않으며 target 재귀 삭제가 없다. DELETE handle을 inventory 전에 보유하면 sharing violation으로 검사가 거짓 성공할 수 있으므로 DELETE reopen은 inventory 관측 뒤에만 한다.

공식 [FSCTL_SET_REPARSE_POINT](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ni-winioctl-fsctl_set_reparse_point) 문서는 SeCreateSymbolicLinkPrivilege 필요를 명시한다. 따라서 junction이 현재 host에서 비관리자 생성 가능하다고 보장하지 않는다. 권한을 추가/활성화하거나 DeveloperMode를 변경하지 않으며 native 실패는 고유 OS 코드와 함께 실패로 남긴다. [REPARSE_DATA_BUFFER](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/ns-ntifs-_reparse_data_buffer)의 mount-point offset/length/UTF16 형식과 [junction 공식 설명](https://learn.microsoft.com/en-us/windows/win32/fileio/hard-links-and-junctions)을 기준으로 local target만 허용한다. 기존 Win32_System_IO feature의 DeviceIoControl과 documented control-code constants만 사용하고 dependency 추가가 없다.

초안 개별 rustfmt exit0. SHA256 `17bcf959bae9cd81d0c29ca1b9697131232b0ffe3fa686485854508e1da6709e`. 컴파일·native 생성·reparse 거절의 실제 성공은 아직 미검증이다. 이 초안은 기존 ignored 심볼릭 링크 실패를 대체하거나 숨기지 않는다.

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

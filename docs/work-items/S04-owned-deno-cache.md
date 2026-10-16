# S04: Windows 원 앱 소유 Deno cache 기반

- 상태: in-progress (Windows cache 기반 통합·native 검증 완료; guardian 수명 연결 대기)
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R05, R08, R12
- 담당: metrics_recovery
- 담당 파일: `/tmp/dlpad-owned-deno-cache.rs`, 이 문서
- 선행 조건: 관리 Deno 분류·typed runtime environment 연결은 주 담당 별도 작업

## 범위와 공유 계약

현재 workspace Rust/Cargo/shared 선언을 수정하지 않고 Windows 전용 기반 모듈을 `/tmp`에 작성한다. `OwnedDenoCache::create_or_reopen(root_path: &Path, original_root: &File) -> Result<Self, CacheError>`, `verify()`, `path()`, `inventory() -> Result<CacheInventory, CacheError>`를 제공한다. 경로 getter 자체는 소유권 증명이 아니다. 실제 root 원 HANDLE identity, 새 `.runtime-cache/deno` 디렉터리 원 HANDLE, SID/보호 DACL, 원 owner record와 lock 파일을 유지하는 wrapper가 권한의 기반이다.

기존 ToolManager root의 ACL은 변경하지 않는다. 원 root identity와 retained handle containment만 검증한다. 새 runtime/cache/owner/lock에는 현재 SID 전용 보호 DACL을 생성한다. 기존 runtime 디렉터리를 발견하면 정확한 기존 owner record가 없거나 잘못됐을 때 실패하여 보존한다. 기존 권한을 수정하거나 unknown 디렉터리를 빈 새 소유 cache로 취급하지 않는다.

owner.json은 version 1, 원 root/runtime/cache/lock native identities를 포함한 deny-unknown JSON이며 4096 bytes 상한이다. 새 파일은 CREATE_NEW로 생성하고 record를 sync_all 한다. 전용 cache.lock에 원 lifetime nonblocking native file lock을 유지하여 immutable owner record 읽기와 byte-range lock 충돌을 분리한다. 실제 helper의 cache authority 전달/wire와 원 app crash 이후 lock 회수는 주 담당 후속 작업이다.

## 인수 조건

- 새 private cache 생성과 exact record reopen, 원 root/native path 교체·reparse·hardlink·불완전 record·잘못된 ACL·Busy 거절 및 bytes 보존.
- 원 pin을 유지하는 동안 원 경로/native ID/ACL/record를 반복 검증. 자동 삭제, 재귀 삭제, ACL repair, 외부 Deno cache 변경은 없음.
- postflight inventory는 전체 entry 4096개/논리 크기 32 MiB 상한을 검사하며 원 파일 hardlink/reparse를 거절한다. stream 내용을 메모리로 읽지 않는다.
- cache budget은 실행 hard quota가 아니다. 사후 검사가 실행 중 증가를 제한한다고 표현하지 않는다. helper 전체 authority와 실제 EJS 호환성은 이 모듈만으로 완료라고 표현하지 않는다.
- Windows native 회귀 컴파일·실행은 주 담당이 source integration 뒤 수행한다. staging rustfmt 성공을 native 테스트 성공으로 쓰지 않는다.

## 구현과 검증 범위

staging 모듈은 Windows CreateDirectoryW/CreateFileW에서 현재 SID private descriptor를 사용하고, final reparse/nonregular/multiple-links를 거절한다. root/runtime/cache를 delete 공유 없이 retained pin으로 유지하며 각 IO 경로의 원 ID와 strict descendant handle path를 검증한다. record native IDs/원 bytes 및 ACL을 재검증한다. 별도 lock 파일은 helper의 owner record 읽기를 가로막는 원 record byte-range 잠금과 구분된다.

inventory는 디렉터리/파일 하나씩 native metadata를 검사하고 pending 디렉터리만 bounded entry 범위로 유지한다. logical bytes에는 owner/lock 파일도 포함된다. entry 개수/산술 상한 초과 시 fixed Bounds 오류를 반환한다. 논리 크기는 실제 합계와 `budget_exceeded: bool`을 함께 반환하여 32 MiB 초과를 숨기지 않는다. 실제 비용 보고 이후 실행 허용·차단 UI 정책은 주 담당 후속 연결이다. 악성 작업이 조사 중 계속 cache를 증가시키는 경우 상한 안의 일관된 전역 snapshot을 보장하지 않는다.

현재 staging 4개 unit fixture는 생성/reopen/Busy, unknown 디렉터리 보존, partial owner record 보존, hardlink 거절, sparse 논리 크기 초과, record version mismatch, foreign 원 root pin 거절, 원 owner hardlink 및 외부 sentinel bytes 보존을 포함한다. 실제 명령은 `.local-tools/cargo/bin/rustfmt --edition 2024 /tmp/dlpad-owned-deno-cache.rs`이며 exit 0이다. Cargo/native 회귀는 아직 실행하지 않았다. owner File sync 성공과 디렉터리 creation의 전원 차단 내구성을 동일한 보장으로 주장하지 않는다. 원 root/cache capability helper 전파는 미연결이다.

최종 staging API `CacheInventory { entries: usize, logical_bytes: u64, budget_exceeded: bool }`는 내용·경로를 출력하지 않는다. owner record 읽기에 방해되지 않도록 잠금은 cache.lock에만 유지한다. `verify()`와 inventory 반환 직전 원 ID/record/ACL을 반복 확인한다. 삭제 기능은 없다. 최종 source SHA는 주 담당 READY 인계 메시지에 기록한다.

## 주 담당 통합 착수

root가 인계 SHA를 대조하여 crates/engine/src/windows_owned_deno_cache.rs, crates/engine/src/lib.rs에 게시했다. 공유 선언은 root만 수정했다. 생산 환경 활성화 전 기반 native 회귀를 실행하며 임시 dead-code 허용은 아직 미연결인 production factory/handoff에 한정한다. Cargo 의존성·lockfile·DB schema 변경은 없다. Windows 전체 검증 결과는 후속 기록하며 현재 컴파일/실행 인수 완료를 주장하지 않는다.

## 주 담당 Windows 실제 검증

[최초 컴파일 증거](../validation/evidence/windows-owned-deno-cache-integration-first/result.json)는 exit101이다. READ_CONTROL을 Security에서 import한 오류를 실제 native 컴파일로 확인하고 Storage::FileSystem import로 수정했다. 이 실패는 삭제하지 않았다. [수정 후 전체 시험 로그](../validation/evidence/windows-deno-owned-classification-cache-corrected/test.log)에서 소유 cache 회귀 4개가 실제 Windows에서 통과했다. 생성/reopen/Busy, 원 root 불일치, 원 owner hardlink, unknown/partial record 보존, 논리 예산 초과 보고와 version mismatch 보존을 확인했다.

독립 읽기 검토에서 원 lock은 guardian terminal까지 유지해야 하며 path getter만 넘긴 뒤 원 객체를 drop하는 방식은 허용할 수 없음을 확인했다. 현재 SID 전용 ACL 보장은 원 runtime/cache/owner/lock 객체에 한정한다. Deno가 실제 생성한 payload 파일의 ACL 및 ACL 확대/reparse/namespace 교체 실패는 별도 native 인수다. 현재 기반 모듈만으로 전체 helper/생산 다운로드 연결 또는 모든 cache payload private 보장을 주장하지 않는다. 실행 중 hard quota도 제공하지 않는다.

## 전체 통합 결과

[수정 후 Windows 전체 검증](../validation/evidence/windows-deno-owned-classification-cache-corrected/result.json)에서 `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked --features cpu -- -D warnings`, `cargo test --workspace --locked --features cpu`, `cargo build --workspace --release --locked --features cpu` 모두 실제 exit0이다. [주 담당 독립 대조](../validation/evidence/windows-deno-owned-classification-cache-corrected/root-review.json)에서 현재 전체 입력 SHA와 전후 snapshot 및 현재 CPU release artifact 일치를 확인했다. 이 근거는 Windows GNU이며 GUI·MSVC·다른 OS·생산 helper/cache 적용 완료 근거로 확대하지 않는다.

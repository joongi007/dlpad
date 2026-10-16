# Owned disk writer failure and resume validation

2026-10-02, Linux only. New `crates/engine/tests/disk_failure.rs` exercises the existing DownloadRepository, guardian-backed external tool, partial directory and SQLite journal without modifying product code or Cargo.

A temporary synthetic Python executable writes exactly 64 KiB of owned partial bytes. `resource.setrlimit(RLIMIT_FSIZE, (4096,4096))` is applied only inside that worker, with SIGXFSZ ignored so the failing append returns actual kernel **EFBIG (27)**. This is a file-size limit failure, **not ENOSPC or actual full storage**. The second test explicitly injects `OSError(ENOSPC)` at the synthetic writer boundary; it verifies application behavior given that writer failure and does not claim kernel/filesystem exhaustion.

Both tests verify download-stage Execution error with bounded errno evidence, persistent Failed state, no registered/final artifact, unchanged partial bytes, database close/reopen, explicit resume with original Job ID and new Attempt, original source URL re-extraction, successful final artifact, preserved unrelated owned fixture, final database reopen and idempotent recovery.

Actual commands (project-local Rust environment):

```text
rustfmt --edition 2024 crates/engine/tests/disk_failure.rs
cargo test -p dlpad-engine --test disk_failure --locked
cargo clippy -p dlpad-engine --test disk_failure --locked -- -D warnings
```

Results: test compilation 9.82 seconds; **2 passed, 0 failed**, runtime 1.59 seconds. Clippy exit 0, no warnings, 10.28 seconds. Windows source snapshot coordinator explicitly released freeze before the new Rust file was copied into the project. No global formatting, product source, shared Cargo/schema or existing test changes.

Actual ENOSPC on an owned regular filesystem file, SQLite/publication ENOSPC, real yt-dlp partial-format recovery and Windows/macOS storage failure remain unverified. No mount, shared disk filling, `/dev/full`, user Documents or actual media was used. No automatic retry loop was introduced. R04 disk-exhaustion acceptance remains open until safe platform-specific actual storage exhaustion can be demonstrated.

Existing eight-hour tests remained running without restart/termination: last read observed engine/DB 3187.30 seconds, guardian 3185.10 seconds, all owned PID/start ticks matching. Archive remained watching at 2026-10-02T13:54:06.755411 UTC with owner PID6202/start48872 matching. These intermediate observations are not eight-hour completion evidence.

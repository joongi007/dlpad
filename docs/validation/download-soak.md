# Synthetic DownloadRepository longevity and recovery

## Scope

Linux engine+SQLite+owned process adapter only. Stdlib Python temporary executable emits synthetic metadata/progress/artifact markers and bounded synthetic bytes. No real media decode, actual network transport/site behavior, FFmpeg, GUI, sleep/logout, or Windows/macOS is verified. A synthetic `network disconnected`/HTTP403 error verifies repository failure handling and original-source re-extraction invocation; it is not an actual socket disconnect or real expired CDN URL experiment.

## Short actual verification

`cargo test -p dlpad-engine --test download_soak --locked -- --nocapture`: 1 passed, 1 explicitly ignored, 7.88s (final source). `cargo clippy -p dlpad-engine --test download_soak --locked -- -D warnings` passed again on the final generation/sample-bound source (8.96s); full workspace integration remains the owner's check. Only the new source was formatted with `rustfmt --edition 2024 crates/engine/tests/download_soak.rs`. Python wrapper syntax compile with `/tmp` pycache passed. No product source, dependency, Cargo or schema changes.

One vertical scenario exercises real create/prepare/execute, live pause request after observing the partial file, Cancelled process acknowledgement mapped to Paused, SQLite close/reopen, same job ID resume with distinct new Attempt, final create-only artifact publication, persisted completion and idempotent recover. ProgressFeed receives real machine lines. 4000×~117-byte synthetic stderr exceeds the 256KiB engine tail limit: the test asserts truncation and capped retained log. The partial is 65,536 bytes and remains unchanged by pause/resume publication.

Separate disconnected and expired fixture modes exit 7 during the actual download stage. Each asserts Failed, no artifacts and preserved partial. Reopen/resume re-invokes extraction using the original source URL, not the prior synthetic expiring media URL, records the next generation, increments distinct Attempt ID, verifies exact new-generation artifact contents and persists Downloaded through reopen. All six extraction calls assert original source and generation sequence 1..6. There is no automatic retry claim.

## Explicit eight-hour run

The ignored `synthetic_eight_hour_download_repository` requires `DLPAD_DOWNLOAD_SOAK_SECONDS` at least 28800 and a nonexistent absolute direct-child `/tmp` `DLPAD_DOWNLOAD_SOAK_OUTPUT`. Short scenario runs first, then one real repository execution remains Downloading for 28800 monotonic seconds and publishes its synthetic artifact. The test asserts duration, progress, bounded log, partial size, persisted completed artifact and zero recovery changes. Final success evidence is written only after these assertions.

Actual launch after the short test passed:

```sh
python3 scripts/download_soak.py target/debug/deps/download_soak-eb1ebcee84bd48b1 /tmp/dlpad-eight-hour-download-20261001 --seconds 28800
```

Owned execution session **76382**. State remains **in-progress** until result.json records completed after actual elapsed time. Runner proof is `/tmp/dlpad-eight-hour-download-20261001-runner/status.json`; detailed worker proof is `/tmp/dlpad-eight-hour-download-20261001/long-job/{worker-status.json,samples.jsonl}`. Output and runner directories must not exist at launch, so prior results/data are never overwritten. Test binary SHA256 and actual owner PID/start ticks are preserved in runner status. Subsequent builds do not replace the running executable inode. This uses a debug integration-test binary and development guardian helper, not the release GUI binary; costs cannot be compared directly with product release RSS.

Every 60 seconds the worker samples named engine owner/guardian/worker PID start ticks, CPU ticks and RSS. It overwrites one 64KiB partial once per sample and emits one coalesced progress observation per second. At eight hours about 481 samples consume less than 1MiB (hard sample-log cap), about 31MiB total overwrite I/O and 64KiB partial footprint. Engine stdout/stderr retention remains 256KiB. JSON status files replace atomically. A runner stop.request terminates only its owned test child; the guardian handles owner socket EOF. A stopped or failed run cannot produce a completed runner result.

This run is additional to headless release guardian longevity session57344. It does not replace that experiment. The original session at elapsed6250s/105samples had all six recorded PID start ticks match authoritative host /proc; its owner PID29575/guardian29577 stayed alive. Sandbox /proc uses a different namespace, so read-only authoritative ownership checks require the authorized host command rather than interpreting a sandbox ENOENT as process death.

## Actual intermediate observation

Runner elapsed136.09s was running; owner103171/start869854, guardian103274/start870029 and worker103276/start870029 all matched authoritative host /proc. The worker had emitted elapsed60.23s sample with owner RSS6,258,688 bytes, guardian2,252,800 bytes, worker10,891,264 bytes and partial65,536 bytes. These are intermediate development-test costs, not final eight-hour success. Existing guardian-only run remained running at6370.64s. Reusing the existing output was rejected with exit2 before spawning a process. Documentation check passed (102 documents).

## 2026-10-02 재개 시 증거 소실

현재 boot ID794988eb-539b-437e-8396-330738b565bb에서 이전 /tmpstatus와 소유 PID가 존재하지 않는다. 위 running 관측은 당시 중간 기록이며 현재 실행 상태가 아니다. session57344/76382의 8시간 완료 결과는 미검증이다. 기존 시험을 성공으로 처리하지 않는다. 신규 실행과 영구 증거는 S04-soak-restart 작업으로 구분한다.

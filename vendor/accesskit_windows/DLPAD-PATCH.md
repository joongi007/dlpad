# dlpad Windows validation property patch

Upstream: accesskit_windows 0.29.2, AccessKit commit `5801c4a49ff9b2c9ec3f77b3c31272d0da4c5cd3`, `platforms/windows`. Registry archive SHA-256: `d2d63dd5041e49c363d83f5419a896ecb074d309c414036f616dc0b04faca971`.

The original per-file inventory is in DLPAD-ORIGINAL-SOURCE.json. DLPAD-SUPPLY.json records the archive and license origins. The archive's Cargo.lock is retained for standalone tests; the unpacking-only .cargo-ok marker is omitted. Original MIT, Apache and Chromium BSD notices are included without rewriting. Full distribution legal coverage remains a separate gate.

Only src/node.rs changes upstream behavior: UIA IsDataValidForForm reads `Node.data().invalid().is_none()`. The existing properties macro provides both native dispatch and property-change event comparison. None is valid; True, Grammar and Spelling are invalid. FullDescription is unchanged. A consumer-tree regression checks all four states and nonempty property dispatch.

The root workspace excludes this vendor crate, as with the existing forks, and patches the same version through Cargo.toml. Root Cargo.lock removes only this package's registry source/checksum; dependency versions and lists stay identical. Standalone tests use the upstream internal lock and separate test dependencies. The patch adds a boolean getter and property comparison, with no new runtime dependency, thread or I/O. Actual binary size and Windows COM acceptance are recorded in the linked work item.

See [validation work item](../../docs/work-items/S00-windows-input-validation-provider-20261012.md).

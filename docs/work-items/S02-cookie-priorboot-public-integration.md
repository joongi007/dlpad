# S02: 공개 recover_sessions의 이전 부팅 권한 통합 시험

- 상태: completed (합성 이전 부팅의 공개 API 인수)
- 분류: test
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: metrics_recovery — 신규 crates/engine/tests/cookie_priorboot_public.rs 및 이 문서
- 선행 작업: PreviousBootAuthority 생산 kernel boot 경로와 root recovery 연결

## 계약과 인수

공개 import_session으로 실제 private 원본 anchor/marker를 만들고 lease를 defer/drop한다. compact recovery record의 boot 문자열만 동일 길이의 다른 유효 UUID로 바꾸고 그 정확한 bytes SHA256을 marker digest에 갱신한다. serde Value 재직렬화로 field order를 변경하지 않는다. 이는 합성 과거 부팅 fixture이며 실제 재부팅 증거가 아니다.

원 recovery record에 기록된 anchor를 사용해 test-local journal을 reopen하고 metadata0 또는 download1의 Intent/Bound를 만든다. download1은 metadata0 Resolved 실제 synthetic receipt를 먼저 요구한다. 모든 FD를 닫고 공개 recover_sessions가 실제 현재 kernel boot를 읽는 생산 library 경로를 실행한다. private for_test_boot를 호출하지 않는다. 네 조합의 회수와 원 source/외부 sentinel 보존, sameboot pending/boot·digest mismatch/hardlink/Busy/unknown entry/partial record의 보존을 비교한다.

## 검증과 handoff

Cargo/제품/schema/다른 파일 수정 금지. 신규 integration test source만 정렬·동결해 root 실제 Cargo에 인계한다. helper source를 test-local modules로 포함하는 단위시험과 공개 production 경로의 결과를 구분한다.

## source freeze

integration fixture은 original compact record 문자열의 boot 필드 exact 1회 치환과 SHA256 갱신만 수행하며 parser Value는 읽기·anchor 추출에만 쓴다. 새 filesystem 관측값으로 anchor를 대체하지 않는다. production library는 보통의 dependency로 호출하여 cfg(test) constructor를 사용하지 않는다. test-local 모듈에 포함된 내부 unit들은 별도 helper 계약 검증이며 그 synthetic constructor 결과를 공개 생산 경로 검증으로 집계하지 않는다.

공개 integration 시험 두 개에 pending 네 조합과 보존 일곱 case를 작성했다. slot1의 metadata0 실제 synthetic receipt 선행과 FD 전체 Drop을 명시했다. sameboot에는 boot 문자열이 바뀌지 않으며 public recover가 pending을 보존해야 한다. hardlink/unknown entry 및 partial recovery record는 원 bytes 보존을 검사한다. kernel boot 파일 및 시스템 설정은 변경하지 않는다.

`.local-tools/cargo/bin/rustfmt --edition 2024 crates/engine/tests/cookie_priorboot_public.rs` 실제 exit 0. SHA256 `ff2c4bec9c1b48526f31056fdb8d75ad3f930f650b2042a3c18a754552d8c9e2`로 root에 동결 handoff했다. 자체 Cargo는 실행하지 않았다. 실제 재부팅 없이 합성 과거 boot record와 실제 현재 kernel 읽기를 결합하는 시험이며, root actual integration 결과가 다음 인수다.

root 첫 `cargo test -p dlpad-engine --test cookie_priorboot_public --locked -- --nocapture` 실제 27 PASS: 공개 API 시험 2개와 포함된 test-local 단위시험 25개다. `cookie-priorboot-public-first/result.json`에 fixture SHA 일치와 exit0를 보존했다. 공개 시험 내부의 네 pending 조합과 일곱 보존 case를 별도 인수로 확인했다.

root가 후속으로 sameboot/Busy의 deferred와 변조의 rejected를 정확하게 구분하는 assertion, cookie copy 전체 bytes·외부 hardlink·추가 foreign entry bytes 보존 검사를 보강했다. `cookie-priorboot-public-second/result.json`의 같은 target test와 `cargo clippy -p dlpad-engine --test cookie_priorboot_public --locked -- -D warnings` 모두 실제 exit0이며 fixture SHA가 유지됐다. 기본/CPU 전체 workspace 검증은 이 신규 test 추가 전 snapshot이고 제품 Rust 구현은 그 이후 변경하지 않았다. 실제 kernel 재부팅·실제 인증·다른 OS의 시험으로 확대하지 않는다.

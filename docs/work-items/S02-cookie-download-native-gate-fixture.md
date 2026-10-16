# S02: 인증 download native gate 수직 fixture

- 상태: implemented / native verification pending
- 분류: validation
- 우선순위: P0
- 요구사항: R02, R08, R10
- 담당/수정 파일: soak_recovery_audit — crates/engine/tests/cookie_download_gate.rs, 이 문서
- 의존성: root 최신 process gate/helper build 및 실제 Linux user manager

## 범위와 인수 조건

임시0700 cache·합성 Netscape cookie·합성 Python tool만으로 공개 DownloadRepository execute_with_session의 정상 metadata/download, metadata 실패, download 실패 및 실제 cancel을 검사한다. 별도 ignored subprocess driver로 backend 환경 race를 피한다. Literal privatecopy argv/결과 omission/DB 비밀 부재/정상 cleanup/source sentinel 보존을 검사하며 각 실행은 새 lease를 쓴다. 기본 tests는 manager에 의존하지 않는다. Cargo/native 실행은 root 담당이며 제품·공유 파일은 수정하지 않는다.

## 구현과 검증 handoff

`native_authenticated_success`, `native_authenticated_metadata_failure`, `native_authenticated_download_failure`, `native_authenticated_cancel`은 실제 manager/helper가 필요한 ignored capability tests다. 각 test는 자기 binary의 ignored worker subprocess에 명시 backend를 전달하므로 전역 환경을 변경하지 않는다. 부모 deadline30초, cancel-ready8초, synthetic worker10초 상한이다. 각 fixture는 temp0700 root/cache, 외부 source와 unrelated sentinel을 보존하며 version에는 cookies 없음/metadata-download에는 같은 literal copy 사용을 검사한다. 결과 logs/metadata와 DB job/options/rawbytes에 합성 secret/privatecopy 경로가 없고 정상 receipt 후 privatecopy cleanup을 검사한다.

이 fixture는 정상 종료/failure/cancel의 confirmed cleanup이 필수다. `SessionCleanupDeferred`를 정상 PASS로 바꾸지 않는다. 불확실 guardian 정리의 privatecopy 보존은 기존 session_cookie_download fault fixture가 별도 검증한다. 한 lease의 slot 재사용 거절 자체는 process/cookies 담당 회귀에 맡기며 이 fixture는 각각 새로운 lease만 사용한다.

소유파일 `rustfmt --edition 2024 crates/engine/tests/cookie_download_gate.rs` 실제 exit0. 초기 sandbox project alias가 read-only여서 apply_patch로 파일 작성 후 허용된 단일파일 formatter만 승격 실행했다. Source freeze SHA256 `96aa412cf2014863e53fd84a52c8c72d02e693bbc9e21623a32da02b3399f126`. Cargo/native/GUI 실행은 하지 않았다.

Root 통합 명령: 최신 helper build 이후 `cargo test -p dlpad-engine --test cookie_download_gate native_authenticated_ -- --ignored --test-threads=1`. Compile/clippy/실제4case 결과는 아직 미검증이며 READY를 PASS로 표현하지 않는다.

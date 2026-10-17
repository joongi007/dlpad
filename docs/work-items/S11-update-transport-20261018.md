# S11: 서명된 앱 업데이트 HTTPS 전송

- 상태: implemented (통합 검증 대기)
- 분류: feature
- 우선순위: P1
- 요구사항: R08, R10, R11
- 담당/수정 범위: update_transport; 루트 통합 전 /tmp/dlpad-update-transport-20261018/transport.rs, tests.rs만 작성
- 선행 작업: 루트 app_update metadata 검증 계약 및 staging의 원본 HANDLE 보존

## 사용자 결과

고정된 공개 릴리스 경로에서 메타데이터와 서명을 읽고, 미리 고정된 키로 원본 바이트 검증을 통과한 경우에만 설치 ZIP 스트림을 제공한다. 자동 설치·실행 권한은 부여하지 않는다.

## 계약과 범위

fetch_candidate는 호출자 key_id_hint와 pinned keys를 받는다. 공개키·인증·쿠키는 네트워크로 수집하거나 전송하지 않는다. open_package는 VerifiedCandidate만 받으며, 호출자가 staging 검증을 끝내기 전 스트림/파일을 신뢰하면 안 된다. 전달 스트림은 서명된 크기 + 1 byte, 전체 deadline, cooperative cancellation으로 제한한다. 긴 I/O는 worker에서만 호출한다.

공유 Cargo/schema/module 수정 없음. 기존 ureq 3.4.2와 verifier만 사용한다. redirect는 최대 4회, 각 목적지를 요청 전에 검증하며 github.com은 최초의 정확한 릴리스 asset URL만 허용한다. signed query는 release-assets.githubusercontent.com의 delivery 경로에서만 허용한다. metadata 64KiB, signature 128 lowerhex + 선택적 LF 1byte, headers 16KiB 제한. 네트워크 요청은 명시적 인증 헤더, cookie, proxy, 연결 풀을 사용하지 않는다.

## 인수 조건

금지 redirect 대상에 요청이 전송되지 않는다. signature 변조·크기 초과·취소·timeout·전송 실패 시 package request가 생기지 않는다. 수신 패키지는 staging에서 원본 파일 identity와 signed hash를 재검증한다. production signing key가 없다면 다운로드/업데이트 가용성을 주장하지 않는다.

## 검증과 handoff

독립 Rust 1.94 harness에서 verifier 단일 모듈의 복사본(테스트 등록만 제거)과 transport를 함께 빌드했다. `cargo test --manifest-path /tmp/dlpad-update-transport-20261018/Cargo.toml --offline --locked`: 13 PASS / 0 FAIL. `cargo clippy --manifest-path /tmp/dlpad-update-transport-20261018/Cargo.toml --all-targets --locked --offline -- -D warnings`: exit 0. `cargo fmt --manifest-path /tmp/dlpad-update-transport-20261018/Cargo.toml`: exit 0. fixture 전용 lockfile이며 실제 workspace lockfile 수정은 없다. 루트가 source를 통합한 뒤 workspace의 정확한 잠금 버전으로 검증해야 한다. 실제 공개 릴리스/서명키/Windows 설치 GUI 네트워크 실행은 이 단위의 범위 밖이다.

통합 파일 매핑: transport.rs → crates/engine/src/app_update/transport.rs; tests.rs → crates/engine/src/app_update/transport/tests.rs. root는 `pub mod transport;` 등록을 담당한다. 모듈 정상 레이아웃 기준이다. 공개 `Transport`는 테스트/worker 구현 계약이며 임의 사용자 URL을 입력받는 제품 API가 아니다. 반환 reader는 `Send`를 요구하지 않으므로 worker 내부에서 생성·소비하고 UI thread로 전달하지 않는다.


Root 실제 workspace 통합: Windows targeted27PASS(HTTP13 포함), 이후 workspace strict Clippy/전체test/release build 모두exit0. 원 Job 자연종료·forced0·private/inputdesktop불변·소스불변을 확인했다. 원격운영키·실제공개 signed 릴리스 경로는 아직 미실행이다.

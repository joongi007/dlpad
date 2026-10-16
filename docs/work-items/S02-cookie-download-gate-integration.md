# S02: Cookie download process gate 연결

- 상태: implemented / integration verification pending
- 분류: implementation
- 우선순위: P0
- 요구사항: R02, R08, R10
- 담당/수정 파일: soak_recovery_audit — crates/engine/src/download.rs, 이 문서
- 의존성: root의 process::run_metadata_with_cookie/run_with_cookie 구현

## 범위

Linux 인증 metadata/download 호출을 lease-aware process API에 연결한다. 각 새 session lease는 metadata slot0/download slot1만 사용하며 session worker가 소비된 두 slot을 재사용하지 않는다. 기존 ProcessSpec, 쿠키 없는 실행, Windows pending-reader 정책을 유지한다. 공유 process/cookies/schema/Cargo 파일은 수정하지 않는다.

## 인수 조건

- Linux cookie 실행에서 download 계층의 begin_reader를 중복 호출하지 않는다.
- typed SessionCleanupDeferred mapping과 실패 후 보존 정책을 유지한다.
- 소스 freeze 후 root가 통합 컴파일/회귀를 실제 실행한다. Agent는 Cargo를 실행하지 않는다.

## 구현과 handoff

Linux cookie metadata는 `process::run_metadata_with_cookie`, download는 `process::run_with_cookie`를 호출한다. Linux download 계층의 별도 reader 예약/confirm은 제거했고 slot0/1 예약은 process API에 위임했다. 쿠키 없는 version/metadata/download는 기존 process 호출이며 non-Linux는 기존 reader 예약/confirm 정책을 유지한다. `checked_with_cookie`는 기존 ProcessSpec을 받아 공통 outcome/exit/truncation 검사와 typed cleanup mapping을 보존한다.

Session worker는 한 lease의 두 gate slot을 새 실행에 재사용하지 않는다. Metadata 또는 download slot이 소비되면 retry/resume/new session은 새 lease를 import해야 한다. Gate 실패에 direct 실행으로 fallback하지 않는다.

실행: 소유 파일에 `rustfmt --edition 2024 crates/engine/src/download.rs`, exit0. Source freeze SHA256 `9e46d75ff537399ef87121a666e1fbaf566cad7509390ffc789f7253eef57343`. Cargo/제품 실행/공유 파일 수정은 없다. Root process API 구현과 통합 compile/clippy/실제 회귀는 아직 이 작업에서 검증하지 않았으며 root가 수행한다.

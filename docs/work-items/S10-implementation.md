# S10: 오프라인 진단 보고서 core

- 상태: implemented-core, integration-pending
- 분류: feature
- 우선순위: P0
- 요구사항: R13
- 담당/수정 범위: diagnostics 담당 — `crates/core/src/diagnostics.rs`, 이 작업 기록
- 선행 작업: S02의 Attempt 연결, 주 담당의 Cargo workspace/serde 등록

## 사용자 결과

실패한 Attempt의 단계·종료 코드·도구 버전·bounded 증거를 네트워크 없이 Markdown/JSON 보고서로 미리볼 수 있는 core API를 제공한다. 위험한 증거는 생략하고, 사용자는 로그를 편집하거나 제외할 수 있다. URL 전체를 보고서 내 안정적인 대체값으로 바꾼다. UI 연결·복사·파일 저장은 통합 담당 범위이며 이 작업만으로 최초 릴리스 완료를 주장하지 않는다.

## 계약과 범위

ADR-0004와 DIAGNOSTICS의 P0 계약을 따른다. schema_version 1, 매 보고서 임의 ID, Unix 생성 시각, Attempt 참조, 수집 범위, 생략/잘림과 잔여 위험을 제공한다. 원본 로그·쿠키·환경 dump·원본 첨부는 직렬화하지 않는다. 선택한 로컬 MPD의 구조만 quick-xml 0.41로 읽는다. 파서는 원격 요청을 하지 않으며 DTD·entity·CDATA·processing instruction을 거절한다. 잘못된 UTF-8/XML·1 MiB 초과·깊이 64 초과·구조 node 256 초과는 원본 fallback 없이 생략한다. 보고서는 실행 권한·외부 전송 기능이 없다.

공개 API: `DiagnosticReport::new`, `add_tool`, `add_log`, `edit_log`, `omit_log`, `logs`, `tools`, `report_id`, `json_preview`, `markdown_preview`, `add_mpd_attachment`, `mpd_attachments`, `attachment_omissions`, `omit_mpd_attachment`; `FailureStage`, `ToolComponent`, `ErrorClass`. core lib export와 dependency 변경은 주 담당이 맡는다. 저장 전 add/edit 정제를 강제하기 위해 원문을 포함할 수 있는 필드는 private이다. 로그 128행, 행 4096 bytes, 도구 16개, URL 대응 128개로 상한을 둔다. 버전은 48-byte 숫자 중심 문법만 허용한다. 큰 증거는 원문 fallback 없이 생략한다.

일반 문자열의 완전한 익명화는 보장할 수 없다. 알려진 민감 키워드·헤더·경로·IP·이메일·assignment·큰 opaque 값은 행 전체를 생략한다. 이 보수적 정책은 일부 유용한 로그도 제거한다. MPD는 BaseURL·SegmentList·Initialization·SegmentURL만 구조화하며 sourceURL/media 유무, byte range, depth, URL 동일성을 보존한다. 상속과 완전한 manifest 의미는 미확인으로 표시한다. 보고서에 첨부/실패 이유를 합해 8개까지만 기록한다. P1 catalog/P2 API는 후속 작업이다. quick-xml은 순수 Rust parser이며 비동기 런타임·네트워크·XML entity 확장 의존성을 추가하지 않는다; lockfile과 전체 비용 측정은 통합 담당 범위다.

## 인수 조건

- secret/path/IP fixture는 JSON과 Markdown에서 누출하지 않는다.
- 같은 URL은 같은 대체값이며 원본 path/query는 출력되지 않는다.
- 편집 시에도 정제하고 위험한 편집은 생략한다.
- 증거를 삭제·편집하면 분류를 남은 증거에서 재계산한다.
- oversized 입력은 UTF-8 경계를 훼손하지 않고 생략하며 잘림을 표시한다.
- Markdown 증거는 indentation으로 감싸서 backtick fence 탈출을 막고 악성 지시를 untrusted evidence로 표시한다.
- 네트워크·AI 키·외부 도구 없이 테스트할 수 있다.

## 검증과 handoff

- 실제 실행: `/tmp/dlpad-cargo/bin/rustfmt --edition 2024 crates/core/src/diagnostics.rs` (RUSTUP_HOME/CARGO_HOME을 /tmp 개발 도구로 지정): 성공.
- 실제 실행: 별도 `/tmp/dlpad-diagnostics-check` harness의 `cargo test --offline`: 4 passed. `cargo clippy --offline --all-targets -- -D warnings`: 성공. 실제 모듈을 path로 참조해 검사했으며 workspace 통합 검증은 주 담당이 실행한다. MPD 추가 후 `rustc --test`로 실제 모듈과 기존 /tmp harness rlib를 직접 링크한 `/tmp/dlpad-diagnostics-mpd-tests` 실행: 7 passed. 공유 Cargo/lockfile을 건드리지 않았다. 최종 workspace clippy/test는 통합 담당이 수행한다. UI·clipboard·저장·3 OS는 미검증이다.
- 일반 상대 경로는 물리 경로 대소문자 차이로 read-only였으며 승인된 workspace의 absolute lowercase 경로 쓰기는 성공했다. 기존 Git은 조작하지 않았다.

## S10 로컬 전달 adapter 확장

- 담당 추가 파일: `crates/engine/src/reports.rs` (engine lib export/UI 연결은 통합 담당).
- `save_report`는 정제된 preview를 먼저 직렬화·2 MiB 검증한 뒤 `create_new`로 새 파일만 만든다. JSON은 serde serializer 결과만 사용한다. Unix permission 0600, write/sync 실패 시 소유 파일의 dev/inode가 현재 경로와 일치하는 경우만 정리한다. 이미 존재하는 파일은 거절하고 보존한다.
- `attach_local_mpd`는 사용자가 고른 regular local 파일을 최대 1 MiB+1 byte만 읽는다. 초과·손상·정제 실패는 원본 fallback 없이 parser의 생략 이유로 처리한다. 원본 파일은 수정하지 않는다. 모든 I/O는 UI worker에서 호출해야 한다.
- `demo_report`는 fixture 전용 합성 실패이며 실제 영상·계정·비밀을 사용하지 않는다.
- 실제 검증: `rustfmt --edition 2024 crates/engine/src/reports.rs` 성공. `rustc --test`로 existing /tmp workspace rlib를 직접 링크한 `/tmp/dlpad-report-export-tests`: 4 passed (기존 파일 보존, valid JSON, 2 MiB 초과 export 거절, 1 MiB 초과 local attachment 거절, secret fixture 제거, 소유 partial 삭제와 replacement 보존).
- 미검증: Windows에서 std만으로 파일 identity를 증명할 수 없으므로 write 실패의 uncertain partial은 삭제하지 않는다. 별도 안전한 handle-based 정리 adapter가 필요하다. Linux 검사도 동시 외부 path replacement가 발생하는 hostile directory의 race-free unlink까지 보장하지 않는다. 실제 clipboard/GUI 저장 UX 및 3 OS 동작은 통합 검증 범위다.

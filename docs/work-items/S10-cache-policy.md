# S10: 진단 자료의 세션 보관과 사용자 파일 보존

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R13, R08
- 담당/수정 범위: diagnostic_cache agent; crates/engine/src/reports.rs, crates/engine/tests/diagnostic_cache.rs, docs/work-items/S10-cache-policy.md, docs/validation/diagnostic-cache-validation.md
- 선행 작업: S10 로컬 정제 보고서와 create-only export

## 사용자 결과

보고서 미리보기와 첨부 읽기에서 진단 cache 파일을 만들지 않는다. 사용자가 저장한 보고서와 원본 첨부는 앱의 cache 청소 대상으로 취급하지 않는다.

## 계약과 범위

[ADR-0004](../adr/0004-diagnostics.md)와 [진단 설계](../DIAGNOSTICS.md)를 기준으로 실제 disk I/O를 조사한다. 현재 보고서는 app의 메모리 상태에만 있으며 자동 진단 cache는 0 byte다. 20 MiB/7일은 저장 cache를 도입할 때의 제안으로, 없는 cache를 만들거나 사용자 파일을 만료 삭제하지 않는다. 명시적 export는 사용자가 관리한다. 원본 첨부는 read-only이며 정제 구조만 보고서에 남긴다. shared schema, Cargo, app UI, 공통 문서는 주 담당이 조정한다.

## 인수 조건

- 첨부/정제/편집/제외/미리보기/보고서 drop 과정은 원본 파일, 기존 사용자 export, job/partial fixture를 변경하거나 cache 파일을 생성하지 않는다.
- 새 Markdown/JSON 저장은 지정 파일만 생성하며 기존 파일 충돌은 보존한다.
- 손상·DTD·초과 첨부는 원본 보존과 fail-closed 처리를 유지한다.
- 보고서/첨부 bounds와 OS별 미검증 범위를 검증 문서에 기록한다.

## 검증과 handoff

자동 disk cache 0 byte/세션 한정 정책을 reports.rs의 공개 모듈 계약과 [전용 검증 문서](../validation/diagnostic-cache-validation.md)에 기록했다. 사용자 파일을 삭제하는 cache 청소 API 또는 불필요한 persistent cache를 추가하지 않았다. 파일과 빈 디렉터리를 포함한 snapshot으로 첨부·정제·미리보기·편집·제외·drop, 명시 MD/JSON 생성, 충돌·손상/DTD/초과/저장 실패를 검증했다.

실행 결과(Linux/WSL):

- `cargo test -p dlpad-engine --test diagnostic_cache --locked`: 최종 3/3 통과, 0.01초.
- `cargo test -p dlpad-engine --lib reports::tests --locked`: 기존 실패 경로 4/4 통과, 0.05초.
- `cargo clippy -p dlpad-engine --test diagnostic_cache --locked -- -D warnings`: 통과.
- `rustfmt --edition 2024 --check crates/engine/src/reports.rs crates/engine/tests/diagnostic_cache.rs`: 통과.
- `python3 scripts/check_docs.py`: 62 문서/로컬 링크 통과(병렬 작업 중 측정).

초기 시험은 token 포함 로그 전체 생략으로 편집할 행이 없어 1건 실패했다. fixture에 정상 관측 행을 추가하고 다시 시험했다. 실제 정제/편집 정책은 변경하지 않았다. 정상 Git 저장소가 없어 기존 .git을 보존했다. 새로운 의존성/API/schema와 실행 파일 code path 변경은 없다. 전체 workspace 통합 검증과 공통 DIAGNOSTICS/P0-status 문서 반영은 주 담당이 수행한다. Windows/macOS 실행·clipboard·Windows 파일 identity·전체 export GUI 검수는 미완료이며 이 작업을 전체 S10/P0 출시 완료로 해석하지 않는다.

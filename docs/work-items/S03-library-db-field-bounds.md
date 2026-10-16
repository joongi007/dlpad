# S03: 라이브러리 DB 필드 선제 byte 상한

- 상태: done (Rust 결과 필드 디코딩 상한 범위)
- 분류: fix
- 우선순위: P0
- 요구사항: R03, R01
- 담당/수정 범위: diagnostic_cache 초안; root 통합·검증 — crates/engine/src/library.rs, 이 문서, 전용 evidence.
- 선행 작업: S03-library-scan-bounds-audit; root 독립 compile/회귀/native 검증.

## 사용자 결과

손상 DB의 거대한 path/file_identity를 String으로 materialize하기 전에 SQL에서 거절한다. invalid 행을 조용히 제외하지 않고 오류를 반환하여 UI의 이전 목록 보존 계약을 유지한다.

## 계약과 범위

원 `crates/engine/src/library.rs` SHA256 `ab65f218e5c702139001046b12aaac194e9e2bcf847f29ed52317f7b329a68fa`의 사본을 `/tmp`에서만 수정한다. core/Cargo/schema/기존 fixture/example/API를 바꾸지 않는다. `library_page` keyset 조건·정렬·256행 cap 및 rename identity 값을 유지한다. Next full-root 재scan과 SQLite publication VM 취소는 별도 작업이다.

path byte cap128KiB는 Windows32766 UTF16 units의 최대UTF8 약98300bytes를 수용한다. identity128bytes는 현재 unix/dev/inode 및 windows:v1/volume/32hex 표현을 수용한다. 이 작업은 identity token의 삭제 권한을 새로 만들거나 형식을 추정하지 않는다. SQL typeof TEXT 및 length(CAST AS BLOB) 검사와 CASE로 거절된 payload를 NULL로 결과화한다. 작은 validity bool을 먼저 읽고 fixed InvalidData conversion error를 반환한다. SQLite 내부의 원 TEXT 검색·처리 allocation까지 상한을 증명하지 않는다.

기존 `location.rs:39`의32KiB UTF8 정책은 선택한 root 입력 상한이며, library DB의 각 하위 파일 전체 경로와 동일한 대상은 아니다. 이 작업은 root 정책을 바꾸지 않는다. Windows UTF16과 UTF8 길이를 동일시하지 않고 한글/emoji의 정상 경로를 byte guard로 불필요하게 자르지 않는다. SQL CASE는 `(root,present,path>after)` 조건 및 LIMIT으로 조회된 결과에서 Rust String 반환을 막지만 SQLite의 거대 TEXT index/key 비교·CAST AS BLOB 처리 과정은 별도 내부 비용이 발생할 수 있다. CAST가 항상 zero-copy이거나 전체 쿼리 heap/RSS가128KiB로 제한된다고 주장하지 않는다. root의 실제 비용 관측과 VM/SQLite limits 후속 정책이 필요하다.

## 인수 조건

1. 실제 SQLite `CAST(zeroblob AS TEXT)`로 path/identity 각각 limit+1 및8MiB를 생성한다. Rust에 거대 String fixture를 먼저 만들지 않는다. SQL BLOB length로 실제 저장 bytes를 확인하고 page 호출은 bounded error로 실패한다.
2. 오류 후 DB 행/원 payload 길이는 유지된다. 임의 skip/전체 page empty-success로 변환하지 않는다. 오류 메시지에 원 경로/identity 내용이 포함되지 않는다.
3. 한글32766자 synthetic DB path 및 정상 Windows identity가 그대로 반환되고 keyset after에서 중복 없이 끝난다. 실제 Windows filesystem 최대길이 생성 시험이라고 표현하지 않는다.
4. 기존 library rename/cancel/page/failed-scan 회귀와 root 전체 clippy/tests를 통과해야 한다. draft fmt만으로 성공 처리하지 않는다.

## 검증과 handoff

초기 draft SHA `71368c186c40a6ccb8ecbeba1a3135bdb6800b69c4f56f9fa617c50c1ad6c2aa` 후 보존 assertion을 추가한 최종 SHA는 `c98ddaf0b649e1b768abff342bb977c6bd71c3dffc91b45b9de52f3c7bc66e64`이다. 오류 전후 실제 BLOB byte lengths와 행 수를 비교한다. 개별 `/tmp` rustfmt는 완료했다. 프로젝트 삽입/Cargo/테스트/native 실행은 하지 않았다. 작성한 실패회귀의 원본 red 및 통합 green은 root가 실행해야 한다. heap/RSS 계측은 수행하지 않았으며 SQL CASE와 선제 bool 읽기가 보장하는 Rust 결과 디코딩 상한만 주장한다.

## root 실패 재현·통합

최초 red는 테스트용 상수누락 E0425 컴파일실패였으므로제품실패재현으로계산하지않는다. 이를testscope상수로보완한 windows-library-db-bounds-red-corrected-20261005-resume는compile성공 후실제oversized assert FAILED/exit101로원쿼리의무제한String decode를재현했다. root는원파일+추가테스트 SHA/내용을확인하고최종draft를통합했으며fmt후 cargo test -p dlpad-engine --locked library::tests 실제8case PASS다. limit+1/8MiB path·identity 실패와오류후원DB보존, Unicode/keyset/정상identity가포함된다. 전체Windowsworkspace회귀를별도실행중이다. SQLite내부메모리상한·실제최대길이OS경로생성은이시험으로증명하지않는다.

root가후속전체fmt/clippy/test/release PASS/current source_final SHA/actualreleaseSHA를독립확인했다. 기존librarycancel/rename회귀를포함한전체WindowsGNU CPU workspace근거는 windows-db-bounds-integrated-workspace-20261005-resume/root-review.json이다. SQLite전체메모리상한은범위밖이다.

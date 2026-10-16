# 라이브러리 파일 identity 검증

2026-10-01 Linux/WSL 개발 환경에서 임시 합성 파일만 사용했다. 사용자 Documents나 실제 영상을 읽거나 변경하지 않았다.

## 구현과 범위

Unix `dev + ino`를 관측 증거로 저장한다. library schema의 nullable `file_identity`와 `(root,file_identity)` index는 주 담당이 조정했다. 이전/새 스캔 각각 identity가 한 개이고, 이전 경로가 사라졌으며 새 경로가 기존 row를 차지하지 않았을 때만 기존 row의 경로를 변경한다. 하드링크로 identity가 중복되면 이전 기록을 새 이름으로 이전하지 않는다. 같은 경로의 새 파일은 관측 identity를 갱신한다.

전체 스캔은 SQLite temporary observation table로 스트리밍하며 성공 후 transaction으로 반영한다. 디렉터리 stack 깊이는 128, 페이지는 기존 256행 상한이다. 추가 Rust dependency는 없고 파일 내용/전체 해시를 읽지 않는다. DB 및 임시 DB 용량은 파일 개수에 비례하며 Rust에 전체 파일 목록을 적재하지 않는다. index와 temporary snapshot의 디스크/I/O 비용이 추가된다.

## 실제 명령과 결과

- `rustfmt --edition 2024 crates/engine/src/library.rs crates/engine/src/library/identity.rs`: 성공.
- `cargo test -p dlpad-engine --lib library --locked`: 9개 성공, 0개 실패, 시험 본문 0.14초. 한글 rename record rowid/identity 유지, 이전/새 하드링크 중복, 같은 경로 다른 파일 identity, 실패 scan rollback, 기존 symlink/페이지/권한 회귀를 포함한다.
- `cargo clippy -p dlpad-engine --lib --locked -- -D warnings`: 성공, warning 0개, 6.96초.

## 제한과 남은 게이트

Linux 실증만 완료했다. Unix inode는 영구 source ID가 아니며 삭제 이후 재사용될 수 있다. 여기서의 unique identity 연결은 문서의 rename 추정이고 출처/Download job 증명을 뜻하지 않는다. source/sidecar 연결 schema는 아직 없다. Windows는 identity 미지원(None)으로 이름 변경을 삭제/추가 관측하며 native file ID adapter 실증은 남았다. macOS 실증도 남았다. GUI 전체 통합, 공개 release 크기/자원 측정과 공통 검증은 주 담당이 수행한다.

# S05: 외부 FFmpeg 선택과 실행 증거

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R07, R13
- 담당/수정 범위: 주 담당; crates/core/src/options.rs, crates/engine/src/download.rs, crates/engine/tests/local_media.rs, crates/app/src/main.rs, catalog.rs, 관련 문서
- 선행 작업: typed argv, Attempt 증거/SQLite, 실제 합성 구간 저장

## 사용자 결과

기존 FFmpeg 실행 파일의 절대 경로를 입력한다. 선택을 실행 계획에 고정하고 실행 전 버전 확인, Attempt 경로/버전 기록과 실패 보고서 전달을 한다. 빈 값은 기존 PATH 탐색을 유지하며 별도로 검증한 버전을 주장하지 않는다.

## 계약과 범위

주 담당이 공유 DownloadOptions 선택 필드와 adapter-local migration v3를 조정한다. 기존 API 유지, 이전 jobs snapshot 변경 없음(선택 None은 argv 동일). 외부 파일 삭제/변경 없음. 관리 설치/공급처는 후속이며 Cargo dependency 변경 없음.

## 인수 조건

경로를 단일 argv로 유지, 경로 변경은 snapshot 불일치로 거절. 파일 없음/버전 형식 오류/취소는 Downloaded 아님. 버전은 Attempt 단위로 DB reopen 후 유지. PATH 밖 한글·공백 경로의 실제 FFmpeg로 빠른/정확 합성 구간 저장과 decode 시험.

## 검증과 handoff

경로 argv 보존·snapshot 변경 거절·DB reopen 증거·파일 없음/폴더/잘못된 버전/비정상 종료·버전 확인 중 취소를 검증했다. 실제 FFmpeg를 한글·공백 경로에 임시 복사해 빠른/정확 구간 저장과 decode 성공. 최종 Rust 82 tests, 실제 도구 1 test(21.78초), fmt/clippy/release 및 isolated UI/Python 안전 4 tests 성공. [통합 결과](../validation/options-validation.md). Windows/macOS 실행과 관리 설치 공급처는 미검증. 외부 파일의 버전 확인 뒤 변경을 막지 않으며 증거는 확인 시점 기준이다.

# S04: TAR.XZ source gate integration regressions

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R05, R08, R13
- 담당/수정 범위: diagnostic_cache; crates/engine/tests/tar_xz_gate.rs, 이 문서만
- 선행 작업: root extract_verified_tar_xz 및 공개 source spec/export

## 사용자 결과

압축 원본 pin이 확인된 stream만 TAR reader에 전달되며 변조·절단·추가 stream·과대 dictionary·취소·시간 제한이 원본/외부 파일을 수정하지 않고 실패하는지 검사한다.

## 계약과 범위

작은 합성 USTAR(선택 abc)를 작성 시 Python 표준 lzma로 압축한 bytes 상수로 고정한다. runtime Python/네트워크/실제 도구/GUI 없음. temp source/output/sentinel만 사용한다. 제품 Rust/Cargo/export는 root 담당이다.

## 인수 조건

정상 component exact bytes; wrong SHA/size, truncation, corrupt stream, trailing garbage, concatenated stream, cancel/deadline, 128MiB dictionary가 모두 실패한다. 매 시도 원 source와 sibling sentinel을 보존하고 output은 별도 owned file이다. 실패 output은 caller 소유이며 partial bytes 가능성을 허용한다; cleanup 권한을 주장하지 않는다. nonempty output 사전 거절은 기존 내용 보존을 확인한다.

## 검증과 handoff

source 작성/fmt 후 root scoped test 대기. fixture 생성은 Python stdlib lzma FORMAT_XZ 기본과 LZMA2 dict_size=128MiB 사용. 실제 Windows/Linux native 및 공식 공급은 이 합성 테스트로 대체하지 않는다.

Root scoped tests 4 PASS / engine all-target Clippy PASS を 인수했다. 리뷰 후 Windows cleanup 실패 무시 위험을 교정: source/output보다 뒤에 TempDir guard를 배치하고 manual Drop 및 PID/sequence naming을 제거했다. 128MiB dictionary는 독립 테스트에서 ToolError::Io ErrorKind::OutOfMemory를 요구한다(lzma-rust2 0.21.0의 memory gate error). 강화된 5개 테스트는 root 재실행 대기다.

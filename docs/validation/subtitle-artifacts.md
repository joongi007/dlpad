# 자막 staging 증거 검증 helper

2026-10-01 Linux/WSL, Rust 1.94.0. 임시 폴더와 합성 파일/sparse 크기 fixture만 사용했다. 사용자 영상·Documents·외부 URL·인증을 읽거나 변경하지 않았다.

## 인터페이스

`download::subtitles::collect_staging_subtitles(staging_root, job_id, source_id, selected: SubtitleOptions, evidence_json)`는 최대 두 SubtitleStagingEvidence(path/bytes/language/extension)를 반환한다. 입력 JSON은 다운로드 이후 `{id, requested_subtitles:{ko:{filename,ext},en:{filename,ext}}}`이며 caller가 선택 URL snapshot에서 관측한 source metadata ID와 비교한다. ID 일치는 암호학적 원본 증명이 아니다. metadata 자체와 실제 실행 argv의 소유·일치 확인은 통합 담당 계약이다. helper는 자동 생성 여부를 파일명으로 추측하지 않는다.

requested_subtitles 누락/null은 빈 결과로서 언어 미제공이다. source ID 없는 JSON은 거절한다. staging 폴더에서 이름만 보고 별도 파일을 찾거나 완료로 등록하지 않는다. 선택 언어 유한 catalog의 exact ko/en만 허용하며 duplicate key/선택 밖 언어를 거절한다.

metadata 128KiB, requested map 최대 두 개, source ID 256바이트, filename 32768바이트 상한이다. 요청 map은 custom streaming serde visitor로 제한하며 다른 metadata 필드는 retain하지 않는다. 허용 ext는 vtt/srt/ttml이다. 현재 core의 vtt/best fallback이 다른 format을 선택하면 helper는 안전하게 실패하며 허용 format 확장은 별도 검증이 필요하다.

canonical absolute nonsymlink staging root의 direct child `job-id.language.ext`만 허용한다. job ID는 최대 128개 ASCII alnum/-/_ 문자다. 파일은 nonsymlink regular, nonempty, 최대 16MiB여야 한다. 내용/전체 해시를 읽지 않으므로 크기가 아니라 metadata만 관측하며 syntax/decode 유효성을 주장하지 않는다. 추가 dependency/schema는 없다.

## 검증

- `rustfmt --edition 2024 crates/engine/src/download/subtitles.rs`: 성공.
- `cargo test -p dlpad-engine --lib download::subtitles --locked`: 경계 회귀 포함 7개 통과, 0개 실패, 0.02초.
- `cargo clippy -p dlpad-engine --all-targets --locked -- -D warnings`: 최종 경계 회귀 포함 성공, 4.79초.

기본 선택·두 언어·same source 요구, metadata 없음/null일 때 폴더 내 unclaimed 파일 무시, duplicate/unknown metadata, 비허용 ext/empty/폴더/용량 초과, 다른 job/path/루트 밖/relative/nested 경로, symlink file/root 거절과 외부 대상 bytes 보존을 검증했다. supported formats/정확 16MiB 상한과 metadata field 상한 회귀도 작성했다.

## 미완료와 통합 계약

주 담당이 download.rs에 시험 전용 module만 연결했다. production download pipeline, after-download JSON capture, UI, 다중 artifact SQLite publication plan은 아직 연결하지 않았다. 반환 값은 staging 후보 증거이며 VerifiedArtifact나 Downloaded 상태가 아니다. 조회/공개 사이 파일이 바뀔 수 있으므로 caller는 publication 시 identity·size를 다시 확인하고, multi-artifact plan 저장→create-only 공개→artifact 등록 및 Downloaded transaction→crash-gap 복구를 구현해야 한다. 사용자/외부 파일을 자동 정리하지 않는다.

이 helper 시험은 실제 yt-dlp 다운로드 성공이나 final publication/crash recovery를 대체하지 않는다. 실제 VTT sidecar 수신 옵션 smoke는 [별도 기록](subtitle-options.md)에 있다. Windows/macOS 실증과 실제 사이트 자막은 미검증이다.

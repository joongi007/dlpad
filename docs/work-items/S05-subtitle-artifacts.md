# S05: 자막 staging 증거 검증

- 상태: implemented
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R07
- 담당/수정 범위: library_identity 에이전트; 신규 crates/engine/src/download/subtitles.rs, 본 문서, docs/validation/subtitle-artifacts.md
- 선행 작업: S05-subtitles; 주 담당의 mod 연결·after-download evidence·다중 artifact journal 후속

## 사용자 결과

선택 언어와 동일 source metadata ID가 확인된 자막 파일만 최종 공개 후보로 받을 수 있다. 폴더의 임의 파일을 자막 완료로 추정하지 않는다.

## 계약과 범위

bounded metadata requested_subtitles를 최대 두 언어로 읽고 절대 canonical staging direct-child 경로, job/language/extension basename, regular/nonempty/16MiB 상한을 확인한다. source ID는 URL snapshot으로 관측한 extractor metadata의 일치이며 cryptographic 원본 증명이 아니다. 결과는 staging evidence이며 최종 Artifact/Downloaded를 만들지 않는다. 공유 schema, download.rs, Cargo, UI, 기존 core는 수정하지 않는다. Git 저장소 부재를 기록하고 기존 .git을 보존한다.

## 인수 조건

- 선택 언어와 source ID가 일치하고 실제 파일이 조건을 통과해야 staging evidence를 반환한다.
- metadata 128KiB, 최대 2개, filename 32768바이트, source ID 256바이트; duplicate language/unknown extension/손상/다른 source는 거절한다.
- symlink, root 밖, 비정규/빈 파일, oversized file은 거절한다.
- requested_subtitles 없음/null은 미제공 빈 결과이며 폴더를 임의 탐색하지 않는다.
- 최종 publication/crash recovery와 UI 미연결을 구현 완료라고 보고하지 않는다.

## 검증과 handoff

임시 synthetic 파일 시험 7개 통과(0.02초), engine all-target clippy 통과(4.79초). [검증 기록](../validation/subtitle-artifacts.md)에 파일/metadata 경계와 production 미연결을 기록했다. 주 담당이 시험 전용 module을 연결했으며 다중 artifact journal·production 연결은 주 담당 후속이다.

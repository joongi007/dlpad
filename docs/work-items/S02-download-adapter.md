# S02: 外部 yt-dlp 다운로드 어댑터

- 상태: implemented-foundation (앱 및 실제 도구 검증 대기)
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R05, R07, R08, R13
- 담당/수정 범위: domain 에이전트; crates/engine/src/download.rs, 이 문서. 통합 검토 요청으로 options.rs 밀리초 파싱 및 process.rs tail 보관 수정
- 선행 작업: S00/S01 출시 게이트, S02 process foundation; module/dependency 변경은 주 담당

## 사용자 결과

명시적 외부 실행 파일의 버전을 확인하고, 메타데이터 조회 후 typed 옵션으로 다운로드한다. 작업 snapshot과 Attempt 및 상태를 SQLite에 저장한다. v2 로컬 migration은 planned_root/path/source/bytes와 Attempt별 도구 경로·버전을 보존한다. tool_version(id)는 현재 마지막 Attempt의 버전만 반환한다. 재시작 복구는 active 상태를 Interrupted로 바꾸며 같은 job ID로 새 Attempt를 만든다. 파일 존재·크기·경로를 검증한 후에만 Downloaded가 된다. 알려진 원본 길이를 넘는 구간은 실행 전에 실패한다.

## 계약과 범위

DownloadRepository::open/create_job/job/recover/execute와 DownloadResult를 공개한다. 저장소는 별도 Connection과 download_jobs 테이블을 사용하고 revision 비교로 중복 시작을 거부한다. 단일 UI worker에서 호출할 동기 API이다. 기존 store schema는 수정하지 않았다. 도구 경로·버전은 실행에 pin하지만 binary hash 검증은 후속이다. snapshot은 argv 기반이며 비Unicode argv는 정직하게 거부한다.

[공식 yt-dlp 옵션 문서](https://github.com/yt-dlp/yt-dlp#usage-and-options)를 2026-09-30 조회하여 config/plugin 차단, no-overwrites, no-simulate, after_move/JSON 변환 계약을 확인했다. 실제 설치 버전의 지원은 아직 합성 fixture만 검증했다.

대상 볼륨 .dlpad-partials/job-id에서 받고, after_move 경로가 해당 디렉터리의 비symlink 파일인지 검사한다. 공개는 hard_link의 create-only 동작으로 기존 파일을 덮어쓰지 않는다. 메모리로 미디어를 읽지 않는다. hard link 미지원 파일시스템은 실패하며 자동 덮어쓰기 fallback이 없다. 공개 완료 이후 staging 링크만 제거한다.

메타데이터는 전용 process::run_metadata API에서 stdout 전체를 최대 8 MiB까지만 보관하며 stderr는 별도 256 KiB tail 및 한 줄 64 KiB 제한을 유지한다. 일반 runner의 제한은 변하지 않는다. metadata 초과는 OutputLimit으로 종료한다. serde의 선택적 Summary 필드(id/title/extractor/duration)만 최대 256/4096/256바이트 문자열 제한으로 추출하고 formats 및 서명된 media URL은 버린다. 원문 stdout은 다운로드 실행 전에 해제하며 일반 DB나 결과 보고서에 저장하지 않는다. 일반 다운로드 로그는 bounded tail ring으로 유지하여 장시간 progress 뒤의 after_move marker를 보존한다. 잘린 버전 stdout은 거부한다.

## 인수 조건

합성 임시 fixture에 버전/JSON/완료 출력, 파일 누락, 비정상 종료, 경로 탈출, 기존 파일 충돌, 로그 flood 뒤 완료, 취소, Interrupted 복구, snapshot 변경, 원본 길이 초과를 시험한다. 실제 영상·Documents·계정·업로드를 사용하지 않는다. 시간 파싱에 .001/.999, malformed/sub-ms/overflow를 추가했다.

## 검증과 handoff

rustfmt 성공. rustc --test로 core를 별도 rlib로 빌드하고 process+download 합성 테스트 12개 모두 통과했다. options 독립 테스트 5개 모두 통과했다. 최초 시도에서 오래된 core rlib export와 SQLite u64 ToSql 오류를 발견하여 최신 core로 재빌드 및 i64 checked conversion으로 수정한 뒤 재시험했다. 주 담당이 전체 Cargo 통합 검증을 수행한다.

미검증: 실제 yt-dlp/FFmpeg 사이트 다운로드, 실시간 진행률 UI, 8시간 soak, 3 OS 파일시스템, Windows 파일-ID 기반 crash-gap reconciliation, 정확한 영상 컨테이너 검증(ffprobe), 프로세스 crash guardian. 현재 검증은 비어 있지 않은 파일의 경로/크기이며 미디어 자체의 재생 가능 여부를 보장하지 않는다. 파일 공개 전 planned artifact를 기록한다. Unix 복구는 소유 root·파일명·크기와 staging/final dev+ino 동일성을 검사하여 Downloaded를 중복 없이 복원한다. 같은 크기의 기존 사용자 파일은 완료로 오인하지 않는다. 공개 이후 DB 저장 실패는 Verifying을 보존하여 복구한다. Windows는 파일-ID 검증 구현 전 보수적으로 Interrupted로 두며 완료를 추정하지 않는다. 이 기반을 S02/P0 출시 완료로 표현하지 않는다.

추가 검증: 공개 직후 프로세스 소멸을 합성한 DB 재오픈 복구(첫 recover=1, 다음=0), 동일 크기 foreign 파일 거부, v1→v2 migration 재실행을 포함하여 process+download 14개 모두 통과했다. Clippy가 지적한 중첩 if를 let-chain으로 정리했다.

추가 metadata 검증: 64 KiB 초과 JSON 성공, 8 MiB 초과 실패, noisy stderr 분리, formats/media URL 미보존, metadata 실행 중 취소를 검사했다.

최종 독립 rustc 테스트: process+download 18개 통과(0 실패). 문서 검사는 다른 담당자의 검증 문서 작성 중 참조 대상 2개 부재로 실패하여 통합 담당에게 전달했다.

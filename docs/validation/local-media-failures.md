# 실제 HTTP 실패·원본 재추출·같은 작업 재개

2026-10-02 Linux/WSL x86_64. 새 `crates/engine/tests/local_media_failures.rs`의 opt-in 시험이며 제품/Cargo/schema와 기존 local_media.rs는 변경하지 않았다. [작업 계약](../work-items/S03-real-http-failures.md), [고정 소스·버전 증거](local-media-failures.json), [실제 실행 로그](local-media-failures.log)를 보존한다.

## 재현과 관찰

명시적으로 선택한 기존 yt-dlp **2026.09.27.232945**와 FFmpeg **4.4.2-0ubuntu0.22.04.1**을 사용했다. FFmpeg가 testsrc2 320×180/20 fps/5초 MPEG-4를 임시 폴더에 생성했다. 합성 media는 694,337 bytes다. 사용자 영상·Documents·인증은 사용하지 않는다.

임시 Rust HTTP 서버는 127.0.0.1의 임의 포트에만 바인딩한다. 같은 `/source.html`에 video 요소를 제공하고, 실패 때 `/media.mp4?ticket=expired`, 복구 때 `/media.mp4?ticket=refreshed`로 미디어 URL을 바꾼다. 원본 SourceUrl snapshot은 바꾸지 않는다. test-only printf로 도구 오류를 대신하지 않고 실제 yt-dlp가 HTTP 응답을 소비한다. 다운로드 재시도는 typed `--retries=0`으로 고정한다.

| 실제 조건 | 실패 증거 | 명시 재개 결과 |
|---|---|---|
| 미디어 HTTP 403 | yt-dlp `HTTP Error 403: Forbidden`, nonzero exit, Failed, artifact 없음 | 같은 원본 페이지 재추출, refreshed 미디어 URL 요청, 같은 Job ID/새 Attempt, Downloaded |
| Content-Length보다 짧은 응답 뒤 연결 종료 | yt-dlp 32,768 bytes read / 661,569 more expected, nonzero exit, Failed, 31,744-byte `.mp4.part` 유지 | 실제 Range offset 31,744로 이어받기, 같은 Job ID/새 Attempt, Downloaded |

각 시나리오의 원본 페이지 요청은 두 Attempt 전체 4회이며 재개 후 증가를 assert한다. Failed에서 완성 경로와 artifact가 없고, DB를 닫고 다시 열어 Failed와 원래 typed 옵션을 확인한다. resume는 stored options를 읽으며 첫 Attempt를 유지하고 번호 2의 서로 다른 Attempt ID를 추가한다. 완성 파일 bytes가 합성 원본 전체와 정확히 일치한다. 다시 DB를 열어 Downloaded/두 Attempt/artifact를 확인하고 recover=0 및 partial 소멸을 확인한다.

## 안전 상한과 실행

media 최대 4 MiB, HTTP 요청 header 최대 8 KiB, 최대 256개 요청, Range 관측 최대 128개를 제한한다. socket read/write는 각각 500 ms, 서버 accept는 nonblocking, Drop은 stop flag와 owned thread join이다. 각 execute/resume에는 30초 취소 watchdog을 두고 완료 때 Condvar로 즉시 정리한다. FFmpeg 생성은 소유 process adapter의 20초 timeout/argv를 사용한다. watchdog에 의한 취소를 정상 HTTP 실패로 계산하지 않는다.

```sh
DLPAD_SMOKE_YTDLP=<existing-absolute-path> DLPAD_SMOKE_FFMPEG=<existing-absolute-path> \
  cargo test -p dlpad-engine --test local_media_failures --locked -- --ignored --nocapture
cargo clippy -p dlpad-engine --test local_media_failures --locked -- -D warnings
cargo test -p dlpad-engine --test local_media_failures --locked
```

실제 opt-in은 처음 1 pass/24.71초, 최종 동일 source hash 재실행 1 pass/23.74초다. 최종 JSON elapsed 29.85초는 Cargo 준비와 버전 수집도 포함한다. 초기 시험 코드의 Debug/PartialEq 없는 API 사용 컴파일 오류를 신규 시험 안에서 수정했다. 이후 clippy 경고 0, 기본 실행은 0 pass/1 ignored로 외부 도구를 자동 실행하지 않는다. 전체 workspace 통합 검증은 주 담당이 수행한다. Git은 정상 저장소가 아니므로 기존 .git을 보존했다.

## 미검증

서버가 재현한 HTTP 403/짧은 응답은 실제 사이트의 서명 만료·로그인 복구, Wi-Fi/인터페이스 단절, HLS 조각 재개를 검증한 것은 아니다. YouTube/치지직·8시간 이상·Windows/macOS·실제 ENOSPC는 별도 R04 게이트다. 디스크를 채우거나 /dev/shm·관리자 mount·기존 사용자 데이터를 변경하지 않았다. 기존 Python 3.10에 대한 yt-dlp deprecation warning은 실제 로그에 보존하며 시험 통과를 도구 환경 지원 완료로 확대하지 않는다.

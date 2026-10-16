# 공개 YouTube 구간 저장의 실제 검증

고정 공개 URL `https://youtu.be/lxOLdQXE2yw`의 1~4초를 production DownloadRepository/DownloadSession으로 저장했다. 별도 읽기 검증에서 **55,277B / 3.008초 / 256×144 VP9+Opus**와 전체 파일 decode exit0을 확인했다. 새 다운로드를 반복하지 않았고 사용자 계정·쿠키·업로드는 사용하지 않았다.

[원 다운로드](evidence/p0-public-bounded-cut-v2-20261015/root-review.json)는 엔진 Exited0/Downloaded revision4·Attempt1·Artifact1 이후 fixture 자체 경로 검증에서 실패했다. 원 artifact의 canonical extended Windows 경로가 원 output 문자열과 starts_with가 달랐다. [기존 파일 읽기 검증](evidence/p0-public-cut-readonly-review-v1-20261015/root-review.json)은 원 plain 검사는 통과, 문자열 비교는 실패, 열린 native root/file HANDLE containment는 성공임을 실제 재현했다. 원 실패를 삭제하거나 성공으로 바꾸지 않았다.

원 typed 구간과 저장 옵션, 도구 버전, 실제 DB 기록을 Windows production open_readonly로 대조했다. DB에 WAL/SHM/journal이 없음을 확인했으며 SQLite immutable로 원자료를 생략하지 않았다. pin된 ffprobe/FFmpeg를 file·pipe 프로토콜에 한정해 길이/144p와 로컬 decode를 확인했고 원34pins·source674·3PE·DB/파일/4tool을 Root가 독립 대조했다.

원 실행은 90초 협조 취소와 200ms 표본의 output32MiB/여유1.5GiB soft 감시를 사용했다. 표본취소 조건은 발생하지 않았고 이것은 hard quota/전체 네트워크 총량 상한이 아니다. 도구 버전은 yt-dlp2026.08.19, FFmpeg n9.0.2-17-g2a571b6068-20260930, Deno2.9.7이었다.

원 다운로드 당시 artifact SHA/native ID는 fixture 실패 전 기록되지 않았다. 이번 읽기 검증이 처음 고정한 파일이 검증 전후 동일함은 확인했지만 원 시점의 identity까지 소급 주장하지 않는다. GUI·관리 도구 활성화·실제 취소/재개·치지직 구간 저장·8시간·clean OS·전체 P0 인수는 별개다. [작업 기록](../work-items/S04-public-bounded-cut-20261015.md).

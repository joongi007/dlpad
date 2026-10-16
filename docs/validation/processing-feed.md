# R01 FFmpeg 처리 배속 feed와 실제 stderr 전달

## 연결 계약

ProgressFeed의 기존 snapshot() 다운로드 B/s/ETA는 유지한다. 별도 processing_snapshot()은 output_time_us, speed(미디어 배속), 조건부 eta_seconds, ended를 최신 한 개만 제공한다. set_processing_duration(Option<u64>)는 알려진 출력 길이 microseconds만 받으며 변경 시 이전 parser/processing evidence를 비운다. 실제 source/range duration과 argv/worker/UI 연결은 통합 담당 범위다. observer는 stderr 전용이며 stdout과 parser를 공유하지 않는다.

명확한 `frame=<nonnegative integer>`로 영상 machine record를 시작하고 기존 bounded FFmpeg parser가 `progress=continue|end`에서 끝난 record만 발행한다. 사람용 `speed=` 또는 stats 전체 줄은 standalone processing evidence가 아니다. max line64KiB, record64lines, latest1, 3초 이후unknown이다. invalid/oversized record는 이전 processing sample을 무효화한다. end 이후 새 frame은 새로운 프로세스 parser를 시작하며 end는 artifact 완료 증거가 아니다. 오디오-only 출력은 frame key가 없으므로 이번 stderr framing으로 미측정이다.

[FFmpeg 공식 문서](https://ffmpeg.org/ffmpeg.html#toc-Main-options)는 -progress의 key=value record와 progress 종료키를 정의한다. 속도는 네트워크 B/s와 다르다. 알려진 출력 duration·positive speed·유효 output timestamp가 함께 있을 때만 ETA를 계산하며 unknown/0speed/end/지나간duration에서는 표시하지 않는다.

## 실제 합성 실험

Linux/WSL2, 외부 FFmpeg4.4.2-0ubuntu0.22.04.1, yt-dlp2026.09.27.232945. 신규 script는 stdlib argv 실행·30초 deadline·각stdout/stderr256KiB 상한·소유 child wait와 127.0.0.1 임시서버만 사용한다. 6초160×90색상영상 및 sine 오디오는 새 /tmp 폴더에 생성하며 사용자 영상/Documents/GUI/외부접속을 사용하지 않는다. 별도 OS/장시간FFmpeg 비용은 미검증이다.

```sh
python3 scripts/ffmpeg_progress_probe.py /tmp/dlpad-processing-probe-20261001-downloader --ytdlp <existing-yt-dlp>
```

초기 --postprocessor-args `ffmpeg:-progress pipe:2 -nostats`만으로 fast/accurate/merge 모두 machine record0이었다. FFmpeg downloader는 postprocessor와 별도 argv 경로다. 추가 reserved --external-downloader-args `ffmpeg:-progress pipe:2 -nostats`로 fast/accurate에서는 아래 실제 stderr가 전달되었다. 제품에 raw postprocessor argv 입력을 추가하지 않는다.

| 합성 경로 | exit | stderr frame/continue/end | stdout machine | 최종 out_time_us | 최종 배속 |
|---|---|---|---|---|---|
| direct -progress pipe:2 null sink |0|2/1/1|0|6000000|581x|
| yt-dlp fast section1..4 |0|2/1/1|0|2900098|21600x|
| yt-dlp accurate section1..4 |0|2/1/1|0|2700098|133x|
| yt-dlp separate video/audio merge |0|0/0/0|0|unknown|unknown|

빠른 자르기 첫 timestamp는 -1199902/speedN/A였다. parser가 InvalidNumber/unknown으로 버린 뒤 최종 positive record를 수용했다. 정확 자르기 첫 sample0/speed0은 measured zero지만 ETAunknown이다. Packet timestamp가 요청 길이3초와 일치한다는 가정을 하지 않는다. 위 합성 배속은 제품 성능 예산으로 해석하지 않는다.

일반 merge의 FFmpegPostProcessor.real_run_ffmpeg는 설치된 정확한 yt-dlp 소스에서 Popen.run stdout/stderr PIPE를 사용해 종료까지 수집하고 성공 stderr를 부모로 내보내지 않았다. [공식 upstream 구현](https://raw.githubusercontent.com/yt-dlp/yt-dlp/master/yt_dlp/postprocessor/ffmpeg.py)도 이 경로를 확인하는 참고자료이며 exact installed source 관찰과 구분한다. reserved pipe:2 argv만으로 일반 postprocessing live 배속을 제공할 수 없다. FIFO/별도 소유 progress sink 등 추가 전달 경계의 설계·실증은 후속이며 **R01 전체 완료가 아니다**. 관측 없는 merge에서 unknown을 유지해야 한다.

## 실제 검증 명령과 결과

- `cargo test -p dlpad-engine --lib metrics::progress_feed_tests --locked -- --nocapture`: 초기3회귀 통과.
- `DLPAD_TEST_PROCESSING_LOG=<owned fast.stderr> cargo test -p dlpad-engine --lib metrics::progress_feed_tests::actual_ffmpeg_stderr_records_reach_processing_feed --locked -- --ignored --nocapture`:1 passed, 실제2900098us/21600x/end; 다운로드1000B/s 유지.
- 동일명령 accurate.stderr:1 passed, 실제2700098us/133x/end; 다운로드1000B/s 유지.
- direct-stderr.stderr와 --include-ignored로 전체4feed회귀 passed(0.01s), non-final known-duration ETA1회 실제관측, final6000000us/581x/end/ETAunknown.
- Python syntax compile passed with task /tmp pycache.

파서는 네이티브 프로세스 소유권·상태 전이를 바꾸지 않는다. 상태 PostProcessing와 숫자 관측은 동일하지 않으며 raw 기록·100%/end만으로 완료를 표시하지 않는다. metrics memory 비용은 독립 mutex와 한 최대64KiB line/parser/latest sample; dependency/schema 추가없음.

기존 guardian8h session57344와 DownloadRepository8h session76382를 보존했다. 이번 관측 guardian7512.85초/download1278.01초 모두running, 기록된9개PID start가 host/proc와 일치했다. 아직8시간 결과가 아니다.

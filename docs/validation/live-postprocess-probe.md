# R01 일반 후처리 전용 progress sink 실증

## 결과와 실제 범위

Linux/WSL2, FFmpeg4.4.2, yt-dlp2026.09.27.232945, 기존 development native guardian helper로 실제 synthetic merge/audio 변환을 실행했다. 일반 PP가 stdout/stderr를 내부 수집해도 **owned FIFO의 별도 machine stream은 live로 관측된다**. 이 문서는 production 구현 완료·Windows/macOS 동작·실제 사이트/영상/UI를 증명하지 않는다. metrics/기존 product/Cargo 파일을 수정하지 않았다.

```sh
python3 scripts/ffmpeg_live_proxy_probe.py /tmp/dlpad-live-postprocess-20261002-final --ytdlp <existing-yt-dlp> --guardian target/debug/dlpad-process-guard
```

실제 exit0, 모든 assertions 통과. 소유 /tmp 폴더0700/FIFO0600, 6초색상영상/sine audio, argv-only yt-dlp와 native guardian, 127.0.0.1 합성 파일 transport만 사용했다. 웹 UI/외부접속/사용자 Documents/실제 미디어 없음.

| 경로 | 경과 | 첫 machine record | record수 | 최종 값 | 결과 |
|---|---|---|---|---|---|
| merge |6.113초|0.260초|13|5888000us/1.03x/end|live 성공, exit0|
| merge owner-loss 취소 |1.411초|0.279초|3|1152000us/1.13x/end=false|ownerEOF, exit-9|
| audio-only MP3 변환 |6.739초|0.562초|13|5981875us/.994x/end|live 성공, exit0|
| invalid progress sink |.418초|없음|0|unknown|exit1, PP global-option 오류|

각 경로의 named owned descendants가 종료·회수되었고 FIFO도 제거되었다. 무관한 owned sentinel은 시험 종료까지 살아 있었다. 취소는 첫 record1초 이후 실제 guardian anonymous control socket EOF를 전달했다. 이전 guardian SIGKILL 한계를 해결했다는 의미는 아니다.

## 교정한 실험과 비용

첫 실험 generic `ffmpeg_i1:-re`는 실제 PP input root key와 일치하지 않아 실행이 .6초 이내 끝났다. FIFO2records 수신은 확인했지만 live/cancel=false여서 성공으로 쓰지 않았다. 정확한 `Merger+ffmpeg_i1:-re`, `ExtractAudio+ffmpeg_i1:-re`로 합성 input pacing을 붙인 뒤 위6초live/cancel을 실증했다. -re는 시험을 느리게 하는 fixture 옵션이며 제품옵션 제안이 아니다.

제품후보 reserved PP value는 `ffmpeg:-progress <owned-endpoint> -nostats`다. 실제시험에서는 이 endpoint만 FIFO로 치환했다. 포워딩wrapper/proxy/new dependency가 필요하지 않았다. script가 전체 progress를 파일에 누적하지 않는다: line+record 총64KiB/64lines, latest sample1, human stdout/stderr 각256KiB 누적허용량 및 tail1024chars, namedPIDFD32개 상한,20초timeout, activeFIFOreader가독립drain한다. FIFO에 O_RDWR hold를 두므로 startupwriteropen은 blocking하지 않는다; process exit 뒤 pending bytes를 배출한 다음 EOF를 무한대기하지 않는다. cleanup은 guardianEOF와 namedPIDFD/namedwait만 사용하며 /proc 전체scan/무관PIDkill이없다.

중간관측 development guardian약2.2MiB RSS, yt-dlp약50MiB, FFmpeg약46~48MiB. debug helper파일은 약35MiB이나 product에는 추가helper를 번들한 것이 아니다. 합성물은 작고 duration/resolution이 고정이며 처리 배속·RSS를 일반 사용자성능예산으로 해석하지 않는다.

## 오디오-only 관측 계약

오디오 record는 frame/fps가 없고 bitrate,total_size,out_time_us,out_time_ms,out_time,dup_frames,drop_frames,speed,progress로 시작했다. 따라서 human stderr의 strict frame-start parser에 전달하면 관측되지 않는다. **전용 machine sink**에 한해서 기존 FfmpegProgressParser를 직접 쓰고 progress 경계로 검증한다. stderr frame gate를 완화해 사람용 speed를 관측으로 바꾸지 않는다. microseconds/positive speed/known requested output duration에서만 ETA를 계산하고 N/A,negative timestamp,0speed,unknownduration,end는 unknown을 유지한다. end는 artifact 완료 증거가 아니다.

## Production 설계 제안과 platform gate

우선안은 추가 proxy나 postprocess분리 없이 per-job **owned machine-progress endpoint** adapter다. endpoint + bounded reader + cancellation/RAII cleanup을 좁은 interface로 두고 typed reserved argv를 생성한다. 기존 stderr observer는 downloader영상용 compatibility 경로이며 dedicated sink는 merge/audio를 담당한다. 두 관측을 한 parser에 동시에 섞지 않는다. 여러 PP가 순차 실행될 때 end 뒤 새 parser epoch를 시작하고, 동시 writer 가능성은 별도검증/unknown 처리해야 한다.

- Linux: 이번 FIFO 증거로 구현후보 채택 가능. 부모가 reader를 먼저 열고 private endpoint를 유지하며, job timeout/cancel/guardian fault 뒤 bounded drain을 중단·close·unlink한다. cache cleanup은 해당 job의 소유 endpoint만 대상으로 한다.
- macOS: FIFO primitive는 후보이나 실제 FFmpeg open/write/cancel/crash 시험 전 동작확정 금지. Linux proc/subreaper 소유증거를 그대로사용하지 않는다.
- Windows: user-local named pipe adapter와 FFmpeg -progress endpoint 호환을 먼저 실제 Windows fixture에서 시험해야 한다. user ACL, remote clients 거절, cancellation-aware reads, namedpipe lifetime을 구현조건으로 둔다. 이 실험은 Windows namedpipe 성공을 주장하지 않는다.

namedpipe가 FFmpeg에서 작동하지 않으면 native helper/proxy가 별도 bounded IPC transport로 relay하는 대안을 비교한다. 그러나 proxy만으로 yt-dlp가 먹는 stderr가 부모에게 전달되지는 않으므로 독립 transport는 여전히 필요하다. yt-dlp와 FFmpeg를 product에서 따로 실행하는 대안은 formatselection/remux/subtitles/partial/recovery/publication 정책까지 확대하므로 이 좁은R01 수정의 우선안으로 권하지 않는다. 전체 progress 파일 누적 또는 periodic truncate는 FFmpeg writer offset과 sparse growth/record race 문제로 채택하지 않는다. 실제platform계약 검증 뒤 공유ADR 결정은 주담당이 조정한다.

[FFmpeg 공식 progress 설명](https://ffmpeg.org/ffmpeg.html#toc-Main-options)과 [yt-dlp 공식 PP implementation](https://raw.githubusercontent.com/yt-dlp/yt-dlp/master/yt_dlp/postprocessor/ffmpeg.py)을 참고했다. exactinstalled 코드·실행 결과는 최신upstream문서와 구분한다. Python syntax compile 통과. production dependency/schema/기존source 변경없음.

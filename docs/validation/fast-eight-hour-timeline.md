# 8시간 미디어 타임라인의 빠른 검증

8시간을 실제로 기다리지 않고도 긴 미디어 길이의 처리를 시험한다. 새 opt-in `long_duration_media` 시험은 FFmpeg가 만든160×90 색상/분당1frame 합성 MP4를 loopback에서 실제 yt-dlp/Rust engine으로 내려받는다. 실제 영상·Documents·사이트·계정은 사용하지 않았다.

[실제 결과](evidence/fast-eight-hour-timeline/result.json): 전체28800초(8h),480frame,29,814B의 원본과 다운로드 bytes 동일. 끝부분28680–28800초 정확 자르기 결과120초/decode성공. 전체/구간 Downloaded 및DB재열기/recover0. 실행22.90초/1PASS/0failed. 최초 compile Arc 타입 실패와 수정 후 성공 raw를 함께 보존했다.

재현:

```sh
DLPAD_SMOKE_YTDLP=<existing-absolute-yt-dlp> \
DLPAD_SMOKE_FFMPEG=<existing-absolute-ffmpeg> \
DLPAD_SMOKE_FFPROBE=<existing-absolute-ffprobe> \
cargo test -p dlpad-engine --test long_duration_media --locked -- --ignored --nocapture
```

영상 길이·전송 바이트·실행 경과 시간은 서로 다른 시험 축이다. 이 시험은 큰 시간값과 끝부분 seek/cut/완료등록에 대한 빠른 증거다. 낮은frame rate·단색압축으로 크기를 줄였으므로 일반8시간영상의codec/고bitrate·4GiB·네트워크 누수나장시간resource/sleep검증을대표하지않는다.

시간 전이를 직접 지정하는 core unit와 짧은 fault fixture는 네트워크 단절·재개·타임아웃을 매개변수로 검사한다. [기존 실제8시간 합성 시험](../work-items/S03-soak-recovery-run.md)은 독립적으로 유지하며8hterminal전PASS로승격하지않는다. [작업 계약](../work-items/S03-long-duration-fast-fixture.md)을 따른다.

## 사용자 서비스 backend 후속 실제 도구 시험

a67d7224소스통합검증의defaulttest바이너리와helper를Linux/tmp에고정해`DLPAD_GUARDIAN_BACKEND=user-systemd`로실제yt-dlp·FFmpeg를실행했다. [명령/exit](evidence/fast-eight-hour-user-service/result.json), [미디어증거](evidence/fast-eight-hour-user-service/media-result.json), [바이너리SHA](evidence/fast-eight-hour-user-service/snapshot.json). 실행4.616초/1PASS이며8h원본28800초·480frame·29814B,끝2분120초decode·완료등록/DB재열기를확인했다. 당시호스트/캐시가다르므로이전22.9초와성능개선율로비교하지않는다. 실제8hwallclock·일반bitrate·다른OS·후속startupwait소스검증의증거는아니다.

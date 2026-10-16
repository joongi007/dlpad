# S03: 8시간 길이 합성 미디어의 빠른 검증

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R05, R08
- 담당/수정 범위: root; crates/engine/tests/long_duration_media.rs·전용 증거/검증 문서 및 PERFORMANCE
- 선행 작업: 기존 실제 yt-dlp/FFmpeg 합성 media 통합

## 사용자 결과

매 개발 반복에8시간을기다리지않고8시간미디어타임라인의길이·완료등록·끝부분구간·복구를시험한다.

## 계약과 범위

160×90/분당1프레임의색상합성 fixture만사용한다. 실제영상/사용자Documents/사이트·계정은사용하지않는다. localhost ephemeral 서버와기존명시yt-dlp/FFmpeg/ffprobe, 실제Rust엔진을사용한다. 파일상한16MiB·도구timeout·소유cleanup·DB재열기·실제ffprobe duration과decode를검증한다. 낮은프레임률은일반영상codec/고bitrate/4GiB 비용을대표하지않는다.

## 인수 조건

실제duration28800초(8h)/480frame 증명, 실제도구download와byte동일·Downloaded/DB복구0, 끝부분구간저장·decode. 빠른시험elapsed와미디어duration을구분한다. 시간기반leak/sleep/realtime8h soak를대체했다고주장하지않는다.

## 검증과 handoff

사용자가8시간시험대안질문을주었고독립빠른경로를추가한다. 기존real-time8h프로세스는그대로유지한다. opt-inignored시험으로기본CI에외부도구필수의존을추가하지않는다.

## 실제 결과

`DLPAD_SMOKE_YTDLP=<existing-yt-dlp> DLPAD_SMOKE_FFMPEG=/usr/bin/ffmpeg DLPAD_SMOKE_FFPROBE=/usr/bin/ffprobe cargo test -p dlpad-engine --test long_duration_media --locked -- --ignored --nocapture`:1PASS/0failed, 실제시험22.90초. ffprobe원본28800초/480frame/29,814B, 끝부분120초정확cut/decode/전체downloadbytes동일/Downloaded와DB재열기recover0. 최초compile실패(Arc cancellation타입)와수정후성공log모두보존했다. [결과](../validation/evidence/fast-eight-hour-timeline/result.json). actualwallclock8h·고bitrate/4GiB·사이트/OS 검증과구분한다.

## 사용자 서비스 보호 경로 후속

root가a67d7224통합소스의defaulttest빌드에기록된long_duration_media-fc6de4ed2a0c6881와nativehelper를Linux/tmp에복사해고정하고DLPAD_GUARDIAN_BACKEND=user-systemd에서실제yt-dlp/FFmpeg전체·끝2분cut/DB재열기시험을추가실행한다. agent의process.rs후속수정/Cargo와실행파일을분리한다. 8시간wallclock/새startupwait소스검증으로확대하지않는다.

사용자서비스실제도구후속1PASS/exit0/4.616초완료. root가28800초/480frame/29814B/끝120초decode/DB재열기를rawJSON에서대조했다. 실제8hwallclock은계속별도미검증.

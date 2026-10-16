# 근거와 갱신

확인일: 2026-09-30. 외부 기능·릴리스·API 정책은 변할 수 있으므로 구현·릴리스 시 다시 검증한다. 문서의 성능 예산과 아키텍처는 dlpad의 제안이며 아래 자료의 성능 보장이 아니다.

| 자료 | 사용 범위 |
|---|---|
| [yt-dlp README](https://github.com/yt-dlp/yt-dlp) | standalone 배포, config, structured progress, 외부 도구·옵션 |
| [yt-dlp EJS](https://github.com/yt-dlp/yt-dlp/wiki/EJS) | YouTube JS runtime·solver 구성 |
| [지원 extractor 목록](https://github.com/yt-dlp/yt-dlp/blob/master/supportedsites.md) | 치지직 포함 지원 범위, 실제 동작은 별도 검증 |
| [치지직 오류 #17063](https://github.com/yt-dlp/yt-dlp/issues/17063), [MPD 오류 #8269](https://github.com/yt-dlp/yt-dlp/issues/8269) | R13 사례: 중복 종료와 해결 배포를 구별. 상세 확인 범위는 DIAGNOSTICS.md |
| [FFmpeg 문서](https://ffmpeg.org/ffmpeg.html) | seek·stream copy·progress·후처리 |
| [FFmpeg 다운로드](https://ffmpeg.org/download.html) | 소스와 플랫폼 배포 공급처 구별 |
| [Iced 0.14 feature 목록](https://docs.rs/crate/iced/0.14.0/features) | tiny-skia/wgpu 선택 가능성, 선택 버전은 S00에서 고정 |
| [YouTube videos.insert](https://developers.google.com/youtube/v3/docs/videos/insert) | 업로드 privacy·검증 제한 |
| [YouTube resumable uploads](https://developers.google.com/youtube/v3/guides/using_resumable_upload_protocol) | offset 조회와 재개 |
| [Desktop OAuth](https://developers.google.com/identity/protocols/oauth2/native-app) | 시스템 브라우저 기반 인증 |
| [GitHub Actions workflow syntax](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax) | CI events·matrix·permissions |
| [checkout](https://github.com/actions/checkout), [upload-artifact](https://github.com/actions/upload-artifact) | CI checkout·artifact 보존 |

디자인 근거는 DESIGN.md의 gtmux 로컬 소스 경로와 토큰 snapshot으로 보존했다. 다른 개발자가 해당 절대 경로를 가지고 있을 필요는 없다. 실제 스크린샷·시각 비교는 S00 작업으로 남아 있다.

2026-10-15 진행률 파서 검수: [공식 fragment downloader](https://github.com/yt-dlp/yt-dlp/blob/master/yt_dlp/downloader/fragment.py)와 [공식 ProgressCalculator](https://github.com/yt-dlp/yt-dlp/blob/master/yt_dlp/utils/progress.py)에서 조각 수로 나눈 예상 용량이 소수로 유지됨을 확인했다. 이 근거로 dlpad의 확정 크기 u64 계약을 유지하면서 예상 크기만 bounded f64로 수정한다. [작업과 실제 검증](work-items/S01-fractional-transfer-estimate-20261015.md).

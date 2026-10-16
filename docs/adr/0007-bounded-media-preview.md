# 0007: 내용 중심 미디어 작업면과 bounded 미리보기

- 상태: Accepted — 개발 후보의 scoped 구현; 전체 UI/릴리스 인수와 별도
- 날짜: 2026-10-07
- 요구사항: R01, R04, R05, R08, R11

## 문제와 결정

URL 문법 확인만으로 영상 준비 성공을 표시하지 않는다. 작업 대상을 미리보기·하나의 저장 구간 band·독립 playhead로 표시하고 넓은 화면에서는 저장 설정을 옆에, 좁은 화면에서는 아래에 배치한다. GTMUX 원 Button/Input/Toolbar/Settings/FileTree의 정보 위계와 독립 hover/pressed/selected/focus를 역할 기준으로 적용한다. 표면 색상이나 단위 테스트 통과는 사용자 UX 인수가 아니다.

core는 1ms 이상 7일 이하 duration에서 start < end <= duration을 유지한다. 재생 위치는 duration-1 이하이고 구간 변경과 독립이다. 수치 입력 오류는 마지막 유효 구간을 보존하고 저장을 차단한다. 실제 원본 frame PTS를 취득하지 않으므로 표시 시간은 요청한 nominal seek이며 frame-accurate 편집을 주장하지 않는다.

새 crate 없이 기존 yt-dlp/ffprobe/FFmpeg와 owned process tree를 argv로 사용한다. UI thread에서 도구 실행·긴 I/O를 하지 않는다. 최신 요청 generation과 cap1 job/completion channel, 하나의 active 요청과 최신 pending 요청으로 제한한다. 오래된 결과는 폐기하되 cleanup ownership 실패는 generation과 무관하게 보존한다. CloseWait는 thread ACK까지 기다리며 ACK 자체를 OS process cleanup 성공으로 간주하지 않는다.

미리보기는 320×180 RGB, 6fps, 최대 2초/12프레임 batch다. 무음이며 전체 영상 stream을 메모리에 적재하지 않는다. metadata stdout 8MiB, 로그 16KiB와 절대 deadline을 적용한다. 원격 전체 수신 bytes/RSS hard cap은 확보하지 못했으므로 output cap을 해당 보장으로 표현하지 않는다. 일반 URL은 하나의 직접 영상 소스인 비 live 영상을 준비하며 video-only DASH도 허용한다. 무음 미리보기는 별도 오디오를 합치지 않고 인증·cookie를 전달하지 않는다. 명시적으로 미리보기 없이 다운로드를 선택하면 기존 다운로드 기능을 사용할 수 있다.

## 실험으로 수정한 결정

실제 FFmpeg의 2초/6fps 합성 영상에서 1.999초 seek는 성공 exit에도 RGB를 반환하지 않았다. noaccurate_seek/fps 옵션도 해결하지 못했다. 따라서 단일 seek는 요청→167ms 이전→1초 이전을 같은 절대 deadline 안에서 순차 시도한다. 반환된 시도 위치를 playhead에 반영하고 구간은 보존한다. 끝 부분 batch의 자연스러운 빈 출력은 Finished로 재생을 정지한다. 그 외 빈/불완전 RGB는 명시 오류다. 모든 프레임률의 frame-accurate seek를 보장하지 않는다.

## 현재 검증과 남은 범위

Windows whole fmt/clippy/test/release와 합성 원본의 metadata·red/blue seek·EOF seek·batch·취소·deadline·원본 보존을 실행했다. Candidate의 RawInput hover/down/up/Tab/drop·실제 재생 시간과 색 변화·CloseWait를 19 PNG로 보존했다. [작업 기록](../work-items/S00-media-workspace-interaction.md).

로컬 drop은 읽기·내용 확인·구간 선택이며 export는 후속이다. 긴 영상의 정밀 확대, 목록 행에서 미리보기로 연결, 모든 원격 사이트·전체 영상 호환, 오디오, physical IME/reader/OS DPI, 사용자 최종 평가·전체 P0는 남는다. native HWND layout/비용/패키지는 현재 작업에서 별도로 인수한다.

2026-10-08 실제 공개 YouTube 형식 목록은 muxed-only 선택 실패를 보였다. 선택자를 `bestvideo*[height<=360]/worstvideo*`로 수정하고 사용자가 지정한 Deno 경로를 Prepare argv와 원 파일 pin에 포함했다. 출력은320×180 그대로이며 fallback 입력의 원 해상도까지360이하라고 보장하지 않는다. worker의 도구 경로 snapshot은 유지하고, 새 준비 요청에서 경로가 바뀌면 이전 worker의 stop ACK 후 교체한다. 준비된 영상이 도구 경로 변경만으로 자동 재준비되지는 않는다. 치지직·YouTube의 공개 metadata/짧은 무음 preview·일시정지는 실제 private GUI에서 별도로 확인했다. 전체 영상 저장·오디오·실제 인증 시험은 포함하지 않는다. [실제 시험과 실패 기록](../work-items/S00-public-video-ui-walkthrough.md).

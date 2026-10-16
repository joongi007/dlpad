# S05: 실제 Windows FFmpeg 합성 구간·decode

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R05
- 담당/수정 범위: root는 이 문서·공유 인터페이스·후속 통합 및 실제 검증, metrics_recovery는 /tmp/windows_ffmpeg_p0_next.md 읽기 감사 초안만 담당한다. 제품 source/Cargo/lock은 root만 수정한다.
- 선행 작업: 현재 Windows CPU 전체검증과 격리native observer/원ownedfixture 계약. 현재동시실행검증의source는동결한다.

## 범위와 인수

기존project-owned 실제FFmpeg의source/version/license와고정합성1초media에서fast/exactcut·decode·실패/취소·외부소유권보존을구분해검증한다. 실제사용자영상/계정/다운로드·외부게시를사용하지않고source별실증은미완료로남긴다.

읽기감사후구체적인최소단위로구현/실행하고root가원로그·파일SHA·프로세스정리를독립검증한다. 아직actual후속인수완료가아니다.

2026-10-05 명시 재개: metrics_recovery 담당은 새 Windows DownloadRepository fixture를 /tmp/dlpad-real-ffmpeg-*에 구현한다. root가 검토 후 프로젝트 통합·원native실행·최종검증을 맡는다. 공유Rust/Cargo/lock/nativeprobe 실행은 root 승인 전 금지한다. 독립license감사는 soak_recovery_audit가 /tmp/dlpad-license-next-*만 담당한다.

2026-10-05 root통합범위는새 crates/engine/examples/windows_real_ffmpeg_cut_fixture.rs와root새nativecollector다. metrics_recovery의/tmpfixture초안만반영하며기존공유API/Cargoengine의존/schema변경없다. stdowned127.0.0.1단일합성media서버/request2초절대기한·128request/8KiBheader/16KiBchunk, 전체150초취소watchdog·outerfiniteJobhost로제한한다. 공식yt-dlp2026.08.19 17,840,399B/SHA66674953fe251b89f4d08c5f0e35e0728679bd67ab3d7d05c0562af101dd3e7a를freshprojectroot로다운로드후기존서명checksum과대조했다. publicRepository fast/exact→ffprobe/decode/DBreopen과missingexternalFFmpeg 실패, actualFFmpegProcessing후ProcessSpeccancel/timeout을별도구분한다. Repository실제cut중cancel/timeout·관리receipt/site는이좁은인수와별도이며미완료다. 원source/tool/license/sentinel은삭제하지않고SHA/nativeidentity를전후검증한다.

추가 인수: 단색 duration 시험을 시간 구간 정확성으로 확대 해석하지 않는다. 고정 red/green/blue/yellow 4구간 합성 source의 원 색상과 fast/exact 500..1500ms 및 정확한 off-GOP350..1350ms 결과의 첫/후반 색을 실제 FFmpeg decode한 RGB3B로 대조한다. 원 fixture 파일 create_new/native ID 유지와 bounded 출력, 원 취소 시험의 무한 입력을 보존한다. diagnostic_cache는 /tmp 초안만 담당했고 root가 diff와 기존 transport tests 보존을 검토해 통합했다.

재현 collector scripts/windows_real_ffmpeg_private_probe.py와 원 경로·manifest·실제파일증가 실패 회귀 scripts/test_windows_real_ffmpeg_private_probe.py를 root가 검토·게시했다. 추가 docs 비용 외 runtime/Cargo 변경 없음. 명령은 python3 scripts/windows_real_ffmpeg_private_probe.py <fresh absolute project evidence> --tools-manifest <original absolute manifest>이며 기존 native host 사용과 originalsource/toolSHA 전후 검증을 유지한다. /tmp 초안의 parent traversal와 해시 read 상한 누락을 root검토에서 발견해 교정 후 게시했다.

actual temporal3case와게시collector재실행이모두PASS다. root가원SQLite integrity/Downloaded3개·Failedmissingtool0artifact, 원source/artifact/toolsSHA와각3B RGB를직접대조했다. firstgreen/lastblue(500..1500)와firstred/lastblue(350..1350), decode/ffprobeexit0/duration1.0를확인했다. 증거 windows-real-ffmpeg-temporal-cut-20261005-resume/root-review.json 및게시collector결과 windows-real-ffmpeg-published-collector-20261005-resume/result.json. parenthost가명시Job0를기록하지않는기존한계를별도S00원Jobterminal작업으로검증한다. 원nativeIDs는producer원보고이며root독립Win32재조회/실제사이트·오디오·관리tool공급인수는미완료다.

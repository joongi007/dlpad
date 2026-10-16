# S04: FFmpeg enabled dependency notice/source 대응

- 상태: completed (enabled mapping·notice 수집; 공급 채택 미완료)
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R11
- 담당/수정 범위: soak_recovery_audit — 이 문서와 새 전용 공급 evidence directory
- 선행 작업: 실제 buildconf, 고정 BtbN source137recipes와125declarations

## 계약과 인수

실제 enabled/disabled configure flags를 고정 recipe에 연결하고 official pinned upstream notice를 bounded create-only로 확보한다. 실제 당시 dependency payload provenance와 recipe 선언은 구분한다. 기존 파일/Rust/Cargo를 변경하거나 빌드·공개 배포하지 않는다. 불확정 대응은 그대로 남긴다.

## 실제 대응과 확보

새 `.local-tools/ffmpeg-enabled-notices-8ehqjqje/`에 [전체 mapping](../../.local-tools/ffmpeg-enabled-notices-8ehqjqje/mapping.json)을 보관했다. 기존 실제 `ffmpeg-version.txt`의 configure 문자열에서 enable flags64개/disable15개를 추출했다. 63개는 고정 BtbN source의 literal enable flag 또는 script basename에 연결된다. 남은 version3는 외부 library가 아닌 FFmpeg 자체 license policy이다. API/hardware flags도 포함돼 64개를 독립 외부 library64개라고 부르지 않는다. 여러 recipe 후보는 모두 보존하며 실제 image 종속성 실행 순서를 추측하지 않는다.

비활성은 debug/avisynth/libdavs2/libdvdread/libdvdnav/libfdk-aac/frei0r/librubberband/schannel/libvidstab/whisper/libx264/libx265/libxavs2/libxvid이다. 비활성 GPL codec script가 recipe archive에 있다는 이유로 LGPL 실제 binary에 포함됐다고 표현하지 않는다. enabled에는 libopus/libvorbis/librav1e/libdav1d/libaom/libass/openssl/libxml2/libjxl/libplacebo/libvpx/libwebp 및 fonts/hardware 관련 dependencies 등이 있다.

exact40-hex GitHub commit 선언41개에 대해 [notice manifest](../../.local-tools/ffmpeg-enabled-notices-8ehqjqje/notice-manifest.json)에 source archive URL/notice URL/bytes/SHA를 기록했다. 처음35개에서 root license를 찾았고 나머지6개는 [정확한 tree 경로 followup](../../.local-tools/ffmpeg-enabled-notices-8ehqjqje/notice-missing-followup.json)으로 실제 COPYING 계열을 확인했다. nv-codec-headers는 독립 license 파일 대신 [원 header notice](../../.local-tools/ffmpeg-enabled-notices-8ehqjqje/notice-nv-headers.json)를 full source header로 보관했다.

예를 들어 [opus exact commit](https://github.com/xiph/opus/tree/503d81b138d76621aae4b12786e90de48aa8db3a), [nvEncodeAPI source notice](https://raw.githubusercontent.com/FFmpeg/nv-codec-headers/eddcea9e27f6b772057c9b3f87de2cc1737faffc/include/ffnvcodec/nvEncodeAPI.h)는 mutable latest를 사용하지 않는다. nvEncodeAPI313554 bytes/SHA256 `8776fddcb8febc6aec4d73989b1f21831eb30306bc583da55b4bf0c14a1dc228`, dynlink_cuda24212 bytes/SHA256 `c970d5817120ea481ba29b9c603bdc9a386985d9fc98a47f6588030dcdba87c1`이다.

공식 VideoLAN/GitLab/GoogleSource/SourceForge/GNU의 별도18 repository에서 pinned commit notice를 요청했고 [other-host manifest](../../.local-tools/ffmpeg-enabled-notices-8ehqjqje/notice-other-hosts.json)에 결과와 실패 URL을 기록했다. dav1d/fontconfig/libxcb/xlib/pulse/aom/bluray/placebo/rist/ssh/vpx/webp/opencore-amr의 notice를 확보했다. opencore-amr 한 source가 nb/wb 두 enabled flag에 대응하며 중복 다운로드하지 않았다. GoogleSource base64 응답은 decoded notice SHA와 wire SHA를 따로 기록했다.

## 정확히 남은 gap

iconv/soxr/freetype/SVT-AV1/drm의 notice 요청은 접근 실패 또는 후보 경로 미확보로 남았다. lame은 SVN revision6835 선언이며 Git commit 형태로 임의 변환하지 않았다. libiconv의 gnulib 같은 transitive source와 fonts/build system의 간접 dependency notices도 완비하지 않았다. 다른 저장소에서 root LICENSE 한 파일을 얻었다고 bundled third-party notice 전체가 확보된 것은 아니다.

source archive URL을 기록한 GitHub dependencies의 전체 source payload는 이번에 다운로드하지 않았으므로 그 payload SHA가 있다고 주장하지 않는다. 확보한 source notice의 digest만 검증했다. recipe의 pinned commit은 당시 공급 binary에 실제 들어간 payload의 provenance 또는 그 runtime version string을 독립 증명하지 못한다. mapping 모든 항목의 actual_payload_provenance_verified=false를 유지한다. 당시 container digest/cache/source download bytes·transitive graph·dependency-specific source/notice 제공 계약은 여전히 필요하다.

라이선스별 법적 해석·특허/codec 권리·재현 build·3 OS 공급 채택은 이 수집 결과로 완료하지 않는다. 공급은 Proposed 상태이다. 다음 개발은 실패 host의 공식 exact path/source archive 확보, transitive source graph, notice bundle/package 표시 및 변조 거절 인수이다.

## 실행 결과와 비용

읽기 mapping/공식 HTTPS notice 다운로드만 실행했다. 최대8개 동시 요청, 요청10~15초, notice512 KiB/tree4 MiB/header1 MiB 상한과 create-only 파일 쓰기를 사용했다. 실패404/403/timeout 등은 manifest에 보존하고 HTML 응답을 notice로 저장하지 않았다. 원 local binary/source/license 파일을 변경하거나 실행하지 않았고 Rust/Cargo/외부 게시/공개 배포는 없다. 추가 source correspondence와 notice 완비는 미검증으로 남긴다.

## 주 담당 독립 파일 검증

root가 notice-*.json의file/bytes/SHA기록을재귀로읽고현재파일을직접해시했다. notice/header64개와tree보조자료를포함해70파일을대조했으며digest/size불일치0. 이수치는license자료개수와보조JSON자료를합친검증수이며actualbinarypayload또는transitive고지완비증거가아니다.

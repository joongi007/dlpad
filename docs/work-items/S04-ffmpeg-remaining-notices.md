# S04: 잔여 FFmpeg dependency notice

- 상태: completed (여섯 exact notice 확보; 세 source archive·전이자료 잔여)
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R11
- 담당/수정 범위: soak_recovery_audit — 이 문서 및 새 공급 evidence directory
- 선행 작업: enabled dependency mapping 및 root 기존 SHA 검증

## 계약과 인수

iconv/soxr/freetype/SVT-AV1/drm/lame의 고정 revision에서 official notice와 source 경로를 확인한다. mutable branch/다른 release notice로 대체하지 않고 bounded create-only 수집 실패도 보존한다. 실제 binary payload 대응성과 transitive 완비·공급 채택은 범위 밖이다.

## 실제 수집과 exact revision

새 `.local-tools/ffmpeg-remaining-notices-lgsdyldh/`의 [manifest](../../.local-tools/ffmpeg-remaining-notices-lgsdyldh/manifest.json), [followup](../../.local-tools/ffmpeg-remaining-notices-lgsdyldh/followup-manifest.json)에 성공·실패 URL/bytes/SHA를 보존했다. 공식 저장소의 기존 recipe commit/revision만 요청했으며 다른 release로 바꾸지 않았다.

| dependency | exact revision | 실제 확보 |
|---|---|---|
| iconv | 1df3087ba8110c7f3ed3eb5f8869b814dbbe00b0 | COPYING(GPL3), COPYING.LIB(LGPL2.1), libcharset COPYING.LIB |
| soxr | 945b592b70470e29f917f4de89b4281fbbd540c0 | LICENCE와 README |
| FreeType | d333439633039de426f943f28a2926c7f97b5ae5 | root LICENSE.TXT, docs FTL/GPLv2, source archive |
| SVT-AV1 | 5ed9ffbcb5086da23f30fe9bc5b00145358462da | LICENSE.md, PATENTS.md, source archive |
| libdrm | b97cbde15c5c3abfe44d78e8f57139e50f612fec | README.rst, xf86drm.h SPDX notice, source archive 및 LICENSES/MIT.txt |
| lame | SVN6835 | 공식 SVN revision 경로의 COPYING 및 LICENSE |

FreeType·SVT-AV1·libdrm archive의 notice 경로를 읽어 별도 JSON으로 보존했다. 압축 내 임의 경로를 filesystem에 추출하지 않았다. libdrm MIT 전문은 원 고정 archive의 [선택 member 증거](../../.local-tools/ffmpeg-remaining-notices-lgsdyldh/drm-extracted-notice.json)로 원 archive와 연결한다.

| source archive | bytes | SHA256 |
|---|---:|---|
| FreeType | 2574080 | e9d4680f9d751add3571e5407f2943a18ece9c02a78e1bb456aae305254d8bc9 |
| SVT-AV1 | 11156613 | 30137902c7d3f30e04f8b0d8a37a459a9ee26fb8877797418a409897ed3cbe65 |
| libdrm | 570781 | 379d0881f6e7509ba5c0dca6ce045d7e68ffcbb4bed4cee5b598ab843fcb8cb2 |

iconv COPYING.LIB26419 bytes/SHA `20e50fe7aae3e56378ebf0417d9de904f55a0e61e4df315333e632a4d3555d95`는 처음 read timeout 이후 동일 exact URL에서 성공했다. soxr LICENCE1024 bytes/SHA `dc98676341fdcd29d9f279c9679d6a75288785b174ded8d1b2e316c366166135`, lame COPYING25281 bytes/SHA `e64f9c5a18f56828c10a575df13ade641aa3af4512a7afe6c411256943b57aaf`를 실제 확보했다.

## 왜 남아 있는가

원 iconv cgit snapshot 요청은 HTTP400, soxr SourceForge code-snapshot ZIP 요청은 HTTP404라 source archive bytes/hash를 확보하지 못했다. 이것은 해당 source가 없다는 증명이 아니며 공식 git export 경로의 후속 수집이 필요하다. lame은 SVN6835의 개별 notice를 확보했지만 전체 revision export/archive는 아직 요청·검증하지 않았다. 다른 버전 release archive를 대응 source로 채우지 않는다.

FreeType는 처음 잘못된 docs/LICENSE.TXT404였고 실제 root LICENSE.TXT를 확인했다. SVT-AV1은 LICENSE가 아니라 LICENSE.md, libdrm은 COPYING가 아니라 LICENSES/MIT.txt 및 source SPDX였다. 실패 원인은 manifest의 해당 HTTP/timeout 결과대로 기록하고 전부 upstream 접근 불가라고 표현하지 않는다. 공식 [FreeType 개발 source 안내](https://freetype.org/download.html)와 [공식 mirror](https://github.com/freetype/freetype), [SVT canonical repository 안내](https://github.com/AOMediaCodec/SVT-AV1)를 읽었지만 actual 수집은 recipe의 원 GitLab exact commit에서 성공했다.

soxr LICENCE는 pffft.c 사용 시 source 내 추가 license를 읽으라고 명시한다. 실제 variant에 포함된 transitive dependency/embedded source notice, iconv gnulib, FreeType의 bundled/transitive modules 등은 별도 mapping 인수이다. SVT PATENTS 문서를 확보한 사실은 patent clearance 판정이 아니다. 당시 binary payload provenance와 법적 notice/source 제공 완비·전체 공급 채택은 여전히 미검증이다.

## 실행 결과와 handoff

공식 HTTPS bounded read만 최대6개 병렬, 요청15~30초, notice512 KiB/source archive32 MiB 상한으로 실행했다. 새파일만 만들었고 기존 root 검증70자료/Rust/Cargo/source/binary를 변경하지 않았다. archives와 notices는 실행하지 않았으며 외부 게시·공개 릴리스는 없다. 공급 Proposed 유지. 다음 작업은 세 전체 source export와 transitive notice 및 root의 새 SHA 독립 검증이다.

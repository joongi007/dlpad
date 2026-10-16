# S04: 고정 dependency source export 후속

- 상태: completed (수집 결과 보존; Windows 최초 릴리스 변경으로 Linux 후속 보류)
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R11
- 담당/수정 범위: soak_recovery_audit — 이 문서 및 새 .local-tools evidence directory
- 선행 작업: S04-ffmpeg-remaining-notices, 고정 BtbN recipe commits/SVN6835

## 계약과 인수

iconv/soxr/lame 전체 official source를 exact revision으로 export하고 bytes/SHA와 embedded/transitive notice 경로를 기록한다. bounded public Git/SVN 다운로드만 허용하고 별도 임시 config를 사용한다. 기존 workitem/source/Rust/Cargo/validation 파일 및 사용자 자료는 수정하지 않는다. 같은 release의 다른 revision으로 대체하지 않으며 binary correspondence/전이자료 완비와 source export 확보를 구분한다. 불가 경로 rawfail은 새 evidence에 보존한다.

## 실제 확보와 제한

새 `.local-tools/ffmpeg-source-export-npjzddrg/`의 [manifest](../../.local-tools/ffmpeg-source-export-npjzddrg/manifest.json)에 공식 source export를 기록했다. soxr는 공식 Git에서 clone한 뒤 `rev-parse <sha>^{commit}`의 exact equality를 확인하고 archive했다. 원 commit `945b592b70470e29f917f4de89b4281fbbd540c0`, tar675840 bytes/SHA256 `cff7e2b6427b01a5f73b32175153d38a919941052ea0e52f1992d196e46493ce`이다.

lame은 SVN CLI가 없어 official revision6835의 HTTP directory index를 읽었다. 모든 directory의 Revision6835 표시와 entry 안전성을 검증하고 32 directories/435 files/원 내용8071505 bytes를 수집했다. [export 결과](../../.local-tools/ffmpeg-source-export-npjzddrg/lame-export-result.json)와 [435개 file manifest](../../.local-tools/ffmpeg-source-export-npjzddrg/lame-file-manifest.json)를 보존했다. content tar8417280 bytes/SHA `9ea1a8dea6570611aa32fe85f4bee0d6076be58a775aa9e310488521e85f465f`이다. SVN properties/executable/symlink semantics는 export하지 않았으므로 full SVN checkout이나 동일 build 재현 bundle로 표현하지 않는다.

[embedded·전이 manifest](../../.local-tools/ffmpeg-source-export-npjzddrg/embedded-and-transitive-manifest.json)에 soxr COPYING.LGPL/LICENCE와 pffft source header, lame license 파일을 archive member별 SHA로 연결했다. source를 실행하지 않았다. BtbN recipe가 지정한 gnulib mirror의 exact `eb72eb6f75f5621c5d648acd11467fd124584617` source9552759 bytes/SHA `b97aaa8c6d2178387802ea9a7b3d53eb307a930a72d050c699f19867160caa6c`도 확보했다. gnulib COPYING는 파일/모듈마다 다른 license라고 명시하므로 전체 archive 확보만으로 실제 libiconv imported module의 notice 완비를 주장하지 않는다.

iconv 공식 HTTPS Git clone은55초 timeout으로 실패했다. [세 official snapshot 후속](../../.local-tools/ffmpeg-source-export-npjzddrg/iconv-snapshot-followup.json)은 Network unreachable였다. 이전 source archive400과 notice 성공을 보존하며 현재 오류를 source 존재 부재나 정책적 접근 금지로 해석하지 않는다. partial clone을 완성 source/hash로 보고하지 않았고 기존 파일을 삭제하지 않았다. exact iconv 전체 source와 imported gnulib graph는 남았다.

## 실행과 변경된 우선순위

Git는 임시 실행 환경에서 system/global config 및 credential helper/prompt를 사용하지 않았다. clone55초/archive45초, archive64 MiB 상한으로 수행했다. SVN content 수집은180초/128directories/2048files/64 MiB, 개별4 MiB/15초와 최대8개 요청으로 제한했다. 원시 실패·성공 결과는 새파일에만 기록했고 Rust/Cargo/기존 문서·사용자 자료·공개 배포를 변경하지 않았다.

진행 중 요청 종료 후 사용자가 Windows 최초 릴리스로 방향을 변경했으므로 이 Linux 수집 결과를 보존하고 새 Linux 공급 작업은 시작하지 않는다. Windows win64 실제 payload/notice/source inventory는 별도 신규 작업이며 Linux source 대응·runtime 비용·license 검증을 Windows 완비로 전용하지 않는다. 공급 Proposed 유지.

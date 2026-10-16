# S04 공개 공급 자료 검증

관측일: 2026-10-01. [작업 계약](../work-items/S04-supply-evidence.md). 이 기록은 공급 자료 조사와 작은 checksum의 검증이며 자동 설치 또는 3 OS 실행 완료가 아니다. 실제 외부 설치를 수정·삭제하지 않았다.

## yt-dlp: 서명된 checksum과 고정 후보

관측 당시 공식 stable은 [2026.08.19](https://github.com/yt-dlp/yt-dlp/releases/tag/2026.08.19), published_at `2026-08-19T23:48:43Z`다. 이전 합성 다운로드에 사용한 외부 `2026.09.27.232945`와 같은 배포물로 취급하지 않는다. [공식 README](https://github.com/yt-dlp/yt-dlp#release-files)는 Linux standalone glibc 2.17+, Windows x64 Windows 8+, macOS universal 10.15+를 안내한다. PyInstaller standalone은 합성물 GPLv3+이며 소스 Unlicense만 표시하면 부족하다. license 자료는 해당 tag의 [THIRD_PARTY_LICENSES.txt](https://github.com/yt-dlp/yt-dlp/blob/2026.08.19/THIRD_PARTY_LICENSES.txt)를 함께 고정한다. 최소 OS 안내는 dlpad 지원 OS/실행 실증을 뜻하지 않는다.

공식 [SHA2-256SUMS](https://github.com/yt-dlp/yt-dlp/releases/download/2026.08.19/SHA2-256SUMS) 1,595 byte와 [분리 서명](https://github.com/yt-dlp/yt-dlp/releases/download/2026.08.19/SHA2-256SUMS.sig) 566 byte, [tag 고정 public.key](https://raw.githubusercontent.com/yt-dlp/yt-dlp/2026.08.19/public.key) 1,676 byte만 임시 폴더로 내려받았다. SHA256은 각각 아래와 같다.

```text
sums: a63701f30755cb4d9317950d69703e9d751d490cb1b7059e1bb501a353fe7dcb
sig:  8d6f9a7c84c57222e3650c61699185878a3468308de702993ce4f20b5f9cf199
key:  45d6b415928b5f3e228b461fa9e6d7eb56a824931c785ece00279a06c7a6d6e5
```

`/usr/bin/gpg --batch --no-options --homedir <이번 시험의 temp/gnupg>`로 별도 0700 keyring을 사용했다. import 후 `--with-colons --fingerprint`, `--status-fd 1 --verify <sig> <sums>`를 실행했다. 예상 fingerprint `AC0CBBE6848D6A873464AF4E57CF65933B5A7581`의 `VALIDSIG`와 exit 0을 확인했다. 합성 문자열 한 줄을 checksum에 추가한 뒤 같은 서명은 `BADSIG`/exit 1로 거절됐다. 사용자 GPG keyring은 사용하지 않았다. 키와 서명을 같은 공식 origin에서 받았으므로 독립적인 key identity 인증까지 검증한 것은 아니다. trust는 `TRUST_UNDEFINED`였으며 '암호학적으로 일치'와 '독립적으로 인증한 공급자'를 구별한다. trust anchor 변경/키 회전은 검토 후 별도 고정한다.

서명 checksum와 [공식 tag API](https://api.github.com/repos/yt-dlp/yt-dlp/releases/tags/2026.08.19)의 asset digest가 일치했다. 아래는 다운로드 이전 후보 pin이며 **실행 파일 bytes를 다운로드·재해시한 결과는 아니다**.

| Asset | 다운로드 byte | SHA256 |
|---|---:|---|
| yt-dlp_linux | 40,446,224 | 58162f9bfdc27458ea47bfcb311cf47028f17d8154a8bf7d689861d46399230a |
| yt-dlp.exe | 17,840,399 | 66674953fe251b89f4d08c5f0e35e0728679bd67ab3d7d05c0562af101dd3e7a |
| yt-dlp_macos | 37,146,048 | 0f192b7ec147ab6288885d6351d9ab67367640029b4377576ef46dd79cf7b202 |

Linux direct executable의 기존 `InstallManifest` 후보는 다음처럼 표현할 수 있다. 앱이 자동 사용하거나 설치하는 manifest가 아니다.

```json
{
  "tool": "yt-dlp",
  "version": "2026.08.19",
  "target": "x86_64-unknown-linux-gnu",
  "source_url": "https://github.com/yt-dlp/yt-dlp/releases/download/2026.08.19/yt-dlp_linux",
  "sha256": "58162f9bfdc27458ea47bfcb311cf47028f17d8154a8bf7d689861d46399230a",
  "license": "GPL-3.0-or-later; THIRD_PARTY_LICENSES.txt at tag 2026.08.19",
  "ownership": "Managed",
  "executable": "yt-dlp"
}
```

## FFmpeg: 빌드 공급 후보와 license

[FFmpeg 공식 다운로드](https://ffmpeg.org/download.html)는 source를 제공하고 BtbN/Linux·Windows, gyan/Windows, evermeet/macOS 등의 빌드를 안내한다. source 서명 키는 `FCF986EA15E6E293A5644F10B4322F04D67658D8`이다. source의 서명은 외부 공급자가 만든 실행 파일의 서명과 다르다.

[공식 license 설명](https://ffmpeg.org/legal.html)에 따르면 기본 LGPL-2.1-or-later, GPL 구성 사용 시 전체 FFmpeg에 GPL이 적용된다. [tag n9.0.2 LICENSE.md](https://github.com/FFmpeg/FFmpeg/blob/n9.0.2/LICENSE.md)에서 version3/nonfree 조건도 확인했다. `--enable-version3`와 외부 라이브러리를 함께 확인하며 `--enable-nonfree` 빌드는 관리 배포 후보에서 제외한다. 파일명에 LGPL이 있다고 license 의무·모든 dependency를 검증한 것으로 취급하지 않는다. dlpad는 외부 프로세스로 실행하며 FFmpeg library를 link하지 않는다. 배포 시 해당 binary의 license/source/build recipe 자료를 마련하는 작업은 남는다.

[공식 안내된 BtbN README](https://github.com/BtbN/FFmpeg-Builds)는 LGPL variant가 libx264/libx265 등 GPL-only 라이브러리를 제외한다고 설명한다. Linux glibc 2.28+/kernel 4.18+, Windows 최소 보장 Windows 10 22H2이며 macOS auto-build는 없다. 날짜별 daily artifact는 14일, 월말 마지막 build는 2년 보관 정책이다. `latest`는 변경되므로 pin으로 사용하지 않는다.

**제안 후보**: [autobuild-2026-09-30-13-08](https://github.com/BtbN/FFmpeg-Builds/releases/tag/autobuild-2026-09-30-13-08)의 `ffmpeg-n9.0.2-17-g2a571b6068-linux64-lgpl-9.0.tar.xz`. 공개 [checksums.sha256](https://github.com/BtbN/FFmpeg-Builds/releases/download/autobuild-2026-09-30-13-08/checksums.sha256) 5,712 byte를 내려받아 해당 행과 [tag API](https://api.github.com/repos/BtbN/FFmpeg-Builds/releases/tags/autobuild-2026-09-30-13-08)의 digest를 대조했다.

```text
archive_size: 137863452 bytes
archive_sha256: 2d41cbea0ca1a15029b638330740f78d6f5aeb062355bf425433fc938705350a
checksums_sha256: e45436a222ab0de3b436940f4dbc5bd6df4d5929d19ed6122ce9c6ae57dd1fe6
```

archive 자체를 내려받거나 실행하지 않았다. 서명·attestation·source/notice 완비·ffmpeg/ffprobe member digest·풀린 총크기·codec·호환성은 미검증이다. LGPL 후보의 정확 자르기 codec 적합성을 실제 합성 시험으로 판단해야 한다. 사용자 확정 공급처 또는 출시 채택으로 표현하지 않는다. 같은 공급 archive 안 ffmpeg/ffprobe를 하나의 배포 세트로 검증해야 한다.

현 환경의 기존 외부 `/usr/bin/ffmpeg`와 ffprobe는 `4.4.2-0ubuntu0.22.04.1`, glibc 2.35/WSL kernel 6.18.33.2에서 version 확인에 성공했다. `ffmpeg -version`의 `--enable-gpl --enable-libx264 --enable-shared`, `ffmpeg -L`의 GPLv2-or-later 출력을 확인했다. launcher 파일 크기 301,544/178,832 byte만으로 shared dependency를 포함한 설치 합계를 주장하지 않는다. 이 환경의 외부 설치 실증은 BtbN 후보나 Windows/macOS 실증을 대체하지 않는다.

## JS runtime와 solver: 별도 선택 구성요소

[현재 EJS guide](https://github.com/yt-dlp/yt-dlp/wiki/EJS)는 YouTube challenge를 풀 외부 JS runtime과 solver를 요구한다. 현재 Deno >=2.3.0(기본 활성/권장), Node >=22, QuickJS >=2023-12-9; Bun 1.2.11~1.3.14는 deprecated다. 공식 standalone/zipimport는 EJS를 포함한다. 기본은 해당 tag에 포함한 solver이며 `--remote-components`로 실행 시 임의 갱신하지 않는다. 예전 announcement의 Deno 2.0/Node 20 조건을 현재로 쓰지 않는다. Deno는 UI WebView가 아니며 실행할 때의 외부 도구 비용이다.

관측한 Deno 후보는 [v2.9.7](https://github.com/denoland/deno/releases/tag/v2.9.7), published_at `2026-09-17T09:04:55Z`다. [공식 설치 문서](https://docs.deno.com/runtime/getting_started/installation/)는 플랫폼별 zip와 `.sha256sum`, Windows 10/Server 2016 version 1709 이상을 안내한다. macOS/Linux 최소 OS 전체 조건은 이번 조회에서 확정하지 못했으므로 target별 binary/OS 검증이 필요하다. [Deno LICENSE](https://github.com/denoland/deno/blob/v2.9.7/LICENSE.md)는 MIT이며 third-party 자료도 보관해야 한다.

[Linux checksum](https://github.com/denoland/deno/releases/download/v2.9.7/deno-x86_64-unknown-linux-gnu.zip.sha256sum) 100 byte와 [공식 API](https://api.github.com/repos/denoland/deno/releases/tags/v2.9.7)를 대조했다. 아래 모두 archive metadata이며 executable hash/설치 크기/실행 실증은 아니다.

| Asset | 다운로드 byte | archive SHA256 |
|---|---:|---|
| deno-x86_64-unknown-linux-gnu.zip | 41,596,794 | c6527f24f4b16031d3ae4fa9f658d5f11534c8d84ce7dc8502420280919c3490 |
| deno-x86_64-pc-windows-msvc.zip | 42,630,221 | a0c3101b4158d1dfb7d6a78a7bf0f3de80c96bb423c152beec8beb22786f2238 |
| deno-aarch64-apple-darwin.zip | 38,469,316 | 5cd46d6268f6f78f5d88bdc7159d20bd44cdaa4b3303474839f87ec6fe7ae25c |

Deno archive/executable는 실행하지 않았고 JS CPU/RSS/cache 비용도 미측정이다. 앱 소유 DENO_DIR가 필요하면 bounded cache/정리 계약을 별도 구현해야 한다. 기존 사용자 Deno cache를 삭제하지 않는다. 작은 오래된 QuickJS를 용량만으로 고르면 solver 성능 저하가 생길 수 있다.

## 설치 연결 전에 남은 계약

기존 `tools::InstallManifest.sha256`는 가져온 **실행 파일** digest다. FFmpeg/Deno **archive** checksum을 이 필드에 그대로 넣으면 검증 대상이 다르다. archive source/tag/size/digest, 압축 해제 entry allowlist와 경로 탈출·symlink·크기 상한, 추출된 component별 executable digest, license/source/notice, minimum OS, 공급 trust를 별도로 고정해야 한다. 현재 manifest는 단일 executable만 추적하여 FFmpeg/ffprobe 배포 세트의 완성 계약이 아니다. 이 조사는 shared schema를 변경하지 않았다.

실행 파일 다운로드→digest 확인→소유 guardian 안 제한된 version 검사→실제 합성 codec/solver 시험→원자 activation/rollback→3 OS 검증은 다음 구현 작업이다. 임시 checksum은 stdlib urllib로 파일당 64 KiB+감지 byte, API는 2 MiB+감지 byte 상한 및 30초 timeout으로 읽었고 tempdir만 정리했다. 다운로드 실패·GPG 실패는 후보를 검증 완료로 승격하지 않는다. 네트워크 기본 sandbox DNS 실패 후 사용자가 허용한 개발 의존성 공개 조회의 escalation이 승인됐다. 사용자 비밀/계정/영상은 전송하지 않았다.

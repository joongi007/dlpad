# BtbN LGPL FFmpeg 공급 후보 실제 실증

2026-10-02 Linux/WSL x86_64. [작업 계약](../work-items/S04-ffmpeg-supply-probe.md). 기존 공급 문서·제품·Cargo·schema를 변경하지 않았다. 신규 `scripts/ffmpeg_supply_probe.py`가 공개 고정 archive를 새 프로젝트 소유 폴더에 내려받고 allowlist만 추출했다. **실행 가능한 Linux 후보이며 관리 배포 채택 완료는 아니다.**

## 공급 pin과 실제 bytes

[고정 BtbN release](https://github.com/BtbN/FFmpeg-Builds/releases/tag/autobuild-2026-09-30-13-08)의 공식 API와 checksums.sha256을 다시 받아 이전 pin과 대조했다. published_at은 2026-09-30T13:09:24Z다. 실제 다운로드는 16.88초, 추출 10.21초다. [원시 공급 증거](ffmpeg-supply-probe.json)에 URL·size·digest·member inventory·argv·결과를 보존한다.

| 검증 대상 | 실제 byte | SHA-256 |
|---|---:|---|
| tar.xz archive | 137,863,452 | `2d41cbea0ca1a15029b638330740f78d6f5aeb062355bf425433fc938705350a` |
| bin/ffmpeg | 142,534,568 | `953632dece404bb1461a4a2285f0191a6105d72c81f9da577eb54326a7fad49a` |
| bin/ffprobe | 142,305,128 | `c0d4ee234730e7b083cc729108ac34f3b718cc4858c3cf8ed6e0dcec545f13c1` |
| LICENSE.txt | 7,651 | `da7eabb7bafdf7d3ae5e9f223aa5bdc1eece45ac569dc21b3b037520b4464768` |

압축된 archive digest를 실행 파일 digest로 사용하지 않았다. 전체 archive 73개 member의 선언 총크기는 450,209,477 bytes다. ffmpeg/ffprobe/license만 선택한 실제 설치 후보는 284,847,347 bytes(271.65 MiB)이고, archive와 함께 유지하면 최소 422,710,799 bytes(403.13 MiB)다. 이번 측정 폴더의 archive/API/log/두 실행 파일/작은 fixture 전체는 423,046,053 bytes다. ffplay·문서 전체를 설치 크기에 포함시키지 않았다.

## 안전 추출과 실행

최대 다운로드 140 MiB, 공개 API 2 MiB/checksum64 KiB, socket20초·다운로드 wall300초 확인(마지막 blocking read 최대20초 추가)을 적용한다. stream tar를 읽어 총 선언크기768 MiB/member4096개/추출본 실행 파일 각각384 MiB·notice2 MiB를 제한한다. 경로 root 일치, absolute/parent/backslash, duplicate, symlink/hardlink/special file을 거절한다. `extractall`과 shell 실행은 사용하지 않는다. 새 private 소유 root의 create-only regular files만 작성한다.

[합성 보안 회귀](ffmpeg-supply-security.json)는 absolute/parent path, symlink, hardlink, duplicate, oversized member 6건 거절 및 allowlist 정상 대조 1건을 확인했다. 압축 dictionary/memory의 일반 악성 LZMA 공격 방어를 제품 installer 수준으로 검증한 것은 아니다. 공급 SHA 일치가 먼저 확인된 이번 고정 archive만 실행했다. source API/checksum/실행 파일에 대한 독립 서명·attestation 검증은 없다.

argv-only process는 새 owned session으로 시작하고 30초 deadline/관측 log256 KiB 제한, 초과 시 소유 group만 종료·wait한다. 최종 helper는 child-only RLIMIT_FSIZE4 MiB도 적용해 log/fixture 성장 상한을 둔다. 로그가 즉시 끝나도 종료 후 크기를 다시 검사한다. 초기 전체 probe 뒤 해당 추가 제한의 실행 helper를 실제 ffprobe로 재검증했다. 사용자의 설치·캐시·media를 변경하거나 삭제하지 않았다.

## 버전·license·codec·실행 비용

두 binary는 `n9.0.2-17-g2a571b6068-20260930`, gcc16.2.0을 보고한다. [buildconf](ffmpeg-supply-logs/ffmpeg-buildconf.txt)는 `--enable-version3`, `--disable-libx264`, `--disable-libx265`, `--enable-libopenh264`를 포함하며 enable-gpl/nonfree는 없다. [실제 license](ffmpeg-supply-logs/ffmpeg-license.txt)는 **LGPL version3-or-later**다. 단순 파일명의 LGPL 또는 upstream 기본 LGPL2.1로 표시하지 않는다. [공식 BtbN README](https://github.com/BtbN/FFmpeg-Builds)는 LGPL variant에서 GPL-only dependency 제외를 설명한다.

합성 testsrc2 160×90/10 fps/2초 + sine audio를 MPEG-4/AAC MP4 101,583 bytes로 생성했다. encode/probe/decode 모두 exit0다. 기본 codec을 명시하지 않은 정확 자르기(0.2초 시작, 0.8초 길이)와 decode도 exit0이며 [cut probe](ffmpeg-supply-logs/cut-probe.txt)는 MPEG-4/AAC 각 0.800000초를 확인한다. H.264 출력/실제 yt-dlp 통합/모든 codec을 이 결과로 완료하지 않는다.

[runtime 증거](ffmpeg-supply-runtime.json)는 binary SHA 재확인 뒤 실행 비용을 다시 측정했다. decode wall0.204초/CPU0.064초, 정확 자르기 wall0.233초/CPU0.094초다. 10ms post-exec RSS 표본 최대는 version21,508 KiB, decode32,592 KiB, probe27,696 KiB, cut36,176 KiB다. 매우 짧은 처리라 표본이 실제 peak를 놓칠 수 있다. wait4의 launch peak에는 fork 시 Python launcher RSS가 포함된다(초기42,060 KiB, 재해시 측정158,132 KiB). 이를 FFmpeg 자체 RSS로 혼동하지 않는다. 작은 fixture 결과는 긴 영상의 처리 비용 예측이 아니다.

## 후보 판정과 남은 계약

이번 pin은 Linux에서 decode/기본 정확 자르기가 가능한 실제 공급 후보다. archive에는 LICENSE.txt만 있고 대응 source/README/빌드 recipe/dependency notice는 발견되지 않았다. release49assets에도 source archive는 없었다. [FFmpeg 공식 법적 안내](https://ffmpeg.org/legal.html)는 정확히 대응하는 소스와 외부 dependency를 확인하도록 설명한다. 해당 FFmpeg commit·BtbN build recipe/dependency source 및 완전한 LGPL3/GPL3 notice 구성, 공급 trust, Windows/macOS/minimum OS, archive multi-component activation/rollback이 해결되기 전 관리 배포를 채택하지 않는다. Source가 없다는 사실은 공급자의 의무 이행 전체에 대한 법적 판단이 아니다.

재현은 새 evidence 경로를 지정한다. 기존 결과를 덮어쓰거나 관리 도구로 활성화하지 않는다.

```sh
python3 scripts/ffmpeg_supply_probe.py --self-test
python3 scripts/ffmpeg_supply_probe.py --evidence docs/validation/ffmpeg-supply-probe-new.json
```

제품 UI는 추가하지 않았고 기존 archive supply gate는 유지한다. 기존 Git은 정상 저장소가 아니라 .git을 보존했다.

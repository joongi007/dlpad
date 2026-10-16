# Deno YouTube 지원 구성요소 후보 실증

2026-10-02 Linux/WSL x86_64. [작업 계약](../work-items/S04-deno-supply-probe.md), [실제 pin·실행·비용 증거](deno-supply-probe.json). 신규 `scripts/deno_supply_probe.py`와 전용 문서만 작성했고 제품/sharedschema/Cargo/기존 공급문서는 변경하지 않았다. **오프라인 실행 가능한 Linux 후보이며 관리 공급 채택이나 YouTube 지원 완료는 아니다.**

## 공식 요구사항과 실제 pin

[yt-dlp EJS guide](https://github.com/yt-dlp/yt-dlp/wiki/EJS)를 다시 확인했다. YouTube challenge에는 JS runtime과 EJS solver가 별도로 필요하고, Deno 최소2.3.0을 권장한다. 공식 PyInstaller/zipimport 배포물에는 EJS가 포함된다. 이번 시험은 고정 Deno2.9.7만 검증하며 bundled EJS의 정확 버전/동작이나 실제 challenge를 실행하지 않았다. 원격 components/npm/package 실행을 활성화하지 않았다.

[공식 Deno release](https://github.com/denoland/deno/releases/tag/v2.9.7) API asset digest, 공개 sha256sum, 실제 다운로드를 비교했다. [공식 설치 안내](https://docs.deno.com/runtime/getting_started/installation/)와 [tag source/MIT license](https://github.com/denoland/deno/blob/v2.9.7/LICENSE.md)를 확인하고 license1074 bytes를 별도 보관했다. ZIP download는5.02초였다.

| 대상 | 실제 byte | SHA-256 |
|---|---:|---|
| deno-x86_64-unknown-linux-gnu.zip | 41,596,794 | `c6527f24f4b16031d3ae4fa9f658d5f11534c8d84ce7dc8502420280919c3490` |
| 추출 deno | 95,830,104 | `ce6a052beb97c2b92de67077e3f3924ba7c9661ede0d8dc2f66a116a3c841f21` |

Archive digest를 실행 파일 digest에 넣지 않았다. 실행 파일91.39 MiB, archive+실행 파일131.06 MiB다. 관측 당시 API/license/fixture/cache/log까지 소유 폴더 전체137,823,202 bytes였다. ZIP에는 deno 하나만 있고 notice/source는 들어 있지 않았다. MIT root license만으로 V8/Rust dependency·embedded snapshot·third-party notice가 완비됐다고 취급하지 않는다. source tag와 binary 대응/source·notice·공급 trust는 관리 채택 전에 더 검증해야 한다.

## 오프라인 동작·종료·비용

[실제 version](deno-supply-logs/version.txt)은 Deno2.9.7 stable/x86_64-unknown-linux-gnu, V8 15.0.245.2-rusty, TypeScript6.0.3이다. 합성 argv 배열 `[3,5,8]`의 합16/개수3과 고정 synthetic 텍스트 SHA-256을 JS로 계산하고 Python 기대값과 비교했다. [fixture 결과](deno-supply-logs/fixture.txt)는 exit0다.

`--no-remote --cached-only --no-check --no-config --no-lock --no-prompt` 및 deny-net/run/read/write/env/sys를 적용했다. entry script와 runtime 내부 전용 cache는 CLI/runtime가 읽고 쓰며 fixture에 일반 파일/네트워크 권한은 주지 않았다. 실제계정/쿠키/영상/원격 solver는 사용하지 않았다. DENO_DIR는 새0700 프로젝트 소유 cache이며 update check를 비활성화했다. 사용자 Deno cache를 읽거나 삭제하지 않는다.

| 실제 실행 | wall | CPU | post-exec RSS 표본 최대 |
|---|---:|---:|---:|
| version | 0.072초 | 0.014초 | 7,056 KiB |
| 합성 JS | 0.259초 | 0.065초 | 26,816 KiB |
| 무한루프 timeout | 1.016초 | 0.855초 | 29,668 KiB |
| 명시 cancel | 0.260초 | 0.168초 | 29,312 KiB |

무한 루프는 ready 출력을 확인한 뒤 소유 group을 종료하고 wait4로 회수했다. timeout/cancel은 exit-9이며 자연 종료를 취소 성공으로 계산하지 않는다. 10ms RSS 표본은 빠른 peak를 놓칠 수 있고 wait4 launch RSS는 pre-exec launcher도 포함한다. 작은 합성 입력의 시간은 실제 EJS solver 성능이나 전체 YouTube 트래픽 측정이 아니다.

관측 전용 cache는 fixture250,768 bytes, timeout/cancel 뒤296,088 bytes다. 실행 중 32 MiB/512개 entry/symlink 금지 예산을 반복 검사하고 초과 시 소유 process를 종료한다. OS directory quota는 아니며 polling 간 초과가 생길 수 있다. child-only RLIMIT_FSIZE4 MiB는 각 cache/log/fixture 파일을 제한한다. JS file-write는 거절하지만 runtime 내부 cache 쓰기는 남는다. 64 KiB log 및 실행 deadline을 종료 후에도 검사한다.

## 추출 실패 경로와 재현

공유 `ffmpeg_supply_probe.fetch`의 HTTPS origin/redirect 및 byte/time 상한을 재사용한다. download48 MiB/API2 MiB/checksum4 KiB/license64 KiB, ZIP member32개/총256 MiB를 제한한다. allowlist deno/notice만 stream copy하고 absolute/parent/backslash/colon/duplicate/nonregular/symlink/encryption을 거절한다. CRC와 길이는 ZIP reader/실제 복사량으로 확인한다. create-only 새 root만 작성하며 extractall/shell을 사용하지 않는다.

`--self-test` 8개는 경로 탈출3개, symlink, duplicate, oversized ZIP, cache-budget 초과를 거절하고 allowlist 정상 대조를 통과했다. cache 초과 시험은 새 임시 폴더의 sparse 합성 파일이며 실제 볼륨을 채우지 않았다. duplicate ZIP 생성의 Python warning은 예상된 공격 fixture다. 새 script와 재사용 fetch의 SHA를 원시 증거에 기록했다.

```sh
python3 scripts/deno_supply_probe.py --self-test
python3 scripts/deno_supply_probe.py --evidence docs/validation/deno-supply-probe-new.json
```

## 남은 gate

Windows/macOS/minimum OS, bundled EJS의 정확 버전 통합·실제 사이트, source/third-party notices/공급 signature·attestation, 관리 multi-component archive 설치/선택/제거는 미검증이다. 기존 후보를 관리 설치로 등록하거나 PATH·시스템 설정을 변경하지 않았다. 기존 .git을 보존했다.

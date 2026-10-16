# S04: Windows FFmpeg 후보 실제 payload inventory

- 상태: completed
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R11
- 담당/수정 범위: soak_recovery_audit; 본 문서와 신규 `.local-tools/ffmpeg-win64-inventory-*`만
- 선행 작업: Windows 최초 릴리스 범위, 공식 BtbN release metadata와 고정 recipe 확보

## 사용자 결과

Windows 후보의 실제 ZIP/component 크기·SHA·license·고정 source 대응을 확인한다. 공급 채택이나 실제 Windows runtime 검증을 선언하지 않는다.

## 계약과 범위

공식 autobuild-2026-09-30-13-08 win64-lgpl-9.0 asset을 신규 폴더로 bounded streaming 다운로드한다. 실행 파일은 실행하지 않고 ZIP metadata와 선택 component만 조사한다. 기존 source 자료는 읽기만 하고 Windows variant를 새 evidence로 복사한다. Rust/Cargo/기존 문서·증거 수정, Linux 공급 후속, 배포는 제외한다.

## 인수 조건

- 공식 metadata의 exact asset size/SHA와 실제 다운로드 일치.
- ffmpeg/ffprobe/license component path·size·SHA와 PE import inventory 보존.
- Windows variant 및 v1 flat 256MiB/component 합 한도 대응을 구분.
- 실제 실행하지 않은 version/runtime 및 binary 재현 provenance 부족은 미검증으로 명시.

## 검증과 handoff

최종 `python3 scripts/check_docs.py`: 실제 exit0, 570 documents PASS. 아래 최초 후보 기록 이후 실제 수집을 완료했으며 제품 채택/Windows 실행은 미완료다.

수집 진행 중. 공식 후보: `ffmpeg-n9.0.2-17-g2a571b6068-win64-lgpl-9.0.zip`, 171535354 bytes, SHA256 `6b264b9e6019103f601d98c292bd332fd87acf1c5e941ddff4fb71760fe63432`. 예전 Linux payload/notice를 Windows 완비 근거로 전용하지 않는다.

### 실제 수집 결과

신규 evidence `.local-tools/ffmpeg-win64-inventory-f5uj2z2m/manifest.json`, `pe-imports.json`에 exact URL·ZIP 48 entries·component·고정 recipe SHA를 보존했다. `/tmp/dlpad_windows_ffmpeg_inventory.py` 실제 exit0, streaming 64KiB/192MiB/240초 상한, 다운로드 23.25초. 공식 metadata size/SHA와 실제 일치. PE header-only `/tmp/dlpad_windows_pe_inventory.py` 실제 exit0. objdump 전체 출력은 1MiB 수집 상한을 넘겨 거절했고 대신 PE header와 import table만 bounded 조회했다. 어떠한 exe도 실행하지 않았다.

| component | bytes | SHA256 |
| --- | ---: | --- |
| ffmpeg.exe | 134497280 | `0e482b853f2fd4821055d20cdce510f0eea6027037c04b994f454a2136e8868e` |
| ffprobe.exe | 134296064 | `b9627185ffbad1d4fc6ef953868ca9ad8df75fc8dd035fcab58aca78caa2d23e` |
| LICENSE.txt | 7651 | `da7eabb7bafdf7d3ae5e9f223aa5bdc1eece45ac569dc21b3b037520b4464768` |

두 exe 합 268793344 bytes는 기존 `archive_manifest.rs`의 v1 `MAX_EXPANDED_BYTES=268435456`보다 **357888 bytes 초과**한다. license까지 선택하면 268800995 bytes이며 각각 component 자체는 256MiB 미만이다. ZIP 크기는 256MiB 미만이지만 전체 ZIP 확장 크기는 418417814 bytes다. 한도 단순 상향이나 설치 통과를 주장하지 않는다. 주 담당이 v2 ZIP layout/component 선택합/전체 상한을 별도로 조정한다. 현재 evidence 디스크 비용 440403816 bytes(약 420MiB): 원ZIP과 선택 실행 파일, recipe, metadata; 설치 시 exe 둘 약256.34MiB. 제품 실행 RSS/startup/Windows DLL 가용성 비용은 미측정.

두 exe 모두 PE32+ AMD64(`0x8664`). import table에는 Windows system DLL과 `api-ms-win-crt-*` UCRT API-set, DWrite/D2D1 등이 있다. 이는 추가 FFmpeg DLL이 import 목록에 없다는 정적 관찰일 뿐 지원 OS 최저 버전/실행 성공 증명이 아니다. raw DLL 목록은 `pe-imports.json` 참조.

### Windows source/license 대응과 잔여

[공식 고정 release](https://github.com/BtbN/FFmpeg-Builds/releases/tag/autobuild-2026-09-30-13-08)의 BtbN commit `6c9aec5fc9a72ec3abedd1fa84db141fa18cf52b` 공식 archive에서 Windows variant·toolchain·공통 variants를 새 evidence로 복사했다. `win64-lgpl.sh`는 `windows-install-static.sh`와 `defaults-lgpl.sh`를 호출하며 defaults는 `--enable-version3 --disable-debug`, `LICENSE_FILE=COPYING.LGPLv3`; 9.0 addin은 release/9.0 branch다. static packaging은 bin 외 doc/presets도 포함한다. 실제 ZIP license SHA는 이미 확보한 exact FFmpeg commit `2a571b606854520cf89804d8030c8b328e621689` COPYING.LGPLv3와 일치한다. GPLv3 참조 원문도 이전 corresponding-source evidence에 확보되어 있으나 ZIP license 파일 하나만으로 모든 dependency notice를 완비한 것은 아니다.

FFmpeg source full archive/고정 BtbN recipe를 확보했지만 build branch 조회·mutable base image/crosstool-ng clone·실제 dependency 당시 payload digest가 모두 고정된 재현 bundle은 아니다. [FFmpeg 공식 license 자료](https://ffmpeg.org/legal.html)를 기준으로 Windows 실제 `-version/-buildconf/-L`을 주 담당이 hidden native host에서 수집한 뒤 enabled dependency를 재매핑해야 한다. Linux enabled configuration/notice 목록을 Windows 완비 증거로 사용하지 않는다. 현재 후보는 **Proposed**, exact runtime version·실제 기능·최소 Windows 환경·v2 제품 설치/삭제·최종 notice/source 배포 구성은 미검증이다.

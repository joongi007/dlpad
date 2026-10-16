# S04: Windows FFmpeg 공급 대응 자료 보완

- 상태: completed (Windows 대응 자료 보완; 공급 채택 미완료)
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R05, R11
- 담당: metrics_recovery
- 수정 파일: scripts/windows_ffmpeg_source_mapping_probe.py, 이 문서, 새 전용 evidence
- 선행 작업: Windows 원 mapped ZIP inventory와 실제 native version/buildconf 및 설치 인수

## 계약과 인수

현재 고정 Windows win64 LGPL 9.0 ZIP의 실제 configure flags를 고정 BtbN recipe와 대응한다. Linux enabled 목록을 Windows 목록으로 대체하지 않는다. 원 ZIP/FFmpeg source/BtbN source SHA를 전후 확인하고 새 bounded 공식 자료만 수집한다. 원 source/사용자 파일/Rust/Cargo/공통 문서/기존 스크립트는 수정하지 않는다. 도구 실행, 재빌드, 외부 게시와 공급 자동설치 채택은 제외한다.

실제 recipe revision과 확보한 source/notice의 digest를 구분한다. 전체 source가 없는데 license 한 파일만으로 대응 소스 완비나 공급 채택을 선언하지 않는다. compiler/container/당시 download cache provenance와 전이 의존성 자료가 부족한 항목은 그대로 남긴다. 새 스크립트는 읽기 전용 mapping과 bounded 공식 수집 및 실패 raw 보존을 수행한다.

## 검증과 인계

아래 실제 수집을 완료했다. native 실행 및 공급 채택은 주 담당의 후속 통합 범위다.

## 실제 Windows 대응과 보완 결과

`python3 -m py_compile scripts/windows_ffmpeg_source_mapping_probe.py` exit0. 공식 공개 수집을 포함한 첫 실행과 재실행 모두 exit0이며 [두 번째 원시 결과](../validation/evidence/windows-ffmpeg-source-mapping-second/result.json)에 9.734초와 원 파일 전후 일치를 기록했다. 첫 결과도 삭제하지 않았다. 스크립트 SHA256은 `6b557f2ca22d4f57956850b1a4135daafe1413fd349ea2ec33ecb08a18022965`다.

실행 명령: `python3 scripts/windows_ffmpeg_source_mapping_probe.py docs/validation/evidence/windows-ffmpeg-source-mapping-second --collect`.

실제 Windows native version log는 `n9.0.2-17-g2a571b6068-20260930`, GCC16.2.0/crosstool-NG1.29.0.7_b1a94f6이다. configure의 enable61/disable19 중 enable60은 고정 recipe literal flag에 대응했고 나머지 version3는 FFmpeg 자체 license 정책이다. [mapping](../validation/evidence/windows-ffmpeg-source-mapping-second/mapping.json)은 복수 recipe 후보와 평가하지 않은 target 조건을 그대로 기록한다. 61개가 모두 독립 외부 library라는 뜻이 아니다. recipe script를 실행하거나 당시 실제 build graph로 단정하지 않았다.

Windows는 pthreads와 schannel을 enable하고 libxcb/xlib/libpulse/libdrm을 disable한다. Linux의 OpenSSL enabled 목록을 그대로 전용하지 않는다. schannel recipe는 Windows 플랫폼 API 선택이며 별도 source download를 선언하지 않는다. mingw recipe는 headers/CRT/winpthreads를 빌드하며 --enable-pthreads를 제공한다. mingw-std-threads는 configure flag가 없는 Windows 전용 헤더 dependency이므로 flag 대응 목록과 별도로 원 recipe를 남겼다.

| 신규 전체 source | exact revision | bytes | SHA256 |
| --- | --- | ---: | --- |
| mingw-w64 | 57b595039040eaa15bece85b7cc71d952281b269 | 15818262 | a68816e314290facd5da1ac96c7eead7e6de6ea4f709a1fbdcf8185497c57eb2 |
| mingw-std-threads | c931bac289dd431f1dd30fc4a5d1a7be36668073 | 33291 | bba362a5fd537ecd223239efa1f7e7c570018e6d5ea7b26cc0515d3ede9da5ad |

mingw-w64의 [공식 site](https://www.mingw-w64.org/downloads/)가 연결한 공식 GitHub repository의 exact commit archive를 사용했다. recipe의 SourceForge origin과 GitHub snapshot을 동일 binary payload provenance로 단정하지 않는다. source archive는 실행/파일시스템 경로 추출 없이 순차 member만 조사했다. mingw-w647084entries/확장134570689B/notice11개, mingw-std-threads21entries/확장141922B/notice1개다. 원 ZIP LGPL 한 파일에 없는 winpthreads의 MIT 계열 및 Lockless 파생 notice, MinGW runtime 묶음 notice, std-threads BSD 계열 notice 전문이 고정 source 안에 확보됐다. 각각 member 경로/크기/SHA를 result의 source_inventory에 기록했다. source 전체 확보와 실제 linkage 포함 여부는 구분한다.

기존 공급 notice73파일을 원 크기/SHA와 다시 대조하고 Windows 실제 recipe의 exact commit URL에만 연결했다. [연결 목록](../validation/evidence/windows-ffmpeg-source-mapping-second/prior-exact-notice-links.json)은 feature57개를 포함한다. 이 수치는 notice의 revision 재사용 근거이며 57개 전체 source나 전이 license 완비를 뜻하지 않는다. 다른 revision을 Windows 대응으로 채우지 않았다.

원 Windows ZIP171535354B/SHA6b264b9e… 및 원 FFmpeg 전체 source17336083B/SHAf02fc080…와 BtbN recipe107893B/SHA81da4f7c…는 전후 일치했다. 원 selected component268800995B와 ZIP 전체 확장418417814B는 기존 Windows inventory/설치 근거를 유지한다. 이번 수집은 바이너리 실행·설치 인수를 재실행하지 않았다.

## 배포 target와 비용 근거

고정 [공식 BtbN README](https://raw.githubusercontent.com/BtbN/FFmpeg-Builds/6c9aec5fc9a72ec3abedd1fa84db141fa18cf52b/README.md)는 Windows7+UCRT를 target하지만 최소 지원 보장은 Windows10 22H2로 구분한다. 이를 dlpad의 Windows7 지원 또는 최소 OS 실제 시험으로 표현하지 않는다. 원 Windows payload는 AMD64이며 toolchain은 mingw32 GNU cross-compiler다. dlpad 소비자 target 문자열이 windows-msvc인 것과 FFmpeg 자체 compiler를 혼동하지 않는다.

두 번째 evidence 총15985113B(약15.24MiB), 첫 결과는 별도로 보존했다. 수집 요청은20초/전체240초, source32MiB·std-threads4MiB, notice/readme128KiB, archive entry30000/전체확장256MiB/notice512개, memory member1MiB 상한이다. 원 largeZIP/source SHA는64KiB streaming이며 전체 source를 메모리에 적재하지 않는다. 제품 설치·RSS 비용은 이번 Python collector 비용과 별개다. 기존 Windows 설치31.128초/선택 검증10.012초 근거는 별도 [설치 기록](S04-windows-ffmpeg-managed-install.md)이며 새 측정으로 덮지 않는다.

## 남은 채택 gate와 다음 수직 작업

**공급 Proposed 및 자동 설치 비활성은 유지한다.** [FFmpeg 공식 안내](https://ffmpeg.org/legal.html)의 대응 source·빌드 설명·외부 library notice 확인 항목을 읽었으며, 자료 보유를 법적 완비 판정으로 바꾸지 않는다. 링크된 checklist는 FFmpeg library linking 관점의 안내이고 외부 executable 사용과 실제 선택 variant를 구분해야 한다.

1. Windows enabled recipe의 전체 source를 각 exact revision으로 확보하고 라이브러리별 patch/submodule/Cargo vendor/embedded code를 따라 transitive graph를 고정한다. 현재 root notice만 있는 다수 source, iconv imported gnulib graph, librsvg의 GLib/Cairo/Pango 및 Rust 간접 의존성, FreeType bundled source 등의 actual build 포함 범위는 남는다.
2. base-win64 Dockerfile의 base:latest, crosstool-ng 무고정 clone, 당시 compiler image digest/cache를 고정한다. ct-ng-config의 GCC16.2.0/binutils2.47/MinGWv14.0.0/libgomp 활성 선언과 이번 mingw recipe commit이 서로 다른 단계인 사실을 보존해야 한다. config만으로 실제 linked runtime version/digest를 확정하지 않는다.
3. 원 binary→source/patch/build graph를 연결하는 공급 attestation 또는 독립 대응 근거, source 제공 장기 보존 및 배포 bundle/notice UI를 마련한다. BtbN README는 월말 build 2년 유지와 최근 daily14개 유지 정책이므로 외부 URL이 영구 source 보존 계약은 아니다.
4. 원 mapped package를 유지하면서 source/notice manifest 고정과 tamper 거절·누락 source 보류 UI를 수직 인수한 뒤 공급 채택을 별도 결정한다. LICENSE.txt만 설치되는 현재 package를 전체 notice 완비로 주장하지 않는다. 실제 공개 배포와 업로드는 별도 사용자 승인 범위다.

이번 scoped 결과는 Windows 특유 두 전체 source와 정확 notice 대응 확보 완료이며 P0 공급 채택 전체 완료가 아니다. Rust/Cargo/공통 문서/기존 원 source·ZIP을 변경하지 않았고 Linux 새 제품 인수는 수행하지 않았다.

최종 `python3 scripts/check_docs.py`: 실제 exit0, 618 documents PASS. 외부 URL과 heading anchor 검증은 이 checker 범위 밖이다.

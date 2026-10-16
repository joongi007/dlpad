# S05: R05 관리 runtime 다음 수직 기능 읽기 감사

- 상태: done
- 분류: audit
- 우선순위: P0
- 요구사항: R05, R06, R11
- 담당/수정 범위: diagnostic_cache — 이 문서만
- 선행 작업: S04 직접 yt-dlp installer·선택/제거·staging, FFmpeg/Deno 공급 후보 실증

## 사용자 결과

FFmpeg/JS runtime의 현재 구현을 실제 source와 증거로 대조해 다음 구현 범위를 구체화했다. 제품 source/Cargo/vendor/runner는 변경하지 않았다. 전체 도구 관리 완료를 주장하지 않는다.

## 실제 코드와 현재 사용자 동작

- R05는 외부 FFmpeg 선택 또는 앱 관리 설치와 관리 설치만 제거를 요구한다. ROADMAP S04는 manifest/rollback/pin/제거·FFmpeg 공급·JS runtime을 함께 갖지만 세부 공급 결정은 ADR0003 Proposed다.
- `crates/core/src/options.rs` DownloadOptions의 ffmpeg:Option<PathBuf>, plan의 --ffmpeg-location 단일 argv 및 저장 snapshot이 있다. 실제 외부 FFmpeg 버전 확인/Attempt 증거가 연결돼 있다. JS runtime typed 필드·--js-runtimes는 core/engine/app source 검색에서 발견되지 않았다.
- `crates/engine/src/tools.rs::InstallManifest`는 tool/version/target/source_url/sha256/license/ownership/executable 단일 파일이다. source archive digest와 실행 파일 digest, FFmpeg+ffprobe components, notice/source bundle을 표현하지 못한다. `import`는 generic 단일 local executable import이며 archive 공급 완료가 아니다.
- `tools/direct_download.rs::download_and_install`는 tool != yt-dlp를 InvalidManifest로 거절한다. HTTPS·256MiB download·120초 전체 설치 deadline·hash/version/owned staging 검증은 직접 executable용이다. archive extraction 모듈은 없다. yt-dlp의 --version 확인을 FFmpeg(-version)나 Deno로 일반화해 쓸 수 없다.
- `tools/selection.rs` installed/active_installed/select_existing은 단일 manifest hash/소유 디렉터리 검증을 한다. remove는 active 거절 및 단일 executable+manifest strictknownfiles 삭제다. 두 component set의 원자 active 전환/rollback/removal은 없다.
- `crates/app/src/managed.rs::candidate`는 2026.08.19 yt-dlp 세 OS 공급만 갖는다. CPU `accessible_tools.rs::perform`도 active_installed/installed에 yt-dlp를 고정하고 Action::Install에서 같은 candidate/direct installer를 호출한다. 관리 FFmpeg/Deno 선택·비용·license 상세는 없다.

## 공급 실증이 증명한 범위

`scripts/deno_supply_probe.py`는 공식 Deno2.9.7 Linux ZIP 고정 pin/실제 archive·executable 별도 SHA, member32/총256MiB/allowlist/stream extraction·경로/symlink/duplicate/암호화 거절, offline synthetic JS와 timeout/cancel을 시험한다. 제품 Rust adapter가 아닌 독립 Python probe다. 실제 실행 파일 95,830,104B, archive41,596,794B, 합계131.06MiB이며 cache 예산32MiB/512entries를 관측한다. source/thirdparty notice/attestation, 다른 OS 및 실제 bundled EJS challenge는 미검증이다.

`scripts/ffmpeg_supply_probe.py`는 BtbN LGPL 후보 tar.xz 실제 archive137,863,452B, ffmpeg142,534,568B/ffprobe142,305,128B/license7,651B를 검증했다. 실행 공급세트271.65MiB, archive 함께403.13MiB. 실제 buildconf/license는 LGPL3-or-later이며 소형 encode/probe/decode/기본 정확 cut은 통과했다. 공급 trust/대응source·외부dependency notice 및 H264/실제yt-dlp/3OS는 남는다. 고정 공급의 allowlist 시험이 일반 악성 압축 dictionary memory 방어를 증명하지 않는다. 기존512MiB staging 예산에는 archive+선택component403MiB가 들어가지만 모든 archive선언450MiB+archive를 전부 풀면 초과하므로 선택 stream 추출이 필수다.

INTEGRATIONS의 “archive·실행 파일 검증 남음”은 2026-10-02 독립 Linux probe 이후 범위를 갱신할 여지가 있다. root 공통문서 담당에게 알려야 하며 이번 감사가 Proposed 공급을 Accepted로 바꾸지 않는다.

## 다음 구현 단위 권고

첫 단위는 **Deno 단일 ZIP 관리 설치→검증된 active 명시 적용→offline synthetic download argv 확인→inactive 제거**로 좁힌다. FFmpeg multi-component보다 작고 ZIP member 하나로 archive/실행 digest 분리와 typed runtime 경로를 끝까지 검증할 수 있다. 공급 trust/notice가 미해결이면 실제 자동 공급 활성화는 gate를 유지하며 동일 backend를 synthetic fixture로 통합한다. 이를 YouTube 지원 완료로 표현하지 않는다. 원격 solver/npm/update를 자동 허용하지 않는다.

구체 수정 범위/API 제안:

1. root 공유계약: tools.rs에 별도 ArchiveManifest{tool,version,target,source,size,archive_sha256,format,components[]}, component{relative_path,sha256,bytes,role,license_reference}를 정의한다. existing direct manifest 유지/명시 migration. Deno ZIP 단일 runtime+notice와 FFmpeg set을 같은 표현으로 수용하되 자동 공급 catalog 활성화는 각각 gate.
2. 신규 tools/archive_install.rs 및 bounded_extract.rs: `install_archive(manifest,cancel)->InstalledToolSet`, SHA 검증된 owned archive만 create_new stream extraction, ZIP deflate decompression memory/entry/expanded bytes/timebound를 설정한다. 새 Cargo ZIP 의존성 exact pin/feature·크기는 root 조정. Python probe를 제품 subprocess installer로 돌리지 않는다.
3. staging_recovery.rs: archive+선택components+manifest/notice 예약/체크포인트 원file IDs와 digest를 추적한다. 기존 nonUnix 회복 부족을 숨기지 말고 unknown 잔여물 보존/capacity failclosed. Final publication crash/DBfail 이전active보존 gate도 함께 검증.
4. selection.rs/tools.rs: `active_components(tool)` 및 set 전체hash검증, set 단위 active전환/rollback, strictrecordedcomponent만 inactive removal. external path·source archive 사용자파일 삭제 금지.
5. options.rs/core 저장snapshot: `js_runtime:Option<JsRuntimeSelection{kind:Deno,path}>`와 bounded absolute argv `--js-runtimes deno:<path>`를 명시 생성한다. 금지 advanced override/기존snapshot 호환/Windows콜론·한글공백 literal 경로 시험. engine download tool version 단계에 Deno --version 증거·실행 전 hash변경 검증.
6. managed.rs+accessible_tools.rs 및 root CPU entry: runtime 종류 선택, 다운로드/설치/notice·cache 비용 설명, 명시 Use, running 동안 intentfreeze, channel1 worker/CloseWait. catalog 문자열은 root 중앙 등록. 기존 Iced/CPU UI parity 여부를 root가 정한다.

FFmpeg 다음 단위는 같은 backend에 ffmpeg/ffprobe/license 세 component 및 -version/-buildconf/license 검사, 실제 임시 합성 media cut/decode로 이어지는 set 설치/활성/rollback/제거다. ffmpeg 경로만 검증하고 ffprobe가 다른 PATH binary로 돌아가는 혼합세트를 성공으로 보지 않는다.

## 실패 인수 조건

- archive SHA mismatch/size·expanded bound/CRC손상/absolute-parent-backslash-colon/duplicate/symlink/hardlink/special/encrypted members를 실행 전 거절; 외부 sentinel·이전 active hash 유지.
- download/extract/version cancel·deadline/loglimit, diskfull·partialwrite·DBfail·publish충돌·crash checkpoint, symlink/identity교체에서 owned확정 files만 정리하고 unknown보존.
- ZIP decompressor 메모리·runtimecache 예산 실제 측정, compressed/installed/peak reserved 총비용 구분. Windows/macOS 미실행을 Linux PASS로 확대 금지.
- Deno wrongversion/changedexecutable/source·notice 누락/activecomponent손상 거절; typed argv path한글공백·Windowsdrivecolon·snapshotresume, running중변경freeze.
- 실제 installed inactive 선택/removal·외부설치보존·active사용중제거거절, appCloseWaitworkerack/privateGUI만 사용.
- offline synthetic JS/yt argv는 실제 YouTube EJS 성공의 대체가 아니다. actualsource/notice/trust와 sites/3OS인수는 별도 gate.

## 검증과 handoff

README→PRODUCT→CONTEXT→R05/ROADMAP→ADR0003/INTEGRATIONS와 관련 source/probe/기존 validation을 읽고 rg/sed/cat로 대조했다. 신규 구현·Cargo·GUI·외부다운로드는 실행하지 않았다. 감사 완료이며 다음 실제 구현은 위 source/API 계약 합의 후 root가 독립 담당을 나눠 진행한다.

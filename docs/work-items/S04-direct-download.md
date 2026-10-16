# S04: SHA256 고정 직접 실행 파일의 관리 설치

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R05, R08
- 담당/수정 범위: diagnostic_cache agent; crates/engine/src/tools.rs 및 tools 하위 신규 모듈/전용 tests, docs/work-items/S04-direct-download.md
- 선행 작업: 관리 root 소유권/digest adapter, S04 공개 공급 증거; HTTP 의존성과 공유 interface는 주 담당 조정

## 사용자 결과

고정된 직접 실행 파일을 제한된 다운로드·hash·version 검증 이후에만 활성화한다. 실패·취소는 이전 설치를 보존한다.

## 계약과 범위

HTTPS와 길이/크기/시간 상한, create-new 소유 staging, streaming SHA256, guardian argv-only version 검사, 새 버전 create-new publication/DB activation을 연결한다. FFmpeg/Deno archive 추출과 UI는 범위 밖이다. Cargo/schema 변경은 주 담당이 조정한다. 외부 설치와 사용자 자료는 변경/삭제하지 않는다. 테스트는 합성 fixture와 명시적인 테스트 transport를 사용한다.

## 인수 조건

정상 size/hash/version에만 activation. 크기 초과/shortread/hash/HTTP/version/timeout/cancel/경로/소유권 실패는 이전active와 외부파일 보존. 제품HTTPS 정책을 localHTTP시험 때문에 약화하지 않는다. 메모리전체body적재/제품shell실행 없음.

## 검증과 handoff

주 담당이 ureq 정확 =3.4.2/default-features=false/rustls만 Cargo/lock에 반영했다. MSRV 1.85는 Rust 1.94.0에 부합한다. rustls/ring/webpki roots 비용과 release 비교는 주 담당이 통합 측정한다. gzip/cookies/JSON/proxy 환경 상속은 사용하지 않는다.

`ToolManager::download_and_install(InstallManifest, expected_bytes: u64, &Arc<AtomicBool>)`을 구현했다. 현재 yt-dlp 직접 executable만 지원한다. HTTPS(redirect 포함), source URL 8192 byte/userinfo·query·fragment 차단, redirect 5, header 64KiB, input buffer 64KiB, download 최대 256MiB와 exact size를 적용한다. 소유 root의 create-new 0700 staging에서 0600 파일을 streaming hash/write하고 sync 후 executable 0700/read-only tracking handle로 봉인한다. guardian에서 `--ignore-config --no-plugin-dirs --version`을 실행하여 정확한 version을 확인하며 자기 파일 변경에 대비해 identity/size/hash를 다시 검증한다. 검증한 exe/manifest를 새 최종 디렉터리로 create-only hardlink하고 Unix 디렉터리 fsync 후 SQLite active를 변경한다. 덮어쓰는 rename이나 recursive 삭제는 없다. DB activation 이전 결과를 active 설치로 반환하지 않는다.

120초 deadline을 transfer와 version/공개 전 단계에 공유하고 version process에는 남은 시간과 10초 중 작은 값을 적용한다. 취소는 단계/읽기 경계에서 검사한다. blocking HTTP read/연결 중에는 남은 HTTP deadline까지 취소 반영이 지연될 수 있다. OS 파일 I/O의 강제 interrupt를 보장하지 않는다. HTTP error 문자열에 URL/원문 headers를 넣지 않는다.

검증(Linux/WSL):

- `cargo build -p dlpad-engine --bin dlpad-process-guard --locked`: helper 빌드 통과.
- `cargo test -p dlpad-engine --lib tools:: --locked`: 직접 다운로드·selection·기존 tools 합계 18/18 통과, real HTTPS opt-in 1개 기본 ignored, 최종 1.44초.
- `cargo clippy -p dlpad-engine --all-targets --locked -- -D warnings`: 통과.
- 공식 pin `2026.08.19`의 40,446,224 byte 실제 HTTPS 다운로드→SHA256→guardian version→post-version hash→임시 root activation opt-in 시험 1/1 통과(초기 5.91초). 최종 config/plugin 차단 argv 재검증은 HTTP 오류 1회(10.22초) 후 동일 상한/정책 재시도로 1/1 통과(6.47초). 원인을 확정하지 않았고 timeout/HTTPS 정책을 완화하지 않았다.

초기 version 실행 시험에서 cleanup identity용 쓰기 clone이 남아 Linux ETXTBSY/guardian readiness EOF가 발생했다. executable을 read-only handle로 봉인해 수정하고 회귀를 통과했다. 실제 loopback HTTP는 private cfg(test) transport만 사용하며 제품 HTTPS policy는 완화하지 않는다. short read/초과/hash/HTTP/body timeout/pre-cancel/read-cancel/expired deadline/version mismatch/version timeout/self-mutating exe/DB trigger failure/소유권·symlink·replacement cleanup 보존을 시험했다.

다른 OS 실행·Windows handle identity cleanup·installer crash staging 복구·느린 연결 취소 latency UI·대체 hardlink 미지원 filesystem·완전한 패키지 notice/source 검수와 앱 관리 UI는 남는다. nonUnix uncertain 실패 산출물은 삭제하지 않는다. 실패 후 만들어진 빈 version 부모 폴더가 남을 수 있다. root를 악의적으로 동시에 바꾸는 사용자 프로세스에 대한 완전한 dir-handle confinement을 주장하지 않는다. 전체 S04/P0 완료를 뜻하지 않는다.

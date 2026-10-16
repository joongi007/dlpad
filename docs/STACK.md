# 기술 스택 결정안

Rust 네이티브, 외부 프로세스 기반 미디어 처리, 이벤트 중심 UI를 기본으로 한다. 개발 workspace는 Rust 1.94.0, Iced 0.14.0과 선택적 eframe 0.33.3을 Cargo.lock에 고정했다. 기본 software, 비교 gpu/alternative feature는 검증용 후보이며 최종 스택은 미채택이다. [실측 결과](validation/S00.md)를 따른다. 최신 버전 자동 추종은 하지 않는다.

| 영역 | 우선 후보 | 선택 이유 / 검증할 비용 |
|---|---|---|
| GUI | Iced, tiny-skia 렌더러 우선 실험 | Rust로 상태·메시지와 테마를 구성. 소프트웨어 렌더러와 wgpu를 동일 화면으로 비교한다. 소프트웨어가 항상 더 작거나 빠르다고 가정하지 않는다. |
| 대안 | egui/eframe, Slint | Iced의 접근성·IME·메모리 목표 실패 시 비교. egui는 도구형 UI와 갱신 비용, Slint는 별도 UI 문법·배포 라이선스 검토가 필요하다. |
| 비동기·프로세스 | Tokio 제한된 feature | I/O·취소·타이머. 작은 고정 워커 수, blocking 작업 별도 제한. |
| 데이터 | rusqlite + bundled SQLite | 단일 로컬 DB, migration·transaction. 임베딩 비용을 측정한다. JSON은 설정/수출용. |
| 직렬화·오류·로그 | serde, serde_json, thiserror, tracing | 명시적 스키마·오류 코드, 제한된 로그 보존. |
| 폴더·OS | directories 또는 플랫폼 API, notify | 문서 폴더/앱 데이터 분리, watcher 이벤트 유실 시 재스캔. |
| 지표 | 플랫폼 API, 필요 시 sysinfo | 전체 시스템을 반복 전수 수집하지 않는다. 지표 의미와 지원 OS 검증. |
| HTTP | reqwest + rustls, 필요한 feature만 | 도구 설치와 선택적 업로드. 미디어 다운로드를 앱 HTTP 계층으로 재구현하지 않는다. |
| 비밀 저장 | keyring + OS backend | Credential Manager / Keychain / Secret Service. 사용 불가 시 세션 로그인. |
| 파일 선택·트레이 | rfd / tray-icon 등 비교 | Linux DBus/GTK 등 숨은 런타임 비용과 배포 의존성을 S00에서 측정. |
| 미디어 도구 | yt-dlp, FFmpeg+ffprobe, 필요한 JS runtime | 별도 버전·소유권·설치 경로를 관리한다. |

Iced의 `tiny-skia` feature는 공식 배포에 존재한다. 실제 workspace에서는 선택 버전의 전체 feature tree를 확인하고 디폴트를 무조건 켜지 않는다. [Iced feature 목록](https://docs.rs/crate/iced/0.14.0/features).

## 빌드 정책

S00에서 정확한 Rust stable 버전을 `rust-toolchain.toml`에 고정하고 rustfmt/clippy를 포함한다. 현재 Rust 1.94.0이 고정되어 있다. Cargo.lock은 앱 저장소에 커밋하며 CI는 `--locked`를 사용한다.

릴리스는 `opt-level = "s"`, thin LTO, codegen-units=1, strip을 시작 후보로 비교한다. `panic=abort`는 크기만 보고 켜지 않는다. 자식 프로세스 정리와 복구 요구를 먼저 검증한다. UPX는 기본 배포에 사용하지 않는다.

네이티브는 WebView를 쓰지 않는다는 뜻이며 모든 OS의 기본 위젯을 그대로 사용한다는 뜻은 아니다. 한글 폰트 fallback·IME·접근성 브리지는 선정 게이트다.

## 현재 의존성 비용

Iced는 default-features를 끄고 tiny-skia/X11/Wayland/thread-pool을 사용한다. GPU는 별도 feature이고 eframe/Glow/AccessKit는 alternative 전용이다. 선택적 의존성이 lockfile에 존재하는 것은 기본 앱에 그 런타임이 포함된다는 뜻이 아니다. 초기 renderer별 uncompressed binary와 RSS는 S00 측정에 기록했다. 완성 앱 크기 목표 달성 주장에는 최종 기능의 재측정이 필요하다.

rusqlite 0.38.0 bundled SQLite는 OS SQLite 편차를 줄이며 C 컴파일과 binary 비용을 추가한다. directories 6은 Documents 조회 adapter, notify 8.2는 native 파일 이벤트와 고정 dirty flag를 사용한다. quick-xml 0.41은 bounded 로컬 MPD 구조만 파싱한다. sha2 0.10은 streaming 도구 digest 검증, fs2 0.4는 앱 데이터 잠금, libc/windows-sys는 필요한 플랫폼 API에 사용한다. 비동기 네트워크·credential store·도구 자동 다운로드 의존성은 아직 도입하지 않았다. CPU/메모리 비용을 숨기는 웹 UI 서버나 WebView는 없다.

## 현재 CPU 통합 후보

Iced actual reader 실패와 Glow RSS 예산 초과 뒤 egui0.33.3/AccessKit 및 직접 egui-winit/softbuffer CPU host를 다음 기능 통합 대상으로 사용한다. [근거와 결정 범위](adr/0001-native-ui.md), [작업 계약](work-items/S00-cpu-stack-direction.md). optional `cpu` feature이며 기본 entry와 최종 Proposed 상태는 유지한다. renderer 신규3crate 다운로드505827B와 Linux 대기RSS49.4MiB/idle0%10초 표본은 일부 화면 비용이다. 모든 P0 화면·3OS·설치·장시간 인수 완료 비용으로 확대하지 않는다.

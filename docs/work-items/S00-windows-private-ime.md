# S00: Windows 격리 화면의 실제 한글 IME

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R01, R06
- 담당/수정 범위: root는 이 문서·후속 통합 및 실행, metrics_recovery는 /tmp/windows_ime_private_capability_plan.md·읽기 감사 및 /tmp/windows_ime_private_probe.cs 초안만 담당한다.
- 선행 작업: 고정 Windows CPU 후보와 private desktop/HWND/원 process identity 관측기.

## 사용자 결과

한글 조합·취소·확정 시 입력과 포커스가 유지되는지 실제 Windows IME 경로에서 확인한다.

## 계약과 범위

사용자 input desktop·전역 키보드·시스템 언어 설정을 변경하지 않는다. 실제 native IME context 및 필요한 capability를 먼저 확인한다. WM_CHAR 합성 입력과 WM_IME 메시지 직접 주입만으로 실제 IME 인수 완료를 주장하지 않는다. 가상 화면 실패 시 visible desktop fallback은 없다.

## 인수 조건

설치된 한글 IME 및 원 앱 context를 관측하고 composition/cancel/commit·한글 문자열·선택/커서/포커스를 확인한다. OS context 경로와 단순 개발 이벤트 seam을 구분한다. 불가능한 capability는 실제 근거를 기록하고 전체 P0 완료로 주장하지 않는다.

## 검증과 handoff

현재 로컬 입력 연결과 실제 native API의 읽기 감사 중이다. 제품 소스·시스템 설정 변경 및 native IME 실행은 아직 없다.

root는 실제조합시험 전에원 app HWND/PID/creation·threadHKL/원HIMC·boundedlocale/조합bytecount를읽는privatecollector 초안을준비한다. 원URL입력의UIA focus만사용하며IME/HKL상태변경과keyboard/layoutload는없다. 이capability관측을실제composition/cancel/commit 완료로표현하지않는다. 설치layout목록을actualTSF/KoreanIME준비완료로오인하지않는다.

첫 readonly private GUI가 통과했다. 원 app HKL68289554·Korean 후보·ImmIsIME=true를 관측했지만 외부 관측자 HIMC는NULL이다. 앱 context 부재를 확정하지 않고 owner UI thread 관측으로 이어간다. root는 기본OFF ime-owner-fixture의 Cargo/entrypoint/software_host 연결을 담당한다. diagnostic_cache는 /tmp/dlpad-ime-owner-fixture.rs, metrics_recovery는 /tmp/windows_ime_owner_private_probe.cs만 담당한다. exact inherited request/ack event와pagefile mapping4KiB를 사용하며 UIthread의파일/pipe I/O 없이 bounded메모리로readonly결과를전달한다. 원nativecontext query/release만사용하며IME/언어/layout상태변경은없다. 실제composition/cancel/commit 인수는 아직 아니다.

## Owner-thread 통합 진행 (2026-10-05)

root는 `crates/app/src/ime_owner_fixture.rs`, `accessible_cpu_main.rs`, `software_host.rs`, `crates/app/Cargo.toml`과 `scripts/windows_ime_owner_private_probe.cs`, `windows_ime_owner_private_native_probe.py`를 통합했다. feature `ime-owner-fixture`는 기본 OFF이고 CPU 후보 전용이다. 기존 windows-sys 0.61.2의 Memory/IMM/HKL/Window binding만 추가하며 새 dependency/version 및 lock 변경은 없다. 시험 feature에서만 프레임당 이벤트 0ms 조회가 추가되고 요청은 한 번만 수행한다. 결과는 원 inherited 4KiB pagefile mapping으로 전달한다. 실제 release 용량 차이는 빌드 뒤 기록한다.

첫 검증은 WSL interop socket 오류로 Windows Rust 실행 전에 실패했다. 권한 내 재시도는 fmt를 통과했지만 Clippy가 HANDLE 숫자 parse의 `map_err`에서 오류 함수 자체를 반환한 컴파일 오류를 발견했다. root가 함수 호출로 수정했으며 원 실패 증거를 보존하고 전체 검증을 다시 실행한다. 두 실패를 성공으로 기록하지 않는다.

원 collector의 직접 생성 manual-reset event/4KiB mapping과 exact HANDLE_LIST만 이번 시험 capability 근거다. fixture가 임의 외부 section의 정확 최대 크기나 event reset mode를 완전 검증했다고 주장하지 않는다. collector 전체 PASS와 owner query PASS를 별도로 검토하며 QUERY_FAILED는 조합 성공이 아니다. 관측자의 원 HWND WM_PAINT 요청은 실제 winit RedrawRequested와 owner paint 경로를 깨운다. 전역 keyboard/layout 변경은 없다.

후속 owner feature의 실제 Windows GNU fmt/Clippy/test/release4명령과 capability실패경로3시험이 모두 통과했다. 원 fixture binary7,881,728B/SHA f88e75bd8ab653e227e47f04d67a27ece6210d4a695f2a325033c1d91ca713c3이며 readonly private GUI도 통과했다. 원 UIthread HIMCpresent=true, HKL68289554/ImmIsIME=true, comp/result0, open=false, context_released=true를 확인했고 자연exit0/Job0/desktop복원을 확인했다. root가 원JSON/현재소스/바이너리/PNG를 대조했다. 조합 인수는 [후속 계약](S00-windows-owner-ime-compose.md)으로 이어가며 기본 feature전체검증과release복원은 진행 중이다. readonly 관측 PASS를 실제 한글 조합 PASS로 표현하지 않는다.

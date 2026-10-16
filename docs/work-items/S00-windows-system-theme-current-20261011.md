# S00: Windows 현재 OS System 테마 인수

- 상태: done
- 분류: verification
- 우선순위: P0
- 요구사항: R06, R08, R11
- 담당/수정 범위: root 실행·통합 및 필요한 Rust/UI 교정; timeline_precision은 /tmp source/evidence 감사와 observer 후보만 준비한다. 같은 파일 동시 수정·Cargo/schema/lock 변경 없음.
- 선행 작업: 현재 CPU whole proof·Light/Dark app 선택/저장 인수

## 사용자 결과

System 선택이 현재 Windows 테마를 첫 화면에 반영하는지 확인하고 실제 OS 변경 실시간 반영과 합성 메시지 시험의 범위를 구분한다.

## 계약과 범위

DESIGN·R06·renderer ADR 및 source/evidence를 대조한다. 사용자 registry/Systemtheme·전역desktop을 변경하지 않는다. 현재OS 값을 읽는 private observer와 합성 설정의 첫 frame을 사용할 수 있다. own window 메시지 주입은 실제OS 변경으로 표시하지 않는다.

## 인수 조건

현재 OS값/query/firstframe 및 선택 유지의 실제 일치 또는 불일치를 기록한다. native HANDLE/Job/source/pixel 범위를 검토한다. 과거 app Light/Dark 선택 시험을 현재 System/liveOS 인수로 확장하지 않는다. 실제 OS변경이 미확인이라면 후속환경 게이트를 유지한다.

## 검증과 handoff

현재 source의 `cpu,first-paint-fixture`를 별도 owned MSVC target에서 fmt/clippy/test/release 실행해 모두 종료0·Job0·원 source 불변을 확인했다. 기본 CPU/설치본을 보존했다. [빌드 root 검토](../validation/evidence/windows-msvc-system-theme-feature-20261011/root-review.json). 진단 binary9568768B는 기본9537024B보다31744B 크며 기본 OFF instrumentation 비용이므로 기본 성능 예산 증거로 사용하지 않는다. 의존성/Cargo/lock 변경은 없다.

`windows_first_paint_private_probe.py --theme System`을 현재 진단 binary SHA f93b03f…303f에 묶어 실제 실행했다. OS reference AppsUseLightTheme=1·high contrast off가 관측 전/원 receipt/소유 메시지 후 모두 같았다. 원 renderer의 첫 successful present는 hidden HWND에서 dark_mode=false/system_theme=Light, 실제background/픽셀RGB255,255,255였고 이후show가 확인됐다. 합성 WM_SETTINGCHANGE는 원 HWND ACK만 검증한다. 자연 종료0·원 Job0·가상 화면 종료·사용자 입력 desktop 불변·원 source/helper/binary 핀을 root가 대조하고 두 sparsePNG를 직접 확인했다. [범위 인수](../validation/evidence/windows-msvc-system-theme-reference-20261011/root-review.json).

이 작업은 현재 안정된 Light OS reference와 System 첫 owner-present의 일치 범위에서 done이다. registry reference만으로 winit 지원/fallback 전체를 증명하지 않는다. 실제 OS theme 변경·ThemeChanged emission·compositor scanout·물리IME/reader는 미검증이며 기존 후속환경 게이트를 유지한다. 이후 Rust 변경의 새 binary로 이 결과를 재실행했다고 주장하지 않는다.

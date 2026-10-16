# S00: Windows 최종 스택의 다음 인수 감사

- 상태: done (읽기 전용 감사)
- 분류: review
- 우선순위: P0
- 요구사항: R06, R08, R11, R13
- 담당/수정 범위: metrics_recovery — 이 문서만
- 선행 작업: STACK/ADR0001/디자인·성능 계약 및 현재 Windows CPU 격리 GUI 근거

## 사용자 결과

기능 구현 반복 대신 최종 renderer/UI 선택의 다음 실제 Windows 인수를 구체화한다. 최초 릴리스 Windows 우선 지시를 적용하지만 한글 입력·접근성·성능 요구를 축소하지 않는다. Rust 네이티브 제약은 Accepted이고 최종 egui/AccessKit/CPU renderer는 계속 Proposed다. 이 감사는 사용자 채택이나 전체 S00 성공이 아니다.

## 계약과 현재 증거

AGENTS/README→PRODUCT→CONTEXT→관련 요구사항→ADR0001 순서와 DESIGN/STACK/PERFORMANCE를 확인했다. STACK은 초기 Iced tiny-skia 후보, 실제 reader 실패·Glow RSS 초과, 후속 CPU 통합을 함께 기록한다. ADR0001의 최종 선정은 OS별 입력·접근성·비용 근거를 요구한다. Windows 우선 변경으로 Linux/macOS 새 출시 인수는 후속이며 기존 Linux 기능 증거는 유지한다.

[Windows CPU native](S00-windows-cpu-native.md)는 원 private Desktop startup/render/ASCII 입력/합성 취소·StopExit/10초 RSS의 당시 binary 성공이다. 실제 한글 IME/reader/최신 전체 peak 비용을 입증하지 않는다. 현재 관리 도구·startup recovery·S10 격리 GUI는 최신 기능별 원 PNG/UIA/worker 종료 근거로 재사용할 수 있지만 그 probe가 측정하지 않은 전체 renderer 비용은 별도다. [Windows runtime 도구 GUI](S04-windows-runtime-tools-gui.md)는 탭/typed Use/잘못된 버전 보존 범위이며 실제 스크린리더 사용성 인수는 아니다.

[Linux scale/paint](S00-cpu-scale-first-paint.md)의 3 warm glyph paint 시간과 Xvfb 200%는 Linux 증거다. `WINIT_X11_SCALE_FACTOR=2`를 Windows DPI 검증으로 옮길 수 없다. `accessible_ime_probe.py`, `accessible_reader_probe.py`는 XIM/Fcitx/AT-SPI/Orca 경로이며 Windows IMM/TSF/UIA/NVDA 증거를 대신하지 않는다. OS theme change와 UI의 Light/Dark 선택도 다른 인수다.

## 남은 게이트의 우선순위와 실행 환경

| 우선순위 | 실제 인수 | 재사용 기반 | private Desktop에서 가능한 범위 / 제약 |
|---|---|---|---|
| 1 | 최신 Windows release의 steady idle resident/private memory·single-core CPU·최대 관측 WorkingSet | `windows_cpu_probe.ps1`의 원 HANDLE/Rss/10초 표본, `windows_runtime_tools_observer.cs`의 standalone native private Desktop/Job/UIA | 원 HANDLE GetProcessTimes와 monotonic elapsed, GetProcessMemoryInfo를 추가한 새 별도 observer로 실행 가능. 기존 RSS 표본은 CPU/private bytes/실행 최대 비용이 아니다. PowerShell policy 실패를 bypass하지 않고 C# native CLI 패턴 사용. |
| 2 | 800×600 logical 및 session 100/150/200%에서 Light/Dark, 오류·report·tools·history 핵심 동작의 시각/키보드 접근 | 기존 Windows UIA/pointer/PNG 및 nested report scroll 실제 collector | 앱 자체 session zoom/theme 선택은 사용자 OS 설정 변경 없이 가능. private HWND만 대상이며 글로벌 SendInput/foreground/clipboard 0. 앱 zoom200%와 실제 OS DPI200%를 구분하고 첫 paint는 실제 glyph 픽셀로 관측해야 함. |
| 3 | Windows 실제 한글 조합 시작·preedit·확정·BackSpace취소·영문 전환·선택 대체 및 후보창 위치 | 현재 Windows matched-key 입력/Value/TextPattern, 기존 Linux IME 결과는 기대값 참고만 | WM_CHAR로 완성 한글을 넣거나 WM_IME_COMPOSITION을 직접 합성하면 actual IME가 아님. private Desktop에서 IMM/TSF 입력 서비스가 실제 동작하는지 먼저 작게 분리 시험해야 함. 입력 데스크톱 전환/실제 사용자 키보드 활성 변경/visible fallback 금지. 서비스가 활성화되지 않으면 환경 미검증으로 기록. |
| 4 | 실제 Windows reader의 제목/상태/버튼·키보드 focus·진행률 알림 과다 방지 | 기존 UIAutomation tree/Text/Value/Invoke/Expanded/Scroll 실제 증거 | UIA semantic client는 독립 실행 가능하지만 actual screen reader가 아님. NVDA/Narrator가 private Desktop의 원 focus/event를 읽는지 별도 feasibility 필요. 사용자 reader 프로세스·기본 Desktop을 조작하지 않으며 실제 reader를 못 실행하면 미검증 유지. |
| 5 | System initial theme·실제 OS theme change, monitor DPI 전환·최종 packaging/driver matrix | readonly OS theme 조회와 현재 설정 UI; 정확 frozen exe hash | System initial 상태는 읽기/PNG 확인 가능. 실제 OS 설정 변경이나 monitor 사용자 작업을 건드리는 전환은 현재 격리 조건에서 실행하지 않음. 수동 theme event 주입을 OS change 성공으로 쓰지 않음. |

DESIGN은 System/Light/Dark 동등 지원, keyboard/한글/RTL/emoji 및 800×600 logical 접근을 요구한다. PERFORMANCE의 idle resident/private 80 MiB 목표와 single-core 평균 CPU 1% 미만을 변경하지 않는다. idle 초과를 전체 machine CPU로 희석하거나 다른 process RSS와 합산해 판정을 바꾸지 않는다.

## 추천하는 다음 단일 수직 인수

**최신 Windows CPU release의 원 private Desktop steady-idle 비용을 실제 UI 상태에 묶어 측정하는 작업**을 먼저 권한다. 화면은 새 private data/media의 다운로드 기본 페이지로 고정하고 실제 startup recovery가 준비 완료이며 worker/tool subprocess 0인 것을 UIA와 원 Job accounting으로 확인한 뒤 3회 fresh launch에서 각각 10초 표본을 얻는다. 이는 지금 사용자 화면이 보이지 않는 환경에서도 가능한 독립 실제 인수이고 실행 공급/계정/미디어 작업에 의존하지 않는다.

새 observer는 기존 원 Process HANDLE/creation FILETIME/assign-before-Resume/kill-on-close Job/private Desktop/PNG/자연 Job0 종료 구조를 재사용한다. 측정은 다음을 고정한다.

1. 원 handle·creation 대조 후 실제 첫 nonuniform glyph paint를 게시한 시점과 startup ready 시점을 따로 기록한다. window 생성이나 UIA node 등장만을 paint 완료로 쓰지 않는다.
2. ready 뒤 안정화 시간을 정확 기록하고 10초 시작/끝의 원 GetProcessTimes kernel+user 누적량과 Stopwatch monotonic 시간을 비교한다. CPU는 `100 × CPU time / elapsed`로 single-core 비율이며 logical processor 수로 나누지 않는다.
3. 같은 원 handle GetProcessMemoryInfo의 WorkingSetSize/PrivateUsage 및 OS PeakWorkingSetSize, 표본 최대 WorkingSet을 각각 이름대로 기록한다. 표본 주기·정확 wall duration·process start→measurement 시점을 저장한다. 표본 최대를 실행 전체 peak로 쓰지 않는다.
4. 80 MiB와 idle1% 목표를 기존대로 비교하되 host ancillary load와 측정 API 실패를 명시한다. 실패를 0으로 기록하지 않는다. executable bytes/SHA, build feature/profile, current source manifest, font/viewport/DPI, 원 PID/creation/Job 증거를 수집한다.
5. 최소 startup/ready/idle-end PNG를 직접 검토하고 원 WM_CLOSE 후 app exit 및 자연 Job ActiveProcesses0/input Desktop 불변을 요구한다. 실패 뒤 Job 강제 정리를 성공으로 승격하지 않는다.

이 작업은 최신 실제 idle gate를 닫을 수 있지만 GUI active peak/10,000 목록 scroll/background·장시간 비용, IME, reader, 200%/테마와 packaging을 동시에 닫지 않는다. 다음 작업은 별도 Windows session200%/Light-Dark 핵심 화면 스크롤 수직 인수이며 그 이후 IMM/TSF 및 actual reader 환경 feasibility를 병렬 독립 조사할 수 있다. 기존 ordinary 기능 구현을 다시 나열하지 않는다.

## 검증과 handoff

이 감사에서는 source 읽기만 수행했다. 새 GUI/Cargo/native/프로세스/시스템 설정 실행 0이며 담당 Markdown 하나만 수정했다. 표의 Windows IME/reader 제약은 실험 전 가능성 구분이고 기능 실패를 확정한 결과가 아니다. root가 S10 GUI를 진행하는 동안 기존 binary/runner/Rust source freeze를 유지했다. 권장 수직 collector는 신규 파일로 작성해 기존 observer를 동시에 수정하지 않고 root가 frozen executable/전체 source를 조정해야 한다. 최종 stack Accepted 전에는 위의 실제 관측 미검증을 남긴다.

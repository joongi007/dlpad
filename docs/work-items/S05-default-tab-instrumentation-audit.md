# S05: 기본 Iced Tab 계측 진입점 감사

- 상태: completed (읽기 감사; native Tab 미검증)
- 분류: validation
- 우선순위: P0
- 요구사항: R01, R07
- 담당/수정 범위: soak_recovery_audit — 이 문서만
- 선행 작업: default 도움말 immutable GUI 증거, native_probe 및 Iced 0.14 소스

## 관측 근거

[첫 실행 provenance](../validation/evidence/p0-default-help-gui-first/binary-provenance.json)의 기본 Iced binary SHA256은 `94e62f13b940c96cea379af8507458859917fd5a9156d6f070c5ca26c0559856`이며 CPU ca8e binary와 다르다. `native_probe.py --page downloads --advanced --seconds 3` 실행은 exit 0, 960×640 private Xvfb였다. [품질 화면](../validation/evidence/p0-default-help-gui-quality/result.json)은 scroll 3회와 정상 회수를 기록하지만 `screen_reader_verified=false`이고 focused widget·caret·입력 값 readback은 기록하지 않는다. 따라서 기존 screenshot 성공은 Tab 성공이 아니다.

`crates/app/Cargo.toml:15`는 Iced `=0.14.0`, x11/wayland/thread-pool이다. AccessKit feature는 별도 CPU의 eframe/egui 쪽에 있다. 기본 Iced에 CPU의 AT-SPI name/FOCUSED/provider가 존재한다고 가정하면 안 된다. 실제 provider 조사도 아직 없다.

`main.rs:1128` URL placeholder는 `https://…`; 이후 저장 경로, 적용/기본 경로 button, `기존 yt-dlp 실행 파일`, `기존 FFmpeg 실행 파일 (선택)`, `품질 높이 (0: 최고 품질)`, 시작/끝, cut pick-list, 자막 button, 고급 옵션 button/입력, 시작/계획 button이 배치된다. 이름은 catalog의 `다운로드 시작`, `실행 계획 미리보기 (옵션 버전은 검증되지 않음)`, `고급 옵션 펼치기`/`고급 옵션 접기`이다. 고급 입력은 `advanced_controls`가 core CATALOG 순서로 생성한다.

현재 `main.rs:2206` subscription에는 window-close 및 fixture event 통계가 있으나 Tab/ShiftTab을 `iced::widget::focus_next/focus_previous`에 연결하는 처리나 keyboard message가 없다. 설치된 `iced_widget-0.14.2/src/text_input.rs:1051` 이후 Named key 처리는 Enter/삭제/방향키/Escape이고 Tab 처리 항목이 없다. 설치된 `iced_runtime-0.14.0/src/widget/operation.rs:50`와 `:55`에 focus_previous/focus_next API는 존재한다. 이는 기본 Tab 정책 누락 가능성의 코드 근거이며 실제 runtime 실패로 단정하지 않는다.

## 다음 실행 가능한 최소 시험

새 전용 `scripts/default_tab_navigation_probe.py`에서 기존 `probe_display`, `OwnedDisplay`, `focus_owned_window`, `key_sequence`, `screenshot`, bounded logs 및 SQLite snapshot을 재사용한다. 위 immutable binary, --fixture, 합성 tool, 0700 임시 data/media/cache, private Xvfb를 사용한다. 처음 URL 좌표만 클릭하고 `x`를 입력해 실제 글자/caret screenshot을 확보한다. 이후 Tab 한 번, `y` 한 번, screenshot, BackSpace 한 번, Shift_L+Tab 한 번, screenshot을 순서대로 기록한다. grab-focus나 후속 pointer 클릭으로 실패를 구조하지 않는다. URL에 `xy`가 남으면 Tab이 입력 포커스를 이동하지 않은 실제 증거이고, 저장 경로에 `y`가 들어갔다면 텍스트 이동의 제한된 시각 증거이다. 별도 root screenshot 검토 전에는 focus/input readback PASS를 부여하지 않는다.

최대 32 Tab을 추가 관측하되 각 키의 PNG와 fixture-events를 보존한다. tool 실행 marker·job 생성 금지, exact PID/start·binary 전후 hash·회수는 필수이다. 이 단계는 executable capability/failure 수집이며 native accessible name을 얻지 못하면 keyboard navigation verified를 false로 유지한다. Clipboard selection-copy/OCR 추측/CPU AT-SPI labels를 값 readback으로 대체하지 않는다.

## Start·계획·고급 인수에 필요한 최소 추가 계측

현재 fixture_trace는 event kind/captured만 최대 64개 기록하며 text/key/좌표/대상 widget을 의도적으로 제외한다. Tab key delivery와 capture는 기록할 수 있어도 어느 control의 focus인지, caret index나 실제 App field가 무엇인지 증명하지 못한다. 따라서 전체 controls에 대한 자동 PASS 수집기는 현재 API만으로 만들 수 없다.

root가 제품 변경을 승인한다면 fixture-only bounded typed snapshot에 실제 widget ID/focus 상태와 합성 입력 field readback을 추가하는 별도 scope가 필요하다. text input마다 고정 ID와 Iced focus operation으로 실제 focus 상태를 읽고, synthetic fixture에서만 field enum/value 또는 비교 digest를 기록한다. 일반 사용자 text/URL/cookie 값을 기록하지 않는다. button의 키보드 focus 지원과 Tab 순서를 별도로 확인해야 하며 focus operation 호출 사실만으로 focused라고 주장하지 않는다.

그 이후 실제 Tab으로 고급 button 도달→Return으로 펼침 상태 변경→세 고급 입력 순서/readback→계획 button 도달→Return으로 실제 preview 생성(도구 미실행)을 인수한다. Start는 실제 focus와 enabled 상태까지만 확인하고 Return을 누르지 않는다. 추가로 synthetic download 시작을 검사하려면 별도 task에서 실제 Start activation/job/tool 실행을 명시적으로 허용해야 한다. 화면 밖 focus는 실패로 보존하고 소유 scroll/시각 표시의 제품 보완은 root 담당이다.

## 검증과 잔여

이 작업에서는 문서와 로컬 의존성/source/evidence만 읽었다. Rust/Cargo/helper 변경, GUI 실행, clipboard 및 사용자 desktop 조작은 하지 않았다. 다음 실행은 위 두-input capability probe이며 자동 focus/caret readback, 전체 Start/계획/고급 traversal, screen reader 및 다른 OS는 미검증이다.

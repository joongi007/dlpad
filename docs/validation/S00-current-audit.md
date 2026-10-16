# S00 현재 스택 게이트 감사

2026-10-02, diagnostic_cache agent. 읽기 전용 감사 및 전용 문서 2개만 작성했다. **판정: S00 전체 FAIL/미충족. Rust 네이티브 제약은 Accepted지만 Iced/tiny-skia 최종 채택은 Proposed를 유지해야 한다.** 기능 구현량과 CLI 시험은 실제 입력·접근성 검증을 대신하지 않는다. [계약](../work-items/S00-current-audit.md), [ADR-0001](../adr/0001-native-ui.md), [최초 측정](S00.md), [격리 IME](isolated-ime.md)를 따른다.

README→PRODUCT→CONTEXT→REQUIREMENTS/ROADMAP→ADR-0001을 읽고 DESIGN/ARCHITECTURE/관련 검증을 대조했다. 정상 Git 저장소가 없으며 `git status --short` exit128, 기존 .git을 그대로 보존했다. desktop GUI/새 build/제품 코드·공통문서 변경은 수행하지 않았다.

## 현재 결과

| 게이트 | 관측/근거 | 판정과 정확 범위 |
|---|---|---|
| Rust 네이티브 UI | app Cargo iced=0.14.0 default_features=false, software→tiny-skia 기본; 웹 UI runtime 없음 | PASS 구현 제약. 최종 toolkit 승인은 별개 |
| Linux software renderer | 최초 960×640 실제 창/한글 glyph, RSS67724KiB; 최신 final-background 같은 창 hidden0→restore2/worker 계속 | PASS 해당 X11/100%/시험 binary. Wayland/driver/배율 matrix는 미검증 |
| GPU renderer | iced-gpu.json exit101, GL 재확인도 surface 오류 | FAIL 해당 WSLg 환경. 다른 GPU/OS까지 실패로 일반화하지 않음 |
| 한글 실제 조합/commit | Fcitx5 Hangul 물리 ASCII gksrmf→한글, BackSpace 조합취소/영어전환 기록 | 부분 PASS 이전 binary/격리 XIM. 최신 binary 재현 없음 |
| 읽을 수 있는 inline preedit | 독립 Fcitx Input Window와 앱 preedit를 분리, baseline probe exit1/preedit=false | FAIL. glyph/clipboard/외부 IME 창으로 대체 불가 |
| 스크린리더 | 기존 renderer JSON 모두 screen_reader_verified=false; 실제 reader/tree 증거 없음 | NOT VERIFIED/필수 미충족 |
| keyboard-only/focus/200%/RTL/emoji/reduced motion | DESIGN 요구, 기존 click/ASCII 입력 시험은 일부 상호작용만 확인 | NOT VERIFIED 필수 시각/접근성 matrix |
| System 첫 프레임/실시간 OS theme | main ThemeChoice::System→None, Light/Dark palette와 저장 정책 존재 | NOT VERIFIED 실제 OS theme event/첫 프레임 |
| 성능/배포 비용 | 초기 renderer 비교/짧은 idle 표본, 최신 binary9882048B | 부분 PASS 크기/단일 RSS관측. 장시간 GUI idle·scroll·배터리 예산 미충족 |
| Windows | 실제 GNU CLI 공통검증/프로세스·identity 증거 | PASS 해당 CLI snapshot. GUI renderer/IME/스크린리더/MSVC/패키지 아님 |
| macOS/3OS remote CI | macOS 실행 증거 없음, remote Actions 실행 안 됨 | NOT VERIFIED |

최신 software 시험 executable SHA256 `e9fa3ce36b3e4a1285ef0ee5930179ee4fa126451730fd0cbdcb2bae020bcc7f`, 9,882,048 bytes를 이번 감사에서 `sha256sum`/`stat`으로 재확인했다. [최종 background JSON](background-ui/final-background.json)의 같은 hash와 일치하며 visible68576/hidden70176/restored70184KiB는 coarse 단일 표본이다. 숨김이 GPU/window 자원 해제나 메모리 감소를 뜻하지 않는다. 초기 software6,226,232B/egui8,797,248B 숫자는 서로 같은 초기 목록 비교이고 최신 기능 앱과 renderer 우열을 직접 비교하지 않는다. 최초 JSON에는 binary SHA 필드가 없으므로 지금 존재하는 executable에 그 결과를 붙일 수 없다.

## 실제 로컬 증거 대조

이번 감사에서 JSON을 파싱하고 파일 SHA256을 계산했다. 아래 값은 **evidence 파일 hash**이며 실행 binary hash와 구별한다.

| evidence | SHA256 |
|---|---|
| [iced-software.json](iced-software.json) | `9ccc34d8359d3ea985e71771f2f125e23716cb520be6e1505b7e9adf64288512` |
| [egui-glow.json](egui-glow.json) | `cb7fd47ba7cc7ff447dc0440bf0c7f73059be54bf2af16439113f608b7a93bfd` |
| [iced-gpu.json](iced-gpu.json) | `3cbc3e304aef8f5e340df0ff1dfb43f8cf7eb00e00886db545a6c7078e9be667` |
| [final-background.json](background-ui/final-background.json) | `68f6a47e4de118d629fe4e72fde8e949438b3ec6f3b221c9f3e23b121a31c6f6` |
| [final-processing.json](background-ui/final-processing.json) | `413f4dd2364f1b6387150a19712bd0dcab3545442aa7fbe4f3bd37c37a75c349` |
| [Windows 초기 최종 CLI JSON](windows-cli-final.json) | `df5214c001a1670039de7d6ab6b92595b2dab5b321077333eca351db93e63ef2` |

`/tmp/dlpad-ime-probe-20261001-*`는 현재 존재하지 않는다. 따라서 [IME 기록](isolated-ime.md)의 commit/preedit 보고는 남지만 원본 PNG/window inventory/config를 현재 다시 검토할 수 없다. `.local-tools/ime/root/usr/bin/fcitx5`, `.local-tools/xvfb/usr/bin/{Xvfb,proot}`, scripts/ime_probe.py 존재는 확인했다. 최신 binary의 IME 성공·실패를 새로 실행했다고 보고하지 않는다.

Pinned local iced0.14.0/iced_winit0.14.1 소스와 Cargo에서 `rg accesskit|accessibility` 일치0건. alternative eframe은 accesskit feature를 갖지만 제품 기본은software다. 이것은 bridge 부족 조사 근거이며 OS가 절대로 읽지 못한다는 증명이 아니다. 실제 tree/role/name/state/focus와 읽기를 관측해야 한다.

## 다음 구체 게이트

1. 최신 후보 binary/hash를 고정하고 owned Xvfb/private D-Bus/Fcitx의 scripts/ime_probe.py로 commit/cancel/Latin transition과 inline preedit를 별도 판정한다. 현재 XIM style/context/event 경계를 작은 재현으로 추적하고 읽을 수 있는 preedit/정확한 commit 및 원시 증거를 저장한다. Wayland·Windows TSF·macOS IME는 각각 실제 OS에서 검증한다.
2. Linux 격리 accessibility bus/reader에서 tree/제목/입력/시작/상태의 role/name을 읽고 keyboard-only Tab/ShiftTab/activation/focus를 확인한다. bridge가 없으면 Iced 구현 또는 다른 native toolkit의 같은 화면/비용을 비교한다. AccessKit feature만으로 통과시키지 않는다.
3. 100/150/200%·800×600 logical 화면의 주요 조작, 긴 제목/RTL/emoji/색상 외 상태/reduced motion을 검사한다. OS System theme 첫 프레임과 실행 중 변경을 따로 관측한다.
4. 같은 기능/fixture/폰트/renderer에서 idle·scroll·active/hidden을 비교하고 앱/외부 tool RSS·CPU를 분리한다. 5초 표본이나 headless guardian soak로 GUI/8시간 다운로드를 증명하지 않는다.
5. Windows/macOS 독립 VM/시험 세션에서 GUI·IME·reader·배포 기동을 검사한다. private CLI 성공을 GUI 입력에 적용하지 않는다. visible desktop 전환/사용자 입력 주입을 자동으로 시도하지 않는다. 정상 Git/remote와 3OS CI 실행은 별도 증거다.

주 담당이 실패를 해소한 결과로 ADR을 Accepted로 전환하거나 다른 native stack으로 수정하기까지 개발 default tiny-skia는 가역적인 선택이다. 이 감사는 사용자 확인이나 스택 확정을 수행하지 않는다.

## 실행 검사

rg 요구사항/ADR/evidence/의존 소스 검색, JSON parse/Python hashlib SHA256, sha256sum 및 stat executable 대조, git status(exit128), localtool/IME 원시 artifact 존재 확인을 실행했다. 코드 변경이 없어 Rust build/test를 반복하지 않았다. docs checker122개 통과.

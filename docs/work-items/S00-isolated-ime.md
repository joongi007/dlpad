# S00: 격리된 X11 실제 한글 IME 조합

- 상태: partial
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R10, S00
- 담당/수정 범위: guardian_fault; 신규 scripts/ime_probe.py, docs/validation/isolated-ime.md, .local-tools/ime/ 전용 도구
- 선행 작업: S00 software renderer spike / isolated Xvfb

## 사용자 결과

Rust native 입력에서 한글 키 조합·preedit·commit·취소를 실제 IME로 검증한다. 한글 glyph 표시나 붙여넣기를 IME 성공으로 기록하지 않는다.

## 계약과 범위

AGENTS/README 문서 순서·관련 ADR와 DESIGN을 읽었다. 기존 app/Cargo/native_probe/x11 helper 파일을 수정하지 않는다. native_probe.probe_display로 소유 Xvfb를 사용하며 실패 시 사용자 desktop으로 fallback하지 않는다. private D-Bus와 임시 XDG config/data/cache/runtime만 사용하고 사용자 Xauthority/IME설정·HOME을 바꾸지 않는다. 프로젝트 전용 IME package extract는 authorized 범위이며 관리자/system 변경은 없다. 기존 .git을 보존한다.

## 인수 조건

- 실제 Hangul IME key compose→preedit→commit을 소유 Xvfb에서 관측한다.
- 조합 중 취소의 결과를 관측하고 입력 손실/중복을 확인한다.
- 정상 영어/한국어 전환과 입력 결과를 clipboard 읽기 또는 저장된 제품 동작으로 검증한다.
- 소유 GUI/IME/private D-Bus만 종료한다. 8시간 session 57344는 유지한다.
- 실패 시 renderer/IME/XIM wiring 원인을 구분하고 S00 stack 결정 evidence를 기록한다.

## 검증과 handoff

환경에 ibus/fcitx/libhangul은 없고 dbus-daemon은 설치되어 있다. Ubuntu Jammy package index에서 fcitx5 5.0.14-1, fcitx5-hangul 5.0.7-1 후보를 확인했다. 프로젝트 전용 extract를 검토한다. 실제 실험 전 성공으로 기록하지 않는다.

## 실제 결과와 남은 gate

실제 Fcitx5 Hangul 조합 commit, BackSpace 조합 취소, 영어 전환, GTK XIM Escape 비교를 수행했다. commit/cancel/transition은 통과했으나 inline preedit는 실패하여 probe exit 1을 유지한다. 외부 Fcitx Input Window와 Iced 자체 preedit를 구분했다. 효과 없는 drawordering 가설과 bounded 임시 trace를 제거하고 upstream 원본을 복원했다. 주 담당이 Cargo path patch/lock 복원을 조정한다. 상세 증거와 정확한 버전/도구 비용은 docs/validation/isolated-ime.md에 있다. S00 전체/다른 OS를 완료로 보고하지 않는다.

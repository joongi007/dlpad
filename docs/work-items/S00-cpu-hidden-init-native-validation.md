# S00: CPU hidden-init 후보 native 재검증

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R11, S00
- 담당/수정 범위: diagnostic_cache; 신규 docs/validation/cpu-hidden-init-native-validation.md 및 전용 cpu-hidden-init-native evidence, 기존 검증 harness는 읽기만 재사용
- 선행 작업: root Linux hidden-init CPU release/fmt/clippy/10 tests 통과

## 사용자 결과

hidden-init 바이너리에서 실제 한글 IME·무음 Orca 이벤트·warm 첫 native 픽셀·200% 핵심 접근성을 다시 확인한다.

## 계약과 범위

binary/source/Cargo frozen. 수정 전 source 및 바이너리 SHA를 전용 source-before.json에 기록한다. 기존28b4 raw 증거를 보존한다. owned Xvfb/private bus/temp data/media/XDG만 사용하며 사용자desktop/clipboard/audio 금지. root는 동일binary perf, 다른 agent는 Windows 담당으로 파일·화면을 공유하지 않는다.

## 인수 조건

실제 레이아웃을 먼저 확인하고 PNG/JSON/로그·시각검토 분리·끝hash를 보존한다. first native glyph 1초 기준을 완화하지 않는다. CPU-only Linux 결과를 전체reader/다른OS/전체P0 합격으로 확대하지 않는다. 소유 직접 프로세스를 회수하고 임시 fixture를 삭제한다.

## 검증과 handoff

새1d4757…15b5 동일source/binary에서 actualIME/무음reader/startup/200% 재검증 완료. 3 warm 첫glyph0.228988/0.186718/0.160565초(각1s이내). 원IME collector exit1/시각검토 구분, 보고서 마지막줄scroll/전체reader/다른OS는미검증. 모든소유직접GUI/process 회수·temp제거. 전용문서 docs/validation/cpu-hidden-init-native-validation.md.

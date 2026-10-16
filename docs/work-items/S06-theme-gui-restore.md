# S06: 저장한 테마의 실제 화면 복원

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R06
- 담당/수정 범위: 주 담당; scripts/theme_restore_probe.py 및 전용 증거/본 문서
- 선행 작업: theme SQLite 저장 및 latest release

## 사용자 결과

명시적 Light/Dark 선택을 다음 실행의 실제 화면에 복원하며, SQLite 설정이 legacy 파일보다 우선한다.

## 계약과 범위

기존 native_probe.py read-only 재사용, 임시 app data/합성 fixture/별도 Xvfb와 private DBus만 사용한다. 제품/Cargo 수정 없음. 최초 관측 화면은 안정화 이후이며 첫 renderer frame 또는 OS System 변경을 증명하지 않는다.

## 인수 조건

SQLite Light/Dark 및 legacy Dark 실제 background 픽셀과 소스 SHA를 확인한다. 손상/큰 legacy 값은 기본값으로 동작하고 fixture bytes를 유지한다. owned GUI/display/bus/temp 정리. 다른 OS/실제 System 변경은 별도 게이트 유지.

## 검증과 handoff

`DLPAD_TEST_FONT=/mnt/c/Windows/Fonts/malgun.ttf python3 scripts/theme_restore_probe.py target/release/dlpad docs/validation/theme-restore/current` exit0. SQLite Dark/Light가 반대 legacy 값보다 우선했고 RGB30/30/30 및255/255/255를 확인했다. legacyDark fallback와4096B초과legacy 기본값도통과했고 원legacybytes/ownedGUI/bus/temp 정리를 확인했다. [실제 결과](../validation/theme-restore/current/result.json). 안정화2초후화면이며 firstframe/SystemOS변경미검증.

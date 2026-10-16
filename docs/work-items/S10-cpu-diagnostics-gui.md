# S10 CPU native diagnostics actual GUI

- 상태: completed
- 분류: verification
- 우선순위: P0
- 요구사항: R13, R01
- 담당/수정 범위: guardian_fault; 신규 scripts/cpu_diagnostics_probe.py 및 전용 docs/validation/evidence/cpu-diagnostics/
- 선행 작업: frozen CPU SHA8fb4a67c27ff89fd883ff6fddaa9136a5fc36c62d74c049665543a513ab06986

## 사용자 결과

합성 실패 Attempt에서 실제 진단 보고서 정제/편집/제외/MPD구조 첨부/내보내기와 원본 보존을 격리 GUI로 확인한다.

## 계약과 범위

제품/Cargo/sharedscripts 불변. 기존 helpers read-only import. owned Xvfb/private D-Bus/AT-SPI 및0700temp XDG/data, 사용자 화면/클립보드/계정/미디어/외부전송금지. 원본 합성 MPD≤1MiB,로그≤64KiB,보고서≤2MiB,시험 프로세스만 정리. 8h 실행 유지.

## 인수 조건

Failed DB Attempt 실제생성; report확장/MarkdownJSON/편집 재정제/개별전체제외; MPD소스보존+DTDoversize실패+구조제외;create-only export성공/충돌원본보존;privateX11clipboard실제읽기;CloseWait/ownedbuscleanup;원시실패PNG/JSON보존. 검증못한 항목은명시.

## 검증과 handoff

구현과 actual bounded experiment 후 source/binary before-after 및 전용validation 기록.

2026-10-03 실제 finalfull-safe-words exit0. 로그3→편집재정제2→개별제외1→전체0, MPD개별/전체제외, DTD/크기초과fail/source보존, 새export/충돌보존, CloseWait0/channelcleanup/DBFailed1attempt 실제검증. 원시하네스실패와부분trial 보존; 상세 cpu-diagnostics-validation.md 참조. Windows S10 실제GUI는미검증.

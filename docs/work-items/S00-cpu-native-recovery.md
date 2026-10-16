# S00 CPU 한글 IME·무음 reader 재검증

- 상태: done
- 분류: verification
- 우선순위: P0
- 요구사항: R08, R11 (Linux native 입력/접근성 부분)
- 담당/수정 범위: stress_recovery; scripts/cpu_native_recovery_probe.py, 이 문서 및 전용 evidence/validation. 제품/shared harness/Cargo/README/P0status 변경 금지.
- 선행 작업: root frozen CPU166946…, 기존 accessible-native-validation과 프로젝트 전용 Fcitx/Orca 존재

## 사용자 결과

현재 CPU 후보에서 실제 한글 조합 확정/취소/Latin전환/선택대체와 keyboard focus·reader event 처리를 확인한다. screenshot만으로 전체 입력 또는 reader 성공을 판단하지 않는다.

## 계약과 범위

AGENTS→README→PRODUCT→CONTEXT→R08/R11→ADR0001, DESIGN 및 기존 native validation/harness를 읽었다. Linux private Xvfb/DBus/temp XDG/합성 media만. 기존8h PID/사용자 desktop·clipboard·audio를 건드리지 않는다. IME는 기존 named field extents를 찾는 accessible_ime_probe.run을 readonly import하고 raw exit1/preedit 미판정은 보존한다. reader는 과거 fixed(300,102)을 재사용하지 않고 실제 namedrole79/extents를 조회한다. Orca는 공식모듈 main/private prefs/speech·braille·sound·mouse review false/2MiB log 상한으로 실행한다.

## 인수 조건

정확 frozen SHA/실제 key input/GetText 기대값/한글 label·entryrole·focus state/Tab후 focus·Orca 실제text changed 및 keyboard locus 처리/소유 종료 증거. 필요한IME/reader환경 부재는 미검증으로 분리하고 독립가능 case를 수행한다. 한글preedit 시각판정은 별도review와 실제text증거로 기록하며 raw IME 이벤트를 추측하지 않는다. 음성·전체readerUX·다른OS·P0릴리스 완료를 주장하지 않는다.

## 검증과 handoff

준비 단계이며 전용도구 존재/readiness를 root에 보고했다. 구현/실행 결과와 미검증은 아래 후속 기록으로 갱신한다.

## 완료 결과 (166946… Linux 기본배율 제한)

current-1 wrapperexit0/제한scopePASS. 실제 ASCII/한글확정/취소/Latin/선택대체 GetText, preedit Text와별도직접PNG 판정, readerfixture entry-focused/Tab→다운로드, 공식무음Orca registry/textchanged/keyboardlocus처리·exit0 확인. rawIME exit1/시각미판정값과전체reader/audiofalse 보존. host reader PID/start 및ownedruntime 잔류없음직접확인. [검증과원시근거](../validation/cpu-native-recovery-validation.md)를 따른다. 이후 metadata source/newrelease를 이 old snapshot으로 검증완료라하지않는다. 제품·shared collector·Cargo 무변경; 다른OS/전체배율/reader사용성은별도미검증게이트다.

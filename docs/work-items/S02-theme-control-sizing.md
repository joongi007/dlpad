# S02: 테마별 입력 크기 일관성

- 상태: done
- 분류: fix
- 우선순위: P0
- 요구사항: R06
- 담당/수정 범위: root; accessible.rs configure_context, 본 문서. diagnostic_cache는 settings GUI/IME harness 전용, library_identity는 Windows CLI 전용 증거.
- 선행 작업: S02-cpu-settings-integration, DESIGN 버튼/입력 기본32px

## 사용자 결과

Light/Dark/System 변경과 저장 테마 재실행에도 기본 선택 버튼 크기32logicalpx를 유지한다.

## 계약과 범위

popup-fixed raw의 Light재실행 실제 ComboBox 높이19px, Dark32px를 root가 발견했다. configure_context가 현재 theme style만 수정했기 때문이며 egui all_styles_mut로 양쪽 style에 같은 spacing을 적용한다. Cargo/schema/새의존성 없음.

## 인수 조건

fmt/clippy/tests/release 후 실제Light/Dark/System재실행 버튼높이>=32px 확인. settings 이동한field의 실제한글IME 별도검증. 단위시험만으로OS/GUI완료주장하지 않는다.

## 검증과 handoff

Linux fmt/CPUclippy exit0(17.30초), CPU entry38시험 PASS, release36.24초. 최종 SHA256 `f052fa5b3803c06b97f82ef360b1ad11776c6c3b0854e9fc20e00381faa75f19`,10486680B. CPU workspace268 passed/0 failed/5 ignored. WindowsGNU CPUclippy/tests/release188passed/1ignored, SHA `d210a7828f384d4422d456c103c6acdaaf5002ae1b0b156d6b0ad91b291bbebf`,5553152B; root가238개현재입력과exe를독립대조했다.

[실제 GUI](../validation/cpu-settings-validation.md) 새styles32 prefix의 Light/Dark/System ComboBox높이32/32/32px,200%높이64px를root가raw extents로확인했다. 테마3개저장/재실행·메뉴닫힘·200%양방향Tab·잠금중종료/선택보존도통과. 실제privateFcitx의preedit.png를root가직접보아한[글]과조합caret를확인했고확정/취소/영문전환/선택교체true를대조했다. 원자동IMEcollector exit1/preeditfalse/imefalse는보존하며시각검수기반관측과구분한다.

[대기 비용](../validation/accessible-performance/cpu-settings-styles32/result.json)은RSS50820KiB/CPU0.09984%/10.016초,ownedGUI/bus/temp정리확인. 설정첫표시0.216–0.222초는first observed painted frame이며정확첫presentation이나busyDB최대5초경로인수가아니다. System OS 실제변경/WindowsmacOSGUI/전체reader는남는다. TextEdit input은21px여서32pxmin_size 개선을다음typed옵션통합으로넘긴다.

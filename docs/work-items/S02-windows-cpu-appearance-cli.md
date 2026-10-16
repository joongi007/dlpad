# S02: Windows CPU 테마 통합 CLI 검증

- 상태: done
- 분류: regression
- 우선순위: P0
- 요구사항: R06, R11, S02
- 담당/수정 범위: library_identity — 신규 workitem/전용 validation 증거만
- 선행 작업: rootCPUsettings/readonlyStore/catalog/sharedentry 연결 및sourcefreeze

## 사용자 결과

테마 모듈과 새 읽기전용Store API가 WindowsGNU CPU기능에서실제로컴파일·시험·release되는지 확인한다.

## 계약과 범위

기존프로젝트Windowsnative runner읽기전용사용,Windows전용cargo-home/target/tmp/privateCLIJob. source/Cargo/runner수정·GUI·clipboard접근없음. before/after 및작업간입력/runner/lock/binarySHA보존.

## 인수 조건

clippy/test/build모두실제exit0,원시시험개수와read-onlyStoreWindowsfailurepath범위확인. source변경시새입력으로소급성공금지. 실제themeGUI/NOFOLLOWsymlink권한제약/3OS는검증결과와분리한다.

## 검증과 handoff

실제3검사exit0/188passed1ignored, source238+runner/lock불변, 새CPUexeSHA READY를root에게전달했다. docs/validation/windows-cpu-appearance.md 및원시증거보존. WindowsNOFOLLOWfinalsymlink는cfgUnix시험뿐이라미검증유지.

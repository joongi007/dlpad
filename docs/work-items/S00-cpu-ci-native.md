# S00/R11: CPU 후보의 3 OS CI 계약과 Windows native 검증

- 상태: in-progress
- 분류: verification
- 우선순위: P0
- 요구사항: R10, R11, S00
- 담당/수정 범위: 주 담당 .github/workflows/ci.yml/scripts/windows_native_probe.py/본 계약; library_identity CPU Windows 전용증거만
- 선행 작업: CPU Linux10tests/clippy/release/actualGUI/IME/reader 및50MiB대기실측

## 사용자 결과

CPU native 후보가 선택되지 않은 기본검사에서 빠지는 일을 방지하고3OS CI에 후보컴파일·lint·tests를명시한다. 현재사용가능WindowsGNU에서실제선택feature도검증한다.

## 계약과 범위

remote CI/공개배포/서명/실제desktop없음. runner feature는cpu/alternative고정choices이며shell문자열없이argv만추가한다. clippy의--separator앞에feature를전달한다. 폴더/소유nativeJob/timeout/취소/정확입력해시기존계약유지. sharedCargo제품source동결후readonlyWindowsprivateCLI에서검증한다. CI설정추가를실제macOS/MSVC성공으로표현하지않는다.

## 인수 조건

default명령argv기존유지, CPU --features선택포함실제native검사exit0/source불변. YAML3OSmatrix후보lint/test/release포함. 실제GUI와CPUWindowsIME/reader·macOS/MSVC는별도게이트로기록.

## 검증과 handoff

구현후py_compile/문서검사와Windowsnativefeature수직검사. 완료전원격CI성공으로쓰지않는다.

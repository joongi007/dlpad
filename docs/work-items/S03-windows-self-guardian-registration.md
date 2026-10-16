# S03: Windows 앱의 headless guardian 등록

- 상태: in-progress
- 분류: integration
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — process.rs, windows_guardian_registration.rs, 이 문서
- 선행 작업: 기존 Windows headless entry와 독립 helper protocol

## 사용자 결과

Windows 제품 앱은 현재 실행 파일의 headless entry를 helper로 재사용한다. 독립 engine 개발 시험은 같은 build directory의 guard binary를 사용한다. 없으면 보호 없는 실행으로 fallback하지 않고 실패한다.

## 계약과 범위

OnceLock current executable 등록. argv/shell/Cargo 의존성 변경없음. desktop/UI를 실행하기전 guardian_entry dispatch가 이미 존재한다. helper 하나/4pipe 비용은 별도측정한다.

## 인수 조건

등록파일우선/독립enginehelper경로/누락failclosed 및 실제 Windows native lint/build. GUI sameapp launch는 별도시험한다.

## 검증과 handoff

구현전상태. 순수경로시험은 native프로세스동작대체가아니다.

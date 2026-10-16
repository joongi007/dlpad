# S03: Windows canonical 경로 prefix 검사

- 상태: in-progress
- 분류: bug
- 우선순위: P0
- 요구사항: R02, R05, R08, R11
- 담당/수정 범위: 주 담당 engine/instance.rs; tools.rs는 diagnostic_cache 별도 담당
- 선행 작업: S00-windows-cli 실제 native GNU test

## 사용자 결과

Windows canonical 경로의 bare drive prefix를 파일로 조회해 정상 데이터 폴더를 거절하던 오류를 수정한다. symlink·parent traversal 검사는 유지한다.

## 계약과 범위

Windows canonicalize의 verbatim prefix를 누적하되 RootDir가 붙기 전에 filesystem 조회하지 않는다. 기존 소유권/locking 계약과 Unix 동작 유지. nativeWindows 테스트 실패 OS code1을 근거로 한다. 공유Cargo/schema 변경없음.

## 인수 조건

native Windows duplicate lock/release/marker·관리 도구 테스트 회귀. Linux 기존 실패 경로 유지. 모든 Windows/MSVC 출시 게이트 완료로 확대하지 않는다.

## 검증과 handoff

nativeGNU app12/core20/engine34pass11fail 최초 기록에서 instance2/tools9 실패. 담당별 수정 후 Windows 에이전트가 실제 재실행하고 주 담당이 통합 기록한다.

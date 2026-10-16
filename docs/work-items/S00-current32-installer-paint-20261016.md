# S00: 현재 설치기 명령 버튼의 실제 GDI 회귀

- 상태: done-scoped
- 분류: validation
- 우선순위: P0
- 요구사항: R06, R08, R11
- 담당/수정 범위: Root / 새 전용 collector·증거와 이 문서. 제품·공유 Cargo/lock 변경 없음.
- 선행 작업: whole32 source675·원 MSVC 설정·원3PE 일치

## 사용자 결과

설치기의 포커스/호버/pressed/disabled 표시를 현재 소스의 실제 Windows GDI painter로 검증한다.

## 계약과 범위

기존 ignored command_paint_memory_tests 세 개를 원 Windows private host에서 명시 실행한다. 96/192 DPI 인수의 메모리 DIB·Light/Dark·filled/neutral·native enabled gate와 로컬 HC fallback 분기다. OS DPI/HC 설정이나 전역 pointer를 변경하지 않는다. 물리 hover 전달·전체 설치 화면 캡처를 이 시험으로 대신하지 않는다. Cargo는 원 whole32 환경/target/cache를 재사용하고 대상 dlpad-install test만 수행한다.

## 인수 조건

현재 source675/원3PE/lock 불변, 실제3 PASS/0 FAIL/0 ignored와 native0/자연Job0/private 종료/input desktop 보존. 원 로그·collector·입력 hash 보존. 실패를 같은 이름 재시도로 덮지 않는다.

## 검증과 handoff

`python3 -B .local-tools/session-recovery-20261013/current32-installer-paint-root-v1/collect.py` 실제 exit0. 원 MSVC 환경에서 `cargo test --locked --offline --target x86_64-pc-windows-msvc --features cpu -p dlpad --bin dlpad-install native::command_paint_memory_tests:: -- --ignored --test-threads=1 --nocapture`를 실행했다. 실제3 PASS/0 FAIL/0 ignored/44 filtered, Cargo 0.49초·시험0.02초. native 자연0/Job0/private desktop 종료/input desktop 불변, source675·원3PE/lock 불변을 확인했다. [Root 검토](../validation/evidence/current32-installer-paint-private-v1-20261016/root-review.json).

실제 GDI painter와 native HWND enabled gate를 실행했지만 hover는 합성 CDIS_HOT 상태다. 물리 pointer 전달·모든 화면 hover/고대비 OS 활성화·실제 DPI 이동 검증으로 확대하지 않는다.

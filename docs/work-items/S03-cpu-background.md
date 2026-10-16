# S03: CPU 숨김·복귀

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R08
- 담당/수정 범위: root; accessible.rs, software_host.rs, 공유 entry/catalog, 본 문서
- 선행 작업: ActivationServer, CPU 다운로드/설정 통합

## 사용자 결과

작업 중 창을 숨겨 유지하고 명시 재실행에만 복귀한다.

## 계약과 범위

기존 private restore IPC 사용, endpoint 실패에는 숨김 거절. 숨김 worker polling에서 surface present를 생략한다. Cargo/schema 변경 없음. 별도 담당은 신규 tools/history 파일만 수정.

## 인수 조건

공통 검증과 private GUI의 숨김·작업 유지·복귀·완료 후 숨김·중단 종료·unsafe endpoint 보존. OS 범위 구분.

## 검증과 handoff

구현 중. 실제 GUI 검증 전 완료 표기하지 않는다.

## callback 후속 인수

무조건 UI polling 제거 후 SHA b8bb 후보 active-hidden/complete-hidden 두 GUI PASS. root는 원시 result passed/hash 및 앞선 AT-SPI rawmask(Enabled/Sensitive 두 bit 없음)와 SQLite integrity를 직접 대조했다. 기존4case와 전용 callback evidence를 유지한다. native Windows callback/macOS·로그아웃·절전·숨김 자원 예산은 이 Linux 수직 기능 완료 범위가 아니다.

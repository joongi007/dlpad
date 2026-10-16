# S03: timeout fixture의 실제 자손 시작 증거

- 상태: in-progress
- 분류: test
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: stress_recovery; crates/engine/src/process.rs Unix tests module 및 이 문서
- 선행 작업: whole startup deadline/public noR 150ms 검증

## 사용자 결과

제품 timeout 정책은 유지하며 readiness 이전 100ms 경합을 자손 소유권 회귀로 오인하지 않는다.

## 계약과 범위

합성 parent/child marker 및 Linux /proc PID/start/parent 관계를 전체 3초 deadline 중 2초 내 실제 관찰한다. 고정 sleep 증대나 Err 허용은 하지 않는다. guardian/제품 정책/기존 실패 로그는 불변. 다른 startup 전용 짧은 시험은 유지한다.

## 인수 조건

반드시 TimedOut; 전체 deadline 이후 추가 정리 2초 이내. 종료 후 동일 PID/start identity가 실행 중이지 않음. 외부 프로세스 신호나 사용자 데이터 사용 없음. Linux identity 검증은 다른 OS 검증을 대신하지 않음.

## 검증과 handoff

시험 구현 및 소유 rustfmt 완료. process.rs SHA `71243073312afe06eeef6f82f6b0b23d3b04e42a71defed860fc78d6d74a30ee`로 sourcefreeze 인계했다. Cargo/실제 시험은 미실행이며 root 통합 검증 대기다. 기존 30/50ms startup 전용 시험은 유지했다. native /proc 증거가 필요한 해당 nested 시험은 Linux cfg로 명시했다. 고정 5ms sleep은 marker 관찰 polling이며 readiness 추정 근거가 아니다. 사전 쓰기 시도는 read-only sandbox로 실패하여 파일이 바뀌지 않았고, 승인된 프로젝트 쓰기 권한으로 계약 먼저 생성 후 구현했다.

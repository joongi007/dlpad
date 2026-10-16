# S01: stderr 폭주와 stdout 완료 증거 분리

- 상태: done
- 분류: bug
- 우선순위: P0
- 요구사항: R01, R08
- 담당/수정 범위: 주 담당; engine/process.rs, 본 계약
- 선행 작업: 최종 workspace 통합 검증

## 사용자 결과

정상 도구의 stderr 폭주가 stdout 최종 파일 증거를 밀어내 작업을 실패시키지 않는다.

## 계약과 범위

기존 전체256KiB 예산 안에서 stdout/stderr 각각 절반 tail을 유지한다. metadata stdout은 기존 별도8MiB이고 stderr는 전체 로그예산을 유지한다. stream 순서를 시간 순서로 추정하지 않는다. 양쪽 truncated는 OR로 보고한다. 전체 stdout 폭주는 여전히 tail을 잘라낼 수 있고 누락 증거로 완료하지 않는다.

## 인수 조건

기존 합성 pause/reopen/flood/expired/reextract 시험과 전체 process 실패·취소·metadata 테스트를 통과한다. 전체 로그 메모리 상한을 늘리지 않는다.

## 검증과 handoff

최초 전체시험에서 Policy(expected exactly one final artifact)로 재현했다. 수정 후 전용 download_soak 회귀1passed/8시간1ignored(7.92초), 이어 최종 workspace 재실행에서 동일회귀7.76초 통과. stdout/stderr 보관 공간 분리로 기존 메모리 상한을 유지한다.

## P0 후속 회귀

stdout 완료 marker를 먼저flush하고 stderr5000행을 늦게 보내는 결정적 회귀를 추가한다. 동시 pipe 스케줄링에 의존한 기존 soak 실패를 보완하며 총4096B budget/truncated/양쪽 evidence 보존을 함께 확인한다. 주 담당이 process.rs만 수정한다.

후속 실제결과: cargo test -p dlpad-engine --lib stderr_flood_cannot --locked 1passed(0.49초); cargo fmt --all -- --check와 workspace clippy(-D warnings) 통과. Windows feature추가는기존windows-sys내IO/ACL/cryptography이며신규crate나lock 변경없음.

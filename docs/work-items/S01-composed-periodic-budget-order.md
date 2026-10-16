# S01: CPU 대기 이후 주기 스캔 작업 구간 시작

- 상태: in-progress
- 분류: bug
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: root — crates/core/src/library_budget.rs, crates/app/src/accessible_library.rs, CONTEXT.md, 관련 ADR 및 검증 문서. diagnostic_cache — /tmp 독립 검토만.
- 선행 작업: 500ms 대기 후보의 GUI CPU 3회 통과와 실제 스캔 완료 시험의 180초 실패 기록.

## 사용자 결과

CPU 예산을 기다리는 동안 2ms 작업 구간을 먼저 열어 사용하지 못하고 지나치는 경우를 방지한다. 이 순서 문제가 실제 180초 실패의 유일한 원인이라고 확정하지 않는다.

## 계약과 범위

작업 구간 2ms/500ms, 누적 CPU 예산 4/1000, CPU 샘플·스캔 중 대기 100ms, 전체 스캔 기한 180초, 유휴 감시 대기 500ms를 유지한다. Stop·명령·우선 감시 원인과 전체 기한을 먼저 확인하고, CPU 대기가 남아 있으면 작업 구간을 열지 않는다. CPU 대기가 끝난 뒤 실제 현재 시각에서 하나의 작업 구간만 연다. 공유 API Budget::at_with_cpu로 이 조합을 검증한다. 기존 카운터 역행·오버플로·I/O 오류 시 롤백 정책을 유지한다. 스키마·Cargo·의존성·스레드·mutex 추가는 없다.

## 인수 조건

CPU 2.4ms 사용량에 따른 600ms 대기 중 작업 구간을 소비하는 반례를 재현하고, 수정 후 구간 보존을 검증한다. 2ms 경계, 지연된 호출의 단일 구간, 누적 구간 보상 금지, 전체 기한 및 100ms 대기 상한을 확인한다. Windows 전체 fmt/Clippy/test/release와 fixture feature 검사를 실행한다. 같은 소스로 실제 완료 180초, Next 5초, Stop ACK 5초, GUI 60초 3회의 각 10초 CPU 1% 및 메모리 80MiB 기준을 통합 검증한다. 이전 실패를 보존하며 일부 통과를 전체 P0 완료로 표현하지 않는다.

## 검증과 handoff

직전 기본 실행파일 3ca7df… 및 fixture 7a3a7c…의 완료 시험은 첫 INSERT를 35.152초에 관측했으나 180초 내 commit을 관측하지 못했다. 강제 종료가 필요한 소유 앱 정리와 사용자 입력 데스크톱 보존을 확인했다. 독립 DB 감사는 최초/최종 10,000행 전체 일치·무결성·DB SHA·cap 및 sentinel 보존을 확인했다. 이 실패 사례의 전체 영상 payload 보존은 아직 검증하지 않았다.

현재 수정은 포맷 명령 exit 0, 사용자 입력 데스크톱 보존까지 확인했으며 전체 Windows 검증을 실행 중이다. 개선된 실제 완료 시간과 CPU 성능은 아직 검증하지 않았다. 별도의 CPU idle credit 제거 초안은 채택하지 않았다.

Windows 전체 `fmt --all -- --check`, `clippy --workspace --all-targets --locked --features cpu -- -D warnings`, `test --workspace --locked --features cpu`, `build --workspace --release --locked --features cpu`는 모두 exit 0이다. root가 현재 소스 543개 SHA, 기본 CPU 실행파일 b278ef012ef1a14cde893f9b531eb19ed2ccc1282e1866ca4989b7ef2a86dccb/7,917,568B, 예산·StopWake·프레임 회귀 9개 실제 통과 및 자연 Job0/가상 데스크톱 정리를 대조했다. 직전 후보와 실행파일 크기는 동일하며 새 의존성·스키마 변경은 없다. [root 검증](../validation/evidence/windows-composed-budget-workspace-20261006-continue/root-review.json).

fixture feature `cpu,library-scan-closewait-fixture`의 전체 Clippy 및 앱 테스트도 exit 0이다. 표준 기능 runner 순수 실패 경로 시험은 직접 `python3 scripts/test_windows_library_periodic_functional_native_probe.py`로 11개 통과했다. 최초 unittest 모듈 호출은 scripts import 경로 오류로 실패했고 직접 실행으로 교정했다. 실제 완료 native 시험은 별도 fresh evidence에서 진행 중이며 완료·Next·Stop·GUI 성능 인수는 아직 남아 있다.

최신 실제 완료 시험도 실패했다. fixture SHA 44012f0982b0c4a1994bca13161ab273870c76f08090f6fcb9b0d66d6e68d30b는 첫 INSERT를 71.047초에 관측했지만 이후 180초 내 commit을 관측하지 못했다. 원 inner app은 소유 Job으로 강제 정리했고 outer 관측 도구 Job은 자연 종료했다. private desktop 폐쇄와 사용자 입력 desktop 불변은 확인했다. [실패 원결과](../validation/evidence/windows-composed-budget-completion-20261006-continue/result.json). 실제 완료시간 개선은 입증되지 않았으며, 새로운 정책 값을 임의로 늘리지 않는다. 원 library thread CPU counter와 대기 구간 계측을 후속 진행하고 독립적인 resize/숨김·복귀 검증을 이어간다.

최신 실패의 독립 DB 검토는 최초/최종 10,000행 전체 일치·integrity ok·DB SHA 불변·CAP 및 sentinel 보존을 확인했다. [범위가 제한된 보존 감사](../validation/evidence/windows-composed-budget-completion-20261006-continue/independent-failed-db-review.json). 전체 영상 payload는 읽지 않았고 완료나 CPU 성공으로 집계하지 않았다.

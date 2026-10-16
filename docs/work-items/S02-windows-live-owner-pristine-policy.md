# S02: Windows 원 live owner의 미사용 reader 정책

- 상태: done (정책 범위)
- 분류: policy
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — windows_records.rs, 이 문서
- 선행 작업: 원 native import journal/marker 연결

## 사용자 결과

원 live import 객체는 reader가 전혀 시작되지 않은 journal을 식별해 정상 정리할 수 있다. 이 판정은 startup 회수 권한이 아니며 Intent 이후에는 취소·Resolved라도 pristine으로 되돌리지 않는다.

## 계약과 범위

pristine은 두 slot 모두 Empty만 뜻한다. reader 완료 receipt 및 원 native 소유권 검증을 대체하지 않는다. wire schema와 Cargo 변경 없음.

## 인수 조건

Empty→Intent→Bound→Resolved 전이 뒤 pristine=false가 유지되고 receipt 완료와 구별됨을 순수 정책 시험으로 검증한다.

## 검증과 handoff

통합 전 상태. 제품 cleanup caller는 원 live owner 및 native journal validation을 별도 요구한다.

`cargo test -p dlpad-engine --lib --locked cookies::windows_records::tests::pristine_is_lost_permanently_when_a_reader_starts -- --exact`: 실제 Linux 1 PASS/274 filtered/exit0. root 소유 파일만 rustfmt edition2024 exit0. 이 시험은 실제 Windows 파일 삭제/제품 import 검증이 아니며 Windows 제품 연결은 diagnostic_cache 별도 작업이 진행 중이다.

# S02: Windows cookie import ownership binding

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: diagnostic_cache — crates/engine/src/cookies/windows.rs 및 이 문서만
- 선행 작업: root recovery_marker/journal/windows_records native contracts

## 사용자 결과

Windows private import가 실제 journal·두 receipt·immutable marker를 생성하고 원 IDs를 보유한다. 정상 정리 전 실제 namespace/ACL/IDs 및 receipt 상태를 검증한다. startup recover와 guardian wiring 완료는 별도다.

## 계약과 범위

CSPRNG nonce→baseDigest→Journal actual6IDs→CREATE_NEW marker actualID/sync. 초기 copy write fault는 live original singleleaf 정리를 유지한다. journal 초기화 시작 이후 failure는 보존한다. 다섯 known entries만 허용하고 DELETEhandles 모두 검증한 뒤 변경한다. Empty journal은 원 live owned 객체에서만 허용, Intent/unconfirmed는 보존하며 bound/resolved는 실제 receipt 일치 필수. Cargo/다른 파일 변경 금지.

## 인수 조건

실제 Windows import 정상 생성/정리, write fault, partial initialize preservation, foreign sibling/marker replacement/receipt mismatch 보존. 제품 reader pending gate 유지. native root 통합 검증 대기.

## 검증과 handoff

구현 source READY, Cargo/포맷/native 검증은 root 통합 대기. 실제 nonce/baseDigest와 CREATE_NEW journal/marker를 보유하고 다섯 original DELETEhandles를 모두 열어 identity/ACL/allowlist를 재검증한 뒤 삭제한다. pristine 판정은 root is_pristine 계약이며 all_receipts_match의 unused-receipt empty 검증도 실행한다. existing write fault tests는 live pristine journal의 원 파일 정리 의미를 유지한다. 신규 native tests는 import 다섯 파일/정상 정리, foreign entry 및 Intent 보존, marker replacement 보존을 작성했다. 초기 journal/marker 실패는 defer 보존으로 연결했다. 실제 Windows 실행 결과는 아직 미검증이다.

파일 disposition은 여러 handle에 대한 원자 transaction이 아니다. 일부 성공 후 후속 실패는 그대로 오류로 남기고 다음 cleanup은 strict allowlist/identity 불일치로 중단한다. 이미 삭제된 원 private copy를 복원하거나 남은 files를 추정 삭제하지 않는다. native 부분 삭제 실패 주입 시험은 후속 인수다. `pub(super) mod recovery` 선언만 root disjoint recovery.rs 통합을 위해 추가했다.

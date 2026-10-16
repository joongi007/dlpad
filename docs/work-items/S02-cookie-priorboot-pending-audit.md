# S02: 이전 boot pending cookie 회수 읽기 감사

- 상태: completed (읽기 회귀 확인; 수정 별도)
- 분류: regression-audit
- 우선순위: P0
- 요구사항: R02, R08, R10
- 담당/수정 파일: soak_recovery_audit — 이 문서만
- 의존성: product gate durable Intent/Bound, 기존 boot recovery와 journal native anchor

## 범위와 인수 조건

Intent/Bound 미확정 reader가 실제 boot 변경 이후에도 정리 거절되어 영구 보존되는지 읽기 감사한다. 이전 boot UUID/native ownership/allowlist 조건을 유지하는 최소 후속 정책을 제안한다. 제품·tests·receipt 수정, 실제 reboot·Cargo·manager 실행은 없다. Empty/legacy/Busy/hardlink/changed entries는 별도 경계로 기록한다.

## 확인한 잠재 회귀

현재 `recover_one`은 원 record.boot가 현재 boot와 같을 때만 journal.recover_completed_readers를 호출한다. 이전 boot에서는 이 단계 없이 owned.cleanup으로 진입한다. 그러나 OwnedCopy.cleanup은 journal.cleanup_eligible을 무조건 검사하며 reader state는 Empty/Resolved만 허용한다. 따라서 원래 reader가 실제 boot 종료로 사라졌더라도 Intent/Bound가 남은 journal은 ReaderUnconfirmed로 정리를 거절한다. recover_one은 defer_cleanup 후 Err를 반환하여 recovery summary의 rejected 범위에 들어간다. Journal.remove_owned_files에도 같은 pending 거절이 있어 앞단 조건 하나만 건너뛰어도 해결되지 않는다.

이는 소스 경로로 확인한 잠재 동작 회귀이며 실제 reboot 재현 PASS/FAIL 결과가 아니다. 기존 이전 boot fixture가 empty journal만 다뤘다면 새 product gate가 만든 pending 상태까지 대표하지 못한다. 같은 boot pending 보존은 의도된 안전 정책이며 완화 대상이 아니다.

## 최소 Proposed 후속 정책

`PreviousBootAuthority`와 같은 crate-private 별도 deletion authority를 boot recovery에서만 만들고 OwnedCopy 및 journal 두 삭제 gate에 명시 전달하는 정책을 제안한다. 정상/sameboot cleanup_eligible 동작과 durable Intent/Bound는 유지한다. Slot을 가짜 Resolved로 바꾸거나 native receipt를 생성/편집해 previousboot 정리를 정당화하지 않는다.

Authority 생성에는 현재 kernel boot UUID를 bounded read하고 유효한 UUID로 검증한 값, 원본 record/marker의 boot·nonce·cookie identity·journal anchor binding 검증, 두 boot가 실제로 다름을 요구한다. 임의 caller 문자열/테스트 next_boot를 production authority로 받지 않는다. 삭제 직전에 현재 boot를 다시 확인하고 provenance가 바뀌면 보존한다. 같은 boot의 PID 부재나 service GC/pathabsence는 이 authority가 아니다. 이전 boot에서는 옛 PID 번호에 signal을 보내지 않는다.

Journal의 pending을 삭제 가능하게 하는 것은 **기록 당시 reader identity가 이전 boot에만 속함**을 검증할 때로 제한한다. Intent capability 및 Bound identity의 boot/nonce/digest와 recovery binding 일치를 검증해야 한다. Empty journal은 원 record의 previousboot ownership으로만 정리하며 sameboot native proof로 취급하지 않는다. 이 정책의 reboot 논거는 VM/kernel boot가 바뀌면 이전 kernel process/FD가 지속하지 않는다는 범위다. 현재 boot의 다른 주체가 외부로 전달받은 cookie copy를 새로 열었다는 권한까지 추정하지 않으며 보호된 cache/원소유 journal lock 계약을 유지한다.

## 보존 경계

- Busy flock은 previousboot에서도 deferred로 보존한다. 잠금을 강제로 해제하거나 journal을 지우지 않는다.
- Current/original boot UUID malformed·boot binding 불일치·record/marker mismatch는 rejected/preserved다.
- Empty journal 및 journal 없는 legacy v1은 원소유 binding과 previousboot가 모두 검증된 기존 경로만 유지한다. Sameboot empty/legacy는 보존한다.
- Hardlink/nlink!=1, symlink, UID/mode/native identity 변경, 잘못된 anchor, receipt 교체/부분 record, extra/changed entries는 모두 preserved/rejected다. Previousboot는 ownership 오류를 덮지 않는다.
- 삭제는 검증된 fixed owned allowlist만 anchored unlink로 수행한다. 외부 source·unrelated sentinel·foreign hardlink 대상은 보존한다. 부분 삭제 오류는 성공이라고 보고하지 않으며 재검증 없이 계속하지 않는다.

## 후속 구현과 검증 범위

Cookies recovery + reader_journal의 명시 previousboot cleanup authority가 후속 최소 수정 범위다. reader_records의 정상 eligibility, process/guardian native receipts와 제품 UI를 함께 개편할 필요는 없다. 담당 조정 후 Intent/Bound 각 slot, mixed Empty/Resolved, empty/legacy, invalid boot/mismatched capability, Busy, hardlink 및 changed allowlist 회귀를 추가한다. 실제 production authority는 /proc 현재 boot에서만 생성하고 synthetic boot selection은 테스트 모듈 내부에만 둔다. 실제 reboot 시험을 하려면 별도 안전 실행/기록 계획이 필요하며 이번 작업은 이를 실행하지 않는다.

## 실행과 handoff

`rg`와 bounded `sed`로 cookies.rs, reader_journal.rs, reader_records.rs를 읽었다. 제품/tests/source 수정·Cargo·native manager·reboot 실행은 없다. Root에 pending 이전boot 거절 경로와 양쪽 삭제 gate의 최소 정책을 보고했다. 실제 runtime 회귀는 미검증이다.

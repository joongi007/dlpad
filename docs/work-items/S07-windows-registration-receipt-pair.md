# S07: 원 등록 묶음과 독립 receipt의 읽기 대조

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R11
- 담당/수정 범위: root — facade/통합/문서/Windows 검증; diagnostic_cache — /tmp pair coordinator 초안; metrics_recovery — /tmp actual 별도프로세스 fixture 초안; soak_recovery_audit — 독립 권한·실패대조 검토
- 선행 작업: 등록 재시작 검사와 외부receipt 재시작 검사 scopedDONE후게시

## 사용자 결과

명시한 두 경로의 원readonlycap을 동시에 유지하고 외부receipt가현재등록원record/seal의정확한사본인지대조한다. 경로를receipt에추측저장하거나자동탐색하지않는다.

## 계약과 범위

기존 RegisteredInspection/ReceiptInspection 두원cap 및canonical RegistrationRecord/Seal 완전동일성 전후검증. cancel/deadline 공동적용, mutation0. payload SHA/nativeID/privateACL/namespace는기존원등록검사 API가검증. 같은byte·다른ID/다른version/nonce/foreign/malformed/siblingwrongreceipt는거절하고모든원자료보존. 역사적소유권/동일SID위조방어/삭제authority/활성화는제공하지않는다. schema/Cargo/lock변경없음.

## 인수 조건

실제producer 종료후두경로freshprocess대조PASS, 독립적으로유효한다른등록의receipt와혼합거절, typed cancel/expiry/foreign 원bytes보존. 전체fmt/clippy/test/release/root독립검토 및원sourcefreeze.

## 검증과 handoff

현재선행sourcefreeze동안/tmp초안만병렬작성. 계획을완료로표시하지않는다.

root 실제검증 source567 Windows GNU전체fmt/Clippy/test/release·설치68회귀(신규3), 별도원producer source563 등록/receipt의새프로세스원ID/SHA/canonical/5+1namespace·inspect/verify 취소/기한4실패·모든원bytes보존 PASS. 잘못된두유효receipt 혼합은실제Windows native unit회귀가검증했고 별도copiedfixture의mixed field는false를유지한다. 두원cap 순차관측이며atomic snapshot이아니다. CPU3658c316…/7,917,568B·신규dependency/lock변경0, 과거GUI비용소급0. root와독립담당이whole/actual 원로그·current SHA map을대조해이scope를완료했다.

[전체검사](../validation/evidence/windows-app-registration-pair-workspace-20261006-continue/root-review.json), [실제원복사본대조](../validation/evidence/windows-app-registration-pair-real-copy-20261006-continue/root-review.json). 실행은 `python3 scripts/windows_current_workspace_probe.py docs/validation/evidence/windows-app-registration-pair-workspace-20261006-continue --execute --seconds 900` 및 `/tmp/dlpad-registration-pair-real-run.py`. P0전체/active설치/cold제거/공급trust/physicalIME-reader/cleanOS/MSVC/서명/remoteCI는후속이다.

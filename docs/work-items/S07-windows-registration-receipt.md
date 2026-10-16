# S07: 독립 개발 등록 receipt 기반

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R11
- 담당/수정 범위: root — schema/app_install facade/통합/native/문서; diagnostic_cache — /tmp receipt coordinator 초안; metrics_recovery — /tmp 실제 Windows fixture 초안; soak_recovery_audit — 기존CLI증거 및 새receipt 실패경계 독립검토
- 선행 작업: fresh 등록·readonly 재검사·등록 CLI scopedDONE

## 사용자 결과

이번 실행의 원 등록 capability에서 정확한 묶음의 외부 receipt를 새 독립 폴더에 기록한다. 재시작 읽기 대조를 위한 별도 관측 자료이며 삭제·active 전환·서명 trust는 제공하지 않는다.

## 계약과 범위

새 root/private ACL/create-only 단일 registration-receipt.json, 원 receipt root/file nativeID와 원 등록record/seal/nativeIDs·domain digest를 결합한다. 등록 target/source 내부나 겹침을 거절한다. 원 FreshRegistration만 작성 입력이며 path/임의JSON에서 작성 권한을 채택하지 않는다. Drop/실패는부분객체보존. schema는root 조정, Cargo/lock 변화없음. 독립 anchor도 같은SID의 악의적 위조 방어·역사적소유권 증명이 아니다.

## 인수 조건

현재 등록 원cap fullverify 전후, cancel/deadline before mutation0, existing/foreign/overlap refusal 보존, canonical64KiB 상한·nativeID일치·부분쓰기 실패 보존. 별도 실제프로세스 ID/bytes 대조와 전체fmt/clippy/test/release. 후속cold removal은 별도작업이며 이 receipt 자체로자동삭제하지 않는다.

## 검증과 handoff

구현전 계약/초안 단계. P0 전체완료와 구분한다.

root는 strict receipt schema와3개실패경로회귀를작성했다. 최대64KiB, 중첩등록/seal preflight는2048B reserve를둔보수상한, 최종실제ID canonical전체상한을다시검사한다. 동일fileIDbytes/다른volume은서로다른native객체이며허용한다. 아직compile/실측전이며완료로보고하지않는다.

2026-10-06 root 실제실행: Windows whole4검사/설치61회귀, 별도CPU5cases/canonicalreceipt2305B/원cursor17·source·외부보존, nativeFileIdInfo 원receipt2IDs/원SHA·size 대조 PASS. 첫 Clippy 큰Err152B 거절을원증거로보존했고원filecapBox와공개UTF16상한/조기hint복제거절을수정했다. source563 CPUdea3c8e0…/7,917,568B, GUI크기0증가·새dependency/lock변경없음. root와독립담당이whole/actual5case/nativeIDs 원로그를대조해이작업scope만완료했다.

[완료대조](../validation/evidence/windows-app-registration-receipt-real-cpu-20261006-continue/completion-review.json), [전체검사](../validation/evidence/windows-app-registration-receipt-boxed-workspace-20261006-continue/root-review.json), [원ID](../validation/evidence/windows-app-registration-receipt-native-identities-20261006-continue/root-review.json). 원observer가입력으로핀한이전root-review.json은수정하지않고최종대조를별도completion-review에기록했다.

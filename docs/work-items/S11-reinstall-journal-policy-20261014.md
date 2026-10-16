# S11: 삭제 후 동일 위치 재설치의 journal 세대 선택

- 상태: proposed
- 분류: bugfix
- 우선순위: P0
- 요구사항: R11, R08
- 담당/수정 범위: Root installer_main.rs + installer_journal_choice.rs; 독립읽기감사 workitem/B만
- 선행 작업: 기존 명시bundle 제거·설치 admission·current native identity/launch 수명 계약

## 사용자 결과

정상삭제 후 같은위치에 새로설치할때 원journal보존 때문에 MISSING/설치차단에 머무르지 않도록 한다. target위치는 유지하고 새journal만bounded generation leaf를사용한다. 이전기록을정상제거증거로추정하거나삭제하지않는다.

## 계약과 범위

ADR0005(Proposed 기반), ADR0009(Accepted 실행수명), 기존canonical5phase/create-only root/nativeID/원자료보존정책 유지. Core removal은bundle4roots만제거하고 외부journal을보존한다. 기존고정packagedjournal은다음fresh create와충돌한다. 새Root후보는32고정sibling(base/-1..-31), readonlyworker/10s취소·deadline, latestrevision/target/journal ACK로pathhint를선택한다. 새schema/Cargo/dependency/삭제권한/자동upgrade/기존journal재작성·이동은제외한다.

## 인수 조건

1. actual 최초설치→원bundle정상제거→원journal5bytes/IDs보존→동일target 새journalfresh설치→verify/run, coldinstaller재시작에서현재bundle에맞는journal재선택.
2. generation0/1..31의foreign/partial/current/duplicate/permission/reparse 상태를보존하며새fresh정책과currentmatch정책을구분한다. installedtarget은유일matchingjournal만선택하고2+match/refusal/취소·deadline/상한초과는추측fallback하지않는다.
3. 존재/부재pathhint는쓰기·실행·삭제권한이아니며각명시operation의factory가현재nativeID/hash/manifest를재검증한다. lateedit/CloseWait/workerpanic/namespace교체ACK는새선택을발행하지않는다.
4. 32present후보의coldcost를같은10s내actual관측; timeout이면오류보존. 기간·기본성능예산을완화하지않는다. orphan부재는정상삭제완료증명이아님을상태/문서에명확히한다.

## 검증과 handoff

독립source진단과Root새후보읽기검토만수행했다. 실제native/test/설치재현0. 원원인/유일성권고/비용/EN_CHANGE가설정정/읽은sourceSHA는 .local-tools/session-recovery-20261013/reinstall-journal-policy-v1/{REVIEW.md,findings.json,source-pins.json}에기록한다. Root가제품수정과actualRust/failure/nativecurrentwhole/private설치/삭제/재설치/cold검증을담당하며성공전미완료를유지한다.

### 최종 Root 후보 읽기 검토 (실제 실행 분리)

Root가unique32scan과동일derived104text SetWindowText생략을반영했다. 현재 두sourceSHA와isolated v2 candidate_after를직접대조했고test-0.log44/0/3ignored·actualexit0/naturalJob0/private/sourceguards를읽기검증했다. 이agent는시험/native를실행하지않았다. coldpackaged matching과explicitargv보존, reparse/cancel/freshcreate/latestACK guard source확인은B/FINAL_REVIEW.md 및 final-review.json에기록했다. Rootwhole/release는진행중이며실제remove→reinstall→coldGUI인수는Root/installer_batch후속으로남는다. 제품수정추가요청없음.

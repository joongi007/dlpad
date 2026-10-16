# S07: 외부 개발 등록 receipt 재시작 읽기 검사

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R11
- 담당/수정 범위: root — facade/통합/문서/전체검증; diagnostic_cache — /tmp receipt_reopen.rs 초안; metrics_recovery — /tmp 별도프로세스 fixture 초안; soak_recovery_audit — 원증거/읽기권한 독립검토
- 선행 작업: 외부fresh receipt 실제인수완료 후게시

## 사용자 결과

새프로세스가 명시한외부receipt를readonly nativecap에서재검증한다. receipt내용의등록대상경로는없으므로 등록된앱현재존재·활성화·역사소유권·삭제authority를증명하지않는다.

## 계약과 범위

ReadRoot/ReadFile로originalroot/file IDs·privateACL·canonical≤64KiB·strictnestedregistrationseal·정확1namespace 전후확인. 파일생성/삭제/ACL변경/도구실행없음. 공유schema/lock변경없음.

## 인수 조건

freshwriter Drop뒤 실제읽기 성공과bytes보존; cancel/deadline/foreign/malformed/oversize/samebytesdifferentID 거절. 실제별도copiedfixture프로세스와root독립rawsource/nativeJob/input검토; 전체fmt/clippy/test/release.

## 검증과 handoff

초안ready이나선행sourcefreeze동안프로젝트코드게시/컴파일하지않는다. 부분scope와P0전체완료를구분한다.

source565 Windows GNU 전체fmt/Clippy/test/release PASS, 설치65회귀(신규4) 및실제선행receipt2305B의별도copiedprocess inspection/canonical/원nativeIDs/4typed cancelexpiry/source·등록5files·외부sentinel 보존을root와독립담당이검증했다. 첫Clippy len_zero거절은보존후수정했다. CPU2234c74a…/7,917,568B·Cargo.lock불변·새dependency0, 기존GUI비용소급0.

실행: `python3 scripts/windows_current_workspace_probe.py docs/validation/evidence/windows-app-receipt-reopen-lint-workspace-20261006-continue --execute --seconds 900`; `/tmp/dlpad-receipt-reopen-real-run.py` 원producer source563receipt→현재source565 검사. root JSONValue 전달시keyorder가canonicalstruct와다를수있음을보정해원rawcanonical과typedfield동일성을분리검토했다. 제품canonical typedserialize 검사는그대로통과했다.

[전체검사](../validation/evidence/windows-app-receipt-reopen-lint-workspace-20261006-continue/root-review.json), [실제원복사본](../validation/evidence/windows-app-receipt-reopen-real-copy-20261006-continue/root-review.json). registeredtarget현재존재/active설치/삭제authority/서명/MSVC/cleanOS/전체P0는이작업완료로주장하지않는다.

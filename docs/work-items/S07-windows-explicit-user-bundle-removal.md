# S07: 명시한 개발 설치 bundle의 정확 제거

- 상태: done (명시 개발 bundle 제거 엔진 범위; CLI/GUI·정식 제거는 후속)
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — 공유 제거 계약·schema/API·통합·최종 검증; metrics_recovery — /tmp bundle_removal.rs 제안 구현; soak_recovery_audit — /tmp 제거 순서·실패 인수 검토; diagnostic_cache — /tmp 기존 cap/native fixture 계약 조사.
- 선행 작업: native bootstrap 설치/검증/취소/CloseWait 인수와 현재CPU 비용 측정. 비용 중 제품 source freeze, /tmp만 작업한다.

## 사용자 결과

사용자가 명시한 완전한 개발 bundle과 journal의 현재 일치를 먼저 검사한 뒤 고정 설치 파일만 제거한다. 외부 journal·source·앱 data·사용자 영상/도구는 보존한다.

## 계약과 범위

AGENTS/README/PRODUCT/CONTEXT/R08/R11/ADR0005의 explicit removal factory를 따른다. 기존 registered_removal, user_bundle_reopen, active/receipt inspection, windows_caps/read-only/delete disposition 계약을 읽고 root가 공유 경계를 합의한다. inspector나 journal로 역사적 ownership/자동 제거 authority를 복원하지 않는다. 별도 명시 factory는 사용자 제거 요청을 받아 원 current readonly caps로 고정 namespace/nativeID/SHA/ACL을 검증하고 root-relative DELETE handles를 획득한다. unknown/torn/replaced/불일치에서는 자료를 보존한다. 실제 file disposition은 원 cap/expectedID 및 bounded cancel/deadline과 함께 관리한다. 자동 startup 제거·추측 경로 삭제·journal 삭제·Downloads/data/외부도구 정리는 금지한다. 첫 단계 /tmp contract만, 정확 파일과 공유 API는 root 검토 후 확정한다. Cargo/lock 변경은 root가 조정한다.

## 인수 조건

완전한 명시 bundle의 정확 고정파일/빈 root 제거, source와 외부journal/sentinel 보존. unknown namespace/torn/ID mismatch/다른 pair 거절과 중간 취소/기한/삭제오류에서 remaining originalcap 상태·mutation/disposition 사실을 정확히 보고한다. 기존 readonly 검사와 launch는 제거하지 않는다. root 전체Windows검증과 actual TEMP synthetic native IDs/bytes/namespace를 대조한다. 계획/단위 통과를 실제 cold 제거 검증으로 쓰지 않는다.

## 검증과 handoff

root 실험방향: 제거 요청은 명시 factory의 현재 ID 검증과 별도 authorization이며 persisted journal만으로 자동 권한을 얻지 않는다. 제거 중 active를 먼저 철회하고 실패하면 남은 원 자료를 보존하는 후보 순서를 조사한다. 기존 registered 제거의 외부 receipt 보존 계약과 나머지 receipt/active/outer 정확 제거를 구분한다. 아직 구현·실행 완료 아니다.

2026-10-06 통합: 수정 파일은 `crates/engine/src/app_install.rs`, `app_install/bundle_removal.rs`, `user_bundle_reopen.rs`, `active_selection.rs`, `receipt_reopen.rs`, `windows_caps/cold.rs`다. root가 공유 경계를 반영했다. inspector를 빌린 상태에서 canonical metadata와 immutable ID를 먼저 생성·검증하고, 오류 없는 이동 단계에서 원 root 네 개만 넘긴다. readonly 파일의 deny-DELETE 공유는 이때 해제한다. DELETE 획득 실패에는 boxed `(원 ReadRoot, 오류)`를 반환하고 미처리 roots도 보유한다. 기존 cold API는 유지한다. 획득 실패의 retry는 authorization을 자동 완성하지 않으며 삭제하지 않는다.

병렬 검토에서 최초 초안의 root 획득 실패 시 원 cap 소실과 부분 root 삭제 후 재시도 실패를 발견해 보완했다. 기존 `payload::reading_pin`의 SHARE_READ|WRITE|DELETE는 원 DELETE handle과 호환되므로 변경하지 않았다. active disposition 수락 뒤 원 handle을 닫고, 다음 payload 전 남은 정확 namespace를 다시 검사한다. 결과의 accepted dispositions는 삭제 요청 수락이며 최종 경로 부재·secure erasure 보장이 아니다. 최종 원 root를 닫은 뒤 `symlink_metadata`의 NotFound만 현재 부재로 보고한다. 존재/기타 오류는 수락 횟수를 보존한 failure다. 원 DELETE cap이 모두 없어진 뒤의 retry도 journal readonly 검증과 부재 관측만 수행하며 대체 root의 권한을 취득하지 않는다. 외부 journal은 원 readonly cap으로 보존하고 root native ID alias도 거절한다. 새 dependency/schema/lock 변경은 없다. 최대 64개 payload의 반복 SHA 비용은 기존 제거와 같은 O(N²)이며 OS I/O의 물리적 취소 지연을 보장하지 않는다.

선행 최신 CPU 성능은 18구간 전부 CPU<1%/WS<80MiB, 최대 CPU0.630450%·WS71,680,000B로 root/독립 검토했다. root는 PNG3개를 직접 확인하고 합성 파일30,000개 정확 bytes/namespace 및 DB3개 각10,000행/integrity/전후SHA를 대조했다. [원 결과](../validation/evidence/windows-msvc-native-bootstrap-default-cost-real-20261006-continue/collector-result.json), [root 검토](../validation/evidence/windows-msvc-native-bootstrap-default-cost-real-20261006-continue/root-counter-pixel-review.json). 해당 source589/CPUa7e60e… 범위이며 새 제거 코드의 검증으로 소급하지 않는다. 현재 제거 엔진 컴파일·실제 제거 인수는 미완료다.

진행 중 검증 기록: 최초 포맷 wrapper는 Windows child에 Linux 경로를 전달해 os-error3/exit125였고 원로그를 보존했다. 올바른 Windows argv로 재포맷 exit0. 첫 전체 검증 Clippy는 시험 fixture의 type_complexity/exit101로 실패해 type alias로 수정했다. 다음 전체 검증은 fmt/clippy 통과, 제거 회귀5개 통과였으나 cold 원root 보존 시험의 예상 InvalidData와 기존 실제 InvalidInput 불일치로 test101이었다. 기존 API 오류형을 유지하고 시험을 수정했다. 이후 최종 부재·active 철회 후 foreign 추가·모든 root 수락 후 cap0 재시도2회귀와 standalone public fixture를 추가해 새 Windows 전체검증을 실행 중이다. 이전 실패를 성공으로 덮어쓰지 않는다.

완료 인수: `windows_msvc_workspace_probe.py --configuration /tmp/dlpad-msvc-workspace-stdout-plan.json --output .../windows-msvc-bundle-removal-final-workspace-20261006-continue --reuse-owned-target`의 fmt/clippy/test/release가 실제0, test840/0/13·새8회귀 PASS다. source591 정확map·CPU11bb9b…8,710,656B·CLIaa8c6d…934,912B·installer65c80d…759,296B 및 원GNU/lock을 root/독립 검토했다. nativefixture 새 example의 unused helper dead-code Clippy 실패도 삭제 수정 후 원증거를 보존했다.

`windows_msvc_user_bundle_removal_collect.py --project /mnt/d/project/dlpad --proof .../windows-msvc-bundle-removal-final-workspace-20261006-continue/result.json --output .../windows-msvc-bundle-removal-real-20261006-continue`는 실제 build+fixture2명령/정상·취소 후 원cap retry·foreign·기한4case를 통과했다. root 추가 native16명령에서 제거된2root의 Windows NotFound, 원 source ID/SHA/합성70k byte·sentinel, 남은2outer의 원ID, journal4개의 현재 canonical chain/nativeID/SHA와 남은2묶음의 domain digest/실제stageSHA를 독립 대조했다. 원16로그/naturalJob0/private/input 및 post-pins도 재확인했고 독립 담당이 원자료를 읽어 확인했다. [원 결과](../validation/evidence/windows-msvc-bundle-removal-real-20261006-continue/result.json), [root 인수](../validation/evidence/windows-msvc-bundle-removal-real-20261006-continue/root-review.json).

미검증 한계: 제거된묶음 원fileID는 root 독립관측하지 않았고, journal 사전SHA의 독립 캡처도 없어 fixture 내부 전후guard 범위로 구별한다. 부분 disposition 뒤 cancellation은 같은 실제 Windows production loop hook 회귀이며 standalone4case의 실제 OS cancel 증거로 확대하지 않는다. 새CPU SHA는 이전a7e60e 비용과 달라 해당18구간을 새CPU 성능으로 소급하지 않는다. CLI연결은 [별도작업](S07-windows-explicit-user-bundle-removal-cli.md), GUI/upgrade/cleanOS/정식installer·전체P0는후속이다.

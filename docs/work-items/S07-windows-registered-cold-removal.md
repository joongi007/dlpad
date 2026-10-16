# S07: 명시적 등록 묶음 제거와 원 객체 재시도

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R11
- 담당/수정 범위: root — 공유 인터페이스·정책·통합·검증·문서; diagnostic_cache — native deletion caps 초안/독립 QA; soak_recovery_audit — coordinator 초안/독립 증거; metrics_recovery — 실제 합성 fixture/receipt native observer 초안
- 선행 작업: 등록+외부 receipt 두 경로 readonly 대조 인수

## 사용자 결과

engine의 별도 `authorize_explicit_registered_removal` factory를 명시 제거 요청에서 호출하면 정확히 검증한 등록 묶음을 제거할 수 있다. 실패는 남은 원 HANDLE을 보유하고 동일 capability로만 재시도한다. 외부 receipt·source·unknown 파일은 제거 대상에 포함하지 않는다. CLI/UI 연결은 후속 작업이다.

## 계약과 범위

수정 파일은 `crates/engine/src/app_install.rs`, `app_install/windows_caps.rs`, 신규 `app_install/windows_caps/cold.rs`, 신규 `app_install/registered_removal.rs`, `app_install/registration_reopen.rs`, `app_install/registration_pair.rs`, 신규 `examples/windows_app_registered_removal_fixture.rs`이다. 공유 schema/Cargo.lock 변경과 새 dependency는 없다. readonly inspector는 DELETE 권한을 제공하지 않는다.

원 readonly child를 해제하되 immutable expected record/seal과 원 ReadRoot/외부 receipt pin을 보유한다. 원 ReadRoot HANDLE을 RootDirectory로 지정한 빈 이름 NtCreateFile(FILE_OPEN)으로 같은 디렉터리의 DELETE access를 취득한다. 각 fixed leaf는 원 root-relative FILE_OPEN과 원 expected nativeID·private ACL·hash·namespace를 모두 재검증한다. 새로운 관측 ID를 expected 값으로 채택하지 않는다. 모든 cap과 preflight가 준비되기 전 disposition은 0이다. 기존 ACL/권한 변경·관리자 실행·path fallback은 없다.

payload→ready→record→seal→빈 root 순서로 원 HANDLE의 FileDispositionInfo를 사용한다. 실패의 accepted disposition 수와 원 remaining cap 수를 구분한다. Drop은 삭제하지 않는다. reader로 retired 파일 이름이 남거나 unknown 파일이 생기면 남은 원 caps를 보존한다. 프로세스 종료 뒤 partial root의 path-only 자동 adoption/retry는 제공하지 않는다. 외부 receipt는 consistency 자료이며 같은 SID 위조 방어·역사적 ownership·publisher trust의 증명이 아니다.

## 인수 조건과 실제 범위

Windows native cap 3회귀와 coordinator 4회귀를 포함한 app_install 75회귀 및 전체 fmt/clippy/test/release가 통과했다. 원 root expectedID 오류, readonly share 충돌, Drop 보존, 비어 있지 않은 root, auth cancel/deadline, foreign 삽입, 첫 disposition 뒤 오류와 원caps 재시도, 실제 reader pending 후 동일caps 재시도를 시험했다. 이전 readonly 회귀의 same-bytes replacement 거절도 전체 suite에 포함된다. 모든 조상 교체 interleaving·ACL 거절 조합·crash 복구를 새 cold 코드에서 실증한 것으로 표현하지 않는다.

실제 copied CPU+합성 notice의 네 독립 등록 묶음에서 producer caps 해제 후 다시 검사해 명시 제거했다. 실패 3행(cancel Interrupted/deadline TimedOut/foreign InvalidData)은 accepted0/원cap5를 유지하고 동일caps retry를 수행했다. 최종 4행은 각 file dispositions5·root accepted·실제 target 부재·외부 receipt 보존을 확인했다. foreign 파일은 engine이 지우지 않았으며 fixture가 자신이 생성한 원 DELETE HANDLE/ID로 별도 정리한 뒤 재시도했다. source cursor17→17와 source·notice·outside sentinel을 보존했다. private desktop/natural owned Job0/input desktop 불변을 원로그로 대조했다.

## 검증과 handoff

`python3 scripts/windows_current_workspace_probe.py /mnt/d/project/dlpad/docs/validation/evidence/windows-app-registered-removal-relative-workspace-20261006-continue --execute --seconds 900`: Windows GNU CPU 전체4검사 PASS, app_install75. source570은 snapshot 항목 수이며 동일 수의 초기 실패 snapshot과 hash map이 다르다. 최초 ReOpenFile 루트 재개방 ERROR5의 7회귀 실패와 단계별 실패 로그를 보존했고, 빈 이름 root-relative FILE_OPEN으로 수정한 최소3회귀와 전체75가 통과했다. ACL 변경으로 해결한 것이 아니다.

`python3 /tmp/dlpad-registered-removal-real-run.py --cpu-sha f3bae8015a8c81d7c05dcb7b952bfa8f24daf3ff11f20da4cd72a239daed70e3`: 실제4묶음/7행 PASS_SCOPED. [root 원검토](../validation/evidence/windows-app-registered-removal-real-cpu-20261006-continue/root-review.json). CPU7,917,568B(+0B), 이전 GUI 비용을 소급하지 않는다. 같은 현재 source의 외부 receipt 네 개를 별도 native observer로 재검증했다. root 및 독립 원본 대조 PASS. [별도 native 원ID](../validation/evidence/windows-app-registered-removal-receipt-identities-20261006-continue/root-review.json), [완료 검토](../validation/evidence/windows-app-registered-removal-real-cpu-20261006-continue/completion-review.json).

남은 payload의 full SHA를 단계마다 다시 검사하므로 O(N²) 읽기이며 64개/256MiB 묶음은 이론상 약16GiB 추가 읽기가 가능하다. 최대 묶음 성능 인수와 단일 OS 호출의 hard deadline 보장은 없다. accepted disposition은 secure erasure가 아니다. 실제 per-user active 설치·실행중 exe 제거·upgrade/rollback journal·clean Windows/minOS/MSVC·서명/공개 배포는 후속 P0이며 ADR 전체 Proposed 상태를 유지한다. 다음 [개발 CLI 연결](S07-windows-registration-receipt-local-cli.md)을 진행한다.

최종 문서 검사: `PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_docs.py` PASS786 documents(외부 URL/heading anchor 제외). root가 실제 fixture·native observer·현재 source/immutable pins를 직접 대조했고 독립 검토도 별도 보관했다.

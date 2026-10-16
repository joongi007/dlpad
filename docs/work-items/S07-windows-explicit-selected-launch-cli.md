# S07: 명시 선택 설치본의 개발 CLI 실행

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R11, R08
- 담당/수정 범위: root — 공유 실행/receipt 계약·통합·검증·문서; metrics_recovery — /tmp 기존 CLI의 열 번째 명령 초안; soak_recovery_audit — 초안/실제 실행 독립 검토
- 선행 작업: 원 소유 런처 및 현재 선택 설치본의 private GUI 두 실행·테마 재시작 인수 완료. source576 전체4검사 완료.

## 사용자 결과

개발 companion CLI 사용자가 등록·receipt·active 세 경로를 명시하여 선택 앱을 실제 실행하고 원 소유 프로세스의 정상 종료·취소·기한 결과를 확인한다.

## 계약과 범위

launch-selected는 기존 strict parser의 세 절대 경로와 명시 timeout을 받는다. inspect_active_selection에서 얻은 원 capability를 launch_explicit_selection에 넘기고 동일 caller cancellation/deadline로 wait한다. 원 Job·File pins는 supervisor 수명 동안 유지하며 argv override·shell·inherited handle·자동 discovery·파일 삭제가 없다. 새 schema/의존성/workspace 변경 없다. 이 명령은 최대600초의 개발용 감독 실행이며 일반 앱의 영구 shortcut/bootstrap·detach 기능이 아니다. 도움말과 문서에서 supervisor 종료/기한 시 자식 앱도 종료될 수 있음을 설명한다.

stdout 한정4KiB·development_only·publisher/installer/upgrade/삭제권한 false를 유지한다. 자연 종료와 취소/기한 및 terminal_unconfirmed를 구별하고 원 created/resume_attempted/may_have_executed/terminal facts를 보존한다. Drop 정리를 성공으로 승격하지 않는다. 기존9명령 parser/receipt 동작을 유지한다.

## 인수 조건

parser 잘못된 경로/누락/중복/unknown/UNC/기한 거절 회귀, Windows 전체4검사. 최신 고정 CPU·원3경로에서 actual private GUI 실행/정상 종료 및 원파일 보존·설정 재시작, 다른 유효 pair/foreign 선택 실패. 실제 취소/기한은 준비/실행 ACK를 근거로 시험하고 timing 추측은 인수로 사용하지 않는다. root와 독립 검토를 모두 통과해야 범위 완료다.

## 검증과 handoff

현재 초안 작성 전이다. 정식 per-user 설치 위치·publication journal·upgrade/rollback·서명/cleanOS/일반 launcher 수명 정책은 별도 게이트이며 이 명령으로 완료 처리하지 않는다. 현재 SDK 공급 조사와 독립 병렬 초안만 진행하고 공유 파일은 root가 통합한다.


root 통합: stagingSHA3633f5bd…를 원CLI hash에 대조하고 적용했다. inspected selection의 총 보유파일 개수는 미집계이므로 새launch branch에서는 retained_file_caps=null/retained_file_cap_count_reported=false와 실제 selection-held/owner-held boolean을 사용한다. 기존9명령 정책은 그대로다. Natural0도 Job 종료 확인이 있어야 성공이며 source QA의 capcount 오해를 수정했다. 전체검증/actual GUI 및 control은 다음 단계다.


2026-10-06 실제 CLI 인수 준비: diagnostic_cache는 scripts/windows_launch_selected_cli_probe.cs 원 Job/이미지/HWND·격리 console 관측기 초안, metrics_recovery는 scripts/windows_launch_selected_cli_collect.py 증거 수집기 초안, soak_recovery_audit는 독립 읽기 검토를 맡는다. Git 저장소가 없으므로 세 담당은 /tmp에서만 작성하고 root가 프로젝트 파일을 통합했다. 기본 CLI Windows 전체 fmt/clippy/test/release는 source576·758passed/13ignored·설치89/parser3으로 통과했다. CPU54c33d95…/7,917,568B 불변, CLI5ae864b8…/754,688B(+31,232B), lock 불변. 독립 전체 검토를 보관했다. 실제 실행 인수는 아직 미검증이다. 전체 GUI suite 한도600초는 네 사례별 기존90초 UI·각 receipt/정리 한도를 합산한 상위 예산이며 사례별 한도는 늘리지 않았다. 가상 화면 실패 시 사용자 화면으로 대체하지 않는다.

실행 첫 수집기는 네GUI 및 Natural17/잘못된pair를 통과했으나 foreign fixture 거절의 예상 kind를 InvalidData로 지정해 FAIL했다. 실제 engine는 알 수 없는 폴더에 PermissionDenied를 반환했고 자식 생성/변경은 false였다. 원FAIL evidence windows-launch-selected-cli-real-20261006-continue를 보존하고 수집기 예상값과 미생성/미변경 guard를 수정한다. 독립 QA에서 worker 미회수시 원HANDLE이 닫히는 실패 경계를 발견하여 참조 process/thread/Job/console/desktop을 종료까지 보존하고 FAIL로 기록하도록 관측기도 보완했다. console 분리 후 desktop 종료 순서로 변경한다. 수정 수집기를 fresh evidence에서 재시험한다.

수정1의 실제 네GUI도 PASS했으나 root가 추가한 no-child guard를 기존 parser preflight receipt에 적용하여 수집기 FAIL했다. 기존 preflight에는 child_created 키가 없으며 retained_file_caps=0/phase=preflight로 보고한다. 누락 키를 false로 둔갑시키지 않고 source 계약에 따라 preflight와 launch-inspect를 구분해 검증하도록 수정한다. corrected-real 원FAIL도 보존한다.


## 최종 범위 인수

[실제 수집 결과](../validation/evidence/windows-launch-selected-cli-final-real-20261006-continue/result.json), [주 담당 검토](../validation/evidence/windows-launch-selected-cli-final-real-20261006-continue/root-completion-review.json), [독립 검토](../validation/evidence/windows-launch-selected-cli-final-real-20261006-continue/independent-review.json).

실제 명령: `python3 scripts/windows_launch_selected_cli_collect.py --proof docs/validation/evidence/windows-launch-selected-cli-workspace-20261006-continue/result.json --output docs/validation/evidence/windows-launch-selected-cli-final-real-20261006-continue`(실제 실행은 동일 파일의 절대 경로). Windows observer/native-ID 컴파일2, GUI suite1의 네 actual case, CLI 추가9, native3query 총15 operation이 원 host 자연 Job0/private 종료/input 불변으로 통과했다. 정상 실행과 동일 세경로 재시작은 Natural0, 실행 ACK 후 CTRL_BREAK는 Cancelled/Interrupted, 동일 시작 기한15초는 TimedOut/TimedOut/CLI1로 실제 확인했다. 별도 helper 자연17은 Other/CLI1로 보존한다. 잘못된 parser6·다른 유효 등록pair·foreign ACL 거절은 자식 실행 없이 예상 오류로 종료했다. foreign은 unknown namespace 검사까지 도달한 증거로 확대하지 않는다.

원 파일34개 SHA와 원 등록/receipt/active의 native10 IDs·SHA/bytes 보존, canonical/domain 및 등록domainSHA/receiptSHA/nested binding을 직접 대조했다. 원 GUI System→Light 및 restart Light PNG를 주 담당/독립 담당이 직접 확인했고 readonly SQLite integrity=ok/theme Light와 조회 전후 DB SHA불변도 확인했다. 이전 두 collector FAIL과 각 네GUI 실제 성공은 별도 원증거로 보존했으며 완료 proof로 재분류하지 않았다.

관측기 failure 경계 수정은 source 독립 검토와 수정 actual suite를 통과했다. worker 미회수 경로를 실제 강제 재현한 근거는 없으며 그 실패 경계의 실제 fault 인수는 후속으로 남긴다. 완료 범위는 개발용 명시 감독 실행이다. 영구 launcher/bootstrap/설치 위치·upgrade journal/rollback·cleanOS/서명·최신CPU 성능/physical IME·reader/전체P0는 완료로 처리하지 않는다.

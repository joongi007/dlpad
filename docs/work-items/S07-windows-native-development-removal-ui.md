# S07: 개발 설치 화면의 명시 제거

- 상태: DONE (개발 제거 UI·기존 설치 회귀 범위; 최신 CPU 비용은 별도 검증 중)
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — engine worker service·공유 API·통합·Windows 최종 검증; diagnostic_cache — /tmp installer_main/catalog UI 초안; soak_recovery_audit — /tmp 확인·취소·CloseWait 계약 검토; metrics_recovery — 현재 제거 CLI collector, 종료 후 /tmp UI native fixture. 프로젝트 파일 수정은 root만 한다.
- 선행 작업: [개발 설치 화면](S07-windows-native-development-bootstrap.md), [제거 엔진](S07-windows-explicit-user-bundle-removal.md), [제거 CLI](S07-windows-explicit-user-bundle-removal-cli.md).

## 사용자 결과

개발 화면에서 현재 설치를 검증한 뒤 제거 확인을 선택하고, worker 종료·실제 outer 부재를 확인하면 설치 파일 제거 완료를 표시한다. 원본·manifest·외부 journal·앱 data·영상·도구는 보존한다.

## 계약과 범위

수정 예정: `crates/app/src/installer_main.rs`, `installer_catalog.rs`, `crates/engine/src/app_install.rs`와 worker용 `app_install/local_remove.rs`, 개발 안내·이 작업 기록, 실제 격리 GUI collector/helper. 한 worker·capacity1 최종 보고·active-only75ms 타이머·원 JoinHandle 종료 확인을 유지한다. 제거 I/O는 UI thread에서 수행하지 않는다. 첫 확인 선택에는 삭제하지 않고 원 경로 snapshot과 보존 범위를 표시한다. 확인 중 입력 경로는 고정한다. 과거 검증 결과를 삭제 권한으로 삼지 않고 worker가 명시 factory로 현재 상태를 다시 검사한다. 제거 취소는 이미 수락한 disposition을 복구하지 않으며 partial 결과와 최초 거절을 구분한다. 자동 제거·path fallback·새 schema/dependency/lock 변경은 없다.

## 인수 조건

private desktop 실제 설치→검증→확인 취소 시 자료 보존, 확인 제거 시 실제 outer 부재와 source/journal/sentinel 원 ID·SHA 보존을 대조한다. foreign/바뀐 경로/불일치에서는 삭제하지 않는다. 단일 worker 및 취소·CloseWait의 실제 ACK 전후 상태를 확인한다. 원caps가 worker 종료 후 없는 UI 재시도를 원caps retry로 보고하지 않는다. 전체 Windows 검증과 root/독립 raw·pixel·native 결과를 검토한다. 물리 IME·reader·DPI·clean OS·upgrade·정식 installer는 이 개발 slice의 완료로 확대하지 않는다.

## 검증과 handoff

현재 계약 조사만 완료했고 UI·service 구현/native 실행은 미완료다. 기존 설치 화면을 worker 결과 enum과 명시 확인 상태로 확장하는 후보를 검토한다. inline 확인을 우선하여 modal reentrancy와 다른 창의 활성화를 추가하지 않는다.

2026-10-06 root통합: 제거CLI 원source proof 인수/독립 검토 후 local_remove service와 installer v2를 반영했다. /tmp UI를 두 독립 담당이 검토했으며 root도 확인·경로snapshot·EN_CHANGE·worker ACK를 직접 검토했다. 설치 후 별도 Verify에서만 제거 요청을 활성화한다. 확인 중 입력을 고정하고 돌아가기는 삭제0/worker0, 제거 후 실제 outer부재 없이는 완료를 표시하지 않는다. 경로수정 후 Verify 재시도는 유지한다. service는 기존 strict local path 검사와 같은caller cancel/deadline을 사용한다. pending ReadRoot의 unpublished API/CLI JSON을 retained_unconverted_bundle_read_root_*로 명확히 했다(DeletionRoot 내부 root/외부journal/OS HANDLE총수 제외). 새 dependency/schema/lock 변경은 없다. 전체compile/test·실제private GUI 인수는 아직 미완료다.

실행 기록: 첫 cargo fmt는 실제 exit0/소유Job natural0/input불변으로 파일을 포맷했지만 root wrapper가 source_unchanged를 요구하여 인수 guard가 실패했다. 원 fmt.log를 windows-msvc-removal-ui-format-20261006-continue에 보존했다. idempotent 재실행 windows-msvc-removal-ui-formatted-check-20261006-continue는 exit0/source불변/natural0이며 현재 전체MSVC검사를 진행한다. python3 scripts/check_docs.py는844문서PASS(당시 docs snapshot), 외부URL/heading은 검사하지 않는다.

Windows 첫전체검사: fmt0/clippy101. 원검증로그를보존했다. 실패는 새 UI 테스트module 뒤 production items가 있어 items_after_test_module lint가 발생한 단일 오류다. root는 allowance를 추가하지 않고 테스트module을native module 끝으로 이동했다. 수정 후 새 전체검사를 수행한다.

2026-10-06 교정전체MSVC 인수: source592 정확map·4검사0·41suites844/0/13, UI보고2/service precontrol1 포함. root/독립 raw/nativeJob와artifact3·GNU/lock 대조PASS. CPU0bf43b…8,710,656B·CLId0e4ae…974,848B·installer2e91bb…798,720B(직전759,296B보다39,424B/38.5KiB 증가), 새 의존성없음. [전체근거](../validation/evidence/windows-msvc-native-removal-ui-corrected-workspace-20261006-continue/root-review.json), [독립근거](../validation/evidence/windows-msvc-native-removal-ui-corrected-workspace-20261006-continue/independent-review.json). native observer6mode를root검토·반영했고 callback 예외를 flag/호출밖 검사로 옮겼으며 threaddescription129UTF16 상한을 적용했다. Source/journal 전후 원nativeID, 제거전7files/4roots관측과 합성128MiB 취소·CloseACK, changed-path Verify회복을 별도 실제 검증한다. 현재 새비용/제거GUI 인수미완료.

2026-10-06 새 제거GUI6case(확인돌아가기·정상제거·foreign거절·경로수정재검증·실제취소·CloseACK), collector12명령과root추가46native명령이 현재source/artifact 범위PASS다. root가case10PNG를직접보고source/journal 원ID·SHA 전후·삭제전7files/4roots·Windowsouter부재·남은5bundle canonical/domain/stagebinding·취소/닫기의원workerACK를 확인했다. 합성128MiB두case의실제CPU prefix/zero padding도별도읽기로대조했다. 삭제된metadata 원canonicalbytes독립보관/exact실행중phase·dispositioncounts는미관측이며특정engine단계로확대하지않는다. [root native+pixel](../validation/evidence/windows-msvc-native-removal-ui-six-root-native-20261006-continue/root-review-with-pixel.json), [직접화면](../validation/evidence/windows-msvc-native-removal-ui-six-root-native-20261006-continue/root-pixel-review.json), [독립6case raw](../validation/evidence/windows-msvc-native-removal-ui-six-root-native-20261006-continue/independent-gui-review.json). 기존설치긍정회귀도2GUI명령·root추가7native명령/3PNG로통과했다. 기존4negative 설치회귀와최신CPU비용인수/추가독립native검토는진행중이며전체P0완료아니다.

기존 repeat회귀첫실행은관측도구FAIL로보존했다. 새 UI는오류/경로수정뒤readonly Verify를유지하는계약인데기존observer가203비활성화를요구했다. root는product를되돌리지않고관측도구를현재계약에맞춰보강했다: Verify활성+Remove비활성/원12파일보존을확인한뒤별도새Verify가성공해야Remove가활성화되는지실제검사한다. 최초FAIL의소유fixture만error cleanup했고사용자데스크톱input은불변이다. 새로운evidence에서재실행하며실패기록을PASS로덮어쓰지않는다.

2026-10-06 scoped 인수 완료: 새6case12실행/rootnative46과 기존설치5case10실행/rootnative27, 총GUI11case22실행·rootnative73명령·직접PNG18장을검토했다. 기존4negative는교정repeat(거절→Remove비활성→새Verify재활성/자료보존), wrongSHA(실제Windowsroot부재), 실제복사중Cancel/Close원workerACK다. [독립native6case](../validation/evidence/windows-msvc-native-removal-ui-six-root-native-20261006-continue/independent-native-review.json), [독립설치negative](../validation/evidence/windows-msvc-native-removal-ui-six-root-native-20261006-continue/independent-bootstrap-negative-review.json). 첫observerFAIL과firstfmt/clippyFAIL은보존했다. 기능/실패/통합회귀범위DONE이며 source592/CPU0bf43b 비용은새3×60/18구간을별도로실행중이다. 정식installer/upgrade/cleanOS/physical/전체P0는미완료. 다음 [재시작기존검사](S07-windows-native-existing-bundle-reverify.md)의typed2roots와UI초안을/tmp에서병렬준비한다. 현재slice는새창의기존bundle검사에CLI가필요한한계를해결했다고주장하지않는다.

최신CPU0bf43b 측정: rootcounter18/18PASS(최대 단일core0.6244603102%/Job WS71,757,824B, 약68.4MiB), root3idle-endPNG와3×10k합성payload/DB정확inventory30k·integrity ok·원DB SHA전후동일을확인했다. [현재counter/pixel](../validation/evidence/windows-msvc-native-removal-ui-default-cost-real-20261006-continue/root-combined-review.json), [원payload/DB](../validation/evidence/windows-msvc-native-removal-ui-default-cost-real-20261006-continue/root-payload-db-review.json).독립cost 최종검토를기다리며 hostquiet/physicalIME·reader·firstpaint/workerrole 완전귀속은미증명이다.

이전source592/CPU0bf43b 비용인수완료: [독립최종](../validation/evidence/windows-msvc-native-removal-ui-default-cost-real-20261006-continue/independent-review.json). source/pins/원raw2·18counter·합성30k/DB3보존을확인했다. 실제sentinel은없는fixture이므로sentinel보존을인수하지않는다. 이후typed2root새source의비용으로소급하지않는다.

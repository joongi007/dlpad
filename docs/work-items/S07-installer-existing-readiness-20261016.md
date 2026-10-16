# S07: 설치기 기존 설치 확인과 실행 준비

- 상태: done-scoped / 사용자 검수 대기
- 분류: fix
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: p0_audit; crates/app/src/installer_main.rs, installer_catalog.rs, 필요시 전용 installer readiness 모듈과 이 문서
- 선행 작업: current33 whole/설치/긴경로·Accepted ADR0009 실행 수명; DESIGN disabled 이유/상태/다음행동

## 사용자 결과

설치기를 다시 열었을 때 기존 설치의 확인 단계와 앱 실행 준비 여부를 상단에서 알 수 있고 같은 위치의 실행 행동을 찾는다. 설치·업데이트 자동 실행은 하지 않는다.

## 계약과 범위

ADR0005 Proposed와 ADR0009 Accepted를 구분한다. 원 worker·revision·root/journal·해시 검증·fresh explicit launch·Job/retained HANDLE/300초 기한·실패보존은 유지한다. I/O를UIthread로옮기지 않는다. engine/Cargo/lock/sharedcatalog/accessible 변경0. 기존설치덮어쓰기/자동updater/P1P2추가0. 모든묶음0.1.0이므로다른버전업데이트를확인했다고거짓표시하지않는다.

## 인수 조건

대기/기존설치확인/완료/실패/입력변경을구분하고, 미확인/진행중/실패에서는실행불가 및이유가보인다. 실제선택된journal의cold검증결과만현재revision에서반영하며presence만으로ready가되지않는다. 중복해시검증을줄여도실행시새검증은생략하지않는다. 기존경로충돌/ambiguous/취소/기한/failedworker는원파일보존과실행금지를유지한다. actualWindows실행/캡처는Root조정후검증한다.

## 검증과 handoff

기존source/원로그읽기로원인분리중. 의미있는회귀를추가하되childCargo/native는실행하지않는다. 최종Root가fmt/clippy/test와격리설치기재실행·상단문구/고정action/worker시간을통합검증한다. 의존성·런타임스레드/프로세스추가없음.

## 원인과 구현

원 v33 `windows-msvc-ui-feedback-v33-install-reinstall-launch-private-v1-20261016/native-gui.log`의 fresh_installer_before_verify는 기존journal찾은후default203/앱실행disabled, after_verify는default212/enabled였다. 전체nativeoperation21.775078초지만개별coldinspect시간은없어ms속도개선수치는주장하지않는다. source에서journal선택이full `inspect_local_existing_bundle`을이미실행하고bool만남겨버려사용자가같은검증을다시실행해야했던것을확인했다.

`admission::choose_verified`는bounded32journal의유일한성공report를해당journal에묶어반환한다. 전체choose성공·원revision/선택일치·joined·취소아님·Existingphase·typedreport성공일때만UIverified/removeeligible에반영한다. 실제앱실행은기존start_launch의freshidentity/기록/실행capability검증을그대로재수행한다. ambiguous/실패/cancel/다른snapshot은ready가되지않는다. 읽기전용자동journal EDIT의programmatic EN_CHANGE는user입력이아니므로중복재검증을예약하지않는다. 사용자target/다른draft변경은원invalidate/revision250msdebounce그대로다. 초기기본입력만debounce를기다리지않고동일worker를바로예약한다.

상단단계는설치확인중/기존앱확인중/설치중/제거중/실행준비완료로구분하고disabled이유와다음행동을visibleintro에표시한다. 앱실행ID212·기존명령순서/일관된자리·Win32nativebuttonsemantics유지. 업데이트/묶음비교/자동교체를표시하거나구현하지않는다. 모든버전0.1.0이므로실제앱버전차이를판정했다고주장하지않는다.

## 준비한 회귀와 실행 한계

새main의admission3회귀: existing_cold_check_retains_only_unique_report_without_second_inspection(실FS/정확1회inspect·source보존·stale拒否), existing_readiness_rejects_ambiguous_cancelled_and_failed_inspection(중복/취소/실패·원파일보존), vacant_location_never_inherits_a_verification_result(새위치에ready이전없음). 기존 edited_source_generation_and_replaced_roots_discard_stale_results/저장실패/취소·기한/정확제거·실행freshguard도유지했다.

child는새Rust를실행하지않았으며회귀는**작성만/미실행**이다. Root가fmt/strictclippy/installer시험및currentnative100/200에서재실행직후checking문구→유일한기존설치자동ready/212활성화, wrongrecord/ambiguous/취소/입력변경실행금지와원Job수명을검증해야한다. 원v33nativehelper의before_verify=disabled203assert는새제품계약에맞춰자동ready212로갱신해야하며원실패·evidence를덮지않는다. 새deps/프로세스/스레드추가0,임시report1개보존비용만추가된다.


### Root 최종 통합 — 2026-10-17

최종 검수본은v35다. Windows whole35 fmt·strict workspace/all-targets clippy·test·release 실제0,1679/0/42·43harness·source675·3PE·lock을 Root가 대조했다. 실제 CPU renderer104프레임/unpainted0·Light/Dark100/200% PNG4개를 직접 검토했다. 새 패키지 전체CRC/SHA와 private 설치·제거·동일위치 재설치·cold 확인(수동 재검증0회)·설치 앱 자연0/Job0을 확인했다. [통합 검수](../validation/ui-review-v35-20261017.md), [최신 실행](../development/windows-user-preview.md).

Explorer 실제 선택과 최종 사용자 UI 승인은 남는다. 자동 시험에서 사용자 desktop·clipboard·외부 viewer를 열지 않았다. 버전 비교·기존 앱 자동 교체는 후속이며 최신 후보 확인에는 새 폴더 또는 portable을 사용한다. 기존 .git은 보존하고 Git조회exit128을 유지한다.

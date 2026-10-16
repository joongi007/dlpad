# S07: 선택 실행의 부모 종료 후 자식 수명

- 상태: done
- 분류: test
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — 공개계약·fixture 통합·실제 Windows 검증; diagnostic_cache — /tmp Rust/native fixture 초안; soak_recovery_audit — 결과 독립 검토
- 선행 작업: S07-windows-selected-supervisor-lifetime의 새 전체 MSVC 검사 및 실제 원 Job 감독 검증

## 사용자 결과

선택 앱의 부모가 종료돼도 소유 자식이 살아 있으면 감독이 완료로 오인하지 않고, 종료·취소·기한을 원 Job에서 확인한다.

## 계약과 범위

CONTEXT와 ADR0005 Proposed의 원 selected process/Job/creation/image capability 계약을 유지한다. 담당 파일은 새 engine example과 프로젝트 전용 synthetic helper/collector이며 경로는 crates/engine/examples/windows_selected_descendant_fixture.rs 및 scripts/windows_selected_descendant_helper.cs, scripts/windows_selected_descendant_build.py, scripts/windows_selected_descendant_collect.py다. 원 Job의 read-only 순간 count 관측을 위해 root가 crates/engine/src/app_install/active_launch.rs에 active_process_count API를 추가한다. 이 관측은 terminal 증명이나 PID 소유권이 아니다. Cargo workspace·lock·공유 schema는 root가 조정하며 이 작업에서 변경할 이유가 없다. 기존 실제 Natural17 helper에는 자식이 없으므로 해당 분기 증거로 소급하지 않는다. 사용자 미디어나 보이는 데스크톱은 사용하지 않는다.

## 인수 조건

원 부모 종료와 실제 살아 있는 자식을 동기 신호·원 HANDLE로 관측한다. 새 wait_cancelled는 원 Job이 비기 전 Natural을 반환하지 않는다. 취소는 Cancelled/원 Job terminal 확인, 기존 bounded wait는 TimedOut/원 Job terminal 확인이어야 한다. 원 fixture 파일 SHA·native ID·사용자 input desktop 보존과 외부 host의 자연 종료를 확인한다. 추정한 sleep 시간이나 Drop만으로 성공 판정하지 않는다.

## 검증과 handoff

현재 소스 감사에서는 common wait의 유한 대기·취소 재확인 정책에 결함을 발견하지 못했지만 실제 descendant-only 분기 증거가 없어 fixture를 작성 중이다. 설계만으로 완료하지 않는다. root가 기존 launch-selected의 새 바이너리 회귀를 먼저 실행하고 이후 source 통합과 전체 검사를 직렬 조정한다.

초안 독립 검토에서 이미 종료한 원 부모의 exit17은 descendant 취소/기한에도 보존됨을 확인했다. TerminateJobObject의126은 원 부모의 이미 확정된 exitcode를 바꾸지 않는다. collector는 자연/취소/기한 모두 원 부모 exit17을 대조하며 reason과 원Job terminal 확인으로 정책을 구분한다. 생산 종료 정책은 변경하지 않는다.

첫 전체 Clippy에서 copied fixture의 CPU 인수가 미사용으로 거절됐다. 원 실패 로그를 windows-msvc-selected-descendant-workspace-20261006-continue에 보존하고, 명시 CPU 인수의 실제 SHA를 검사하도록 수정한 corrected 전체 검사를 실행 중이다. formatting 실행은 소스 변경용이며 검증 성공으로 표시하지 않는다. native30 canonical ID·namespace/domain digest·7파일/case·원helper SHA/cursor 보존을 collector에 통합했으며 실제 실행 전이다.

2026-10-06 corrected source577 전체 MSVC fmt/clippy/test/release actualexit0·760passed/0failed/13ignored다. CPU594676ba…/8,709,632B·CLIa0bb2ab8…/795,136B. 각 source before/after 동일하며 원 GNU/lock 보존이다. 실제 descendant helper와 example 빌드는 이어서 진행한다. 전체 검사만으로 native 분기 인수를 완료로 표시하지 않는다.

실제 첫 natural case가 InvalidData로 실패했으며 원 host의 Job0/private/input 정리는 정상이다. 첫 helper 단독 호출은 host가 필수 argv를 요구해 usage2였으므로 helper 동작 증거가 아니다. SHA 고정 최소wrapper 재시험은 helper 부모 exit17을 확인했지만 outerhost5초 회수보다 자식10초가 길어 forcedcleanup이었다. 이를 native 인수로 쓰지 않는다. 원 fixture의 실패 경계를 단계·원 HANDLE creation/Wait/Job count·outcome/경과의 bounded JSON으로 계측해 진단한다. 원2초/최소1초/정책을 완화하지 않는다.

계측 재현에서 원 helper 부모는 Natural65를 반환했고, 자기 소유TMP에 System.ArgumentException을 기록했다. no-STD 선택 시작과 valid-STD 단독 wrapper의 차이를 근거로 synthetic helper가 stdin/stderr도 자체 pipe로 제공하도록 수정해 대조한다. 이는 helper의 표준 스트림 준비 수정이며 제품의 inheritfalse/선택caps/Job/기한 정책은 유지한다. 앞선 실패를 소급해 성공으로 바꾸지 않는다.

bounded 예외 stack에서 원인을 확인했다: System.Reflection.RuntimeAssembly.get_Location → FileIOPermission.CheckIllegalCharacters가 선택 런타임의 extended DOS 실행 경로를 거절했다. stdin/stderr pipe 대조만으로는 해결되지 않았으며 그 실패도 보존한다. synthetic helper는 원 native module GetModuleFileNameW를 bounded 조회하고 exact extended DOS prefix만 제거해 같은 자체 child를 실행하도록 수정한다. 제품의 final-path pin/extended CreateProcess 경로·원 Job/정책은 그대로다.

2026-10-06 범위 완료: instrumented source577 전체 MSVC4검사·760passed/13ignored 및 원 artifact/GNU/lock 보존, module-path-real 실제12명령/3case·자연대기10026ms·원Jobactive3 관측·Natural/Cancelled/TimedOut 뒤 부모exit17/원Jobterminaltrue·원파일21·canonical nativeID30·namespace/digest/SHA/cursor17을 root/독립 검토했다. 원 실패와 CLR예외/가설 대조를 모두 보존했다. runtimeimage 검증은 원 selected leader 범위이며 descendant 자체imageID 독립조회·실제payload/UI·정식installer/장시간/전체P0는 미검증이다. 현재 CPU594676ba…의 새 loaded GUI 성능 측정을 이어서 진행한다.

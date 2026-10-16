# S07: Windows 사용자 설치의 active coordinator

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R11, R08
- 담당/수정 범위: root — 공유상태/인터페이스·통합·native검증; metrics_recovery — /tmp 기존기반과 최소활성화/복구 계약 읽기 감사; diagnostic_cache — 현재 S00 인수 종료 뒤 분리파일 구현 여부 배정; soak_recovery_audit — 현재 S00 증거 인수 종료 뒤 독립실패 검토 배정
- 선행 작업: 준비·등록·receipt·정확 제거와 개발 CLI scoped DONE. 현재 GUI/IME의 Rust source freeze 중에는 제품 파일을 수정하지 않는다.

## 사용자 결과

준비한 앱 묶음을 사용자가 실행할 설치 버전으로 명시 선택하며 재시작 시 설치 일관성을 검사한다. 부분 prepare/등록을 정상 active 설치로 표시하지 않는다. 공개배포·관리자등록·PATH·updater를 전제하지 않는다.

## 계약과 범위

README→PRODUCT→CONTEXT→R11/R08→ADR0005와 앱설치 capabilities를 읽고 기존 DB/도구 registry 권한과 분리한다. 초기 감사는 /tmp 읽기 자료만이며 root가 durable coordinator와 부분 복구/동시성/원active보존·boundedjournal 계약을 확정한 뒤 파일 담당을 배정한다. 새 persisted 상태는 CONTEXT/ADR에 기록한다. 기존 사용자 appdata/Documents/실제 설치를 시험대상으로 사용하지 않는다. source/lock/shared변경은 root 담당이며 이번 상태의 구현완료를 미리 주장하지 않는다.

## 인수 조건

명시 첫 설치의 active선택·readonly재검사와 원data/외부receipt보존; 실패·취소·기한·중간변조·unknown 파일·기존설치/동시실행을 거절/보존. 복구 기록은 명시 관측/권한과 실제 작업을 구별하고 경로자료에서 삭제권한을 만들지 않는다. 실제 Windows syntheticTEMP 및 root/독립원증거 검증 후 scoped완료한다. upgrade/rollback 정식수명과 cleanOS/서명은 별도 미검증으로 남길 수 있으나 P0전체 완료로 표시하지 않는다.

## 검증과 handoff

현재 최소 공유계약 읽기 감사 단계이며 실제 제품동작 추가 전이다. S00 현재바이너리 검증 완료전 sourcefreeze 유지. 의존성/lock 변경 없음.


root 최소수직범위 결정: 이미 검증된 RegisteredPairInspection을 caller가 명시해 새 private active-selection root에 선택을 기록한다. 재시작은 active/registration/receipt 세 경로를 모두 caller가 명시하고 전체binding을 readonly로 대조한다. marker가 가리킨 임의 경로를 자동 탐색하거나 실행하지 않는다. 이는 선택 일관성의 개발용 기반이며 정식 설치 journal/upgrade/자동실행 완료가 아니다. 실제 prepare→register→receipt→선택→producer종료→readonly재검사 인수 후 다음 full coordinator를 확장한다. 기존 폭넓은 workspace/journal 제안보다 작은 범위로 failure/원data보존을 먼저 검증한다.

파일담당: root app_install facade/sharedAPI·schema최종·등록/receipt기반연결·통합; metrics_recovery /tmp active_schema.rs와 파싱회귀 초안; soak_recovery_audit /tmp 원caps/reopen/overlap·최소공유API 읽기검토. diagnostic_cache는 현재 ownerIMM 실제후독립 검토를 먼저 끝낸다. 현재 sourcefreeze 종료 전 프로젝트Rust수정 없음.

현재 sourcefreeze 종료 후 root가 active_schema.rs 초안을 검토해 통합하고 windows_caps.rs의 FreshDestination readonly사전관측·원ReadRootfinalpath, registration_reopen.rs/receipt_reopen.rs의좁은 원pathdelegates를추가했다. active_selection.rs runtime초안은 diagnostic_cache /tmp, actualCPUfixture초안은 soak_recovery_audit /tmp으로파일분리한다. 최종공유schema/rootexport는root가조정하고 아직컴파일/인수완료가아니다. FreshDestination은원ancestors·즉시parent noDELETEshare·existingtarget거절을추가하며 create/discard권한을제공하지않는다.

root통합검토에서 create_new 실제API·large opaque failure boxing·escapedJSON예산을수정했다. readonly sourceQA 새blocker없음. 회귀는 schema5/runtime기본4+보충3으로확장하며 두publiccreator승자1/패자noadopt, privateACL동일·samebytes/differentmarkerID 거절, postwrite실제취소 cap보존을검증한다. runtimepostwrite실패뒤 canonical marker가조회가능해도이전operation/sync완료증명이아님을문서화한다. 실제fixture의foreign시험을 별도active-foreign에옮겨normalrootpristine를유지하고freshprocess명시3path+canonicalSHA readonly mode를추가했다. 아직native실행전이다.


2026-10-06 scoped DONE: Windows native 전체 fmt/clippy/test/release PASS, 설치 관련87회귀 중 새schema5/runtime7 PASS. 실제CPU53d12740…/7,917,568B(이전대비+0B), lock/dependency불변. 실제synthetic6cases·producer종료후별도process canonicalSHA·정상묶음원nativeID10·기존두등록쌍12파일/notice/sentinel/cursor17보존을root/독립대조했다. [실제증거](../validation/evidence/windows-active-selection-real-cpu-20261006-continue/root-completion-review.json), [전체검사](../validation/evidence/windows-active-selection-workspace-20261006-continue/result.json). 앱실행/자동탐색/정식설치journal·upgrade/rollback·P0전체완료는범위밖이다. 다음 [CLI연결](S07-windows-active-selection-local-cli.md)을진행한다. 위단락은진행이력이며최종인수는이단락이다.

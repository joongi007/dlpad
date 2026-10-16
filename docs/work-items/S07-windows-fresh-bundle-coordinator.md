# S07: 새 설치 묶음의 단일 생성 조정

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — 공유 API·통합·user_bundle.rs·facade·실제 검증; diagnostic_cache — /tmp windows_caps 및 transaction destination seam 초안; soak_recovery_audit — /tmp receipt/active destination seam 초안; metrics_recovery — 인터페이스/실패 인수 독립 검토
- 선행 작업: 명시 준비/등록/receipt/active 원 capabilities와 source577 원 Job 감독 인수

## 사용자 결과

명시한 새 폴더 하나에 등록·receipt·선택 기록을 순서대로 생성하고, 선택 기록을 마지막에 게시한다. 중간 실패는 기존 설치·데이터를 보존하며 완료로 표시하지 않는다.

## 계약과 범위

대상은 새 caller-owned 합성 root이며 KnownFolder 기본값·기존 설치 교체·shortcut·자동 startup discovery·journal 회수는 후속이다. root가 기존 Source preflight와 source pins를 공유한 root factory seam을 선택한다. verified payload 이전에는 outer root도 생성하지 않는다. fixed registered/receipt/active 세 자식은 원 outer HANDLE-relative FILE_CREATE로 만든다. 외부 parent path를 다시 열어 생성하지 않는다. 현재 ActiveSelectionRecord를 재사용하며 공유 schema/Cargo/lock은 바꾸지 않는다. opaque 실패 값은 원 outer·확인된 단계caps를 보존하고 Drop/delete/자동회수를 하지 않는다.

담당 파일은 crates/engine/src/app_install/windows_caps.rs, transaction.rs, receipt.rs 또는 실제 receipt 구현 파일, active_selection.rs, 새 user_bundle.rs와 crates/engine/src/app_install.rs다. 정확한 receipt 담당 경로는 읽기 확인 후 root가 기록한다. 단계별 직렬 의존이 있으므로 구현은 /tmp에서 독립 seam만 병렬 준비하고 root가 통합한다. 현재 GUI 비용 측정 동안 Rust/source/binary를 변경하지 않는다.

## 인수 조건

잘못된 SHA·source alias·취소·기한은 root 생성 전 거절한다. foreign/existing root를 채택하거나 덮어쓰지 않는다. after-copy/등록/receipt/active failure에서 이전 묶음과 source cursor/ID/SHA를 보존한다. 완료는 원3root의 전체 canonical/native consistency 검사 후에만 반환한다. cold readonly 검사에서 missing/foreign/namespace/digest/동일bytes새ID를 거절하고 삭제 권한을 부여하지 않는다. 실제 fresh fixture의 생성·cold검사·원Job 실행과 실패 경로를 Windows native로 확인한다. 전원손실 atomicity·upgrade·전체 P0 완료로 확대하지 않는다.

## 검증과 handoff

공유 계약을 선택해 초안을 준비 중이다. 아직 구현·native 인수 완료가 아니다. source577 성능 측정 종료 후 root가 소스 통합과 전체 검사를 진행한다.

병렬 초안에서 구체적 제약을 발견했다: fresh payload/metadata의 writable·DELETE 원cap을 보유한 채 cold ReadRoot.open_file의 shareREAD 검사를 실행하면 공유 모드가 충돌한다. 원cap을 버리고 경로를 재개방하는 방식은 원cap 보존 계약에 맞지 않는다. root는 기존 cold API/share 정책을 유지하고, 원 FreshRoot의 축소 권한 readonly 복제 및 원root상대·expectedNativeID 지정 SHAREALL observer를 별도 private seam으로 검토한다. 원freshcap의 share 정책이 외부 새writer를 제한하는 것과 readonly observer 자체가 쓰기를 금지하는 것은 구분한다. 생성·쓰기·DELETE 권한을 observer에 부여하지 않는다. metrics_recovery는 pair/registration/receipt·coordinator 초안, diagnostic_cache는 기존 create_child stage를 보존한 windows_caps 초안을 맡고 root가 통합한다. 아직 초안이며 실제 bootstrap 완성을 주장하지 않는다.

추가 root 결정: 기존 FreshRoot 원생성 권한에는 FILE_LIST_DIRECTORY가 없으므로 해당 권한을 readonly 복제 과정에서 새로 만들어낼 수 없다. 원 fresh directory 생성 시 읽기 목록 권한을 먼저 포함한 뒤, observer는 LIST/ATTR/READ_CONTROL/SYNCHRONIZE의 부분집합을 DuplicateHandle로 받고 DELETE·WRITE를 제외한다. 기존 ancestor도 축소 권한으로 복제한다. 이 변경은 새 읽기 목록 capability이며 기존 삭제·쓰기 권한을 확대하지 않는다. 일반 cold 검사 공유 정책은 유지한다. 세부 Windows 실제검사 전이라 시행 완료·권한 인수로 표현하지 않는다.

root는 원sourceSHA·stagedSHA·exactdiff를 대조하고 Windows caps/transaction/registration/registration_reopen/receipt/receipt_reopen/registration_pair/active_selection·app_install facade·새 user_bundle 및CLI의11Rust파일을 통합했다([통합 기록](../validation/evidence/windows-fresh-bundle-integration-20261006-continue/integration.json)). readonly원cap변환은 expectedIDs/fullcanonical/SHA/namespace를재사용하며 원 fresh쓰기cap은 Drop까지 유지한다. receipt전/active전과factory내부/최종 exactouter namespace 검사로 unknown항목을 보존하며publication을거절한다. UserBundleFailure는 private boxedstate로 실패반환시에만boundedheap할당하고 기존public getters는 유지한다. 별도 StageFailure도 boxing하여 큰 오류variant를완화했다. 이는 실제Clippy실패해결을 주장하는 것이 아니라 통합전코드검토에 따른 선택이다. 신규schema/dependency/lock변경없음. 실제Windows포맷·전체4검사를 진행하고 native묶음생성/재시작검사 후에만scoped완료로표시한다.

첫 실제Windows fmt는 exit1 parseFAIL로 보존했다. windows_caps 초안의4추가테스트가 마지막cold export의brace 안으로삽입되어있었고 root가 cfg(test)module 안으로이동하고coldexport를복구했다. 독립read-only검토는구문오류를잡지못했으며그보고만으로compile성공을주장하지않는다. 원FAIL/private원Job 자연종료/input복원과교정실행은분리저장한다. 이후Cargo 실제전체4검사로확인한다.

실제 Windows 전체 검사 첫 시도는 fmt/clippy 통과 후 테스트 exit101로 중단했다. 열린 payload HANDLE의 공유 정책 때문에 root 교체 시험의 디렉터리 rename이 Windows 오류5로 거절됐다. root는 원파일 공유 권한을 완화하지 않고 빈 원root HANDLE/ID를 유지한 교체 조건으로 시험을 수정했다. 릴리스 빌드는 당시 미실행이며 새 전체 검증 결과와 구분한다. 독립 검토에서 preflight 실패의 stage error 존재를 capability 보유로 잘못 보고하는 추가 결함도 발견했다. 현재 전체 실행의 입력을 보존한 뒤 getter 및 실패 경로 회귀를 수정하고 다시 검증한다.

빈root 교체 시험 교정 후 실제 전체4검사는 root/독립 PASS다: 774passed/0failed/13ignored·app_install101개, source579 및 원 GNU/lock/payload 증거 대조. CPU f1f3e4aa…8,709,632B·CLI76d4103b…841,728B. evidence/windows-msvc-fresh-bundle-corrected-workspace-20261006-continue의 root-review.json·independent-review.json에 저장했다. 이어서 retained_capabilities_present를 stageerror 존재 대신 실제 Prepare/Receipt root·filecap 및 Register/Active의 필수 원묶음 보유에 연결하고 wrongSHA/existingouter 두 회귀 assertion을 추가했다. 해당 수정의 새 전체검사와 실제 CLI 인수는 진행 중이며 직전 PASS를 새 입력에 소급하지 않는다.

범위 인수 완료: capability교정 source579의 전체4검사·774/0/13 및 실제 fresh CLI14명령/11행·GUI3case·원NativeID10, 별도 복사중control3명령/2case를 root/독립검토했다. 밝은테마PNG와읽기전용SQLite Light/integrity·원source/등록/receipt/active/foreign 보존을확인했다. 최초parse/시험rename/collectorcasekey FAIL은보존한다. 이source579의정확map 인수이며 이후wake계측통합 source579는같은항목수지만별도검증중이다. durablejournal·upgrade/rollback·기본설치위치/UI·cleanOS/서명·전체P0 완료는범위밖후속이다.

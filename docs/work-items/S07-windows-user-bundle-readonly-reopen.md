# S07: 명시 개발 묶음 cold readonly 재검사

- 상태: done (명시 readonly 재검사 범위; 정식 installer와 분리)
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: metrics_recovery — /tmp user_bundle_reopen.rs/example; root — facade/CLI/문서/통합
- 선행 작업: fresh bundle scoped DONE, ReadRoot/ActiveSelectionInspection

## 사용자 결과

하나의 명시 outer 경로로 고정 세 자식의 현재 readonly consistency를 재검사한다.

## 계약과 범위

원 readonly outer와 기존 ActiveSelectionInspection 수명을 유지하고 namespace 전후 검증. 저장schema/deps0, 자동 경로 탐색/repair/create/delete/launch0. ADR0005 Proposed 유지. fresh 생성caps를 cold권한으로 채택하지 않는다.

## 인수 조건

원 outer namespace 정확3children, 기존pair/active canonical/ID/SHA 검증, unknown/missing/mixed 실패 보존, 취소/기한 inspect+verify4실패 bytes 동일. 원 실CPU/notice/nativeIDs는 example streaming snapshot으로 보고하며 root의 독립 query/원source/sentinel 후검토가 필요하다. meaningful unit 초안은 pre-cancel/deadline missingroot 미생성을 확인한다. 추가 실제valid fixture unknown/replacement 회귀는 root 통합 후 기존 생산fixture 루틴과 연결한다.

## 검증과 handoff

컴파일/format/Clippy/native 미실행. root 필요 export: cfg(windows) mod user_bundle_reopen; pub use user_bundle_reopen::{inspect_user_bundle,UserBundleInspection}. 신규 example Cargo 선언은 root 담당. 실행 argv 예: windows_user_bundle_reopen_fixture.exe --fixed-user-bundle-reopen ABS_BUNDLE. 90초 내부기한/16KiB JSON. 외부 임의경로 삭제권한이나 역사적 소유권을 부여하지 않는다.

병렬초안의ReadRoot/기존selection API를독립검토했다. root는13번째inspect-user-bundle의CLI/parser를/tmp에작성하고감사에서발견한main operation allowlist누락을수정했다. help·확인된pair/receipt/registration표시와미관측retainedcapcount null/count_reportedfalse를보강했다. 별도소유negativefixture의foreign sibling/누락child 거절·원ID/SHA복원을추가했다. 실제source통합/compile/native인수전이며현재CloseWait sourcefreeze종료뒤진행한다. 저장schema/의존성/Cargo.lock변경없다.

수집기초안감사에서example의공급SHA만으로실행되는gate를발견해실제MSVC Cargo example build proof/rawguard/source/artifact/wholeproof 결합을필수로보강했다. decodedwhole 원rawSHA와immutable복사본SHA를직접대조한다. root producer의actualwhole result키차이(effective_values없음)를확인하고원archived effective-configuration values를bounded SHA-pin해사용하도록수정했다. 11nativeID전후보존과canonical record의독립비교는별도범위이며후자는false다. 미실행stage이며현재제품소스동결을유지한다.

원stage SHA를대조한뒤root가새module/example·13번째CLI·두proof-bound Python 수집기를통합했다. 공유facade export만추가하며Cargo/lock/schema불변이다. 새source전체4검사·실제example/CLI 성공및typedfailure/negativefixture/원11ID보존인수를이어진행한다.

현재source581 exactmap Windows fmt/clippy/test/release실제exit0·813/0/13을root/독립인수했다. CPU75d05b1462f7baee4b1f9d61b469d238b93e1daa61f2bbce9d502e4c8f634b1d/8,710,656B는크기불변이지만SHA가달라기존342…성능으로소급하지않는다. CLI0d1b240c62c4ba3060e51e5d8a9705b669a8555de8752da62e472353310d21de/847,872B는6,144B증가, dependency/schema/Cargo.lock변경없다. 현재원ID/typed4/negative2 실제example을proof-bound빌드한다.

실제첫readonly collector는원nativequery4개/example exit0·typed4/negative2후JSON말미줄바꿈누락때문에hostterminal이같은행에붙어strictguard가ValueError로거절했다. raw실패evidence와actualexample exit0를분리보존했다. root는example출력에newline을추가하며terminalguard를완화하지않고같은검사를새source전체4검증후다시실행한다. 최초collectorFAIL은완료근거가아니다.

최종 newline교정 whole4·813/0/13과 실제17명령/7행·원11nativeID/7파일SHA, typedcancel/deadline4·별도소유namespace negative2를root/독립인수했다. root는canonical레코드와원ID 및registration의도메인분리digest/receipt/active 결합을직접대조했다. 원FAIL collector/개행문제를보존하고strictterminalguard는유지했다. 같은최종CPU75d…의기본18구간18PASS(최대CPU0.9358915954219397%, WS72,212,480B), 30k합성files/3readonlyDB 보존·원raw/픽셀·pins도root/독립검토PASS다. API의다른유효pair혼합별도새fixture/정식installer/자동repair/실제crash journal·physical/cleanOS는범위밖잔여다.

실제 실행: `python3 scripts/windows_msvc_workspace_probe.py --configuration /tmp/dlpad-msvc-workspace-stdout-plan.json --output docs/validation/evidence/windows-msvc-user-bundle-readonly-reopen-newline-workspace-20261006-continue --reuse-owned-target`(native실행에는절대output사용); `python3 scripts/windows_msvc_user_bundle_example_build.py --proof .../result.json --output ...`; `python3 scripts/windows_msvc_user_bundle_reopen_collect.py --proof ... --output ... --bundle OWNED_TEMP/bundle --example .../inputs/example.exe --example-proof .../result.json --example-sha256 1a4f8c7cb7ea4df58ebf4c262597f2d3b4ed438451630b644cc05368f012a90f`; `python3 scripts/windows_msvc_whole_gui_cost_probe.py CURRENT_PROOF NEW_EVIDENCE --no-concurrent-dlpad-probes`. 모든정확argv/rawexit/원Job·source는[최종whole](../validation/evidence/windows-msvc-user-bundle-readonly-reopen-newline-workspace-20261006-continue/result.json), [실제재검사](../validation/evidence/windows-msvc-user-bundle-readonly-reopen-newline-real-20261006-continue/root-review.json), [성능](../validation/evidence/windows-msvc-user-bundle-readonly-reopen-default-cost-real-20261006-continue/root-review.json)에보존한다.

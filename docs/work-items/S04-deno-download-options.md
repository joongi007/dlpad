# S04: 선택 Deno 다운로드 옵션·Attempt 진단 연결

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R07, R11
- 담당/수정 범위: metrics_recovery — crates/core/src/options.rs만; diagnostic_cache — crates/engine/src/deno_runtime.rs만; root — engine download.rs/lib.rs, app·test Options constructors, 진단 연결과공통문서
- 선행 작업: Deno 정확output/parser/ownedmetadata process 및 archive 검증선택 실제공식PASS

## 사용자 결과

명시 선택한 Deno 경로를 metadata/download의 같은literal argv에사용하고 실제버전을 Attempt component와 실패진단에저장한다. 기존저장옵션의runtime없음은defaultNone으로복원한다. UI경로선택과공급 채택은후속이지만 backend사용자의typed옵션은동작해야한다.

## 계약과 범위

DownloadOptions.deno Option<PathBuf>를 추가한다. storedversion1은serde default/skipNone backwardcompatible로기존payload를읽는다. plan은absUnicode/noParent/noControl/boundedpath검증후 --no-js-runtimes, --js-runtimes, 단일OsString deno:<path>를추가하며shell/인위따옴표를쓰지않는다. raw옵션의runtime/remotecomponents override는기존allowlist로거절한다. worker는canonical원file/type/nativeID/owned --version/strict3line stablehosttarget/버전bounded확인및posthash로선택한다. root는canonical경로를option snapshot/metadata/download동일plan에반영하고 기존download_attempt_components(component='deno')를재사용한다. 새table/Cargo/deps는없다.

공식Deno2.9.7 stable출력근거를활용하되 다른runtime버전/EJS실사이트호환이나publisher trust/notice를완료로표현하지않는다. 제품remote solver다운로드를새로허용하지않는다. 긴I/O는기존worker에서실행한다. 파일동시수정없으며root가공유API·constructor 변경을통합검증한다.

## 인수 조건

저장옵션구버전복원/roundtrip/runtimepath·Windowsdrive·한글공백literal/unsafe입력·rawoverride 거절; 실제합성metadata/download동일argv·Attempt버전재열람/실패진단; version/nonzero/tamper/cancel/timeout 실패의옵션·DB·foreign보존. 다른OS실제runtime/GUI·EJS사이트/최신전체workspace는따로검증한다.

## 검증과 handoff

작업시작범위기록. 직전새Deno전체collector는tools.rs module순서fmt차이로실패하고소스불변종료를확인했다. root개별skip_children rustfmt로교정했다. 선행Linux63PASS/Windows42PASS및공식제품selectPASS와새옵션연결검증을구분한다.

## core 옵션 구현 인계

DownloadOptions.deno Option<PathBuf>와 StoredOptions의 serde(default,skip_serializing_if None)를 추가했다. 기존 version1 JSON은 deno 없음으로 복원되고 None은 기존 argv와 JSON을 유지한다. 선택 경로는 절대 Unicode/control 없음/parent traversal 없음/32768바이트 이하를 검증한 뒤 --no-js-runtimes, --js-runtimes, 'deno:'+원 path 한 OsString으로 plan에 넣는다. 원 canonical identity는 engine 담당이다. 기존 숫자 advanced whitelist는 그대로여서 js-runtimes/no-js-runtimes/remote-components raw override가 거절된다.

3회귀는 기존 JSON/선택 roundtrip 및 argv 동일, 한글·공백·문자그대로와 Windows drive literal, 잘못된 상대·parent·control·oversized 경로 및 raw override 실패를 검증한다. core 내부 fixture constructor에는 denoNone을 추가했다. 개별 rustfmt(skip_children) exit0이고 Cargo는 root 통합을 기다려 실행하지 않았다. 다른 source/schema를 수정하지 않았다. source SHA eb34fea9f3366f9c3d8386e77b3e513c34c91dd39ccd442b57a2cada3d0edba4.

## 첫 통합 컴파일 실패 교정

실제 Linux/Windows all-target Clippy는 새 Deno 시험이 기존 sibling tests::options fixture를 scope 없이 참조하여 E0425로 실패했다. 공통 domain fixture를 tests-parent 범위 pub(super)로 제한 공개하고 sibling 시험에서 명시 import하여 교정했다. 제품 옵션/argv 정책과 시험 assert는 변경하지 않았다. 최초 실패를 보존하고 root 재검증을 기다린다. 자체 Cargo는 실행하지 않았다.

## Deno 다운로드 옵션 통합 검증 (2026-10-04)

선택 Deno를 하위 호환 optional 저장 옵션으로 연결했다. metadata와 다운로드는 동일 canonical runtime을 literal argv로 전달하며 실제 버전을 Attempt와 실패 보고서에 남긴다. runtime 검사 중 확인된 사용자 취소는 Cancelled/Paused로 분류하고 정리 미확인은 보존한다.

Linux core24개·Deno관련12개 및 CPU all-target Clippy가 통과했다. 최신 Windows GNU CPU workspace Clippy와 test는 terminal exit0이다. root는 docs/validation/evidence/deno-download-options-native-cancel-classification/{clippy,test}.json의 source_before를 현재 파일과 독립 대조하여 불일치0, source_unchanged 및 input_desktop_unchanged를 확인했다. Windows 공식 runtime 실행·GUI runtime 선택·최신 전체 release 빌드·macOS와 전체 P0는 미완료다. 이전 snapshot의 typed argv/Attempt 미연결 기록은 현재 backend 상태와 구분한다.

# S11: 업데이트 ZIP에서 별도 새 설치 준비

- 상태: done
- 분류: feature
- 우선순위: P1
- 요구사항: R08, R10, R11
- 담당/수정 범위: Root crates/engine/src/app_install/update_prepare.rs·app_install.rs·공유 문서/최종 검증. release_audit 별도 worktree app_update/bundle.rs와 bundle/tests.rs의 crate-private streaming bridge. updater_audit는 기존 feature CI 읽기 감시만 담당. 같은 파일 동시 수정 없음
- 선행 작업: 서명 후보/원 StagedPackage, read-only BundleObservation, 기존 create_journal_aware_bundle와 FreshRoot

## 사용자 결과

검증된 원 ZIP에서 기존 앱과 별도인 새 경로에 다음 버전을 준비한다. 기존 설치와 미디어·외부 도구·프로필은 변경하지 않는다. 이 API는 아직 UI에 연결하지 않으며 자동 전환/실행/rollback 완료를 뜻하지 않는다.

## 계약과 범위

기존 원 HANDLE·Windows private FreshRoot/CreatedFile와 journal-aware 설치 기능을 재사용한다. ZIP 전체 검증 후 최대7개 regular member를 원 caller-owned empty File로 제한된 버퍼로 streaming하고, 같은 member SHA/크기와 원 ZIP SHA/identity를 다시 확인한다. source path를 재오픈하지 않는다. 실패한 부분 출력과 원 capability는 보존하며 경로 기반 recursive cleanup은 하지 않는다. 성공에도 기존 A나 global launcher 선택·프로필 DB·네트워크·운영키·프로세스는 변경하지 않는다. 기존 development_only/법적 미완료 표시는 유지한다. 새 dependency/schema/lock 변경은 없다.

## 인수 조건

정상 ZIP에서 새 고정 설치/journal의 실제 identity·manifest 일치를 검증한다. 취소/timeout/손상/기존 target·foreign sentinel/부분 준비 실패는 원 ZIP과 이전 설치를 보존한다. streaming bridge는 namespace·empty regular/distinct outputs·손상·중단을 검사하고 메모리 상한64KiB를 유지한다. Windows 실제 engine strict lint와 회귀, Linux 해당 parser 시험을 실행한다. 전체 교체/rollback으로 보고하지 않는다.

## 검증과 handoff

Root가 실제 Windows 원 stdout과688개 source pin을 확인했다. `cargo clippy -p dlpad-engine --all-targets --locked --offline -- -D warnings` exit0; `cargo test -p dlpad-engine --lib --locked --offline app_update -- --test-threads=1` 51PASS/0FAIL; `cargo test -p dlpad-engine --lib --locked --offline app_install::update_prepare -- --test-threads=1` 7PASS/0FAIL/1ignored; 이어서 해당 ignored 실제 ZIP fixture를 명시 선택해1PASS/0FAIL을 확인했다. 네 명령 모두 exit0이며 `.local-tools/update-integration-20261018/native-preparation-v1/native-tests.log`에 원 결과가 있다. 경과45.38초, natural-zero/final-zero=True, final-active0, forced=False, private-desktop-closed/input-desktop-unchanged/source-unchanged=True다. 전체 workspace/release 새 빌드 통과 주장과 구별한다.

실제 fixture는 CI49c의 installer ZIP SHA2569ed4360afd65c060401d4c2eb58cd64c5d03a33aecbeaafaf9baed6ad3200dc5를 입력으로 사용했다. 크기/원 SHA를 고정하고 test-only Ed25519 키로 합성 후보를 만들어 원 StagedPackage를 거쳐 새 고정 bundle/journal와 manifest5개의 native identity/hash를 검증했다. 운영키·공개 signature·네트워크 전송·프로세스 실행을 검증한 것이 아니다. 이 시험은 환경변수를 명시하는 ignored fixture이며 소스 checkout/일반 CI가 로컬 ZIP에 의존하지 않는다. 이전 앱/프로필/미디어/외부도구 sentinel 보존, 기존 세 target/foreign 보존, 취소/기한/손상 preflight0출력, invalid PE의 부분 원capability/7파일 보존, readonly seal/foreign namespace 거절을 실제 시험했다.

CreatedFile writer를 그대로 기존 pin_source에 넘기면 share-read가 원 WRITE/DELETE와 경합하므로, reading_pin(ReOpenFile) 원 readonly HANDLE을 먼저 보유하면서 writer를 닫고 기존 pin_source로 share-read를 고정한다. 경로 재오픈·원 HANDLE 공백·공유 cap API 변경 없이 실제 Windows 성공을 확인했다. Drop은 원 파일을 지우지 않는다.

Root Linux workspace targeted app_update45PASS/0FAIL이며 기존 media_preview.rs:591 unused_mut 경고가 남아 Linux strict workspace 검증 완료라고 하지 않는다. `cargo fmt --all -- --check` exit0. 하위 agent의 별도 pinned1.94 harness37PASS/strict Clippy 보고만으로 인수하지 않고 Root가 원 두파일 hash와 bridge 코드를 검토하고 위 native/Linux 시험을 통합 실행했다.

새 dependency/lock/schema 변경0, UI idle worker/network/추가 cache 생성0이다. 요청 시 original ZIP 여러 전체 hash와 member stream/readback·기존 설치 verifier의 추가 I/O가 발생한다. 각 streaming 버퍼64KiB·metadata 각각64KiB·7member/전체 expanded256MiB 한도를 유지하며 이 수치는 프로세스 RSS 상한이나 지연 SLA가 아니다. 새 PE 크기는 후속 빌드 측정 대상이다.

이 작업의 인수는 별도 새 설치 준비 API까지다. 실행 중 A의 CloseWait/Job0·typed supervisor IPC·B readiness/profile compatibility·append-only launcher 전환·실패 복귀·운영 trust key·native UI의 실제 교체는 후속이다. 운영 교체 완료로 표시하거나 현재 수동 설치 안내를 바꾸지 않는다.


동일 engine 구현에 notice fixture 수정만 통합한 뒤 Windows engine 전체도 주 harness635PASS/0FAIL/18ignored와 filtered child1PASS를 확인했다. 첫 실행의 guardian 선행 빌드 누락8FAIL은 보존하고, 원 guardian build 뒤 재실행해 인수했다. 전체 workspace/GPU/공개 배포/실제 앱 전환 성공을 이 결과로 대체하지 않는다.


후속 c5963b5 hosted 전체 CI39066254029도 통과했다. 새 원 installer ZIP(SHA2566af3ab081610f121afff5e017561c61c2d4dfe5aa8687691fc59547e67f83f57)을 test-only candidate로 사용해 동일 명시 fixture1PASS/0FAIL을 다시 실행했다. 운영키/자동 앱 실행은 제외한다. CPU10,430,464B·installer4,423,680B로49c 대비 각각512B 증가했다. 이는 모든 후속 변경을 합친 PE 차이이며 모듈별 원인/메모리 비용 분해가 아니다.

# S04: 도구 화면 첫 진입의 기존 설치 표시

- 상태: complete
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R07, R13
- 담당/수정 범위: root crates/app/src/accessible_tools.rs 통합·전체 검증·문서; release_preparation은 /tmp 후보와 시험만 작성한다. Cargo/schema/lock 변경 없음.
- 선행 작업: 수동 설정 및 managed Use 실제 Windows GUI 인수

## 사용자 결과

도구 화면을 처음 열면 기존 설치와 활성 도구가 표시된다. 사용자가 설치 목록 갱신을 먼저 찾아 눌러야 하는 단계를 줄인다.

## 계약과 범위

기존 bounded worker·mutation gate·소유 cache 정책을 재사용한다. UI thread에서 도구 조회/해시/복구 I/O를 실행하지 않는다. 최초 enabled 진입에서 한 번 갱신하며 disabled 상태에서는 보류한다. 실패를 매 frame 자동 재시도하지 않는다. 긴 canonical 경로 표시 개선은 별도 후속이다.

## 인수 조건

첫 진입에서 명시 Refresh 없이 기존 active inventory/Use를 실제 private Windows GUI로 확인한다. 빈 목록, disabled 이후 enabled, 반복 frame 중 중복 submit 방지, 실패 이후 명시 Refresh, 종료 중 작업의 기존 안전성도 검증한다. 수동 경로 선택·저장 정책과 외부 파일 보존을 유지한다. 실제 whole 및 새 설치 후보 검증 전 완료로 표시하지 않는다.

## 검증과 handoff

실제 첫 진입 PNG와 source에서 inventory=None이며 자동 조회가 없는 것을 확인했다. managed Use의 기존 명시 갱신 인수는 통과했으며 이를 실패로 재분류하지 않는다. 구현·실행은 후속이며 현재 source 변경 없음.

root는 accessible_tools.rs에 pending flag 한개와 기존 worker를 쓰는 최초 enabled Refresh를 통합했다. 기존 mutation gate와 실제UI context를 유지하고 제출직후 controls를 busy로 갱신한다. 실패자동재시도 없음, explicitRefresh 유지, implicitUse/install 없음. Cargo/lock/schema/중앙문자열 변경없다. rustfmt --check 통과. 첫 scoped명령은 잘못된 package/target 선택으로 Rust시험 전 exit101이며 실패로그를보존했다. 실제 package dlpad의 dlpad-cpu 대상으로 교정한 두 번째 실행은 진행중이다. 실제 privatefirstopen 및 whole/새설치ZIP 인수는 후속이다.

최신 전체 fmt/clippy/test/release: windows-msvc-tools-first-open-whole-20261011/root-review.json 1,063passed/0failed/28ignored/43harness. rootsource620·원raw/naturalJob0·원SHA를독립대조했다. actualscoped16회귀통과도별도root검토했다. CPU SHA806bf03e44b897cb082d92148563b044c1f7b47ce7cb607d36d1319b1081e129/9,625,600B로직전candidate와크기동일. worker/channel/의존성/lock/schema 추가없음; 새초기Refresh의 실제idle/fixture비용은미측정이며 이전수치를소급하지않는다. 현재실제private GUI noManualRefresh 인수를실행중이다.

실제 private Windows managed fixture의 save/reopen/firstopen Use/reopenManaged 네 단계와 closed SQLite4 integrity를 root가 검토했다. 명시 Refresh 요청 없이 원 active version 및 canonical path와 enabled Use를 관측했고, unsaved 다른 도구 draft를 유지하면서 선택한 slot만 Automatic으로 저장했다. PNG를 직접 확인했다. 전체 source620/raw natural0/Job0/원 파일SHA/nativeID를 대조했다. 빈 목록·disabled·반복 frame·실패 명시 재시도·stopped 경우는 실제 Windows scoped16 회귀로 검증했으며 별도 빈/foreign GUI 인수로 표현하지 않는다. 새 설치 묶음 인수와 최신 idle 비용은 진행 중이다.

새 설치ZIP exact7member와 실제5파일 설치·재검증을 root 독립 검토했고 README 현재 링크를 새 후보로 전환했다. 이번 최초 Refresh 기능은 scoped 완료다. 최신 idle/긴 canonical path 표시/수동 tool executable browser/물리 OS 최종 인수는 별도 작업이며 전체P0 완료가 아니다.

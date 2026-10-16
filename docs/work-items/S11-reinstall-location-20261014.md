# S11: 삭제 후 같은 위치 재설치 경로 진단과 복구 안내

- 상태: done (삭제 후 재설치 scoped 인수)
- 분류: bug
- 우선순위: P0
- 요구사항: R11, R01
- 담당/수정 범위: Root — crates/app/src/installer_main.rs, installer_journal_choice.rs, README/CONTEXT/ADR0005, 사용자 실행 안내/상태 문서 및 이 작업 기록. installer_catalog.rs는 읽기 확인만 했고 수정하지 않았다. 하위 installer_batch — 독립 private 재현 helper와 별도 작업 기록. windows_auto — 제거/journal 정책 읽기 조사와 별도 작업 기록.
- 선행 작업: v24 설치 후보

## 사용자 결과

삭제 후 같은 설치 위치를 선택하면 누락 원인과 다음 행동을 알 수 있고, 검증된 빈 위치에는 재설치할 수 있다.

## 계약과 범위

ADR0005/0009, 기존 retained root·fresh create·삭제 권한을 유지한다. 실제 사용자 설치/데이터는 수정하지 않고 소유 임시 fixture만 사용한다. 남은 journal이나 알 수 없는 폴더를 자동 삭제하거나 덮어쓰지 않는다. Cargo/schema/lock 변경은 Root만 조정한다. Git 조회 exit128으로 기존 .git을 보존하고 독립 helper namespace를 사용한다.

## 인수 조건

1. 설치→제거→같은 target 재설치에서 정확한 오류·target/journal 존재를 관측한다.
2. 소스 payload 누락, target 부재, parent 부재, journal 잔존을 구분하고 다음 행동을 안내한다.
3. 기존 파일 보존·cancel/stale/reparse 거절을 유지하며 성공/실패 회귀와 Windows private 실행을 검증한다.
4. 새 설치본은 실제 빌드/패키지 검증 후에만 완료 또는 최신본으로 표시한다.

## 검증과 handoff

2026-10-14 사용자 MISSING 오류 보고로 재현 착수. 정확한 삭제 방식/버튼/경로는 비동기 질문 중이며 독립 개발을 계속한다. 아직 원인 또는 수정 완료를 주장하지 않는다.


### 원인과 후보 구현

기존 native 제거는 bundle만 제거하고 immutable sibling journal5를 보존한다. 기존 admission은 target 부재/journal 존재를 Existing으로 안내하고, 이어 Verify는 없는 bundle에서 NotFound를 반환해 MISSING을 표시한다. journal 자체삭제·덮어쓰기나 제거완료 추정 대신 packaged 설치에 한해 기본 sibling과 -1..-31의 총32 후보를 worker에서 검사한다. target 부재에는 최초 빈 journal을 fresh create 요청에 사용하며, target 존재에는 기존 서비스의 실제 current bundle 일치가 유일한 journal만 선택한다. cancel/deadline/reparse/상위폴더 확인과 작업 직전 native 재검증은 그대로 유지한다. 명시4path 개발모드는 변경하지 않는다. 새 schema/의존성/lock 변경은 없고 startup worker 검사 상한32/10초가 추가된다. 원 기록32개가 찬 경우 기존 기록은 보존하고 다른 설치 위치를 안내한다.

독립 Rust module tests 최초3 PASS, Windows 설치기 최초43 PASS/0 FAIL/3 ignored와 workspace strict clippy 실제0. 이후 ambiguous multiple-match 거절 회귀와 동일text EDIT 재설정 방지 guard를 추가해 재검사 중이다. v24 실제 private 재현v1은 제거후 Install 비활성을 관측했지만 helper의 Install-ready 가정으로 실패하여 원FAIL을 보존했다. 수정한 재현v2는 실제 활성 Verify를 요청하고 정확한 MISSING을 수집한다. 후보는 아직 배포 ZIP에 포함되지 않았다.


Windows 실제 v24 재현v2 compile0/native0/observer0에서 제거 완료 후 target 부재·journal5 원hash보존·201 비활성/203 활성/default203을 확인하고 실제203 Verify로 사용자 MISSING을 재현했다. Root는 원30pins/5journal/원operationlogs/private natural0/Job0과 after-reinstall.png를 독립 검토했다. 현재shipping3은새build중이므로 원재현 pins3의 재hash는 주장하지 않는다; 원 generated7/staged frozen inputs와 child의 build전후 대조를 보존한다. E/windows-msvc-ui-feedback-v24-reinstall-private-v2-20261014/root-review.json.

최종 후보 Windows 설치기44 PASS/0 FAIL/3 ignored. Module은제거후기록보존/새generation/재시작일치/모호한2개거절/상한/취소를 검사한다. readonly104는ES_MULTILINE이므로WM_SETTEXT의EN_CHANGE loop는실제결함으로확정하지않는다. 동일text reset생략은불필요한update를줄이는guard로유지한다. 전체4검사/release/native후보/최신ZIP은진행중이다.


최종 source671 Windows 전체4검사(fmt, strict workspace/all-targets clippy, test, release)는 모두actual0. test1542 PASS/0 FAIL/42ignored/43harness. Root가현재source·원4log·3PE·GNU원본·lock를독립재검사했다. 전체proof SHA4981392dad59c2cdf3948ec0ede5a7648b1a589e39fdae995d615e7ba43ca70b. 설치기4,432,384B/ff66a5a9efb7fb417a8fab21fc32f2dd3dad068aad80a52574e9c3ce7f0d4027는v24보다6,656B 증가했고CPU/CLI는원바이트동일. 앱CPU 성능을재측정하거나새버전성능성공으로확장하지않는다. [현재 전체검사](../validation/evidence/windows-msvc-ui-feedback-v25-whole-20261014/root-review.json).


패키징 최초graph는일반sandbox의vsock setup 한줄오류로 actualCargo/nativehost 시작전에 실패했다. 원FAIL을보존하고Root는허용된require_escalated privatehost에서rustcversion actual0/private/inputunchanged로환경을구분했다. 승인검토거절·보안변경·WSL재시작은없었다. 같은authorizednative권한으로새graph-v2를실행중이며원FAIL을성공으로바꾸지않는다.


### 최종 인수

새v25 package6phase와child독립review 모두actual0. Root가원1,136입력·474archive members CRC/SHA·456notice source·generated7를독립재검산했다. 설치ZIP17,164,983B/SHA291d2d7a077b2c46494283d4470f34360c8e85296df3287ad45b46c6b4237996,portable18,215,479B/SHA05dfd34ddb82a8032600fe59c5853704f5e8f8c61db83456c9618a940926a915. [패키지 검토](../validation/evidence/windows-msvc-ui-feedback-v25-20261014-package-root-review.json).

actualprivate2installer에서Install201이활성화된후같은target에새journal-1로재설치했고MISSING=false를확인했다. 첫installer0→freshinstaller의actualmatching -1→Verify203 exactsuccess→두번째0/Job0,앱실행0회다. Root가원33pins/5oldjournal/5newjournal/5finalpayload/fullsource671과after-remove/after-reinstall/fresh-verified3PNG를독립확인했다. before/afterprepare의journal_directory_exists는oldBASEjournal을측정하므로새journal의사전존재로해석하지않는다. after-removePNG는admissionpending이며ready201은별도ACK뒤rawproof다. [native 검토](../validation/evidence/windows-msvc-ui-feedback-v25-reinstall-private-v3-20261014/root-review.json).

README/Windows사용자안내/현재P0상태는v25로갱신했고이전v24README는relative links를원대상으로보정하여보존했다. 실제코드검사명령: python3 -B scripts/windows_msvc_workspace_probe.py --configuration docs/validation/evidence/windows-msvc-settings-ime-commit-whole-resumed-20261013/effective-configuration.json --output docs/validation/evidence/windows-msvc-ui-feedback-v25-whole-20261014 --reuse-owned-target (fmt/clippy/test/release 각각0). native/packaging exactargv는각collector와전용workitem에있다. source공유파일은Root만수정했으며하위완료보고외에원자료를Root가직접검증했다. 원v1helperFAIL/v24재현/sandboxgraphFAIL은보존한다.

미검증: 사용자실제설치/물리IME·reader/cleanOS,32기록fullhistory비용/상한초과UX,upgrade/rollback/서명/공개릴리스. 실제사용자데이터·기존도구·main.git 수정/삭제/재구성은없다. 앱CPU는바이트동일이므로기존strict3CPU실패를지우거나새성능완료로표현하지않으며별도mainframe원인분리를준비한다.

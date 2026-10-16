# S11: 별도 소스 Git commit과 bundle 준비

- 상태: completed
- 분류: release-preparation
- 우선순위: P0
- 요구사항: R10, R11
- 담당/수정 범위: root scripts/isolated_git_source_bundle.py, scripts/test_isolated_git_source_bundle.py; release_preparation /tmp 후보
- 선행 작업: source export schema2, 최신 통합 검증과 문서 갱신

## 사용자 결과

원 프로젝트의 비정상 .git을 그대로 두고 새 /tmp 소스 복사본에 실제 commit과 검증 가능한 bundle을 만든다. remote URL과 인증을 추정하지 않는다.

## 계약과 범위

정확한 export inventory·원본 SHA·변환 SHA·실제 Git object SHA256을 대조한다. 최소 환경, 빈 hooks/template, signing·credential·network protocol 차단. shell 없이 argv로 실행하며 기존 destination은 거절하고 실패 산출물은 보존한다. 소스와 evidence를 구분한 제출용 복사본이지 원 저장소 복구가 아니다.

## 인수 조건

root guard 회귀와 실제 별도 init/add/commit/fsck/bundle verify, exact tree/object bytes 검증. 원 .git 변경 없음, remote·push·외부 CI 없음. 사용자 데이터와 도구는 포함하지 않는다.

## 검증과 handoff

구현 후보 통합 후 root 검증 중. 실제 외부 Git 제출과 Actions는 URL·권한 및 별도 원격 실행이 필요하다. 로컬 bundle을 원격 CI 성공으로 표현하지 않는다.

root 통합: unittest guard7PASS. Windows 우선 CI docs job에 동일 회귀를 연결했다. 실제 commit/bundle 실행은 최신 패키지·문서 고정 뒤 수행하며 지금은 미실행이다.


### 최종 검증 후 로컬 Git 실행

새Windowswhole1011/0/24·S10표시/JSON/DB·18idle구간·현재5파일설치·실제구간저장/독립decode를통과했다. root는최종문서까지freeze한소스를새 `/tmp/dlpad-source-review-final-20261011`로export하여원본개인자료/도구/evidence제외·문서/정책guard를검토한다. `/tmp/dlpad-source-git-final-20261011`의create-only실행은helper의각git명령/정확tree·objectSHA256/bundleverify/sourcepostpin을요구한다. 실행명령은 `python3 scripts/isolated_git_source_bundle.py --output-root /tmp/dlpad-source-git-final-20261011 --git-sha256 587ef21868c948b883993e23209b86a72a6ddc06aab1545c697ffc31075acd4a --execute`다. 실제실행결과/commit/bundleSHA는 [최종 실행 기록](../validation/evidence/isolated-git-final-20261011/root-result.json)에기록하며RUNNING/FAIL은완료로계산하지않는다. originalempty.git은보존하고remote/CI/publication은실행하지않는다.

### 최신 소스 snapshot 재고정

별도 Git foundation 인수는 기존 실제 commit2044cbab/bundle 검증으로 완료했다. 이후 추가된 Windows 실패 복구 시험과 실행 계획 스크롤 수정·whole1013/0/26·현재 설치/idle 인수를 포함한 새 소스 snapshot을 `/tmp/dlpad-source-review-plan-scroll-20261011`에서 먼저 검토하고, `/tmp/dlpad-source-git-plan-scroll-20261011`에create-only commit/bundle로 고정한다. 이 snapshot의 실제 commit/hash/성공 여부는 [실행 결과](../validation/evidence/isolated-git-plan-scroll-20261011/root-result.json)에 보존한다. RUNNING/FAIL을 완료로 계산하지 않는다. originalempty.git은 그대로 보존하며 원격 업로드·실제CI는 별도 미실행이다.

CI build-input/graph/원 archive binding 보조 도구와 설치 중단·typed 거절 재현 도구를 포함하는 후속 source snapshot을 준비한다. [진행 및 최종 결과](../validation/evidence/isolated-git-ci-readiness-20261011/root-result.json). 현재 custom tool path의 후속 GUI 인수는 진행 중이며 이 snapshot이 미래 GUI 성공을 주장하지 않는다.

후속 CI-readiness source snapshot 실제 인수: commit090595d10b3c77dafa6bba4c43bbbe94a376c3e3, [bundle](../validation/evidence/isolated-git-ci-readiness-20261011/dlpad-source.bundle) 5,424,631 bytes SHA256 a7a49dac4a4bd05572120ffca488998072dedc13f3cbce7f422e5e5a326c574c. Root가 별도 review receipt byte 일치, 원 source와 committed object tree, actual10개Git operation exit0, Git binarySHA, exactbundleSHA 및 원empty.git 보존을 재검토해 PASS_SCOPED. 복사본 docs830/projectdocs886 PASS, source1788파일의 high-specificity secret marker0(전수 비밀 감사 아님), Git guard7PASS. 이 실행 전 snapshot이며 이후 GUI 인수 기록·원격 저장소·Actions·runtime updater를 완료로 포함하지 않는다.

## 진단 커서·CI capture 통합 snapshot

원래 empty invalid `.git`은 재확인하고 보존했다. 수정된 native UI·실제8시간 soak fixture·CI whole format·raw capture producer·7개 작업 연결 workflow를 포함하여 `/tmp/dlpad-source-git-diagnostic-caret-ci-20261011`에서 격리된 Git snapshot을 생성했다. 실제 Git10작업, exact tree와1800파일의 현재 원본/커밋 object SHA256를 root가 다시 대조했다. export 문서835개를 확인했다.

보존 묶음은 `docs/validation/evidence/isolated-git-diagnostic-caret-ci-20261011/dlpad-source.bundle`, commit `68c48493ab7cf985aaccd2941180442f2952dad2`, SHA256 `1d230559f9aa9b8f7cfe82a06b61bf8c83b91979eb7e04b8ee6e100afccc0672`, 5,507,844B다. 이전090595d 묶음을 덮어쓰지 않았다. 이 report 이후 문서 추가와 이후 수동 도구 preference 통합은 snapshot 이후 변경으로 분리한다. 원격 URL/인증/CI 실행은 없어 remote 업로드·updater 완료를 주장하지 않는다.

## 수동 도구 설정 및 CI 소스 snapshot

격리repo `/tmp/dlpad-source-git-tool-preferences-ci-20261011/source` commit `08c1771413484418e55e7030ad4b3cffe1f601a2`. [bundle](../validation/evidence/isolated-git-tool-preferences-ci-20261011/dlpad-source.bundle)5,503,379B SHA256 `eee268805ebab08404f20152c98947698f2ffe1e2b311f70c4872b657685c279`. 실제1,806 exported files/838 docs검사, Git10operations0·fsck·bundle verify·정확tree 및 각committedblob SHA256를 root 별도tmp readonlyGit5commands와object재조회로확인했다. 검토 최초는기존create-only rawfilename collision로거절돼 기존원본을덮어쓰지않고별도freshreviewdir로 재실행했다. 기존invalid .git/network/remote 미변경. 수동 경로 기능·current1055source·CIorchestration·README/history를 포함하는 당시snapshot이며 이후GUI시험결과/진행문서수정까지 포함했다고표시하지 않는다. 원격Actions·업로드·updater완료는아니다.

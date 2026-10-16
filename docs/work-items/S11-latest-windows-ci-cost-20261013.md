# S11: 최신 Windows 비용·로컬 CI 검증 갱신

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R10, R11
- 담당/수정 범위: windows_auto — /tmp 준비·읽기 검토; root — source/interface/실행·프로젝트 문서 통합
- 선행 작업: 최신 UI focus/plan whole-v2 및 실제 UI/설치 검수

## 사용자 결과

현재 배포 후보의 비용과 로컬 CI 재현 가능성을 정확한 source/artifact에 연결하고, 이전 snapshot 성공을 최신 결과로 표시하지 않는다.

## 계약과 범위

기존 scripts/ci_*와 Windows cost collector, 관련S11 work-item/evidence를 읽어 최소 fresh 실행을 준비한다. /tmp 후보만 병렬 작성하고 프로젝트 source/Cargo/schema/공유 문서·native 실행은 root 담당이다. Gitinvalid를 유지하며 원격/hostedActions/배포는 실행하지 않는다. 실제8시간 시험 반복과 물리사용자desktop 시험은 이 작업 밖이다.

## 인수 조건

current whole/source/원본 artifact pins를 확인하고 fresh output에서 실제명령·원로그·bounded owned lifecycle·비용 scope를 독립 대조한다. 준비/단위검사/로컬CI를 hosted 성공 또는 전체OS/UI 인수로 표현하지 않는다. CI7 계약이 별도7명령 실행을 요구하면 기존whole4 재사용으로 대체하지 않는다.

## 검증과 handoff

준비 단계다. 실제 최신 비용/CI 실행은 아직 하지 않았다. root UI/설치 검수와 독립적으로 정확 argv·gate·선행 조건만 /tmp에 준비한다.


Root독립준비 검토: /tmp/dlpad-ui-focus-plan-ci7-preparation-20261013의candidate.patch와argv를읽고 guard18개를실제실행하여PASS했다. Cargo.lock 예외는exactD드라이브프로젝트경로/old→newSHA/134403bytes/wholebeforeafterpin만허용하고나머지nonreleasepin을엄격보존한다. 실제nativeCI는7captures+debugguard prerequisite이며currentwhole4로대체하지않는다. source/export/runtime/원logs/도구pin/현재lockbaselinereceipt는fresh출력에연결한다. preparation/export/native는아직실행하지않았다.

모든root소유UI/설치/native27ID실행을종료한뒤currentCPU776d17c…의idlecost3×60s/18fixedwindows를fresh출력current-ui-focus-plan-idle-cost-20261013에서실행중이다. operatorno-concurrent-probesflag는hostquietness실측을의미하지않는다. Python문서검사는999PASS이며현재README/상태표가최신exe/설치ZIP에연결된다.

## 2026-10-13 고정 이전 snapshot의 CI와 현재 상태

source654/F42 통합본의 로컬 native CI7과 debug guardian 선행 build, 총8 Cargo 호출은 실제 성공했다. root는 43 test harness/1272·0·30,1940 export records와 pinned exporter의299 문서변환/생성 marker, 도구·원 source·raw log·artifact 보존을 독립 검토했다. [CI root 검토](../validation/evidence/current-ui-semantics-repaint-local-ci7-20261013/root-review.json). 이 결과를 후속 source655/1278 테스트 통합본의 최신 CI로 소급하지 않는다.

기본 F42의 18 고정 구간은 CPU16PASS/2FAIL(1.2483007755%,1.5665082560%), 메모리18PASS다. 세60초 평균은0.520171%,0.676708%,0.598196%로 평균 목표와 strict 구간 진단을 구별한다. 후속 source655의 기본 비용은 아직 미측정이다.

WSL에서 .git은 readonly tmpfs projection이다. 별도 실제 Windows MinGit read-only rev-parse는 현재 exit128로 유효한 worktree가 아니었다. 역사적인 실제 .git 보존 여부를 projection metadata 비교로 증명하지 않는다. .git 재구성/삭제/공개 push는 하지 않았다. [실제 Windows 관측](../validation/evidence/windows-physical-git-readonly-20261013/root-review.json).

후속source655/8ab7 기본앱strict18actualCPU17PASS/1FAIL(1회차5구간1.0861521808%),memory18PASS/3자연종료0·Job0·3DBimmutable-read각10000/integrityok를root검토했다. 평균0.5988757384%,0.5987914980%,0.5723074711%이며referencequietHost는미검증이다. 원FAIL과이전F42의2FAIL을각snapshot에유지하고원인/성능개선인과를단정하지않는다. [최신 기본실패 검토](../validation/evidence/current-ui-feedback-default18-cost-20261013/root-review.json).

### 최신 compact 준비와 source freeze 순서

최신 기본CPU0093/MSVC1284 source655의whole·네트워크/파일선택·v10설치 검수는완료했다. /tmp/dlpad-current-ui-compact-ci7-preparation-20261013은새sourceexport/currentproof gate/15steps7captures8Cargo/cache759/offline의미실행후보이다. UI후속설치실행 수명기능3파일통합이예정되어이준비를현재CI성공으로보고하거나바뀐source에실행하지않는다. 최종source가끝나면freshproof/helper/export로실행한다. 기존1272 CI 실제PASS와v8strict18 CPU17/1/메모리18PASS를최신0093 또는새launch source로소급하지않는다.


## 최종 v13 준비

최종 completion-copy 전체 검사는 실제1,297/0/30·43harness·source656 및3artifact 독립 대조를 통과했다. /tmp/dlpad-completion-copy-ci7-preparation-20261013은 그 exact proof로만 source·문서 export를 동결하며15단계/7captures/guardian 선행을 포함한8Cargo·cache759·offline·원본 보존을 요구한다. 아직 준비/로컬CI 실행을 성공으로 보고하지 않는다. Root가 최신 화면/패키지 안내 문서를 정리한 뒤 실제 실행하고 export의 모든 원본·변환 행을 독립 대조한다. 최신 default18 비용은 모든 CI/GUI 종료 뒤 단독으로 실행하며 원v8의strict 실패를 대체하지 않는다.

## v13 실제 최종 로컬 CI와 기본 비용

최신 고정 source656/whole1,297/0/30의 로컬 CI15단계·7captures·8Cargo와 export1,954행/1,955config파일을 원로그·pin·원cache759·tool10·원본보존에 연결해 root가 독립 검토했다. hosted/publish는 실행하지 않았다. 이후 문서 갱신은 고정 export에 포함되지 않으며 별도 문서 검사로 검증한다. [현재 로컬 CI](../validation/evidence/current-completion-copy-local-ci7-20261013/root-review.json).

최종 v13 default CPU의 단독3×60초 측정은 실제 완료했으며 reported CPU 평균0.545937/0.598683/0.598861%와 메모리18구간은 목표 안이다. 추가 strict18구간은17CPU 성공/1실패(3회4구간1.094067%)이며 원FAIL과 outerexit2를 유지한다. 앱은 세 번 모두 자연exit0/Job0로 종료했고 크래시는 아니다. root는 raw18 kernel/user counter·연속 구간 수학과3개 readonly DB각10,000행/integrity/namespace/schema/index,3만개 합성3바이트 파일 전부를 대조하고3readyPNG를 직접 보았다. 최종 reported60초 평균의 마지막 accounting counter는 producer에 없어18구간 weighted mean과 구분한다. quiet host·worker별 작업 인과·실행 중 scan generation·물리 UI는 미검증이다. [현재 비용 원실패 검토](../validation/evidence/current-completion-copy-default18-cost-20261013/root-review.json).

첫 readonly 감사는3만 파일마다 부모 chain을 반복 조회하여9P 비용이 컸다. 정확한 소유PID/starttime/argv를 pidfd로 대조하여 그 감사만SIGTERM(exit143)하고 원측정/앱/다른 작업/WSL은 종료하지 않았다. 별도 v2는 원 directoryFD와 perfileO_NOFOLLOW/fstat/최대4바이트·전후namespace를 검증하여 모든 파일 읽기를 유지한다. symlink/FIFO/변조/namespace/dir교체의8개 순수 guard를 root도 실제 통과했고 원헬퍼와 중단receipt를 보존했다. v2 독립 감사terminal0의 판정은 strictFAIL 확인이다. [감사 중단 기록](../validation/evidence/current-completion-copy-default18-cost-20261013/root-audit-interruption.json), [실행·guard·pins](../validation/evidence/current-completion-copy-default18-cost-20261013/root-audit-v2/root-execution-receipt.json). strict 비용의 인과 조사와 최초 릴리스 물리 환경 인수는 후속이다.


v14 최신배율 소스의 whole1,305/0/30·현재3PE·63개PNG·archive·native실행/변조/27ID 통합을 확인했다. 새로운 CI7은 별도 freshowned source/runtime/export/output을 사용하여 문서동결 후 실행한다. v13 CI/비용을 현재소스로 소급하지 않으며 strict18 원FAIL과 외부세션143도 그대로 보존한다. CI 준비는 /tmp 후보이고 실제 결과를 아직 성공으로 기록하지 않는다.


v14 CI 실제 실행은 outer143로 끝나 final result/log가 없고 testterminal도 없어 전체PASS가 아니다. fmt·guardian debug·clippy 등 부분receipt는 원자료로 보존한다. 읽기전용nativeprocess관측은 고유runtime python0/unknown0이며 원Job0/과거lifecycle를 증명하지 않는다. PowerShell script 정책 거부를 우회/변경하지 않았고 기존 C# 진단 방식으로 현재상태를 조회했다. nativehost argv 조건 오류도 별도observerFAIL로 보존했다. 소스동결을 해제하고 후속두UI수정을 진행한다. [미완료 분류](../validation/evidence/current-scale-viewport-reflow-local-ci7-20261013/root-incomplete-classification.json).


v16 네필수명령은원fmt/clippy/test3actualreceipt+같은frozen소스의새releaseactualreceipt로1,309/0/33을독립대조했다. 이는CI7가아니며v13마지막CI7와v14미완료143·strict18원FAIL을그대로유지한다. 후속목록identity sourcewhole은별도진행중이고최신CI/비용을예측하지않는다.

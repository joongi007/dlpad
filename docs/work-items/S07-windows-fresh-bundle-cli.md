# S07: 새 개발 설치 묶음 CLI 연결

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — 공유 core API·통합·native 검증·문서; metrics_recovery — /tmp dlpad-local-install.rs command/parser/tests 초안
- 선행 작업: S07-windows-fresh-bundle-coordinator의 원 capability factory·readonly observer 검증. 현재 초안 연결만 독립 수행하며 실제 core 통합 이후 인수한다.

## 사용자 결과

명시한 manifest/source와 새 outer 경로 하나로 registered·receipt·active 개발 묶음을 생성하고 별도 프로세스에서 기존 readonly 검사·선택 실행을 사용할 수 있다.

## 계약과 범위

후보 명령 prepare-user-bundle은 명시 --manifest·--source-dir·--bundle-root를 사용한다. 기존11개 명령의 옵션·기한·원cap·실패 semantics를 유지한다. create_fresh_user_bundle의 source 검증 후 생성·고정3children·active 마지막·opaque 실패 caps 보존을 사용한다. 기존 root를 채택/교체하지 않고 실패나 Drop에서 자동 삭제하지 않는다. 사용자 Documents·실제영상·전역설치·기존선택 자동탐색·journal/upgrade/rollback·정식publisher trust는 이 작업의 인수가 아니다. Cargo workspace/lock/schema 변경은 root 조정하며 현재 새 dependency는 필요 없다.

## 인수 조건

정확 command/parser 실패 회귀, 실제 합성 source의 새 묶음 생성→별도 cold3경로검사→원 selected Job 실행, 취소/기한/잘못된SHA/기존foreignouter의 보존을 확인한다. runtime until exit는 기존run-selected를 사용한다. stdout 원 typed stage·mutation·pre-exitcap 보존과 자연 terminal을 구분한다. full4검사 및root/독립 exact source/원files/NativeIDs 대조 후 scoped 인수한다. 공개 배포/P0 전체 완료는 별도다.

## 검증과 handoff

현재 core API와CLI는 초안 단계다. metrics는 CLI 파일만/tmp에 작성하고 root가 core/공유facade와 통합한다. 실제source/CLI/native 검증 전에는 완료로표시하지 않는다.

12번째prepare-user-bundle 명령/parser2개회귀와 core연결을 root가 원SHA·exactdiff로검증한뒤통합했다. 독립read-only검토에서는 기존11cmd/timeout 및 새정확3flags의추가차단결함을발견하지않았다. 실제parser/native검증은 아직실행전이며 core통합실제전체4검사 이후 별도 process의생성·coldinspect·원selectedJob실행과실패보존을 확인한다.

실제 Windows 전체 검사 첫 시도는 fmt/clippy 통과 후 테스트 exit101로 중단했다. 열린 payload HANDLE의 공유 정책 때문에 root 교체 시험의 디렉터리 rename이 Windows 오류5로 거절됐다. root는 원파일 공유 권한을 완화하지 않고 빈 원root HANDLE/ID를 유지한 교체 조건으로 시험을 수정했다. 릴리스 빌드는 당시 미실행이며 새 전체 검증 결과와 구분한다. 독립 검토에서 preflight 실패의 stage error 존재를 capability 보유로 잘못 보고하는 추가 결함도 발견했다. 현재 전체 실행의 입력을 보존한 뒤 getter 및 실패 경로 회귀를 수정하고 다시 검증한다.

capability 보고 교정 후 전체4검사는 root/독립 PASS(774/0/13, source579 정확map·원 GNU/lock 보존)다. 현재 CPUa6d9223a…8,709,632B·CLI97217440…841,728B다. 실제 CLI collector를 프로젝트 scripts/windows_msvc_user_bundle_cli_collect.py에 통합하고 --help·AST를 확인했다. badSHA의 coordinator 이전 generic preflight 거절과 existingouter의 coordinator 단계 실패를 분리하며 없는 capability 필드를 false로 추정하지 않는다. 현재 실제 생성·cold검사·격리GUI3case·원NativeID10/SHA/namespace 시험을 실행 중이다. wrapper가 REVIEW_REQUIRED를 반환해도 root 픽셀/SQLite·독립검토 전 완료로승격하지 않는다.

첫 실제 fresh CLI collector는 CLI8행(생성·cold검사 성공2/거절6)·GUI3case·native3query까지 실행했으나 최종 Python binding에서 NOTICE.txt vs 기존 helper의 notice.txt JSON키 대소문자 차이로 KeyError를 반환했다. Windows 파일 open 성공과 JSON exactkey를 구분하며 이 wrapper FAIL을 보존한다. root는 fixture이름을 기존 고정 helper와 일치하는 notice.txt로 교정하고 fresh evidence에서전체 수집을 다시 수행한다. 이전 GUI3case는 native실행 사실이며 wrapper최종인수로승격하지 않는다.

범위 인수 완료: capability교정 source579의 전체4검사·774/0/13 및 실제 fresh CLI14명령/11행·GUI3case·원NativeID10, 별도 복사중control3명령/2case를 root/독립검토했다. 밝은테마PNG와읽기전용SQLite Light/integrity·원source/등록/receipt/active/foreign 보존을확인했다. 최초parse/시험rename/collectorcasekey FAIL은보존한다. 이source579의정확map 인수이며 이후wake계측통합 source579는같은항목수지만별도검증중이다. durablejournal·upgrade/rollback·기본설치위치/UI·cleanOS/서명·전체P0 완료는범위밖후속이다.

# S03: Linux 사용자 서비스 guardian 통합

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R10, R11
- 담당/수정 범위: root; process.rs/guardian.rs·tests/guardian.rs·공유 module 연결·통합 시험·ADR/CONTEXT/상태 문서. user_service.rs는 metrics_recovery 독점
- 선행 작업: 실제 제품 guardian 사용자 서비스 실험, ExecStartEx no-env-expand 실증

## 사용자 결과

Linux 앱과 guardian이 동시에 강제 종료돼도 소유 도구의 detached 자손을 사용자 manager가 정리하는 backend를 통합한다.

## 계약과 범위

ADR-0002는 Proposed이며 실험 근거로 보완한다. 설치나 관리자 권한 없이 기존 사용자 systemd/libsystemd capability만 사용한다. 고유 transient unit의 guardian MainPID와 control EOF/R/T/K 계약, stdout/stderr·종료 코드·FD 상한·literal argv를 유지한다. launcher는 직접 Child로 기존 엔진 wait 계약을 유지한다. AddRef/동일 bus/InvocationID/정확 unit 소유권을 검증하고 원 app endpoint의 service-side 중복 FD는 시작 뒤 즉시 닫는다. 다른 unit/manager/기존8h 시험·사용자 데이터는 변경하지 않는다.

초기 backend는 명시적 개발 선택으로 통합 시험하며 충분한 전체 실증 후 기본 정책 변경을 별도로 판단한다. capability 실패를 약한 process group-only 경로로 자동 fallback하지 않는다. 새 Cargo 의존성 없이 기존 libc 동적 로드를 검토하며 runtime 필요 조건과 비용을 기록한다. nonUTF8 D-Bus argv/환경은 failclosed하고 모든 environment 값은 로그·진단에 출력하지 않는다.

## 인수 조건

실제 Rust 엔진을 통한 literal argv·환경·stdout/stderr EOF·exit42·cancel/timeout·owner crash·guardian SIGKILL·동시 crash·무관한 sentinel 보존·unit GC 및 FD leak 검사. capability/입력 실패에서 외부 tool 미실행. 기존 실제 GUI/일반 download 회귀, Linux/Windows 공통 검증. systemd 없는 Linux/다른 OS 지원 범위는 실제 증거와 구분한다.

## 검증과 handoff

구현 전 기록. Python 실험 wrapper 결과를 Rust 엔진 구현 완료로 표현하지 않는다. source/interface 통합·빌드는 root가 직렬 조정한다.

## FD0 entry 1차 실증

기존directguardian8회귀PASS, controldescriptor3unitPASS. 실제FD0 entry시험은readinessR 뒤 Python stdin=None으로 실패했다(원시user-entry-failure.json). 원socketFD0를toolspawn전닫으면CLOEXEC null FD가0에할당될수있어,spawn까지원FD0를유지하고privateclone은>=3/CLOEXEC로고정했다. 영구integration `user_guardian_entry_preserves_null_stdin_and_literal_arguments`는R·literal$한글공백·nullstdin·exit42 PASS. [원시회귀](../validation/evidence/user-service-integration/user-entry-regression.log). 전체nativebackend연결전단계이며R08완료아님.

## 제품 기본 보호 정책 채택 단계

actualengine13case와정확53PID/10unitGC를인수한뒤등록된앱(default/main·accessible·CPU세entry)의Linux기본을user-systemd로변경한다. capability실패에는direct로fallback하지않고명시direct선택도등록앱에서거절한다. 독립engine개발helper와기본host-independentfixtures는기존direct를유지하고strictbackend13case는명시선택ignored시험이다. Launcherprocess는mainregister이전이므로현재자기executable을serviceguardian으로사용해단일앱배포가adjacenthelper에의존하지않게한다. 채택후실제앱GUI·실행계획/숨김/복귀·release지원조건/비용/추가실패인수는계속검증한다.

## 기본 backend 통합 회귀 (진행 중)

root가 fmt/defaultclippy/CPUclippy/defaulttest210PASS/CPUtest349PASS/defaultrelease를현재소스에서실행해exit0을확인했다. 각test7ignored는외부도구/manager등별도실증이며성공수에넣지않는다. [명령과원시로그](../validation/evidence/user-service-final/workspace-result.json). CPUrelease와단독앱기본경로GUI는이기록시점실행중/대기이므로완료아님. 이전strict13case에대해root가53원PID·10cgroup경로부재및sentinel생존기록을 [직접대조](../validation/evidence/user-service-engine-full-20261003/root-review.json)했다. 정상Git저장소없음은재확인했으며기존.git은변경하지않았다.

CPUrelease도exit0완료. immutable단독앱SHA `ea75955c3335612097b39cf7d9587c3fd1dc13f07ca3bc6ac854b93de5e596cc`,12438904B를별도tmp로고정했다. [snapshot](../validation/evidence/user-service-final/immutable-app.json). 제품기본GUI3flow와비용표본은독립격리환경에서하위에이전트에전달했고결과를기다리는중이다. 최신Windows/MSVC/macOS검증을이Linux빌드로대체하지않는다.

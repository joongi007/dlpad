# S11: 재설치 수정본의 현재 소스 패키징과 인수

- 상태: done / v25 개발 패키지 및 소유 private 재설치 범위
- 분류: packaging
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: advanced_units — installer-v25-reinstall-helpers 및 이 작업 계약; root — 제품 main/journal_choice·전체 통합; installer_batch — 별도 private 재설치 시험
- 선행 작업: S11-reinstall-location-20261014, v25 Windows whole와 독립 Root review

## 사용자 결과

원 설치 기록을 보존하면서 새 journal generation 선택과 기존 설치의 startup matching을 수정한 개발 설치본을 제공한다. ZIP 생성과 실제 재설치 동작 인수를 구분한다.

## 계약과 범위

원 v24 helper·ZIP·기록을 변경하지 않고 새 v25 namespace에서 graph → candidate → portable review → generate → installer ZIP → review를 실행한다. 실제 whole·Root review·현재 소스와 세 실행 파일의 SHA/크기를 정확히 연결한다. source671은 Root 실제 통합 결과이며 임의 허용 수가 아니다. graph258·선택 notice missing0·전체 lock appendix missing43을 유지하고 달라지면 원 산출물을 보존한 채 중단한다. generated7/manifest5/CPU와 installer mapping·stream CRC·SHA·safe namespace·deterministic ZIP·before/after 보존을 검증한다.

수정 파일은 전용 helper namespace와 이 문서뿐이다. 제품·공유 source·Cargo/lock·README·원 evidence는 수정하지 않는다. 신규 dependency/feature/schema는 없고 버전은0.1.0 개발후보를 유지한다. 기존 offline cache를 사용하며1.2GB 재빌드를 수행하지 않는다. 두 ZIP 약35MB의 디스크 공간은 실제 산출 후 기록한다. ADR0009의 설치 앱 수명은 유지하며 ADR0005의 Proposed upgrade/trust/cleanOS 범위를 완료로 승격하지 않는다.

## 인수 조건

현재 v25 whole의 실제 fmt/clippy/test/release 네 operation·source671·current3·Root SHA/raw totals가 일치한다. 새로운 graph/두 ZIP/generated 경로는 create-only다. 원 notice bytes와 generated7/archive 모든 member를 독립 대조한다. generated7 완료 즉시 Root/native 담당에게 정확 경로를 전달한다. 실제 설치→제거→재설치→cold matching은 별도 소유 private fixture에서 수행하며 실패·원 journal/사용자 파일 보존·자연 app/supervisor exit/Job0를 Root가 통합 확인한다. 패키지만으로 native/physical/IME/reader/서명/법적 coverage/공개 릴리스를 주장하지 않는다.

## 검증과 handoff

Root v25 whole는 source671,1542 PASS/0 FAIL/42 ignored,43 harness로 독립 확인됐다. proof SHA4981392dad59c2cdf3948ec0ede5a7648b1a589e39fdae995d615e7ba43ca70b. CPU/CLI는 v24와 동일하며 installer는4,432,384B로6,656B 증가했다. 이는 Root whole 근거이며 패키지·재설치 시험의 성공을 뜻하지 않는다.

helper는 AST·원 byte copy·gate 정적 검토만 완료했고 실행 결과는 아직 없다. Root 승인 후 전용 argv를 실행하며 기존 v24와 비용 strict FAIL을 보존한다. 패키지·private native 원자료의 독립 검토 이후만 상태를 갱신한다.


### 실제 패키징 결과

최초 일반 sandbox graph는 WSL vsock 오류로 Windows host/Cargo graph 시작 전 실패했다. 원 FAIL을 보존했고, Root의 별도 escalated private 검사 성공 후 fresh graph-v2에서 재실행했다. WSL endpoint 고장이나 제품 실패로 단정하지 않는다. 보안 설정·WSL·원 기록을 변경하지 않았다.

`installer-v25-reinstall-helpers/interop-v2/pipeline.py`의 graph/candidate/portable-review/generate/installer-zip/review 여섯 phase는 모두 exit 0이다. graph는 기존 offline tree/metadata 세 operation만 수행했고 cache 재빌드는 없다. Generated 7파일과 manifest 5항목을 직접 full validate한 뒤 native 담당에 경로와 SHA를 전달했다.

Portable ZIP은 18,215,479B, SHA `05dfd34ddb82a8032600fe59c5853704f5e8f8c61db83456c9618a940926a915`, 467 members이다. Installer ZIP은 17,164,983B, SHA `291d2d7a077b2c46494283d4470f34360c8e85296df3287ad45b46c6b4237996`, 7 members이다. 두 ZIP 합계 35,380,462B이며 1.2GB cache를 새로 만들지 않았다.

독립 archive auditor는 exit 0으로 actual whole/source671/current3/graph-v2/metadata-lock258/selected notice missing0/원 notice456/appendix missing43을 대조했다. 모든 ZIP member의 stream CRC·SHA와 safe/deterministic namespace, generated7/current manifest5, 모든 original notice bytes 및 source before/after 보존을 확인했다. 기존 report schema의 exact_inputs/candidate_members/installer_members/generated7/actual_original_notice_sources를 생성했고 Root의 별도 strict package 검토 대상으로 전달했다. [Whole](../validation/evidence/windows-msvc-ui-feedback-v25-whole-20261014/result.json), [fresh graph](../validation/evidence/windows-msvc-ui-feedback-v25-20261014-graph-v2/result.json).

실제 설치→제거→재설치→cold matching은 별도 installer_batch/Root 소유 private fixture에서 수행됐고 두 installer 자연 종료·Job0, old/new journal 및 최종 payload 보존을 Root가 독립 확인했다. 이 패키징 담당은 해당 native를 직접 실행한 것으로 보고하지 않는다. [Private 재설치 Root review](../validation/evidence/windows-msvc-ui-feedback-v25-reinstall-private-v3-20261014/root-review.json).

버전은 0.1.0 개발후보이며 공개 배포·서명·법적 전체 coverage·clean OS·실제 upgrade/rollback·physical/IME/reader/전체 UX 게이트는 별도다. 과거 v24 비용의 strict 실패를 이 installer hotfix로 해결했다고 주장하지 않는다. 제품/Cargo/README와 원 v24 helper/ZIP/FAIL은 변경하지 않았다.

문서 검사 `python3 scripts/check_docs.py`는 실제 exit 0으로 1,101개 문서와 로컬 링크 검사를 통과했다. 외부 URL과 heading anchor는 이 검사 범위가 아니다.

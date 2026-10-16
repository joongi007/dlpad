# S11: v26 기본 CPU 개발 패키징

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: installer_batch 신규B/ui-feedback-v26-package-v1/helper와본문서; Root실제whole/패키징/통합
- 선행 작업: v26 whole·Rootreview·현재source/current3 actualPASS, 원v25graph-v2 baseline보존

## 사용자 결과

최신도구캐시상태·검색·진행표시·도움말통합후기본CPU로새portable/installer ZIP을준비한다. 기존v25ZIP을덮어쓰지않는다.

## 계약과 범위

exactwhole namespace windows-msvc-ui-feedback-v26-whole-20261014, outputtagui-feedback-v26-20261015. actualRootsource count와3PE/hash/raw43를derive하며미래수치를예측하지않는다. release기능cpu만·진단feature금지. selected258/wholemissing43·legal/sign/releasefalse를유지한다. oldgraph최초FAIL아닌v25graph-v2 acceptedbaseline사용. S/Cargo/schema/currentartifact/원E/helper변경금지.

## 인수 조건

원7source/5payload/generatedexactnamespace·두ZIP전memberCRC/SHA·원noticeSource/proof/lock/toolchain/current3/graph재대조 및beforeafter. create-only새경로와원v25보존. 공개게시/정식배포/upgrade/legalfullclaim없음.

## 검증과 handoff

준비helperPythonAST 통과; package/native실행0. actualwhole후Root가argv/HANDOFF 순서로실행한다. B원본provenance/basepins/새sourcepins/diff를기록한다.

### 첫 whole 실패 보존과 v2 proof 파생

Root 첫 whole26에서 fmt/clippy는0이지만 app(GPU)3회귀 실패로 test101/release미실행이므로패키징하지않았다. 원wholeFAIL과B/ui-feedback-v26-package-v1을불변보존했다. 새B/ui-feedback-v26-package-v2는원helper핀/diff와함께exact `windows-msvc-ui-feedback-v26-whole-v2-20261015/result.json` 및siblingRootreview만허용한다. 같은v26-20261015 출력은아직부재이고실행0이다. source count/hash는actualRootreview로derive하여변경/예측하지않는다. graphcollector/fullpayloadvalidator/공유ZIPparser bytes불변·모든258/43missing/current3/CPUonly/CRC/SHA/새namespace guard유지. 새AST검사만통과했으며 v1 pure5시험을새v2실행으로소급하지않았다. actualwholePASS전실행금지·상태in-progress유지.

## Whole-v3 / graph-v3 새 준비

정상 HLS의 float 예상 용량 처리와 도구 작은 화면 후속 수정 때문에 원 whole-v2 / graph-v2는 보존하고 패키지 생성을 시작하지 않았다. Root가 승인한 새 proof는 `windows-msvc-ui-feedback-v26-whole-v3-20261015/result.json`이다. B/ui-feedback-v26-package-v3은 v2 원본 핀·diff를 보존하며 새 graph-v3와 자기 receipt 경로만 변경했다. actual count/hash는 미래 Root review에서 읽으며 현재 예측하지 않았다.

새 Python AST 검사와 collector/payload validator byte identity 및 새 출력 부재를 확인했다. package/native 실행은 0회다. selected258 / 전체 missing43 / 기본 CPU / 모든 ZIP CRC·SHA / generated7 / manifest5 / 원 notice와 source closure guard는 그대로다. Root의 actual whole-v3 PASS 이전에는 실행하지 않는다. 전체 P0·물리 UI·정식 릴리스 완료는 주장하지 않는다.

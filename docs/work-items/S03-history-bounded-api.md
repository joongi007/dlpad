# S03: bounded 이력과 명시 재개 검증 API

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R10
- 담당/수정 범위: root; crates/engine/src/download.rs, main.rs test literal, 본 문서
- 선행 작업: typed stored_options, 기존 keyset history

## 사용자 결과

작업 이력에서 최신 도구 버전을 보고 원래 옵션으로 재개한다. 손상하거나 큰 journal 값은 UI 메모리에 무제한 읽지 않는다.

## 계약과 범위

JobSummary에 finite tool_version 추가; validated_resume(id)는 bounded job JSON/ID/상태/stored options를 검사한다. DB/schema 변경 없음. 실패 상세는 저장되지 않으므로 추정하지 않는다.

## 인수 조건

큰/손상 job JSON, invalid ID/version, terminal state와 변경된 stored options 거절; 정상 실패/Paused 재개 및 keyset 회귀. UI I/O worker 처리.

## 검증과 handoff

구현/실제 시험 전 완료로 표기하지 않는다.

## 실제 bounded 인수

Linux engine history 3회귀 및 CPUHistory 실제GUI의원snapshot Height720/동일job2attempt 재개와keyset260seed를확인했다. latestversion/invalidID/큰손상JSON/변경options 거절시험PASS. 이후workspace default200/CPU325PASS 및Windows240PASS는각snapshot범위이며새실패경로2시험은Unixfixture안에있어Windows실행으로확대하지않는다. Windows공통purefixture이전은별도후속coverage다.

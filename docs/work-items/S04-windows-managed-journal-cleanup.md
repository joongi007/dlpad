# S04: Windows managed journal original-handle completion cleanup

- 상태: in-progress
- 분류: safety
- 우선순위: P0
- 요구사항: R03, R05, R08, R13
- 담당/수정 범위: diagnostic_cache; tools/windows_managed_journal.rs, 이 신규 문서만
- 선행 작업: native journal/records4checks PASS; root marker-last/guardian integration

## 사용자 결과

확정완료receipt를원filecap으로검증한journal/8receipt만정리한다. marker는root가마지막에처리한다. pristine는원livecreateobject에서reader未開始かつ8actualfiles空のみ허용,재개방/partial/unknown은보존.

## 계약과 범위

cleanup_completed(self)/cleanup_pristine(self) consuming API로원DELETEhandles全preflight후FileDispositionInfo;所有receipt/journal/root/markercloneを返却前closeして実際9names absence확인. 원marker/executionlock은삭제않음. originalIDs/ACL/path/linkcount/actualreceipt locked검증,Drop자동삭제없음. partialdispositionfailure는잔여gate보존/rootmarkerclear금지. session원receipt가Jobterminal권한인지guardian증명은별도caller필수.

## 인수 조건

정상actualreceipt cleanup/emptylivepristine정리 및reopenedpristine거절. Intent/missingreceipt/foreignreplacement/oversize/ACL변경실패보존/sentinel0change. 실제Win32삭제handle권한/rootcloneclose/nativefileabsence 검증. Cargo/공유marker/tools/process 변경금지.

## 검증과 handoff

구현/전용fmt후rootWindowsnative tests/Clippy대기. 실제미실행을완료로표현하지않는다.

### 실제 Windows 통합 검증

주 담당이 `python3 scripts/windows_current_workspace_probe.py .../windows-managed-lifecycle-lock-capability-retry --execute --seconds 1800`을 실행했다. fmt, 전체 workspace Clippy(-D warnings), workspace test, release build가 모두 통과했다. result.json 및 최종 소스 SHA를 현재 파일과 독립 대조했다. native journal 7개, 삭제권한 없는 guardian journal 3개, runtime 제어 pipe 2개 및 v3 실제 정리 후 재사용 시험이 통과했다. 사용자 input desktop은 그대로였다. 문서 검사는 587개 PASS다. 실제 관리 도구 실행 adapter 및 startup 자동 회수는 아직 연결되지 않았으며 이 시험을 실제 다운로드/앱 crash 전체 인수로 확대하지 않는다.

# S10: 현재 v32 실패 보고서 JSON 저장과 덮어쓰기 거절

- 상태: done-scoped — 실제 Windows 저장·충돌 검증 완료, 읽기 높이 후속
- 분류: validation
- 우선순위: P0
- 요구사항: R13/R08, ROADMAP S10 최초 릴리스 로컬 진단
- 담당/수정 범위: field_reveal. 새 `.local-tools/session-recovery-20261013/current32-s10-export-prep-20261016/` 및 이 새 문서만.
- 선행 작업: whole32/Rootreview/current3PE, 기존 실제 synthetic cookie fixture·private 성공 observer. Root 실제 실행 권한·자원 조정.

## 사용자 결과

실패 원인에서 정제된 JSON 미리보기를 확인하고 새 파일로 저장한다. 같은 경로를 다시 저장하면 거절하고 기존 결과와 실패 이력을 유지하는지 현재 v32에서 검수한다.

## 계약과 범위

[진단 계약](S10-failure-diagnostics.md), [ADR-0004](../adr/0004-diagnostics.md)를 따른다. 기존 API/schema/제품/공유 스크립트/Cargo/lock/원 evidence는 바꾸지 않는다. 실제 사용자 Documents·영상·쿠키·계정·clipboard는 사용하지 않는다. 새 private desktop/합성 fixture와 소유 HWND만 허용하고 visible fallback은 없다. Proposed인 전체 S10 인수를 이번 좁은 성공으로 승격하지 않는다.

## 인수 조건

정확 current32 proof/source/3PE 및 known synthetic fixture SHA. 실제 metadata 실패7·원 call sequence·Privacy 검사·JSON preview/export 동일 첫페이지·새 저장·동일 경로 거절·원 JSON/SHA 유지. 원 cookie/sentinel·실패DB bytes 유지, 자연app0/Job0·workerdrain·input desktop 불변. readonly production DB Failed1/Attempt1/Artifact0와 원 PNG를 Root가 직접 검토한다. 실패는 원자료를 보존하며 임의 재분류하지 않는다.

## 검증과 handoff

준비 단계: 기존 문서/현재 source labels·export renderer·schema와 원 성공 observer를 읽고 Python AST parse/핀을 확인했다. C# compile/native/Cargo 미실행. exact argv와 미검증 범위는 scratch HANDOFF.md에 있다. Root 실제 실행 후 결과와 근거를 추가하며 다른 문서는 담당 Root가 통합한다.


## Root 실제 실행과 독립 검토

HANDOFF의 exact current32 argv로 `collect.py` 실제 exit0. C# compile0·합성 metadata-failure7·앱 자연0/Job0/private/input 복구를 확인했다. 원 JSON971B/preview971B SHA가 실제 동일했고 Failed1/Attempt1/Artifact0, DB 진단 전후 SHA와 immutable readonly SQLite integrity를 Root가 대조했다. 원 입력·source675·현재3PE/lock과 원 도구 fixture pins 불변이다. 동일 저장 경로 두 번째 요청은 거절되고 원 JSON bytes/SHA가 유지됐다. [Root 검토](../validation/evidence/current32-s10-json-export-private-v1-20261016/root-review.json).

Root는 current-preview/current-saved/current-collision/terminal PNG4장을 직접 열어 실제 저장 성공·거절 안내를 읽었다. 그러나960×640에서 미리보기 viewport는약64px/3줄로 눌려 있다. 기능 검증 통과를 전체 진단 UI 인수로 확대하지 않으며 최소 읽기 높이 후속을 별도 준비한다. 클립보드·물리 입력·첨부/편집/제외 전수·큰 보고서/CloseWait 인수는 이 실행에 포함되지 않았다.

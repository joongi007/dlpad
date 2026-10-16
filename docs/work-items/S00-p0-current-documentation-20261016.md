# S00: 현재 Windows P0 인수 범위 문서 정리

- 상태: implemented — 문서 작성, Root 통합·문서 검사 대기
- 분류: documentation
- 우선순위: P0
- 요구사항: R04/R08/R10/R11/R13, ROADMAP S00/S06/S07/S10
- 담당/수정 범위: field_reveal. 새 `docs/validation/P0-current-20261016.md` 및 이 새 작업 기록만.
- 선행 작업: whole32와 현재 native Root 검토, S07 수동 A→B→A 검토, 치지직 metadata 검토. 실행 중 5GB v4 결과는 대기.

## 사용자 결과

현재 사용할 수 있는 개발 후보와 실제 확인된 UI/복귀 동작, 출시 전 남은 작업을 한 표에서 읽는다. metadata·합성 fixture·private UI와 실제 저장·물리 OS·출시 인수를 구별한다.

## 계약과 범위

README→PRODUCT→CONTEXT→요구사항/ROADMAP 및 ADR-0001/0004와 아키텍처를 대조했다. Windows 우선·S10 P0 최초 릴리스 계약 유지. 원 실패/원 evidence·기존 docs·code·schema·Cargo/lock 변경 없음. Root가 기존 README/CONTEXT/현재 상태 문서의 연결을 통합한다. 수동 side-by-side 결과를 일반 DB downgrade·자동 durable revision/updater 완료로 확대하지 않는다.

## 인수 조건

완료 범위에는 존재하는 실제 JSON 링크와 한계를 같이 적는다. 5GB v2/v3 실패·v4 진행 중, default18 NOT RUN·현재 CI OS112 실패를 숨기지 않는다. 이전 OS/feature/snapshot 검증을 현재 Windows shipping 성공으로 쓰지 않는다. 실행 중 결과는 Root의 실제 terminal 검토 뒤 갱신한다.

## 검증과 handoff

원 Root JSON과 수동 ABA10PNG를 읽었다. 새 문서의 상대 링크 존재 여부를 확인했다. Cargo/native 실행 없음. Root 통합 후 `python3 scripts/check_docs.py`로 전체 문서 검사를 수행한다. 기존 여러 역사 문서에 남은 오래된 문구 수정은 Root 담당이며 이 작업에서 건드리지 않았다.


Root 후속: PERFORMANCE에 실제 v32 원 counter/18구간 및 실패를 반영했고, DIAGNOSTICS의 이미 검증한 cache0 byte 정책을 미완료 만료 정책으로 표시하던 낡은 문구를 수정했다. 실제 Windows clipboard·전체 export 인수는 계속 미완료다. 제품 source/Cargo/schema 변경 없음.

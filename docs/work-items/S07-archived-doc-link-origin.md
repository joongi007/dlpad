# S07: 고정 증거 문서의 원래 링크 기준 검증

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R11
- 담당/수정 범위: root — scripts/check_docs.py와 이 작업 기록
- 선행 작업: 실제 companion ZIP 원 입력 보존 완료

## 사용자 결과

검증용으로 원문 그대로 보존한 Markdown도 내용·링크 검사를 유지하되 상대 링크는 SHA가 고정된 원래 프로젝트 문서 위치에서 해석한다.

## 계약과 범위

evidence/*/inputs의 Markdown만 해당 evidence input-pins.json의 동일 파일명·동일 SHA·정확 단일 프로젝트 원본 매핑을 검증한다. 입력 pins는256KiB 한도, SHA가 다르거나 origin이 불명확하면 원위치 override를 거절하고 기존 링크 기준을 유지한다. 임의 외부 경로나 전체 evidence 검사 제외를 허용하지 않는다. 원 자료와 pin을 수정하지 않는다. 신규 의존성/lock 없음.

## 인수 조건

원 ADR 사본의 broken17link 실제 실패를 보존하고 원위치 기준 검사 성공. 원pins 변조/외부 origin/중복 origin을 인정하지 않음. 기존 프로젝트 문서 검사 유지.

## 검증과 handoff

문서 검사 exit1에서 고정 ADR 사본17links가 재현됐다. 구현 후 실제 검사와 실패경로 확인 예정.

실제 absolute lower checker `python3 /mnt/d/project/dlpad/scripts/check_docs.py` exit0/PASS794. 임시 fixture에서 pinned origin 수용·변조pin/외부origin/중복origin3거절을 확인했다. Windows upper/lower alias는 문자열suffix가 아닌 원root samefile로 매핑한다. 일반 상대 script 명령과 독립검토 결과는 후속 기록한다.

일반 상대 `python3 scripts/check_docs.py`도 exit0/PASS794. 원root identity의 upper/lower alias에서도 pins를 매핑했다. 독립 읽기검토 PASS: 이는 archived bytes·원링크 좌표 검증이며 현재 origin bytes 일치 또는 모든 invalid mapping에서 checker 실패를 보장하지 않는다. invalid mapping은 override하지 않아 일반 링크 검사 규칙으로 처리한다.

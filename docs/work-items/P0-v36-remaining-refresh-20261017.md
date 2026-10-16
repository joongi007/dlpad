# P0: v36 잔여 작업 정리

- 상태: done-scoped
- 분류: documentation
- 우선순위: P0
- 요구사항: R02, R03, R04, R05, R06, R08, R10, R11, R13
- 담당/수정 범위: Root; 이 작업 기록, docs/validation/P0-current-20261016.md, P0-status.md, P0-remaining-20261011.md
- 선행 작업: v36 통합 검증과 사용자 설치기 문구 질문

## 사용자 결과

현재 검수본, 개발로 처리할 일, 실제 환경 인수, 공개 운영과 후속 범위를 한 표에서 확인한다.

## 계약과 범위

README, PRODUCT, CONTEXT, REQUIREMENTS 및 ADR 목록의 기존 범위를 유지한다. 제품 코드·패키지·schema·의존성은 바꾸지 않는다. 기존 검증 기록과 실패, snapshot 한계를 보존한다. 설치기의 중단 문구 변경은 미구현 후속으로 명시한다.

## 인수 조건

- 최신 검수본 v36과 실제 Explorer 사용자 확인 미완료를 표시한다.
- UI 문구 개선, Windows 실환경, R13 복사, 치지직 저장, 공개 운영을 구분한다.
- 완료된 8시간 시험을 새 잔여 작업으로 돌리지 않는다.
- 앱 updater와 Linux/macOS·P1/P2를 현재 P0와 구분한다.
- python3 scripts/check_docs.py 통과.

## 검증과 handoff

`python3 scripts/check_docs.py` 실제 exit0: 1206 documents PASS, hash-matched packaged upstream notices 133개 보존. 외부 URL과 heading anchor는 검사기 검증 범위 밖이다. 현재 표의 로컬 anchor는 제목과 대조했다. 문서 정리만 완료했으며 목록의 개발·실환경 인수는 미완료다. Git 조회는 exit128(fatal: not a git repository)이었으며 기존 .git을 보존했다. 새 제품 테스트나 빌드를 실행한 것으로 보고하지 않는다.

# S00: UI/UX 우선순위와 최신 인수 범위 정합성

- 상태: in-progress
- 분류: quality
- 우선순위: P0
- 요구사항: R01, R13
- 담당/수정 범위: 주 담당은 docs/validation/P0-status.md, P0-remaining-20261011.md와 본 기록, 별도 pixel review를 담당한다. diagnostic_inputs는 /tmp/dlpad-ui-current-priority-audit만 작성하며 제품 파일을 수정하지 않는다.
- 선행 작업: 현재 whole1145, 최신 설치 검증, normal Metadata 보고서 및 CloseWait v4 검증

## 사용자 결과

기능 검사·캡처 수집을 UI/UX 완성으로 오해하지 않도록 현재 설치본과 실제 확인 범위를 일치시키고, 디자인 가이드에 어긋나는 남은 결함을 코드와 화면으로 찾는다.

## 계약과 범위

DESIGN.md와 기존 modern-ui-audit를 기준으로 Rust 네이티브 제약을 유지한다. 기존 실패 기록과 설치본을 보존한다. 공개 배포·사용자 데스크톱 시험은 수행하지 않는다.

## 인수 조건

현재 설치본의 source 범위와 검사 수치를 문서에 정확히 연결한다. normal release 보고서와 DEBUG CloseWait를 구분한다. 직접 본 두 PNG의 SHA와 좁은 시각 판정을 기록한다. 코드 감사 후 수정·실제 화면 재검증이 끝날 때까지 전체 UI/UX 완료로 표시하지 않는다.

## 검증과 handoff

Git status는 exit128로 정상 저장소가 아님을 확인했고 기존 .git은 변경하지 않았다. 이전 docs 검사 handle83743은 Unknown process id로 현재 사용할 수 없다. 당시 성공 여부를 추정하지 않고 새 문서 검사를 실행한다.

현재 문서 검사 `python3 scripts/check_docs.py`는 exit0,927문서·114 notices 보존으로 통과했다. 직접 본 두 PNG의 SHA와 좁은 판정을 evidence/ui-current-priority-pixel-review-20261012.json에 기록했다. 이후 자막 UI 수직 작업과 후속 snapshot 구분 문구를 추가했으므로 새 문서 검사를 실행한다.

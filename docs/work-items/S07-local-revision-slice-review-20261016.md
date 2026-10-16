# S07: 로컬 upgrade/rollback 최소 수직 slice 읽기 조사

- 상태: proposed — 조사 보고 완료, 구현 미착수
- 분류: architecture review
- 우선순위: P0 S07
- 요구사항: R08, R11
- 담당/수정 범위: field_reveal; s07-local-revision-slice-review-v1 scratch REPORT/읽은 source pins만
- 선행 작업: ADR0005 Proposed, 기존 fresh-only caps/immutable activev1, revision6 순수회귀

## 사용자 결과

실행 가능한 첫 native A→B→A 시험 범위와 실제 사용자 cold upgrade/rollback까지 남은 경계를 Root가 구별해 구현 순서를 결정한다. 네트워크 updater/P1/P2는 다루지 않는다.

## 계약과 인수 조건

기존 API/파일을 직접 읽고 re-use 가능한 cap과 없는 writer/commit/recovery/launcher/data compatibility를 구분한다. schema/Live/S/Cargo/원 Evidence 수정0. 순수정책 Activated를 실제 native commit로 과장하지 않고 제안의 인수 한계를 명시한다.

## 검증과 handoff

REPORT.md와 source-pins.json을 기록했다. 명령은 rg/sed/cat 및 보고서/핀 작성 Python뿐이다. 빌드/테스트/GUI/native/네트워크 실행0. Root가 scratch 제안을 검토하고 별도 수직 작업을 승인·담당 배정한다. 이번 보고서로 전체 S07/P0/updater 완료를 주장하지 않는다.

Root가 기존 caps·immutable active 선택·data 호환 결손을 검토했다. 보고서의 제안 첫 slice는 실제 사용자 upgrade 완료가 아니므로 순수실험과 별도 유지한다. 현재 UI P0 후속통합/성능 실제검증 우선이며 제품schema결정과 구현은 아직 없다.

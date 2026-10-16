# S00: 진단 입력창 단일 테두리

- 상태: in-progress
- 분류: bug
- 우선순위: P0
- 요구사항: R08, R13
- 담당/수정 범위: root — crates/app/src/accessible_diagnostics.rs, 이 작업 기록
- 선행 작업: S00-diagnostic-empty-editor-density-20261012

## 사용자 결과

진단 로그 입력창의 하단 이중 테두리를 제거하고 배경·커서·선택 표시를 유지한다.

## 계약과 범위

DESIGN.md와 기존 UI ADR을 따른다. TextEdit 문서 테두리 대신 고정 viewport의 배경과 단일 테두리를 그린다. ID·입력 제한·여백·스크롤·보고서 반영 정책은 유지한다. 의존성·공유 schema 변경 없음.

## 인수 조건

기존 진단 편집·포커스·커서·스크롤 회귀와 전체 Windows 검사를 통과한다. 새 실행 파일로 Light/Dark 캡처에서 중복 테두리 제거와 배경·커서를 확인한다. 미반영 초안은 보고서를 변경하지 않으며 자연 종료와 격리 화면 보존을 검증한다. 단위 검사만으로 시각 인수를 완료하지 않는다.

## 검증과 handoff

root가 v5 typed-unapplied-editor.png에서 두 하단선을 직접 확인했다. 독립 source 감사는 기본 TextEdit frame과 별도 viewport stroke의 중복을 확인했다. 실제 검증 결과는 후속 기록한다.

- Windows 대상 진단 회귀: `/tmp/dlpad-diagnostic-single-frame-targeted.py`, session68792 exit0, source_unchanged=true. 상세 원본은 `../validation/evidence/windows-msvc-diagnostic-single-frame-targeted-20261012/diagnostic-single-frame.log`. 전체 검사와 새 native 캡처 인수는 진행 중이다.

- 문서 검사 session44596 exit0:933문서/114 hash-matched packaged notices. 전체 collector 최초 실행은 상대 output 경로를 거절해 시작 전 exit1; 절대 경로로 수정한 session63120에서 fmt/clippy/test 단계 exit0 및 source_unchanged=true를 확인했다. release 및 독립 검토는 아직 진행 중이다.

- 전체 Windows session63120 exit0 및 독립 원본 검토 session68048 exit0:1153성공/0실패/30ignored,43 harness. CPU9,693,184바이트 SHA `ff379a7770cdd66ad9b0939bbf97384761c2855b51da1f006a563ac5f5bffec7`. 이전 밀도 후보와 동일 크기이며 Cargo/dependency 변경 없음. 비용 전체 인수를 의미하지 않는다. `../validation/evidence/windows-msvc-diagnostic-single-frame-whole-20261012/root-review.json`. 새 native 캡처 검증 대기.

- Light session38095 및 Dark session13693 실제 native 진행 중 독립 검토에서 observer Main의 `summary=Deserialize(Run(...))`가 앞서 기록한 theme receipt 두 값을 덮어쓰는 시험 오류를 발견했다. 실행 중 파일 수정·재시작 없이 원 결과를 보존하고, 새 v2 후보에서 결과 대입 뒤 receipt를 기록·원 seed bytes 재확인하도록 수정한다. 제품 오류나 테마 시각 인수로 계산하지 않는다.

- v1 Light38095/Dark13693 모두 exit2: 실제 앱은 자연 exit0/원 Job active0이나 theme receipt 유실로 collector가 거절했다. 양쪽 실제 typed 캡처를 root가 직접 확인해 단일 하단 테두리·배경·한글 초안·끝 커서를 좁게 확인했고 각 evidence에 root-scoped-pixel-review.json을 기록했다. 전체 성공으로 계산하지 않는다.
- 수집기 v2는 result 대입 뒤 실제 원 theme 파일 SHA를 다시 확인하고 receipt를 기록한다. 제품 source 불변. 새 Light38810/Dark8833 private 시험 진행 중이며 기존 실패는 보존했다.

### 새 Light/Dark 실제 흐름 통과

- v2 Light38810/Dark8833 collector 모두 exit0. 독립 검토 Light79328/Dark95453 모두 exit0 `PASS_SCOPED_NORMAL_PRODUCT_REPORT_TRACE`. 자연 app0/원 Job active0/private desktop 종료와 입력 데스크톱 보존·실제 원 theme bytes·미반영 보고서 전체 바이트 동일을 원본으로 검토했다.
- root가 두 새 typed-unapplied-editor.png를 각각 직접 확인해 Light/Dark 배경·단일 하단 테두리·한글 초안과 끝 커서를 좁게 인수했다. 각 새 evidence에 root-independent-native-review.json과 root-scoped-pixel-review.json 보존. 모든 hover/long/disabled/배율과 물리 IME 및 전체 UI/UX 완료는 아니다.
- 제품 source는 Windows 전체1153/0/30 검사와 동일하며 기존 v1 두 실패 기록을 보존했다. 다음 작업은 비활성 시작 원인 접근이다.

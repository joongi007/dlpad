# S00: 종료 대기 화면의 명확한 안내

- 상태: in-progress
- 분류: quality
- 우선순위: P0
- 요구사항: R08, R13
- 담당/수정 범위: 주 담당 crates/app/src/catalog.rs·accessible.rs 및 이 문서
- 선행 작업: 실제 CloseWait release/timeout 시험과 root 대기 PNG 검토

## 사용자 결과

종료가 이미 요청된 상태에서는 “종료 준비 중”과 작업 정리 완료 후 종료 안내·작은 spinner를 보여준다. 더 이상 선택할 수 없는 동작을 안내하는 문구를 표시하지 않는다.

## 계약과 범위

문구는 중앙 catalog다. 기존 창 ID를 명시적으로 유지해 제목 전환에 따른 위치·상태 초기화를 피한다. worker·취소·대기·세 버튼 disabled 정책은 유지한다. 새 의존성 없음. 일반 작업 중 선택 화면은 유지한다.

## 인수 조건

Windows 전체4검사, 실제 waiting frame에서 새 제목·안내·disabled 버튼·원 event/worker ACK·해제/타임아웃·자연 종료 확인. 일반 prompt와 대기 상태의 창 ID를 유지하며 body/footer와 겹치지 않는다. 실제 primitive spinner의 비용은 대기 상태 범위로 기록하고 idle 비용 통과로 확대하지 않는다.

## 검증과 handoff

두 원 시험 release/timeout은 실제20표본·정제 보고서/빈 partial·readonly DB·자연 app0/Job0를 독립 대조했다. 검토기의 inputs.json64KiB 제한이 실제73931B보다 작아 실패해 해당 파일만128KiB로 보완했고 첫 실패를 보존했다. 제품 수정 후 native waiting-frame 인수는 아직 남아 있다.

수정본 Windows whole fmt·clippy·test·release는 모두 자연 종료0이다. 주 담당 원본 로그·소스·artifact 대조1145/0/30을 완료했으며 실제 새 waiting 화면은 distinct v4 feature 빌드 뒤 확인한다. 초기 최소 크기 수정과 함께 새 ZIP을 준비하고 있으며 기존 README 설치 링크는 유지한다.

CPU artifact9,690,624B(직전보다5120B 감소), installer4,310,016B(변화없음). 새 dependency 없음. [원본 검토](../validation/evidence/windows-msvc-bootstrap-closewait-clarity-whole-20261012/root-review.json). 이 크기 비교는 runtime CPU·메모리 측정이나 UI 최종 인수를 대체하지 않는다.

새 v4 DEBUG fixture release/timeout 모두 자연 종료0 및 독립 raw/5build·2case/원event·파일·ACK·readonlyDB 검토 통과. 실제 title/help 전체 client geometry와 기존background안내 부재를 확인했고 root 대기PNG의 spinner·문구·disabled 버튼을 직접 봤다. [두경로 독립검토](../validation/evidence/windows-msvc-current-closewait-release-v4-20261012/root-independent-two-case-review.json), [root픽셀](../validation/evidence/windows-msvc-current-closewait-release-v4-20261012/root-waiting-copy-pixel-review.json). 이는 별도 DEBUG barrier fixture 범위이며 일반release에서 실제 작업을 고의로 오래 막은 전체 종료 인수가 아니다.

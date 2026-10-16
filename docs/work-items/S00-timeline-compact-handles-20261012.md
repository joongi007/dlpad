# S00: 좁은 화면의 구간 손잡이 조작 여유

- 상태: in-progress
- 분류: quality
- 우선순위: P0
- 요구사항: R05, R08
- 담당/수정 범위: 주 담당 crates/app/src/media_timeline.rs와 이 문서
- 선행 작업: range-v3 실제 800×600/200% 실패 좌표 및 캡처

## 사용자 결과

시작·끝 손잡이를 함께 볼 수 있는 스크롤 여유를 확보한다.

## 계약과 범위

DESIGN의 최소28logical px hit area를 유지하고 두 손잡이의 수직 분리를 보존한다. 기존32px 손잡이를28px로, 전체 트랙 예약 높이를88px에서80px로 줄인다. band16px·접근성 ID·구간/seek 정책·키보드·줌을 유지한다. 의존성·schema 변경 없음. 기존80logical px span이 작은 본문에 정확히 맞아야 했던 문제를72px로 줄인다.

## 인수 조건

실제 Windows800×600/200%에서 두 손잡이가 본문에 완전히 보이고 조작 가능해야 한다. 기존 pointer drag·keyboard·zoom·disabled 실패 경로 테스트와 fmt·clippy·test·release를 통과한다. native 픽셀 확인 없이 완료로 표현하지 않는다.

## 검증과 handoff

구현 후 검증 진행 중. 원본 range-v3 실패를 보존하며 observer의 과도한 wheel step도 별도로 진단한다. 최종 UI/UX 인수는 남아 있다.

실제 whole fmt/clippy/test/release는 자연종료0,1139passed/0failed/30ignored이며 원본로그와source/binary 해시를 독립대조했다. range-v5 실제6matrix fullbounds와RangeValue100..1900ms/시간문자열동기화/전체구간0..2000ms·빈입력복귀가 원본receipt와독립review에서 통과했다. 이는 UIA조작이며 물리drag·seek·실사이트 검증이 아니다. 독립검토의 첫hash bound 실패는 보존하고256MiB 상한/64KiB streaming·파일identity검사로 고쳐 통과했다. 실제 캡처에서 lower handle과끝시간glyph가겹쳐 예약높이를96px로 늘려 아래16px를추가확보했다. handle span72px/hit28px는그대로다. 이 마지막spacing수정의whole/native검증은 진행중이다.

# S00: 미리보기 자리표시자의 재생 오인 제거

- 상태: in-progress
- 분류: quality
- 우선순위: P0
- 요구사항: R04, R05, R08
- 담당/수정 범위: 주 담당 crates/app/src/media_workspace.rs 및 이 문서
- 선행 작업: current-ui-network-density의 실제 다운로드 캡처 검토, ADR-0007

## 사용자 결과

영상이 준비되지 않은 미리보기에는 눌러도 재생되지 않는 재생 버튼 모양 대신 영상 자리표시자를 보여준다. 준비된 영상의 실제 재생 버튼은 유지한다.

## 계약과 범위

GTMUX의 명료한 동작 표현과 PRODUCT의 다음 행동 원칙을 따른다. 장식은 hover allocation이며 클릭 동작을 추가하지 않는다. 미디어 준비·worker·타임라인·저장 gate·배율 기하는 변경하지 않는다. 의존성·schema 변경과 UI 전체 개편은 없다.

## 인수 조건

- 준비 전·로딩·오류의 비조작 장식에 재생 삼각형을 사용하지 않는다.
- 실제 준비된 미디어의 재생 버튼과 키보드 identity를 유지한다.
- Rust 검증과 최신 Windows Light/Dark 캡처로 변경 범위를 확인한다. 코드만으로 시각 인수를 완료하지 않는다.

## 검증과 handoff

실제 initial-download-960.png에서 비조작 원형 재생 장식을 확인했다. 현재 구현·변경 후 검증은 진행 중이며 전체 UI/UX 인수는 미완료다.


변경 후 Windows whole은 fmt·clippy·test·release와 원본·pin 대조를 통과했다(1,139 성공/0 실패/30 ignored). 실행 파일 증가512B, 의존성 변경0. current-ui-placeholder-affordance는149개 원본/접근성 receipt를 독립 검토했다. 주 담당은 Light/Dark100/150 및 Dark재시작 장식·URL 끝 caret을 직접 확인했다. 전체 입력/물리IME 인수는 아니다.

200% 별도 관측의 두 실패를 보존했다. target-discovery 후보에서는 실제 초기 idle Text와 이후 도구 누락 Text 전환을 기록하고 bounded scroll 후 Light/Dark의 전체 장식/안내를 직접 확인했다. 두 private 자연 종료 및 원본 독립 검토도 통과했다. offscreen 노드 제거를 실패 원인으로 확정하지 않는다. 원본 실패는 강제 소유 Job 정리를 기록하며 성공 실행의 자연 종료와 구분한다.

최신 합성 loopback 미디어 준비·실제 red/blue 재생·Job 정리·도구/원본 pin 검토는 통과했다. 타임라인 접근 확장 시험은960100에서24회 이후 좌표 변화가 없어 실패했고800/150200은 실행되지 않았다. 구간 조작·시간 입력·전체 사용자 인수는 미완료다. 다음 UI 소스 수정 이후 이 결과들은 고정 이전 snapshot에만 적용한다.

2026-10-12 후속 실제 range-v3는 준비된 영상의 960/800×100/150/200% 중 첫5조건에서 전체 Slider bounds가 본문에 들어오는 것을 확인했으나 800×600/200%에서24회 budget을 소진해 실패했다. 원본 실패와 자연 cleanup0/forcedfalse를 windows-msvc-current-ready-media-range-v3-20261012에 보존했다. 초기 두 Slider의 span160px와 보수적 본문160px, 큰 wheel 이동의 overshoot 가능성이 있어 세밀한 양방향 조작을 후속 확인한다. 이후 RangeValue 및 시간 동기화 단계는 미실행이다. 전체 구간 UI/UX 인수 완료가 아니다.

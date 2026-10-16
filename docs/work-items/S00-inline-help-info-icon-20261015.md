# S00: 라벨 옆 정보 아이콘

- 상태: in-progress
- 분류: polish
- 우선순위: P0
- 요구사항: R07, R10
- 담당/수정 범위: advanced_units ui_components.rs scratch만; caller 라벨행 위치는 Root
- 선행 작업: 사용자 떠있는 `?` 시각 거절; Root 현재 공급/native 동결 종료 후 통합

## 사용자 결과

라벨 옆의 선택적 설명을16px 원형 정보 아이콘으로 인지한다. 글꼴에 따라 생김새가 달라지는 물음표 대신 native vector 도형을 사용한다.32px 클릭 영역과 이름/설명은 그대로 읽힌다.

## 계약과 범위

impeccable product register와 DESIGN의 compact/native/상태별 일관성을 따른다. Help 한 종류만 empty native Button + vector icon을 쓰며 primary/secondary/choice/ghost/danger 전체 스타일은 바꾸지 않는다. 기존 scopedID·Response·AccessKit Button title/description·hover/focus/pinned Popup·Escape focus복귀·밖클릭 새target·disabled 정리를 유지한다. 기본 muted, hover text foreground, pressed darker accent, pinned accent ring/tint, disabled mutedalpha와 기존disabled배경, focus 기존outline으로 구분한다. 새 timer/I/O/dependency/font/schema 없음. 문서/Live/S/evidence를 수정하지 않은 scratch workitem이며 Root가 최종 기록을 통합한다.

## 인수 조건

actual Context가 한글 상태명과 icon을 같은행에 정렬하고32hitarea/16vector를paint한다. Light/Dark100/200에서 idle/hover/pressed/pinned/focus/disabled를 실제Response와Shape로 확인한다. 기존 실제help2회귀를 그대로 유지한다. orphan caller 배치는 별도Root수정이며 helper만으로 전체UI 해결을 주장하지 않는다.

## 검증과 handoff

Linux rustfmt1.94 edition2024 두 후보파일 성공. Cargo/C#/native 시험은0, Root RED/GREEN 실행 대기다. before/REDtest-only/GREEN 전체파일,3diff,exactpins,HANDOFF 보존. 기존 `?` 원화면이 사용자 기준에 부적합했다는 피드백과 시각개선 후보이며 실제physical/reader/전체UI완료는 미검증이다.

Root 통합 검증: 원 정보 아이콘 RED는 실제 Context 0 통과/1 실패로 missing vector를 재현했다. 최초 GREEN은 기존 help 2 통과/새 focus 시험1 실패이며 원 증거를 보존했다. Escape 같은 pass 후반의 focus 복원보다 앞선 paint를 검사한 시험 시점을 다음 settled frame으로 교정했다. 새 GREEN-v2는 정보 아이콘·기존 keyboard/pointer 총3개와 입력 레이아웃1개 모두 통과했고 strict Windows workspace/all-targets clippy도 exit0이었다. 각 원로그/Job0/private/input 및 Live/S 불변을 Root가 대조했다. UI 두 파일은 이 결과 뒤 통합했으며 새 whole-v27와 private 시각 검수는 실행 전후를 별도로 기록한다. 실제 사용자·reader 인수는 미완료다.

새 whole-v27 실제 네 명령 exit0, source674, 1,593 통과/0 실패/42 ignored/43 harness. Root가 원로그·현재 소스·3PE·lock을 독립 확인했다. CPU는 10,306,560바이트 SHA1576b78a5f01c78515ed3349a918038baafa973f698ee04bae9fe1a66be6d6f2로 직전 whole-v26-v5보다3,072바이트 늘었다. CLI/설치기 바이트는 같다. 새 dependency/lock/schema는 없고, 도움말마다 벡터3개를 그리며 runtime 전체 비용 절감은 측정하지 않았다. 실제 private Help/Tools/Search/Progress 화면은 실행 중이며 전체 UI 사용자 인수로 승격하지 않는다.

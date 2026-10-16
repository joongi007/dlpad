# S01: 조각 다운로드의 소수 예상 용량

- 상태: in-progress
- 분류: bugfix
- 우선순위: P0
- 요구사항: R01, R04, R08
- 담당/수정 범위: Root engine/metrics.rs, app/accessible.rs, CONTEXT.md
- 선행 작업: 원 v26 전체·GUI 실패 근거 보존, 실행 중 source freeze

## 사용자 결과

yt-dlp 조각 다운로드가 예상 용량을 소수로 보내도 속도·ETA·진행률이 사라지지 않는다. 예상 진행률은 전송 중 100% 완료를 주장하지 않는다.

## 계약과 범위

공식 fragment.py와 utils/progress.py는 예상 크기의 소수를 보존한다. 다운로드한 크기·확정 크기는 엄격한 u64를 유지하며 예상 크기만 finite, nonnegative, u64 상한의 f64로 읽는다. DB/schema/lock/dependency 변경은 없다. 확정 양수 크기를 우선하고 범위 밖 값은 원 관측을 지운다. 3초 freshness와 bounded 최신 관측 하나를 유지한다.

## 인수 조건

25/100.5와 1/1.5의 실제 파서·feed·UI 표시(24.8%, 66.6%), 속도/ETA 보존, 예상 동일값의 미완료 99.9%, 확정 total 우선, zero·negative·oversize·malformed 실패를 검사한다. Context 검사는 실제 Windows native GUI와 별도다.

## 검증과 handoff

후보 준비 중. 전체 Windows 검사와 private 합성 전송은 새 근거로 실행한다. 새 의존성 0, 관측의 같은 8바이트 필드를 소수로 해석하며 상시 thread·queue 추가 0. 기존 v26 whole-v2 성공은 변경 후 현재 증거로 사용하지 않는다.

Root 선택 GREEN 실제 검수: managed-tool-isolated-native-value-green-v1-20261015에서 입력 9, 엔진 전송 4, 진행률 Context 1(64조합), 도구 compact Context 1(6장면)이 모두 통과했다. 각 명령 exit0/소유 Job0·private desktop 종료·input desktop 불변 및 Live/후보 source 불변을 Root가 원로그와 receipt로 대조했다. 전체 Windows 네 명령은 새 whole-v3에서 실행 중이며 실제 GUI/새 설치본은 아직 미완료다.

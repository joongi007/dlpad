# S00: 확대 화면의 상태 지표 줄바꿈

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R01, R08
- 담당/수정 범위: 주 담당 통합; diagnostic_cache는 /tmp 후보만. crates/app/src/accessible_metrics.rs, 이 문서
- 선행 작업: S00-current-five-screen-ui-polish-20261011

## 사용자 결과

좁은 확대 화면에서 네트워크·디스크 지표를 읽기 쉬운 정보 단위로 줄바꿈한다. ‘측정 불가’의 마지막 두 글자만 다음 줄에 남지 않게 한다.

## 계약과 범위

DESIGN의 컴팩트 상태 표시를 따른다. 현재 지표의 freshness, pending_visible, worker ACK와 측정 가능 여부는 바꾸지 않는다. 창보다 큰 정보는 필요한 줄바꿈을 유지한다. 설정 화면 담당과 파일이 겹치지 않는다.

## 인수 조건

실제 Context·AccessKit에서 800폭/200%의 지표 bounds와 줄바꿈을 검사하고, 가상 Windows Light/Dark 캡처로 확인한다. 긴 값도 창 밖으로 넘기지 않는다. 측정 실패를 정상 수치로 바꾸지 않는다.

## 검증과 handoff

current-ui-settings-focus-800-zoom-20261012의 200-settings-after PNG에서 ‘불가’가 단독 다음 줄로 내려가는 현상을 주 담당이 직접 확인했다. /tmp 후보를 준비하며 native 실행·프로젝트 통합은 주 담당만 수행한다.

### 실제 Windows RED 확인

`windows-msvc-scale-footer-ui-red-20261012/test.log`에서 회귀 테스트 실패를 확인했다. 전체 행에 들어가는 상태 지표가 남은 폭 때문에 분리됨. 컴파일 성공, 27 passed / 2 failed / 2 ignored; 소유 Job natural-zero=True, forced=False, input desktop unchanged. GREEN 통합 및 화면 검증은 아직 진행 전이다.

### 첫 통합 검증 결과

`windows-msvc-scale-footer-library-ui-whole-20261012`: fmt 및 strict clippy 통과. workspace test는 첫 UI harness 228 passed / 2 failed / 3 ignored에서 중단되어 release build는 실행되지 않았다. 배율 선택 회귀와 기존 테마 문맥 회귀는 통과했다. footer 좁은48px wrap 및 library wheel 안내 도달 회귀는 실패하여 원인 조사 중이다. 전체 통과 또는 UI 완료로 주장하지 않는다. 소유 Job 자연 종료/정리 및 input desktop unchanged 확인.

### 폭 교정 v2

Ui.max_rect 확장 가능성을 발견하여 두 지표 배치 전에 max_rect와 clip의 교집합 폭을 고정했다. 실제 원인 확정은 수치 포함 Windows 회귀 실행 후 판단한다. fitting/oversized 전제 검사를 강화했고 기존 줄 수·clip·AccessKit 인수 조건은 유지했다. `windows-msvc-footer-v2-library-scroll-diagnosis-20261012` 실행 중.

### 실측 원인 정정 및 재검증

진단 실행은 실제 row=4984, clip=5000, natural text=106.84375를 기록했다. egui 0.33.3 context.rs의 pending zoom 첫 pass가 전달 screen_rect를 이전 content_rect 기반으로 덮어쓰는 동작을 확인했다. 따라서 앞선 max_rect 확장 가설을 이번 회귀 실패의 확정 원인으로 주장하지 않는다. 두 warmup pass 및 실제 viewport 48/400과 pixels_per_point=2 조건을 추가했다. 전체 문구 wrap/clip/AccessKit 조건은 유지했다. `windows-msvc-scale-footer-library-ui-whole-v2-20261012` 실행 중.

### v2 workspace test 종료 확인

원본 `windows-msvc-scale-footer-library-ui-whole-v2-20261012/test.log`를 주 담당이 집계하여43 harness / 1,111 passed / 0 failed / 28 ignored를 확인했다. 테스트 Job natural-zero/forced=False/input desktop unchanged. release build는 아직 진행 중이므로 전체 검증 완료로 기록하지 않는다. 교정된 footer/library RED와 GREEN 테스트 본문은 rustfmt 후 바이트 동일성을 독립 확인했다.

### v2 전체 검증 완료

`windows-msvc-scale-footer-library-ui-whole-v2-20261012/root-review.json`에서43 harness / 1,111 passed / 0 failed / 28 ignored, fmt·strict clippy·locked CPU workspace test·release build의 실제4명령 및 current source/hash/artifact를 주 담당이 독립 확인했다. CPU 9,670,144 bytes SHA256 `252b7bbaf3a476e574af5925ca392c3680f891a8f5c6b25024d40be88562c92f`; Cargo workspace/lock/schema 변경 없음. `current-ui-scale-selected-context-layout-20261012` 가상 화면 실행 중. 사용자 시각 인수·설치본 갱신은 미완료다.

### 교정된 동일 시험의 독립 RED/GREEN

`windows-msvc-scale-footer-library-corrected-original-red-20261012/test.log`: 원 생산 코드 + 동일 교정 시험에서141 passed / 2 failed. footer는 실제 logical400/row384/natural106.84375/clip400×300에서 문구 분리, library는24px incremental wheel에서도 안내 전체 도달 실패. 이전 whole-v2 동일 시험 GREEN1111/0와 비교하여 수정 효과를 확인했다. 원본 파일 SHA 및 포맷 후 테스트 본문 byte 동일성을 확인했으며 RED 종료 후 저장한 GREEN bytes를 정확히 복원했다. Job natural-zero/forced=False/input desktop unchanged.

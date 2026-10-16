# S05: 설명 있는 주요 옵션과 자르기 선택

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R07, R08
- 담당/수정 범위: 주 담당; crates/core/src/options.rs, crates/app/src/main.rs, catalog.rs, scripts/native_probe.py, scripts/x11_diagnostics.py, scripts/test_native_probe.py, 검증/요구사항 상태 문서
- 선행 작업: S02 worker와 typed argv 정책, 실제 합성 빠른/정확 구간 저장 검증

## 사용자 결과

빠른/정확 구간 자르기를 선택하고 고급 옵션을 펼쳐 속도 제한·재시도·조각 동시성을 입력한다. 범위 오류는 해당 입력 옆에 표시한다. 기본값·비용·충돌·검증 여부와 실행 계획을 확인한다.

## 계약과 범위

순수 옵션 validation을 UI/plan이 공유한다. 입력 길이 상한, 예약 옵션 우회 없음, argv 배열. 실행 중 선택과 옵션 입력 잠금. SQLite schema/Cargo 변경 없음. 공급처/관리 설치/전체 S05 완료는 제외.

## 인수 조건

빈 값 기본 복귀, 경계/불법 값 거절, retry override 중복 없음, 정확 cut argv, 전체 영상 모드에서 cut 플래그 없음. 잘못된 값으로 Start 시 worker 시작 안 함. Xvfb screenshot과 공통 검사.

## 검증과 handoff

공통 fmt/clippy/test/release 성공. 빈 값·경계/불법 입력·retry 중복 방지·긴 붙여넣기·작업 시작 거절·정확 cut/전체 영상·실행 중 변경 차단을 검증했다. Xvfb에서 옵션을 펼치고 하단으로 스크롤한 실제 화면을 확인했다. 최종 통합은 Rust 82 tests, 실제 도구 1 test, Python 안전 4 tests. [실행 결과](../validation/options-validation.md). 실제 IME/다른 OS는 별도 게이트. 전체 S05/P0 완료는 아니다.

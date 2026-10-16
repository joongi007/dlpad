# S01: CPU 라이브러리 화면 공유 API 조사

- 상태: done
- 분류: investigation
- 우선순위: P0
- 요구사항: R03, R08, S01
- 담당/수정 범위: library_identity — 전용 workitem/validation만
- 선행 작업: CPU Candidate/host 고정, Windows CLI 증거 재검토

## 사용자 결과

기존 Iced 라이브러리의 저장위치·목록·refresh·다음page·nativewatch를 CPU 후보에도 제공할 수 있도록 기존 엔진 계약과 독립 구현 범위를 확정할 근거를 정리한다.

## 계약과 범위

제품소스/Cargo/runner 수정과GUI빌드 없이 main.rs/accessible.rs/engine library/watch를 읽기 전용 조사한다. 새 구현은 아직 승인되지 않았으며 신규 accessible_library.rs 소유/공유 Candidate 연결을 제안한다.

## 인수 조건

기존 화면과 실제 engine API 대응, UI thread I/O 방지, pagination/scan/watch 상한, 실패시목록보존·rootchange stale result 정책을 명시한다. Windows 새hidden-init evidence source235/runner/lock/exe/raw시험 독립검토를 함께완료한다.

## 검증과 handoff

windows-cpu-hidden-init/independent-review.json: 현재source235 mismatch0, runner/lock/exe 일치, raw131passed/1ignored/0failed. 계획근거는 docs/validation/cpu-library-parity-analysis.md. 제품구현/GUI시험 미실행.

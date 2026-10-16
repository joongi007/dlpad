# S00: 동적 native thread pin과 종료 tail 구분 진단

- 상태: prepared (설계/실행후보만, compile/native 미실행)
- 분류: diagnostic
- 우선순위: P0
- 요구사항: R01/R03/R08, S00 성능 원인분리
- 담당/수정 범위: p0_audit — 이 전용 B 폴더만. Root — 실제 채택/현재-source proof·실행/통합. software_host/frame_phase/worker_phase/Cargo/schema/lock은 변경0
- 선행 작업: Root 현재 UI 후보 통합 및 fresh feature4검사; 다른 native/package 모두 terminal·source freeze

## 사용자 결과

strictCPU failure의 초기 pin 밖 새 thread 비용을 발견하고 이미 retained HANDLE로 수집한 종료 tail과 구분한다. 성능 예산·주기·기능을 바꾸지 않는다.

## 계약과 범위

기존 v25 phase 실제 원자료와 source 함수를 읽었다. fixed numeric-only sidecar·lifetime128·poll750/window6·원process/thread HANDLE/creation/Wait0/ownerJob·비원자 timestamp·new admission 이전CPU미포함 계약. source 최적화·실제 app worker에 새 clock/span·공유 protocol 변경은 하지 않는다. description 원문 수집을 끄며 경로/URL/secret 로그 없음.

## 인수 조건

DESIGN/REGRESSIONS의 정상종료tail·lateborn·중복/다른identity·counter오류·상한·stop/cleanup·unseen shortthread 한계를 실제 검증한다. coverage_complete/default_budget_acceptance는false다. 검증 없이 원strict 실패를 성공으로 덮지 않는다.

## 검증과 handoff

읽기/official Win32 문서 확인과 scratch 파생/텍스트 guard만 실행. C#/Rust compiler·native·desktop·서버실행0. frame-observer.cs는 실행후보이며 실행결과가 아니다. observer.patch는 immutable v2 baseline 대비이다. source-pins.json로 읽은 현 source/lock을 고정했다. Root가 소스 freeze 후 새 exact proof를 연결하고 실행한다.

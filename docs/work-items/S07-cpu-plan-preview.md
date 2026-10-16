# S07: CPU 실행 계획의 정제된 미리보기

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R07, R08, R11
- 담당/수정 범위: metrics_recovery — 신규 crates/app/src/accessible_plan.rs, 전용 작업 문서
- 선행 작업: core DownloadOptions.plan; root shared entry/Candidate/catalog 연결

## 사용자 결과

실행 전 정책으로 검증된 argv를 JSON 배열로 확인하되 원본 URL 전체를 catalog 생략 표시로 대체한다.

## 계약과 범위

순수 builder만 소유하며 shell/I/O/process/DB/Cargo/schema 변경 없음. root가 module을 선언하고 UI 연결 및 typed error catalog mapping을 맡는다. Proposed UI/renderer 결정을 변경하지 않는다. 문자열은 기존 OMITTED_SOURCE를 사용한다.

## 인수 조건

원본URL query/fragment 미노출, userinfo URL 정책 거절, Unicode/공백/따옴표 및 typed 품질·구간·고급 argv 보존, invalid옵션 실패, JSON escape 확대와 oversized path에서 실제출력256KiB 상한 유지, nonUnicode경로 실패. bounded writer가 초과 append를 거절하고 path clone 이전 preflight를 수행한다.

## 검증과 handoff

module undeclared 상태의 독립 source이며 제품 컴파일에 연결하지 않는다. 작성한4회귀는 root module 연결 뒤 실제 Cargo에서 실행해야 한다. CPU GUI/preview 카탈로그/키보드 인수는 후속 root 통합 범위이며 완료를 주장하지 않는다. 새 의존성과 런타임 worker 없음; 한 요청 출력 최대256KiB 및 validated argv snapshot 비용.

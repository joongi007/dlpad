# S01: Windows 목록 단계별 재확인 비용 분석

- 상태: in-progress
- 분류: performance
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: root 임시 실험 복사본·원증거·통합, soak_recovery_audit /tmp profiling 초안; 제품 library 소스 변경은 root만
- 선행 작업: S01-windows-native-library-scanner 현재 GUI60초 종료 및 소스 동결 해제

## 사용자 결과

파일 1만 개 목록의 주기 재확인 CPU를 낮추면서 추가·삭제·이동 관측과 취소·실패 복구를 유지한다.

## 계약과 범위

현재 preflight, native observation INSERT, rename 검증, publication SQL 및 commit 단계의 wall/CPU를 임시 실험 복사본으로 분해한다. clock/watch 주기와 파일 identity 검증을 완화하지 않는다. 공유 Cargo/schema/lock 변경은 root만 조정한다. 하위는 /tmp만 쓰며 실제 네이티브 실행은 root가 직렬로 수행한다.

## 인수 조건

실험 코드와 입력/source SHA, 합성 1만 파일 보존·DB 무결성·원 Job 자연 종료를 기록한다. 측정이 원 제품 binary와 다른 경우 제품 성능 인수로 표현하지 않는다. 개선 채택은 의미 있는 실패 회귀와 동일 GUI 기준 재검증 뒤 결정한다.

## 검증과 handoff

현재 native scanner GUI 첫 회차 평균 CPU0.728%·메모리 PASS이나 10초 구간3/6 CPU1.731/1.413%로 strict FAIL, 두 번째 평균1.118%로 FAIL이다. 전체3회 종료 전 최종 결과는 미확정이다. 단계별 병목 원인은 아직 추정이며 계측 후 결정한다.

계측 선택: default-off `library-scan-profile-fixture`는 새 의존성 없이 scan당 약20회 thread CPU 조회와9개 고정숫자 단계값만 추가한다. 일반 빌드는 cfg로 계측 코드가 제거되어 런타임 비용0이다. 모든 엔트리 loop 안의 조회/Instant는0이며 traversal+INSERT CPU를 결합값으로만 보고한다. feature 계측 바이너리는 제품 GUI 성능 인수로 쓰지 않는다.

coarse native 실제실행3회 PASS: CPU0.078125/0.109375/0.109375초, unchanged두회차 preflight약16.6ms/traversal+INSERT약29.3ms/candidate EXISTS약17.4ms/rename UPDATE약7.2ms/UPSERT약31.7ms였다. SQL publication 단계가 남은 비용의 상당부분이다. 프로파일feature 빌드/Clippy와 목록24회귀 PASS, root 원로그/source검토완료. 계측 binary이므로 원GUI예산 성공으로 표시하지 않는다. 다음 exact unchanged snapshot skip의 안전성 검토를 진행한다.

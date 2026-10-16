# S02: EOF keeper의 Rust guardian 진입점

- 상태: completed (guardian 진입점 및 실제 제품 연결 범위)
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당 파일: root — process/guardian.rs, 이 문서; stress_recovery — 별도 user_service.rs 통합
- 의존성: EOF keeper 독립 실제 6case, 공통 내부 marker 계약

## 사용자 결과와 계약

초기 manager identity 확인 전 오류에서도 launcher의 EOF writer가 닫히면 수명 유지 helper가 종료된다. 기존 helper 실행 파일의 별도 내부 marker를 사용하며 쿠키·receipt·도구 argv를 받지 않는다. stdin은 현재 사용자 소유 readonly pipe이고 추가 argv는 없다. 실제 EOF는 exit0, 데이터와 잘못된 FD는 거절한다. 제품 timeout을 추가하지 않고 UI thread와 별개 프로세스에서 대기한다.

## 인수 조건과 검증

실제 pipe EOF·byte 입력·쓰기 FD·regular file 거절을 검증한다. keeper service는 parent 밖에 있고 원 reader pidfd/native empty 및 durable receipt까지 수명을 유지한다. 다른 service를 부모 Stop으로 종료하지 않는다. helper 하나와 unit 하나, pipe 한 쌍의 추가 비용은 실제 성능 검증에 반영한다. 새 Cargo 의존성/schema 변경 없음. 현재 구현 중이며 native/Cargo 통합은 source 동결 후 root가 수행한다.


## root 통합 검증 결과

첫 제품 keeper 검증에서 실제 pipe 검증 단위 시험이 통과했다. 제품 초기 실패 보강 후 `cookie-keeper-product-second`의 실제 service3/download4/recovery2 시험이 통과했고 `cookie-keeper-startup-third`의 정상 대조 및 세 시작 실패를 포함한 native2도 통과했다. 초기 실패에서 source/sentinel/private copy와 Intent를 보존하고 도구 실행 없이 소유 unit을 회수했다. 결과·원 실패 로그는 해당 evidence에 보존한다.

전체 workspace 현재 검증은 [별도 작업](S03-cookie-keeper-current-workspace.md)으로 실행 중이다. 추가 helper 비용, 실제 제품 foreign sibling, release env 무시 실제 실행, 다른 OS 및 재부팅 인수는 이 진입점 완료로 대체하지 않는다.

# S02: 제품 process의 쿠키 durable 시작 gate

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: root — process.rs, process/guardian.rs, 이 문서; metrics_recovery — cookies.rs; soak_recovery_audit — download.rs
- 선행 작업: 실제 native launcher3PASS, 원래 journal/receipt 파일 검증

## 사용자 결과

Linux 세션 쿠키를 사용하는 metadata와 download가 실제 소유 컨테이너 identity를 journal에 저장한 뒤에만 도구를 실행한다. 종료 확인과 실제 receipt resolve가 모두 성공해야 복사본 정리를 허용한다.

## 계약과 범위

ProcessSpec와 일반 실행 API 유지. Linux 전용 run_metadata_with_cookie와 run_with_cookie는 각각 slot0/1을 사용한다. 한 worker의 새 lease에서 슬롯별1회만 실행한다. agent별 파일 소유를 분리한다. 공유 schema/Cargo 변경 없음. guardian의 control/writer/record lock 세 FD만 child에 전달하고 launcher가 즉시 CLOEXEC를 복원한다. C→I→durable bind→B→R를 취소/기한 안에 수행한다.

## 인수 조건

제품 DownloadService 실제 합성 정상 metadata/download, 취소, metadata 실패/download 실패와 actual receipt·copy 정리/보존 확인. 취소전예약없음; bind/receipt 실패는 typed cleanup deferred. 기존 Windows/쿠키없는 실행 유지. privatefixture/합성source만 사용.

## 검증과 handoff

구현 중. 부모 release 경합 정책은 별도 actual experiment 후 통합한다. 전체 same-boot startup 복구와 전체 P0 완료로 표시하지 않는다.

root 실제 통합 `cargo check -p dlpad-engine --all-targets --locked` exit0, helper build exit0, `cargo test -p dlpad-engine --lib --locked` 244PASS/2ignored, `cargo test -p dlpad-engine --test cookie_download_gate --locked native_authenticated_ -- --ignored --nocapture --test-threads=1` 실제4PASS(8.82초). [원시와 source/helper SHA](../validation/evidence/cookie-product-gate-first/source-and-helper-sha256.json)를 현재 source 동결 상태에서 보존했다. 실제 파일/DB/omission 인수를 포함한fixture를 root가 실행했다. 이후 same-boot 복구/parent release 경합과 전체workspace검증은 별도 범위다.

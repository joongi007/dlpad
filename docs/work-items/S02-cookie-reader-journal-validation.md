# S02: durable reader journal 실제 Linux 검증

- 상태: done
- 분류: test
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: root — cookie_reader_models.rs Cargo 연결·검증, 이 문서
- 선행 작업: reader_journal source 동결

## 사용자 결과

고정 두 reader의 디스크 Intent/Bound/Resolved 및 receipt·lock 검증을 임시 폴더에서 실행한다. 실제 제품 launch gate와 다음 시작 recovery는 미연결이다.

## 계약과 범위

신규 anchored journal 모듈을 Linux integration Cargo 시험으로 연결한다. 제품 cookies/process 경로와 Cargo dependencies는 변경하지 않는다. 실제 OS 임시 파일·권한·fileID·lock 및 주입된 쓰기 실패를 사용한다.

## 인수 조건

5개 actual journal 실패/정상 시험과 기존8개 model 시험을 실행한다. 원본 sentinel/unknown 파일 보존, partial commit poisoning, receipt mismatch 거절·상한 유지.

## 검증과 handoff

root 실제 Cargo 결과 대기. File durability foundation 성공을 app crash·service gate·재시작 회수 완료로 확대하지 않는다.

Root 실제 명령 `cargo test -p dlpad-engine --test cookie_reader_models --locked`: 13 passed, 0 failed. 실제 파일·lock 시험5개와 model8개 통과. 제품 launch/recovery 연결은 후속 작업이다. 원시 로그: docs/validation/evidence/cookie-reader-journal-validation/test.log.

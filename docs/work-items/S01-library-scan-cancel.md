# S01: 라이브러리 스캔의 협조적 중단

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: 주 담당; crates/engine/src/library.rs, 본 문서, 전용 검증 문서
- 선행 작업: 기존 reconciliation transaction, CPU library panel 공유계약

## 사용자 결과

종료 요청 후 전체 폴더 관측의 다음 안전 경계에서 스캔을 중단하고 기존 목록을 보존한다. 단일 filesystem/SQLite 호출 중 즉시 중단을 보장하지 않는다.

## 계약과 범위

기존 reconcile_library API와 schema를 유지하고 reconcile_library_with_cancel(root,&AtomicBool)를 추가한다. Cancelled 오류를 명시한다. directory entry와 DB publication 및 commit 직전 취소 확인 후 transaction rollback한다. CPU panel 담당은 새 모듈만 소유하며 본 파일을 수정하지 않는다. 실행 중 Windows GUI snapshot 수집이 끝난 뒤 소스를 변경한다.

## 인수 조건

시작 전 취소는 filesystem을 열지 않는다. 관측 도중 및 최종 publication 직전 취소는 기존 DB snapshot을 유지한다. 이후 fresh scan은 정상 성공하고 삭제·추가를 반영한다. 비취소 기존 동작·깊이/권한/rename 실패 회귀를 유지한다. UI thread join 없이 종료 ack를 기다리는 panel 연결은 별도 작업이다.

## 검증과 handoff

아직 구현·시험 완료가 아니다. 정상 Git 저장소가 없으므로 기존 .git은 보존한다.

실제검증: `cargo test -p dlpad-engine library::tests --locked` 10 passed/0 failed/0 ignored(다른 시험은 filter), 24.59초 빌드/0.15초 실행. 시작전취소·관측중·publication직전·commit직전rollback·후속freshscan 성공을 확인했다. 기존8개 라이브러리 회귀도통과했다. CPU통합 `cargo clippy --workspace --features dlpad/cpu --all-targets --locked -- -D warnings` exit0, CPUentry20tests pass. 취소 flag 확인 뒤 commit의 짧은 경쟁은 정상완료로 결정될 수 있으며 개별 OS/SQLite 호출 강제중단은 제공하지 않는다. root/sharedUI 종료ack는 별도integration작업이다.

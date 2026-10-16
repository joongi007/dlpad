# S01: 파일 identity와 외부 이름 변경 정합성

- 상태: implemented
- 분류: feature
- 우선순위: P0
- 요구사항: R03
- 담당/수정 범위: library_identity 에이전트; crates/engine/src/library.rs, crates/engine/src/library/identity.rs, 본 기록, docs/validation/library-identity.md
- 선행 작업: S01-library-foundation, 주 담당의 store.rs migration 합의

## 사용자 결과

외부 이름 변경을 파일 identity로 확인할 수 있는 경우 기존 관측과 연결한다. 하드링크와 확인할 수 없는 플랫폼은 불확실한 이름 변경을 추정하지 않는다.

## 계약과 범위

ADR-0002와 ARCHITECTURE의 전체 스캔 transaction, symlink 제외, bounded pagination 계약을 유지한다. Unix dev+ino identity를 해시나 파일 내용 읽기 없이 관측한다. 공유 DB schema는 주 담당이 조정한다. Git 저장소가 없어 branch/worktree를 만들 수 없으며 기존 .git을 보존한다. 앱 UI, download/process/report/core options, Cargo 파일은 수정하지 않는다.

## 인수 조건

- 한글 경로 rename 뒤 동일 identity와 기존 row가 유지된다.
- 하드링크와 삭제/다른파일 재생성은 rename으로 잘못 연결하지 않는다.
- 실패한 전체 스캔은 이전 path/identity를 보존한다.
- schema 이전 DB는 nullable identity로 안전하게 동작한다.
- 비Unix는 identity 미지원으로 명시하며 파일 목록 갱신은 유지한다.

## 검증과 handoff

주 담당이 nullable identity migration을 승인하고 구현했다. Linux library 회귀 9개가 0.14초에 통과했다. 세부 결과와 inode 재사용/비Unix/source linkage 제한은 [검증 기록](../validation/library-identity.md)에 있다. 전체 통합 공통 검증은 주 담당에게 인계한다. Windows/macOS 실증과 출처 sidecar linkage는 미완료다.

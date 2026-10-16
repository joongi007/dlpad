# S00: 시작 준비 상태와 비활성 이유 표시

- 상태: done-scoped / 사용자 검수 대기
- 분류: bug / UX
- 우선순위: P0
- 요구사항: R08, R10, R13
- 담당/수정 범위: 주 담당; crates/app/src/accessible.rs, crates/app/src/catalog.rs, 검증 기록과 최신 검수 문서
- 선행 작업: v33 사용자 검수; 설치기와 상세 탐색기 작업은 별도 담당

## 사용자 결과

앱 시작 시 다운로드와 일부 설정이 잠긴 이유를 상단에서 확인한다. 복구 실패를 계속 확인 중으로 표시하지 않는다. 완료되면 준비 안내가 사라진다.

## 계약과 범위

DESIGN.md와 ADR0005/0009에 따라 기존 native panel과 중앙 문자열을 사용한다. I/O, schema, 도구 검증 정책은 바꾸지 않는다. 설치기/상세 구현 파일은 병렬 담당자만 수정한다. Git 저장소 인식이 실패하여 기존 .git을 보존하고 파일 소유권으로 분리한다.

## 인수 조건

- 확인 중/실패/완료가 구분되고 실패 시 재시도할 수 있다.
- 100/200% 작은 창에서도 안내와 버튼이 겹치지 않으며 초점을 강제로 옮기지 않는다.
- 실제 Candidate Context 회귀와 Windows workspace 검증을 실행한다.

## 검증과 handoff

진행 중. cargo/rustc가 WSL PATH에 없으므로 프로젝트 소유 Windows MSVC 도구와 private desktop host를 사용한다. 실제 OS IME/reader와 사용자 UI 승인은 별도다.

### 2026-10-17 통합 중간 결과

작업 시작 namespace 20261016을 유지한다. 새 상단 안내는 이전 작업 확인과 도구 설정 읽기가 끝나면 사라진다. 확인 실패는 RECOVERY_FAILED로 구분하고 재시도를 상단에 배치했다. 복구 중에는 다운로드 진행률 바를 표시하지 않는다. 추가 schema/의존성/상주 worker는 없다. 준비 중에만 폰트 캐시의 문장 높이를 측정하며 준비 완료 후에는 그 비용이 없다.

실제 Windows targeted v3에서 첫 프레임 clip 실패를 잡았다. v4의 Context lock 재진입 실패, v5의 Rust E0500 컴파일 실패도 원 로그로 보존했다. 폭을 Context lock 밖에서 읽고 안내 문자열을 소유하도록 고친 v6의 실제 Candidate 회귀는 1통과/0실패다. 800×600/400×300 logical viewport, Light/Dark, 실제 glyph clip·초점 비탈취·준비중/실패/완료 전이를 확인했다. 일반 sandbox의 WSL interop 실패는 별도 보존했고 승인된 native private host로 실행했다.

전체 Windows whole34 fmt/clippy/test가 성공했고 실제 총계는 1679통과/0실패/42ignored/43harness다. release·패키지·native 설치 검증은 진행 중이며 완료로 쓰지 않는다. Cargo.lock과 의존성/schema는 변경하지 않았다. source675를 동결했다. 설치기와 로컬 상세는 각각 S07/S00 작업 기록을 참조한다.

### v35 캡처 반영

whole34의 실제 CPU renderer PNG4개를 Root가 확인한 뒤 복구 worker에도 노출되는 다운로드 취소/일시정지 행의 첫 프레임 clip을 발견했다. 해당 행은 ready 이후 다운로드 worker에만 표시하도록 고쳤다. 같은 실제 Candidate 회귀에 다운로드 제어 문구 부재 검사를 추가했으며 Windows targeted35-v1은 통과했다. v34 portable은 source post-check가 거절한 원 결과로 보존하고 사용자 경로에서 제외한다. 최종35는20261017 namespace로 새 whole/graph/package 검증을 수행한다.


### Root 최종 통합 — 2026-10-17

최종 검수본은v35다. Windows whole35 fmt·strict workspace/all-targets clippy·test·release 실제0,1679/0/42·43harness·source675·3PE·lock을 Root가 대조했다. 실제 CPU renderer104프레임/unpainted0·Light/Dark100/200% PNG4개를 직접 검토했다. 새 패키지 전체CRC/SHA와 private 설치·제거·동일위치 재설치·cold 확인(수동 재검증0회)·설치 앱 자연0/Job0을 확인했다. [통합 검수](../validation/ui-review-v35-20261017.md), [최신 실행](../development/windows-user-preview.md).

Explorer 실제 선택과 최종 사용자 UI 승인은 남는다. 자동 시험에서 사용자 desktop·clipboard·외부 viewer를 열지 않았다. 버전 비교·기존 앱 자동 교체는 후속이며 최신 후보 확인에는 새 폴더 또는 portable을 사용한다. 기존 .git은 보존하고 Git조회exit128을 유지한다.

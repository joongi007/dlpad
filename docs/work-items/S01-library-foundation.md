# S01: 저장 위치와 라이브러리 기반

- 상태: implemented, 통합 검증 대기
- 분류: feature
- 우선순위: P0
- 요구사항: R02, R03
- 담당/수정 범위: library agent — crates/engine/src/library.rs, crates/engine/src/store.rs, crates/engine/src/watch.rs, 이 문서
- 선행 작업: S00 확정 후 UI 통합; 현재 격리된 엔진 구현만 수행

## 사용자 결과

OS Documents 조회와 명시적 저장 폴더 생성, Unicode 파일 목록과 추가/삭제 반영을 제공한다. 접근 불가 루트를 삭제로 판단하지 않는다.

## 계약과 범위

ADR-0002와 ARCHITECTURE 저장 계약을 따른다. Store는 blocking SQLite adapter이며 UI thread에서 호출하지 않는다. schema v1은 settings, library_entries, schema_migrations이며 공유 변경은 주 담당이 통합한다. OS Documents 조회는 directories의 native API를 사용하고 조회만으로 폴더를 생성하지 않는다. 스캔은 디렉터리 iterator 최대 128개, 페이지 최대 256행으로 제한한다. symlink와 partial을 제외하며 전체 스캔 성공 뒤 transaction을 commit한다. unsupported non-Unicode 경로는 오류로 이전 관측을 유지한다. notify 8.2 native recursive watcher는 이벤트를 두 atomic bool(dirty/error)에 합쳐 앱 event queue가 커지지 않는다. 파일 Access 이벤트는 스캔 자체가 다시 스캔을 유발하지 않도록 무시한다. backend 오류는 고정 크기 marker로 보존하며 다시 스캔하도록 dirty를 설정한다. Drop은 소유 backend를 해제한다. debounce/재연결, UI 연결, file ID rename 추정은 통합 담당 범위다.

## 인수 조건

임시 폴더에서 Unicode 추가/삭제, 접근 불가 루트의 기존 행 보존, bounded paging, symlink 제외, depth 실패 rollback, migration 재개와 settings 유지. 사용자 Documents와 영상은 테스트에 사용하지 않는다. Watcher missing root, 100,000 이벤트 coalescing/error acknowledgement와 Linux 임시 파일 write 2초 deadline 테스트를 추가한다.

## 검증과 handoff

[notify 8.2 공식 watcher 계약](https://docs.rs/notify/8.2.0/notify/trait.Watcher.html)을 확인했다. Native watcher는 OS watch handle과 thread 비용이 있고 backend 내부 buffering까지 앱 atomic coalescing이 제한한다고 주장하지 않는다. 루트 rename/removal 뒤 watch 재생성은 통합 담당이 처리하며 사용자 새로고침은 계속 제공한다. 테스트는 소스에 포함했으며 workspace 연결과 실행은 주 담당이 수행한다. 권한 거절 동작은 transaction rollback 구조로 보존하며 root 사용자 환경에서 chmod 실패 주입을 실제 검증했다고 주장하지 않는다. Windows/macOS OS Documents 실동작 미검증. SQLite bundled는 시스템 SQLite 의존성을 없애지만 binary build 비용을 추가하며 정확한 측정은 통합 build에서 기록한다.

# S03: 라이브러리 scan·paging·idle 비용 읽기 감사

- 상태: done
- 분류: audit
- 우선순위: P0
- 요구사항: R03, R01, R13
- 담당/수정 범위: diagnostic_cache; 이 문서만. 제품 Rust/Cargo/scripts 읽기 전용.
- 선행 작업: 기존 library reconcile/CPU panel; root loaded-cost 원시 collector 종료.

## 사용자 결과

전체 폴더를 메모리 Vec로 보관하는지, 페이지/이벤트 큐 상한과 남은 SQLite·idle 비용 위험을 현재 코드로 판별한다. 감사 완료이며 개선 구현이나 성능 gate 완료가 아니다.

## 계약과 범위

README/PRODUCT/CONTEXT/R03·금지사항과 기존 scan 취소/상태 복구 계약을 기준으로 `library.rs`, `store.rs`, `watch.rs`, `accessible_library.rs`, `accessible_metrics.rs`, `accessible.rs`를 읽었다. native/빌드/새 cost 시험은 실행하지 않았다. root 원시 `windows-library-loaded-cost-20261004-resume/gui/result.json`을 읽기 확인했다.

## 인수 조건과 관측

- **전체dir Vec 없음:** `library.rs:118–165`는 열린 ReadDir 스택으로 DFS하고 깊이128에서 실패한다. 파일별 INSERT를 SQLite TEMP `library_observations`에 수행한다. Rust 측 대기 항목은 깊이에 비례하며 전체 파일 목록을 collect하지 않는다. TEMP 테이블/인덱스의 DB 저장량은 파일 수에 비례한다.
- **page/queue bounded:** `library_page:209`는 keyset `path>? ORDER BY path LIMIT` 및 엔진256 상한이고 CPU PAGE_SIZE128이다. command/result 채널 각각1, snapshot send는20ms retry+stop 확인; watcher는 dirty/error AtomicBool로 coalesce한다. UI `show_rows`는 실제 가시 row만 렌더링한다. 페이지 이동은 앞 페이지를 누적하지 않는다.
- **SQLite 단일 statement 전체 materialization은 Rust에서 없음:** INSERT 한 행씩, reconciliation SQL은 SQLite 내부 실행이다. Store는 busy timeout5초만 설정하며 temp/cache/mmap 예산 및 SQLite VM progress handler를 명시하지 않는다. bundled rusqlite 구성은 sqlcipher가 아니므로 build.rs의 TEMP_STORE=2 구문을 현재 설정이라고 오인하지 않는다. bundled sqlite 기본 TEMP_STORE1이지만 실제 Windows PRAGMA/cache 비용은 이번 audit에서 실행 확인하지 않았다.
- **실질 개선 1—paging 재scan:** `accessible_library.rs:374` Next도 `scan→reconcile_library_with_cancel→library_page`여서 매 페이지 클릭이 전체root 스캔/transaction을 반복한다. 검증된 최신 snapshot의 page-only query와 Refresh/watch reconciliation을 분리하면 같은128 keyset 계약을 유지하면서 반복scan 비용을 줄일 수 있다. 원시 stale·rename/delete 조건의 페이지 정책을 먼저 정해야 한다.
- **실질 개선 2—cancel 지연:** 파일 스캔 loop 및 publication 전/commit 전에는 cancel을 확인하지만 `library.rs:172–201` 대규모 UPDATE/INSERT SELECT/DELETE 각 VM 수행 중에는 확인하지 않는다. 같은 thread의 atomic cancel은 SQL 종료까지 반영되지 않을 수 있다. bounded VM progress/interrupt 계약을 도입하고 취소 시 transaction rollback·이전 관측 불변을 회귀로 검증할 필요가 있다. 현재 UI stop acknowledgement 자체를 무한이라고 단정하지 않는다.
- **실질 개선 3—row 바이트 상한:** page의 `row.get<String>` path/file_identity에 SQL byte 제한이나 엄격 token 검증이 없다. 정상 filesystem 경로는 OS 제약을 받지만 손상/수정된 DB의 한 거대 TEXT가128행 상한을 우회할 수 있다. SQL CASE/length(CAST AS BLOB) 선제 제한 후 finite path/identity 검사와 실패 시 이전 UI rows 보존 회귀가 필요하다.
- **watch 반복 비용:** worker recv_timeout100ms 뒤 `watch.poll`은 실제root metadata/identity를 읽는다. `WatchSession`은 access-only hints를 무시하여 자기 read가 무한 rescan을 유발하는 흔한 경로를 막지만 외부 변경 폭주에는 각 coalesced dirty마다 full scan할 수 있고 명시 debounce/rate limit은 없다. 이벤트 수만큼 큐가 늘지는 않는다. native hint 없어도30초마다 authoritative full reconcile이 발생한다.

## idle 실패 원시와 다음 인수

root actual 3rounds는 WS84,668,416/79,466,496/79,773,696 bytes, CPU0.3115%/1.08394%/1.24777%였고 round2/3 idle<1% budget 실패다. 각 원 owned Job app1/natural exit0/desktop close를 보존한다. host quiet/전체 host 부하는 확인되지 않았으며 원인을 단정하지 않는다.

idle start는8.29–8.51초이고10초 관측이라 정상30초 periodic rescan이 이 구간을 직접 설명하지 않는다. 첫 watch attach dirty의 추가 스캔이 관측 중인지 원시 `scan_inflight_directly_observed=false`여서 확정하지 못한다. `accessible_metrics.rs:142`는 매1초 결과 변화와 무관하게 ctx.request_repaint를 호출하며 모든 탭에서 metrics.poll한다. library 자체 pending/closewait 때100ms repaint 및 실제 download worker 때250ms repaint는 조건부다. 따라서 (a) 원 scan 시작/끝 계측, (b) watch root-check 횟수, (c) metrics tick/실제paint count를 같은 bounded 관측 구간에서 분리하는 것이 다음 최소 회귀다. budget/timeout을 완화하지 않는다.

필요 후속은 root가 정한 독립 수직 작업이며 본 audit는 source 변경·새 테스트 실행·Windows 성능 통과를 주장하지 않는다.

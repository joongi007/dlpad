# S01: 목록 다음 페이지에서 전체 재스캔 제거

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: root crates/engine/src/library.rs·crates/app/src/accessible_library.rs·CONTEXT/ADR0002·검증/공통 문서; diagnostic_cache /tmp 테스트 초안만
- 선행 작업: worker CPU 원귀속·현재 bounded128 DB keyset paging

## 사용자 결과

다음128개 페이지는 마지막으로 commit한 목록에서 읽어 불필요한 전체폴더 재스캔을 줄인다. 시작·수동 새로고침·위치 변경·파일감시의 전체freshscan은 유지한다.

## 계약과 범위

페이지 조회와 재확인을 분리하며 root/data분리·원DB·cancel·단일worker/channel1·128행상한과 serial ACK를 유지한다. 새 의존성/schema/lock 변경 없음. 주기 idle CPU 예산 해결과 별개이다.

## 인수 조건

128/128/4 페이지의 누락·중복 없음, 페이지 읽기는 원DB 관측을 변경하지 않음, 외부 추가는 후속Refresh로 반영, 취소·잘못된root·corruptDB 실패 보존. 최신 native회귀/full검증 통과와 미검증 실제 GUI 구분.

## 검증과 handoff

현재 전체workspace가 동결 상태로 실행 중이므로 제품 구현은 terminal 뒤 진행한다. 소스와 새 API는 root만 수정한다.

### 첫 red→green과 독립 검토

Windows release 원140행 회귀는 기존scan경로에서 변경14B/추가13번째row로 snapshot assertion FAIL(exit101)을 재현했다. read_page 분리후 두앱의 accessible_library 회귀는exit0이다. 현재추가260행전체키셋/Stop·DB미생성/corruptDB원bytes/겹치는root4회귀와fixture포함Clippy를 실행한다. read_page는Store::open을사용하므로 파일권한수준의read-only DB라는의미가아니며, 목록관측을재조정하지않는다는뜻이다. 단일OS/SQLite호출의즉시중단을보장하지않는다. 독립검토에서workerACK순서의구체회귀는발견되지않았다. 원전체검증과GUI는후속이다.

### DB 미생성/미migration 보강

없는DB를Next가새로만드는원실패를Windows CPU target에서exit101로재현했다. root는Store::read_library_page_readonly를추가하고기존boundedpagequery를재사용했다. READ_ONLY/NO_MUTEX/NOFOLLOW 및기존5초busybound를사용하며from_connection/migration을호출하지않는다. 앱root/data 검증은DB열기와분리하고일반directory/Windowsreparse root를거절한다. missing/0B/legacy/corrupt·exclusiveBusy보존회귀를추가해native검증을이어간다. 잠금중즉시cancel이나WAL/hotjournal sidecar전체불변은주장하지않는다.

### 최종 scoped 인수 (2026-10-06)

현재두앱19개페이지회귀·enginereadonly2회귀와fixture포함Clippy가PASS다. 최신Windows CPU 전체fmt/clippy/test/release 네명령과541소스SHA를root검증했고실행파일8f21c516…/7895040B(+2048B, 의존성/lock/schema변경없음)를확인했다. 현재binary격리GUI는Next 실제media-00128/첫페이지부재·추가/rename/delete후Refresh·invalidroot원행보존·자연WM_CLOSE/Job0/private/input유지를PASS했다. root는page2PNG를직접보았고UIA와readonlyDB원cursor media-00127 뒤정확128행media-00128..00255의ID/크기/무결성을대조했다. fixture의의도된rename/delete/add를제외한원payload보존은runner에서검증했다.

명령: native cargo test -p dlpad --release --locked --features cpu accessible_library::tests (두후보각19PASS), cargo test -p dlpad-engine --release --locked readonly_page_ (2PASS), fixture포함cargo clippy --workspace --all-targets --locked --features cpu,library-scan-closewait-fixture,library-scan-profile-fixture -- -D warnings; python3 scripts/windows_current_workspace_probe.py <freshout> --execute --seconds900 (fmt/clippy/test/release4PASS); 기존windows_library_loaded_native_probe.py 최신SHA/collector로실행(PASS).

[전체](../validation/evidence/windows-library-page-readonly-workspace-20261005-continue/root-review.json), [실제GUI/DB](../validation/evidence/windows-page-readonly-loaded-gui-20261005-continue/root-review.json), [readonly회귀](../validation/evidence/windows-library-page-readonly-green-20261005-continue/root-review.json). 이수직작업은완료지만idleCPU·watchonly/CloseWait·물리IME/reader·다른OS/전체P0는별도인수이다. 단일OS/SQLite호출중강제중단과WAL/hotjournal sidecar전체불변은미검증이다.

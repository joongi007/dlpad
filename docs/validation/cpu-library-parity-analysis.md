# CPU 라이브러리 화면 공유 엔진 계약

2026-10-03, 읽기 전용 분석. 기존 Iced 실제 Library 화면은 저장root input·Apply/Default·Refresh·Next와 filename/size/local row다. 10000-row viewport는 fixture 전용이며 실제데이터화면과 혼동하지 않는다. 현재 CPU Candidate에는 라이브러리 목록이 없다.

기존 engine 재사용 계약:

| 동작 | 공유 API | 정책 |
| --- | --- | --- |
| OS 기본위치 탐색 | library::default_library_root() | 탐색만, 사용자폴더 생성없음 |
| 명시저장위치 준비 | library::create_library_root(Path) | 명시 Apply 후 worker에서만 실행 |
| 관측정합성 | Store::reconcile_library(root) | 전체성공에만commit, symlink/partial제외, nativeID유일rename, 오류시기존DB보존 |
| 목록page | Store::library_page(root,after,128) | keyset 최대256, UI상한128, Next는현재tailpath |
| 변경관측 | watch::WatchSession::new(root)/poll() | fixed atomic dirty/error, 재시도spacing, periodic authoritative reconciliation |
| 여유용량 | metrics::volume_free_bytes(root) | 실패/unknown은수치를숨김 |

Iced scan_page는 root/data canonical overlap 양방향거절과 library.sqlite Store를사용한다. 새 후보도 이조건과startup recovery gate를유지해야하며 전체scan을UIthread에서하지않는다. Store의library_page에는썸네일/검색/metadata/open/delete API가없으므로 이화면parity에그기능을invent하지않는다. DBscan이같은Store파일에접근하므로 download와SQLitebusy실패를UI에보여주고 무한retry하지않는다.

독립 파일 제안: 신규 crates/app/src/accessible_library.rs가 LibraryPanel/단일ownedworker/WatchSession을소유한다. root는 accessible.rs에panel field/tab/action routing만연결하고중앙catalog c::SAVE_ROOT/APPLY_ROOT/DEFAULT_ROOT/REFRESH/NEXT/LOCAL/errors를재사용한다. Candidate 다운로드 outputroot와Library active root는명시Apply에서한번snapshot공유; 실행중root변경은막는다. Engine/Cargo/schema변경필요없음.

제안API `LibraryPanel::new(data:PathBuf,root:PathBuf)`, `show(&mutself,ui:&mutegui::Ui,can_change_root:bool)->Option<PathBuf>`, `poll(&mutself,ctx:&egui::Context)`, `busy()/request_stop()/join_finished()`; 정확공유계약은구현시root합의한다. I/O·canonicalization·rootcreation·Store·watchpoll은단일worker에서수행한다. UI→worker에는capacity1명령과latest refresh bit/checkedroot generation, worker→UI에는최대128row snapshot1개 및boundederror를둔다. page결과오래된generation은폐기한다. root변경성공전기존entries/root/cursor보존, 실패scan은기존entries유지; refresh가Next와겹치면refresh를한번coalesce해첫page로재관측한다. Drop/StopExit는worker중scan cancellation경계가없는현재engineAPI를즉시중단가능하다고주장하지않고phase/report를root와합의한다.

인수: synthetic temp10000files에서 DBcount10000·UI128row 상한·Next 중복 없는 page, 외부 rename/add/delete 실제 관측, disconnect/depth/SQLite 실패 시 이전 목록 보존, root/data overlap 및 symlink 거절, dirty burst capacity, rootchange 오래된 결과 폐기, keyboard/AccessKit 이름/읽기, idleCPU/RSS 및 StopExit 실제 검증. 한국어 catalog와 실제 OS private 화면을 사용하고 사용자 Documents에는 접근하지 않는다. 위 내용은 미구현 계획이며 P0 완료 근거가 아니다.

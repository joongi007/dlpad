# S01: CPU 파일 목록과 종료 대기 연결

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: 주 담당 accessible.rs, accessible_main.rs, accessible_cpu_main.rs, 본 문서; library_identity accessible_library.rs 독점
- 선행 작업: CPU hidden-init 실제 GUI, library panel 및 cooperative scan 취소

## 사용자 결과

파일 목록 탭에서 선택한 저장 폴더를 관측하고 명시 적용한 경로를 다운로드에도 사용한다. 종료는 소유 다운로드와 스캔의 중단 확인 후 완료한다.

## 계약과 범위

LibraryPanel lazy 생성, 초기 사용자 Documents 생성 없음. 긴 I/O는 panel 단일worker. 실행 중 저장루트 변경 금지, 종료대기중새작업금지, UI thread unfinished join 금지. 기존 다운로드/설정 좌표 높이 유지.

## 인수 조건

module 연결·fmt/clippy/시험/release, 파일목록128상한·Next·외부변경·루트실패보존·스캔취소·workerpanicack·CloseWait 검증. 실제 GUI는 새binaryhash에 연결하며 기존hidden-init증거로 확대하지 않는다.

## 검증과 handoff

구현 전 계약. 실제 GUI/3OS 결과는 후속 기록한다.

최초공통Clippy는 panel snapshot nestedif 경고로실패했고 owner가수정했다. 저장루트 TextEdit의 labelled_by도연결했다. root는panel초기생성시ctx등록/루트Apply중새다운로드차단/CloseWait중새작업방지를보강했다. 현재 fmt/CPUalltargetClippy exit0, CPUentry20tests pass(다운로드6/panel9/host5). 실제GUI와release는진행중이며완료로승격하지않는다.

Linux 통합 release 완료:39.68초/exit0, SHA256 `185442b2dd209aa8e2ce43251c5ef695c835fe1c60c33c4de3bfb0d15d7971ad`, 10359544B. 새 의존성이나 Cargo/DBschema 변경 없음. hidden-init 후보10172280B보다187264B 증가했다. Windows native CPU clippy/test/release는7.59/17.47/31.13초exit0,153passed/1ignored. 주 담당은 현재source236개SHA를 독립 대조해 불일치0을 확인했다. [Windows 증거](../validation/windows-cpu-library/result.json). 실제GUI·reader·비용은 별도검증중이며CLI성공으로대체하지않는다.

최종 현재 source 공통 검증: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`, `cargo build --workspace --release --locked` 모두exit0. 기본workspace193passed/0failed/5ignored. CPUworkspace `cargo test --workspace --features dlpad/cpu --locked`228passed/0failed/5ignored. release38.72초. 문서196개검사통과. root는실제stdout에서시험개수를집계했다. 실제GUI인수전작업in-progress유지.

현재 binary185442...의 별도 대기비용은 ownedXvfb/privatebus에서RSS50748KiB(49.6MiB)/idleCPU0%10.290초였다. [직접측정](../validation/accessible-performance/cpu-library/result.json). 파일목록tab을열지않은baseline이며10000파일scan비용과구분한다. Linux첫GUI는10000파일/외부변경/invalidroot보존을관측했으나ownedDBlock중CloseWait가false여서종료인수로사용하지않는다. Windows새hashCancel/빈Librarytab는관측됐고StopExit는입력경계실패로별도대조중이다. 초기실패를보존하며GUI전부PASS로승격하지않는다.

새Windows38e7 후보의StopExit는문자별선언VK/scan-code를맞춘독립harness대조로PASS했다. 제품source수정없이입력경계를교정한결과이며command+A/실제modifier기전을확정하지않는다. root가 matchedkeys result.json과SQLite를직접읽어ownedworkerJob/startFILETIME/cleanup·primaryexit0·inputdesktop불변 및Cancelled/Attempt1/artifact0/integrityok를확인했다. [검증 문서](../validation/windows-cpu-library-validation.md). Linux corrected pagination은관측됐으나busy확인실패raw를유지하고CloseWait최종시험을기다린다.

## 실제 GUI 통합 인수 완료 범위

boundedLinux파일목록 수직기능을완료한다. [목록 GUI](../validation/cpu-library-gui-validation.md)의185442 snapshot에서10000파일/첫128개및다음128개/외부변경수렴/invalidroot보존/Tabfocus를확인했다. root는rawsummary의00000/00127/00128/00255경계와페이지2끝PNG를직접대조했다. [CloseWait](../validation/cpu-library-closewait-validation.md)의새8fb4snapshot은실제textentry상태로busy를확인하고닫기prompt의StopExitaction후DBlock동안앱생존,해제뒤116ms exit0/ownedGUI회수를보였다. root가entry-state/result.json을직접읽어조건을확인했다. 이전buttonEnabled proxy실패는AccessKit버튼상태의부적절한사용이었고raw를보존한다. 내부stopack메모리를직접읽은증거가아닌사용자행동과외부종료관측범위다.

Windows38e7의빈Librarytab/실제다운로드Cancel/StopExit도전용증거를직접검토했다. 새S10/fixturefix는CLI167tests만현재확인된범위이며새WindowsS10 GUI로확대하지않는다. actualDocuments/redirectedDocuments/다른OS목록전체/reader전체/전체P0/최종S00게이트는남는다.

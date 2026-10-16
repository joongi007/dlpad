# S01: CPU 라이브러리 panel

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R03, R08, S01
- 담당/수정 범위: library_identity — 신규 crates/app/src/accessible_library.rs 및 전용 docs
- 선행 작업: 기존 engine library/watch, root 후속 Candidate/module/CloseWait 연결

## 사용자 결과

CPU 후보에서 저장 위치 선택·128개 목록·Refresh·Next·native watch 갱신을 제공한다.

## 계약과 범위

LibraryPanel new/show/poll/busy/request_stop/stopped 계약. single worker, command/result channel 상한1, worker 소유 WatchSession, UI thread 긴I/O/Join/Drop blocking 금지. 제품 기존파일/Cargo/catalog/engine 수정하지 않는다. Apply 성공 전 기존root/rows 보존, root/data overlap 차단. Stop은 새명령을 막고 진행 scan 끝에ack한다.

## 인수 조건

합성 tempfile 기반 boundedpage·root/dataoverlap·실패목록보존·root변경·stopack 회귀. 실제GUI/reader/10000성능은root 통합 후 별도검증. 즉시scan취소를 주장하지 않는다.

## 검증과 handoff

신규 panel/협조취소/CloseWait 연결 완료. root Linux fmt/CPU Clippy/default193시험/CPU242시험/fullrelease 통과, 생산 CPU SHA8fb4a67c27ff89fd883ff6fddaa9136a5fc36c62d74c049665543a513ab06986가test-onlyfixture보강전후동일했다. panel9회귀는Linux 및WindowsCPU에서실행해통과했다.

WindowsGNUCPU전체Clippy/test/release는최신S10통합소스에서도exit0/167passed1ignored. compiledinput237개 및runner전후동일, raw는docs/validation/windows-cpu-diagnostics-fixturefix/에보존했다. 최초fixture충돌실패도별도로보존한다.

실제Linux boundedpanelGUI는docs/validation/cpu-library-gui-validation.md: 합성10000file DBcount/128keyset/virtualrows/Next/addrename삭제수렴/invalidroot기존선택보존/Tabfocus 확인. docs/validation/cpu-library-closewait-validation.md의새SHA8fb4 actualentry-state busy→namedStopExit→lock보유중생존→해제후exit0으로narrowCloseWait gate 충족. 이전collector 실패를보존하고새검증으로과거binary를확대하지않는다.

완료범위는Linux boundedlibrarypanel과공통CLI회귀다. R03전체기능·전체P0·전체reader사용성·Windows/macOSlibraryGUI·3OS인수를완료라고표현하지않는다. GUI수집원시한계와exactbinaryscope는전용문서를따른다.

# S03: CPU 작업 이력과 검증된 재개 요청

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R03, R08, S03
- 담당/수정 범위: diagnostic_cache; 신규 crates/app/src/accessible_history.rs 및 전용 validation 문서
- 선행 작업: root CPU entry integration, engine DownloadRepository history/stored_options

## 사용자 결과

최대128개 이력 페이지의 상태/시도와 검증된 같은 Job 재개를 제공한다. 완료·취소·invalid snapshot 재개를 거절한다. 도구/실패 정보는 root와 bounded engine API를 조정해 표시한다.

## 계약과 범위

HistoryPanel::new(data),show(ui,enabled),poll(),busy(),request_stop(),stopped(),take_resume()->Option<String>. 단일 owned worker가DB I/O,command/result 각capacity1,UI는최대128행. 새로운재개는worker가현재row와typed snapshot을검증한뒤id한개만root에게전달하고실제실행은root담당. root entry/Cargo/catalog/schema/engine수정금지. UI context는show에서등록,worker완료wake,stopack는DB자원unwind뒤. 새로운label이필요하면root요청. 사용자desktop시험금지. 강제종료guardian은rootexisting계약.

## 인수 조건

bounded paging/실패시이전행보존/같은id만typed snapshot검증/terminalinvalid snapshotfailclosed/stopack/workerpanicack. UIthreadI/O 없음,완료channel막힘에서stop보장. 도구/실패data는bounded+redacted engine공유계약확정후연결. root통합검증으로실제재개동작확인,단위시험만으로GUI완료주장금지.

## 검증과 handoff

공유API제안전달. source범위독점/하위agent없음. 기존GUI증거원시보존.

## 실제 완료 결과

root 통합 CPU59tests/alltargetclippy/boundedAPI3tests/release PASS 보고. actualGUI SHA522e2304fc38618df696cfca1f126e08bc77634f48ac1db631d3c797fe8f8cbd에서 missing/corrupt/원snapshot재개·128페이지·종료대기 PASS. 최초physicalPause timeout raw보존, actualPauseaction 계측 대조로 같은후보 재검증. docs/validation/cpu-history-validation.md 및 cpu-history/summary.json. 모든소유GUI종료,source/Cargo추가변경없음. rawfailure상세journal미저장/다른OS·전체reader는미검증.

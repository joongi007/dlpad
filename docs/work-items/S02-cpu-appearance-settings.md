# S02: CPU 테마·배율 설정

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R06, R08, S02
- 담당/수정 범위: library_identity — 신규 crates/app/src/accessible_settings.rs 및 전용 docs
- 선행 작업: root공유읽기전용boundedStore API/catalog/entry/host/Candidate 연결

## 사용자 결과

CPU후보의System/Light/Dark테마를첫frame전읽고선택을유지하며75–200% 배율을25step으로선택한다.

## 계약과 범위

AppearanceSettings load_initial/show/poll/busy/request_stop/stopped. startup만window/eventloop전I/O,저장singleworker/channel1/latesttheme/revision stale결과차단. 기존theme.sqlite serde문자열및legacytheme.json4096bytes호환. 실패System고정오류·기존bytes보존. 제품기존file/engine/Cargo/schema/catalog수정금지,root읽기API/공유연결담당. apply_to_context startup API와session-only배율,active+latestdrain후stopack를승인받았다.

## 인수 조건

privatefixture malformed/oversize/symlink/permission/DBbusy 원본보존과latesttheme저장순서/stopack/panic. firstdisplay실제비용·OSSystem변경·theme/scale GUI/IMEcoordinates는root후속actual검증. Store5초timeout을빠른첫paint완료로추정하지않는다.

## 검증과 handoff

신규모듈및11회귀(Unix2조건부)구현/ownrustfmt exit0/sourcefreeze. root readonlysetting API 및APPEARANCE_FAILED 제공완료. docs/validation/cpu-appearance-settings.md 기록. root공유연결완료, 실제LinuxCPUClippy/38tests(appearance11) 및WindowsCPU전체3검사188pass1ignored(appearance9양binary) 통과. actualGUI/SystemOSevent/firstpaintcost/3OS는후속이라in-progress유지.

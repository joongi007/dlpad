# S02: CPU 설정 GUI 검증 준비

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R06, R11, S02
- 담당/수정 범위: diagnostic_cache; 신규 scripts/cpu_settings_probe.py, 전용 validation/evidence
- 선행 작업: root accessible_settings/CPU integration 및 새 binary READY

## 사용자 결과

System/Light/Dark 선택의 실제 표시와 SQLite 저장·닫기 후 재실행 첫 native frame을 확인한다. 200%에서 이름으로 좌표를 재조회하고 keyboard focus/한글 field 접근을 확인한다.

## 계약과 범위

제품/Cargo/기존 probe/source 수정 금지. root READY 전 GUI 실행 금지. owned Xvfb/private bus/temp data/media/XDG만 사용하며 사용자 desktop/audio/clipboard 금지. 외부 OS 테마 설정을 바꾸지 않는다. System의 실제 OS 변경 실시간 반영은 별도 gate로 남긴다. 화면 배율이 영구 저장되지 않는 현 계약을 테마 persistence와 구분한다.

## 인수 조건

AT-SPI named combo/action 선택, 저장 theme SQLite value, raw PNG/first-painted pixels, 닫기 exit0 후 동일data 재실행 first observed painted frame theme를 비교한다. 선택된 밝게/어둡게와 native background pixel을 비교하고 OS System preference는 관측값만 기록한다. scale200% 실제 선택 후 positive native resize 1600×1200,800×600logical 조건에서 bounds와 Tab focus/한글 field GetText 확인. 한국어 문자열이 존재하는 것과 실제 물리 한글 IME 검증은 구분한다. 모든 child/temp cleanup 및 시작/끝SHA.

## 검증과 handoff

준비 단계. GUI 미실행. READY SHA를 필수 인자로 받으며 mismatch는 실행 전에 거절한다. --validate-only는 자식/GUI를 시작하지 않는다.

## 최초 후보 결과

SHA52b55의 테마3개 SQLite 저장/닫기 exit0/재실행 painted 관측은 성공. 초기 collector는 static label과 ComboBox의 동일 Name을 혼동하여 role29를 제외하도록 수정했다. action popup 잔류는 raw PNG 보존, root 수정 예정. 최종 abc 실패는 재실행 후 owned window keyboard focus를 명시 확보하지 않은 collector 계약 누락을 확인했다. 기존 raw를 유지하고 새 READY까지 GUI 중단했다. 첫 presentation이나 물리 한글 IME 성공으로 확대하지 않는다.

## 최종 후보 결과

SHA330096e38bedd88fede1751dd84ec11ac518c00c9a2a47ac1a84b201617aaa1b actual GUI exit0. popup Escape 없이닫힘/3테마 저장·재실행 first observed painted frame/200% entry abc·ShiftTab scale·Tab entry/save-in-flight lockStopExit·해제 뒤Light저장 확인. 내부workerack/실제한글IME/SystemOS변경/다른OS는 별도 gate. docs/validation/cpu-settings-validation.md, cpu-settings/summary.json. 소유GUI종료/root해제통보.

## 후속32px/이동IME gate

root가 Light style의19px 높이를 발견해 all_styles_mut 수정 중이다. 담당 범위에 기존 accessible_ime_probe.py의 named role79 필드 좌표 발견/전용 증거를 추가한다. 새 READY 전 GUI 금지, 기존330096/52b55 raw 보존. Theme별 실제 ComboBox extents h>=32 assert 추가, 새후보 fullsettings 및 privateFcitx 실제물리키 시험 수행.

## styles32 최종 검증

SHAf052 actual settingsfull exit0: themes별ComboBox32px/200%64px,기존인수조건전체PASS. 이동Settings 한글entry(8,201,944,21)에서 privateFcitx물리키commit/cancel/Latin/selection 성공, inlinepreedit PNG 직접판정. rawIME exit1/preeditfalse는수정하지않고 별도reviewJSON으로분리했다. rootPNG리뷰요청완료. 두GUI/자식/temp모두종료,source재빌드/성능시험해제통보.

root가preeditPNG독립확인. 남은gate:field높이21logicalpx→DESIGN32min_size후속(typedoptions통합root담당), startupbusytimeout5초첫표시실제GUI/SystemOSevent미측정. raw자동판정미수정/조건부LinuxIME관측.

## input32 후속 준비

root typed options/IME/Library min_size32 후보 build READY 대기. 담당은 settings·실제IME 기존 전용harness만 수정. settings100%fixture entry32px,200%64px assert 추가. IME namedrole79 자동좌표유지. Library 저장 폴더 field는 actualrole79 GetExtents32px 여부만새prefix에서관측하는 계획이며 옵션harness는guardian_fault 담당. 제품/Cargo/다른harness수정없음;새READY전GUI금지.

## input32 최종 증거

SHA2459ba126c6bcc13470b470c8a384d9d8ecef779893fd832411db0a78932f559 actual settings exit0/IME raw exit1(원시보존+시각review분리). Settings100%32/200%64 및 Libraryroot32 실제확인;기존동작재통과. IMEentry32px 실물리키commit/cancel/Latin/replace true, inlinepreedit직접확인/root검토요청. 두GUI정리/성능슬롯해제,전용doc/evidence업데이트.

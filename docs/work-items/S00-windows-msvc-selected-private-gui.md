# S00: MSVC 설치 선택본의 격리 GUI 인수

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R06, R08, R11
- 담당/수정 범위: root — 도구 설정·collector 통합·native 실행·증거/문서; diagnostic_cache — /tmp collector 초안; 독립 검토는 실행 후 별도 배정
- 선행 작업: S07-windows-msvc-workspace-experiment의 실제 전체 검사와 MSVC release 산출물. 실행은 선행 PASS 이후에만 한다.

## 사용자 결과

새 MSVC 앱도 명시 선택한 설치 복사본에서 시작하고 테마 선택을 저장하며 재시작할 수 있는지 확인한다.

## 계약과 범위

기존 GNU target와 증거는 보존한다. 새 MSVC target의 앱과 fixture를 사용하고 새 합성 데이터 폴더·private desktop에서만 시험한다. 담당 초안은 /tmp에 작성하며 프로젝트 파일 수정은 root가 통합한다. 기존 scripts/windows_selected_gui_collect.py의 증거 계약을 MSVC 경로에 맞게 확장할지 별도 collector로 분리할지는 실제 proof 구조를 검토해 결정한다. schema/Cargo.lock/dependency 변경은 없다. 비용·실제 OS IME·cleanOS·서명·제품 installer는 이 짧은 실행 시험으로 완료하지 않는다.

별도 scripts/windows_msvc_selected_gui_collect.py, windows_msvc_launch_selected_cli_collect.py, windows_msvc_gui_producer_build.py를 통합했다. 기존 GNU collector의 역사적 증거 계약과 target은 유지한다. 공유 current whole proof guard는 scripts/windows_msvc_workspace_probe.py에 두며 source before/after·네 actual operation·exit0/collector 오류없음·private/input·locked MSVCtarget을 대조한다. six negative proof case(exit/source/collector/target/missing/unlocked)를 실제 거절했다. crates/engine/examples/windows_selected_gui_fixture.rs의 manifest target은 cfg(target_env)에 맞추고 새 전체4검사를 실행했다. 같은 파일의 agent 동시 수정은 없다.

## 인수 조건

현재 source map과 MSVC CPU SHA/bytes의 실제 설치 복사본 일치, private GUI 시작·System→Light 선택·원 선택 경로 재시작 Light 표시, SQLite readonly integrity/theme와 원 파일/native ID 보존, 자연 Job0·강제 종료 없음·사용자 input desktop 불변. root가 원 로그와 화면을 확인하고 독립 검토를 보관한다.

## 검증과 handoff

collector 준비 중이다. 현재 MSVC 전체 테스트는 통과했으며 release 빌드가 진행 중이다. 실행 전 결과를 완료로 쓰지 않는다.

현재 source576 새 전체4검사와 producer 실제 MSVC build가 PASS다. producer SHA175de9af5bbbc959a5286f1c6c8a5fa56e367ae7a6db5da5339276c91d5ceabb/726,528B, [원 build 증거](../validation/evidence/windows-msvc-gui-producer-20261006-continue/result.json). GUI actual collector는 fresh windows-msvc-selected-gui-real-20261006-continue에서 실행 중이며 root/독립 화면·DB 인수는 아직 미완료다.

실제 [MSVC 설치본 GUI](../validation/evidence/windows-msvc-selected-gui-real-20261006-continue/result.json)는 6native 명령·두 자연 GUI 종료·MSVC manifest·원 source/pins·canonical 관계/nativeID10 보존을 통과했다. root와 독립 담당이 selected/restart PNG를 직접 확인하고 readonly SQLite integrity=ok, theme=Light, DB SHA 불변을 대조했다. collector의 REVIEW_REQUIRED는 원 기록으로 유지하고 별도 root/independent review에 scoped PASS를 보관했다.

이어 [기본 MSVC CLI 실행](../validation/evidence/windows-msvc-launch-selected-cli-real-20261006-continue/result.json)도 15native 명령·13결과로 통과했다. 네 private GUI의 실행/재시작은 자연0, 취소는 Interrupted, 기한은 TimedOut이며 원 Job 종료·input 불변을 확인했다. 실제 input pins20·보존 파일34·nativeID10·원 설정 Light 유지, parser/다른 등록쌍/foreign 거절을 root와 독립 담당이 확인했다. Natural17 실패 대조는 과거 GNU helper와 현재 MSVC CLI의 교차 호환 시험이며 MSVC-built helper 증거가 아니다. source576·CPU121f02cd…·CLI a3001bd2…·GNU 원 앱/CLI·lock 불변. first presentation·OS 테마 변경·물리 입력·비용·정식 installer·cleanOS/서명 및 전체P0는 이 완료 범위에 포함하지 않는다.

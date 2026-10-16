# S10: 소유 HWND 합성 입력 pacing 실험

- 상태: in-progress
- 분류: test
- 우선순위: P0
- 요구사항: R13, R08, R11
- 담당/수정 범위: root 통합·실행; scripts/windows_s10_report_input_batch_probe.cs, scripts/windows_s10_report_input_batch_native_probe.py 및 본 문서
- 선행 작업: 진단 edit90초 누적 입력 실패, original suite freeze

## 사용자 결과

실제 제품 정책을 바꾸지 않고 진단 GUI 자동 시험의 문자별 대기를 줄여 편집 인수를 완료할 수 있는지 확인한다.

## 계약과 범위

최대8 UTF16 unit/backspace 묶음마다 원 PID/FILETIME/HWND를 확인하고 기존 F24 envelope를 각 문자에 유지한다. clear와 최종 actual readback exact 검사를 그대로 요구한다. 1024 입력 상한·90초 worker·150초 outer·private desktop·원본 보존 조건을 유지한다. 기존 suite의 고정 수집기는 수정하지 않는다. 별도 exe·private desktop·임시 데이터의 실험은 기존 suite와 공유 상태가 없어 병렬 실행하며 성능 측정을 하지 않는다.

## 인수 조건

실제 edit scenario에서 모든 입력값 exact readback과 정제·동일 보고서 ID·DB 보존·정상 종료를 확인한다. 묶음 입력의 성공을 IME·실제 사용자 키보드 인수로 확대하지 않는다. 실패를 timeout 증가나 clipboard fallback으로 우회하지 않는다.

## 검증과 handoff

root는 원 수집기와 사본의 두 입력 루프 차이를 검토했다. 실제 native compile/실행과 root DB/export 검토 전에는 채택 완료가 아니다.

후속 실제 시험 `windows-s10-input-batch8-edit-20261004T140348Z`는 compile와 edit native assertion을 통과하고 worker72,544ms에 반환했다. root는 `S10 public observation [url-1]` 재정제, URL/서명/쿠키 원문 부재, 원 ID/attempt/stage/tools 유지, export SHA·DB Failed/artifact0·원 synthetic source/sentinel 보존과 실제 final PNG를 직접 확인했다. 정상 WM_CLOSE·Job 비움·private desktop 닫힘을 기록했다. 기존 문자별 대기 edit90초 실패는 보존한다. 한 시나리오의 합성 입력 실험은 통과했지만 원 live suite의 수집기는 아직 변경하지 않았고 전체 GUI/IME 인수로 확대하지 않는다.

별도 재현 runner는 기존 scenario runner의 C# source 선택만 묶음 실험 수집기로 바꾸며 각 case의 원 privacy/불변/종료 검사와90초 제한을 유지한다. 원 mpd-remove도 최종 export path 입력에서90초에 도달해 원 실패를 보존하고 묶음 입력의 fresh mpd-remove 실행을 시작했다. 원 suite는 수정하지 않는다.

2026-10-04 재개: 원11case suite는 terminal 결과5PASS/6FAIL이며 입력·조작 누적90초 도달 실패를 보존한다. 묶음 mpd-remove 재시험은 통과했고 root는 최종 attachment0, 원 ID/attempt/stage/exit/tools, DB Failed/artifact0/cookie option 부재, 정상 종료·Job0·private desktop 닫힘과 실제 final PNG를 확인했다. 중간 첨부 export는 보존되지 않아 전이는 native assertion 범위이다. mpd-reattach fresh 묶음 시험을 시작했으며 아직 완료가 아니다.

후속 mpd-reattach, mpd-all, mpd-unsafe, mpd-large fresh 묶음 시험 모두 compile/native PASS. root는 실제 export 동일ID/attempt/stage/exit/tools, DB Failed/artifact0/--cookies 부재, 원 프로세스 종료·Job0·private desktop 닫힘과 실제 final PNG를 확인했다. unsafe/large는 기존 유효 첨부1개를 보존하면서 UnsafeXml/TooLarge omission을 추가한다. 제외 결과 attachment0과 혼동하지 않는다. 각 fresh evidence의 root-review.json에 기록했다. 원11case suite 5PASS/6FAIL은 그대로 보존하며 이후6case 묶음 재검증을 별도 근거로 사용한다. clipboard/IME/최대보고서 stress와 전체S10 인수는 남는다.

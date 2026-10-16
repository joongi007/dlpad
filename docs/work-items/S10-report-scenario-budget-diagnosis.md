# S10: 누적 GUI budget 진단과 독립 scenario 인수

- 상태: in-progress (staging READY; 새 실제 실행 미완료)
- 분류: validation
- 우선순위: P0
- 요구사항: R13, R08
- 담당/수정 범위: diagnostic_cache; `/tmp/windows_s10_report_save_scenarios.cs`, `/tmp/windows_s10_report_save_scenarios_probe.py`, 본 문서만
- 선행 작업: S10-report-all-controls-focus-scroll 실제 제품 통합; windows-s10-all-controls-focus-20261004T131828Z 원시 실패

## 사용자 결과

전체 GUI 인수를 임의 timeout 증가 대신 fresh GUI·fixture별 작은 scenario로 분리한다. 실제 멈춘 작업과 누적 비용을 구분할 step evidence를 남기며 원 실패는 보존한다.

## 재현·원인·계측

원 evidence 파일 timestamp: initialJSON22:19:31.306, initialMD22:19:58.222, log index0 setter22:20:02.770, edit-field 원35char setter22:20:11.952, clear0 성공22:20:14.732,90초 worker deadline result22:20:14.790. 마지막까지 전진했으므로 특정 UIA provider 호출 hang이라고 확정하지 않는다.127~128char path 입력·기존 path backspace·35ms/char와 매action subtree/PNG 비용이 전체 시나리오에 누적됐다. Job0/input desktop unchanged는 유지됐지만 UIA worker가 복귀하기 전 root가 정리해 private_desktop_closed=false이며 전체 인수 실패다. timeout90초는 유지한다.

root의 추가 실제 source 대조: 이전 appdata root/private-cache는 tool 허용 cache root/private-cache 또는 root/data/session-cookies와 맞지 않는다. 앱이 appdata/session-cookies를 덧붙여 실제 metadata fixture64 rejection을 만들었다. staging Main은 root/data로 교정했으며 fixture 코드는 바꾸지 않는다. initial report Metadata exit7 또는 Download exit9, actual calls version+metadata(+download), original synthetic cookie SHA 확인을 인수에 추가했다. 원64 report 저장 관측을 정상 planned metadata failure 증거로 확대하지 않는다.

## 계약과 구현

native C# argv8개: binary binarySHA fixtureTool fixtureSHA freshEvidenceDir metadata-failure|download-failure single|all scenario. scenarios: save/edit/omit-one/omit-all/mpd-attach/mpd-remove/mpd-reattach/mpd-all/mpd-unsafe/mpd-large. 각 호출은 새 process/privateDesktop/Job/root/source/SQLite/report를 생성한다. 초기 JSON 새 export·worker 반환을 모든 case에서 확인하고 report_id/attempt/stage/exit/tools는 이후 JSON export마다 정확히 불변이어야 한다. Markdown에는 동일 reportID가 있어야 한다.

save: 실제JSON/MD 새 저장과 sentinel 충돌. edit:URL 재정제/문장유지/로그수유지. omit-one:nonempty1개감소, omit-all:nonempty초기에서직접0이며 별도case. attach:구조1/privacy. remove/all:actualMPD0 header와fresh0attachments→attach 반환 뒤제거→JSON0. reattach:attach→실제header→개별제거/헤더없음→재첨부→JSON1. unsafe/large:정상attach header확인뒤 각각DTD·1MiB+1 추가→최종JSON originalattachment1+omissions 및모든sourcebytes보존. 기존 create-only export/sourcehash/원 Job 종료/privateDesktop복귀/clipboard0 권한 계약은 유지한다.

bounded checkpoints `s10-steps.json`은 worker/case/field/click/export begin/end의 monotonic elapsed_ms·UTC·고정라벨만 최대256행/64KiB 기록한다. 원90초 worker/150초 outer owned Job budget 유지. 별도 무제한 UIA retry나 새 UIA 호출 thread/Thread.Abort를 도입하지 않는다. provider 호출이 실제 hang이면 begin/end의 마지막 경계와 original host terminal을 읽고 failclosed로 남긴다. fresh scenario 분할의 성공 시간과 raw output을 실제 재검증해야 한다.

Python runner staging은 기존 scripts/windows_s10_report_save_native_probe.py를 읽기 복제하여 required --scenario 하나당 native compile+GUI1회만 수행한다. 기존 project source 경로를 기대하므로 root가 원 scripts 담당권한으로 C#·runner 통합한 뒤 실행한다. 각 case는 다른 fresh outputdir 필수이며 한 case 성공으로 전체를 표시하지 않는다. source/immutable exe 전후 검증·원시실패 보존·root DB/PNG독립검토는 유지한다.

## 인수 조건과 handoff

C# staging SHA256 `263523129cfe5b3e92c3cde26e3ffed1fd8c34744c9bf77d49d934a02c68cee0`; Python staging SHA256 `59f8158471a954cfbf6c3575df3d15dcf32aa3c3363f9cd4eef5db03200e1566`. Python py_compile exit0. C# compile/native/GUI 미실행, Rust/Cargo/project scripts 수정 없음. actual save-inflight CloseWait와 clipboard는 여전히 미검증이며 scenario 분할로 완료하지 않는다. root가 최신 frozen product를 각case90초내 실행하고 actual stage/calls/privacy/count/source/sentinel/DB/Job/desktop를 독립 검토해야 한다.

## 현재 게시본과 호출 호환성

root가 source SHA `9c4c2962213d26c4a4dca1f0c5972c6ea3f02131b006dc7ea20b6a02b11856b4` C#와 `59f8158471a954cfbf6c3575df3d15dcf32aa3c3363f9cd4eef5db03200e1566` Python runner를 실제 프로젝트에 게시했다. staging 호출의 VerifyDiagnostic 여분 logOmission 인수 오류는 이 C#에서 교정됐다. root 실제 C# compile PASS이며 save GUI session16869가 진행 중이다. terminal/인수 결과는 아직 없고 이 기록은 compile과 진행 사실만이다.

현재 재현 명령은 `python3 scripts/windows_s10_report_save_native_probe.py /absolute/fresh/project/docs/validation/evidence/<case> --scenario save`이며 다른case도 각각 fresh output으로 호출한다. runner는 native collector에 정확8개 argv를 전달한다. 기존 standalone7개 argv 호출은 explicit Eight explicit arguments required로 failclosed이며 호환인 것으로 안내하지 않는다. 마지막8번째 scenario를 생략해 암묵적 전체 인수를 실행하지 않는다.

기존 `scripts/windows_s10_report_save_probe.ps1`은 이전 Restricted 실패와 embedded observer 역사 보존용으로 **deprecated**다. 현재 standaloneC#8argv를 호출하는 wrapper가 아니라 자기 embedded CookiePrivateGui.Run을 실행하는 별도 옛전체시나리오이므로 signature가 자동 갱신된다고 주장할 수 없다. 이전 문서의 PS `-File` 호출 예시는 당시 준비 이력이며 현재 실행 권고가 아니다. 이 PS 파일을 동작 가능한 최신 launcher로 소개하거나 Restricted를 바꾸는 수정은 하지 않는다. root가 원 파일을 관리하며 필요하면 별도 deprecated 안내를 게시한다. 시험 정책 변경·Bypass·-Command 재해석은 계속 금지한다.

Markdown invariant는 본문을 JSON으로 parse하지 않는다. 초기 JSON에서 serialize한 immutable dictionary만 parse해 reportID를 얻고 Markdown에 그 ID가 있는지 확인한다. JSON은 ID/attempt/stage/exit/tools 정확 불변 검사를 수행한다. 모든 nativecase의 실제 terminal/root DB/PNG 검토가 남는다.

## 실제 save scenario 누적 실패와 최소 분리

`windows-s10-save-scenario-20261004T133252Z/gui-single` 실제 result는 FAIL이다. Metadata7·actual_fixture_stage_exit_verified=true와 JSON/MD 새 export를 확인했다. steps의 Markdown export end=72606ms, sentinel field begin=72607/end=89188ms, collision click begin=89189ms 뒤 원90초 worker deadline이 도달했다. 이 진행 근거를 provider hang으로 해석하지 않는다. private_desktop_closed=false, owned_job_active_before_cleanup=1, owned_job_terminated=true, owned_job_empty_after_cleanup=true, input_desktop_unchanged=true를 그대로 보존한다. worker Join 실패를 실제 desktop close 성공으로 바꾸지 않는다.

최소 `/tmp` 후속은 `save`를 initialJSON+MD만으로 끝내고 `save-collision`을 fresh initialJSON→sentinel 충돌로 독립 분리한다. 원90초 worker/150초 host budget 및 exactreadback·stage/calls·report 불변·원source/cleanup 인수는 유지한다. 기존35ms/char 입력과 backspace는 이번 최소 패치에서 변경하지 않는다. 후속 실제 전송속도 실험이 필요하면 bounded owned HWND WM_CHAR 방식과 매단계 exact readback으로 검증하며 clipboard/globalinput은 도입하지 않는다.

C# staging `/tmp/windows_s10_report_save_split.cs` SHA256 `7bfc1efe3e44114cc82b66a293a799f47f046f62797ae186fd0b638403882546`; Python `/tmp/windows_s10_report_save_split_probe.py` SHA256 `c9bb9d0f59bbeadcdd0e64b804407ac87478cf842184c9e25ed1c6569478677a`. Python py_compile exit0. Native compile/GUI 실행과 프로젝트Rust/Cargo/scripts 수정 없음. root 게시·실제각freshcase 인수 전 준비 상태다.

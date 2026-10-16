# S07: 명시 선택한 복사본의 격리 실행 인수

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R11, R08
- 담당/수정 범위: root — 기존 private launch collector 재사용·evidence·문서·최종검증; soak_recovery_audit — /tmp 실행계약·완료후독립증거검토
- 선행 작업: S07-windows-active-install-coordinator scoped DONE; 현재CLI전체4검사 PASS. CLI실제시험 종료후 native실행한다.

## 사용자 결과

명시 선택 기록의 원 복사본이 Windows private desktop에서 실제 시작하고 테마 설정을 재시작에 보존하는지 확인한다. 제품 자동 launch를 추가한 것으로 표현하지 않는다.

## 계약과 범위

원 canonical 선택기록·원nativeID·SHA와 CPU proof를 대조한 뒤 기존 windows_app_prepared_launch_probe.cs/windows_settings_private_probe.cs를 재사용한다. 후속 원소유 런처의 읽기 getter와 개발 example을 root가 조정한다. 공유 schema·Cargo 의존성 변경은 없다. native소유Job/원exe file핀을 유지한다. visibledesktop fallback/전역입력/실사용Documents/실제영상0. 프로젝트증거collector와원핀을보존한다.

## 인수 조건

원 선택파일의 volume+128bitID/SHA 전후동일, private 두실행·자연Job0·inputdesktop불변, 실제Light 선택과readonly SQLite integrity/theme 및화면독립대조. 원등록5파일/receipt1/active1/source/notice/sentinel 변형없음. 제품installer/upgrade·서명·physicalIME를이시험으로완료표시하지않는다.

## 검증과 handoff

아직실행전이며root native실행직렬조정. GUI성능60초/8시간시험은별도snapshot이며이기능시험에서소급하지않는다.

root읽기검토 결정: 기존preparedobserver의 원exeFile pin/SHA 전후대조는 CreateProcess가실제로선택한childimage ID의resume전검사를대체하지않는다. 따라서과거helper만재사용해currentactive실행인수로승격하지않는다. [원선택소유런처](S07-windows-selected-process-launch.md)의실제childimage검사를선행한뒤이GUI인수를진행한다. 기존prepared실행의당시좁은증거는보존한다.

원selected-process 실제3case scopedDONE후현재GUI후속을진행한다. 새publicreadonly process_id 관측getter는root만조정하며권한전달/PID기반ownership을추가하지않는다. 파일담당: metrics_recovery /tmp/windows_selected_gui_fixture.rs producer; diagnostic_cache /tmp/windows_selected_gui_probe.cs adapter; root 공유getter/프로토콜·통합·collector/native검증; soak_recovery_audit 현재CLI최신증거후독립GUI검토. 원GUIchild는Rust런처에만소유되고C#PID/creation/image관측HANDLE은readonly검증/UI관측용이다.
프로토콜초안: first argv --selected-gui-new CPU_ABS SHA; restart --selected-gui-restart ACTIVE REG RECEIPT OWNEDROOT(모두절대caller명시). 각READY(JSON<=16KiB) phase selected-gui-ready에 child_pid/child_creation_filetime/owned_root/data_root/registration_path/receipt_path/active_path/selected_executable/active_sha256, TERMINAL selected-gui-terminal에 Natural0·jobterminal·원files/canonical보존. productionargv0 only/noinherit. Windowsfixture는owned dlpad-cli-* private host에서만, rootTEMP/data/media/tmp를한회processenv설정한다. 1회Systemseed→LightUI선택·WM_CLOSE→자연종료후2회restart원3경로+동일dataRoot readonly선택검사→Light초기상태·WM_CLOSE, 원Job0·inputDesktop불변·fullSHA/nativeIDs/DB/pixels독립대조. 현재아직producer/adapter초안단계다.


통합 검증 기록: 첫 Windows 전체검증은 fmt 통과 후 GUI example의 canonicalize Result 전파 누락 두 곳으로 컴파일 exit101이었다. 원 실패 증거 windows-selected-gui-workspace-20261006-continue를 보존하고 두 곳을 수정했다. 수정본 전체검증은 windows-selected-gui-corrected-workspace-20261006-continue에서 진행한다. C# 중복 helper/누락 Save 선언·기존 출력 폴더 보호·attribute 초기화 guard·재시작 원 root DTO 비교를 컴파일 전에 보강했다. 아직 실제 GUI 인수 전이다.


첫 실제 GUI 인수는 first 시스템→밝게 선택·두 실행 child/producer Natural0와 원 selected Job0 및 SQLite integrity=ok/theme=Light까지 확인했다. 그러나 restart의 C# parent Job 단회 accounting 조회가 0을 확인하지 못했고 finally forced cleanup을 수행해 전체 FAIL이었다. 원 windows-selected-gui-real-20261006-continue를 보존한다. parent accounting을 최대5초/20ms 협조적 조회하며 실제 iterations/active/total/terminated/elapsed/OSerror를 기록하도록 관측기를 수정했다. timeout/queryfailure/forced는 여전히 FAIL이다. rawstdout도 명시UTF8로 설정한다. Rustsource576/제품binary는 그대로며 새 evidence에서 재인수한다.


최종 좁은 인수 완료: [수정 실제7명령 주 담당 검토](../validation/evidence/windows-selected-gui-corrected-real-20261006-continue/root-completion-review.json)와 [독립 검토](../validation/evidence/windows-selected-gui-corrected-real-20261006-continue/independent-review.json)가 PASS다. current source576 CPU54c33d95…/7,917,568B·CLI f97300cc…/723,456B·전체4검사/755passed13ignored/설치89를 유지한다. 실제 시스템→밝게 선택 및 원3명시경로 재시작의 밝게 UI·두 PNG 직접검토·readonly SQLite integrity=ok/theme=Light·쿼리 전후DBSHA 불변·원9파일SHA/10nativeIDs·원root/selectedDTO·자연CPU/producer/Job0·격리화면close/input불변/forcedfalse를 대조했다. 재시작 parent accounting ACK는 실제2회/21ms였고 원단조회FAIL 원인은 확정하지 않는다. 각 실행 직후 DB SHA를 따로 기록한 것은 아니므로 first DB SHA=restart DB SHA를 주장하지 않는다. GUI 성능/8시간/physical/cleanOS/MSVC/서명/정식installer·upgrade/P0전체완료는 범위 밖이다. 새 dependencies/lock 변경 없고 개발 example+원HANDLE readonlygetter가 추가되었다. 재현 명령은 scripts/windows_selected_gui_collect.py --proof 현재전체증거/result.json --output 새절대evidence경로이다.

# S00: CPU accessible candidate isolated download acceptance

- 상태: completed (bounded Linux CPU candidate GUI acceptance)
- 분류: verification
- 우선순위: P0
- 요구사항: R01, R04, R08, R13
- 담당/수정 범위: guardian_fault; scripts/accessible_download_probe.py, 신규 전용 CPU validation/evidence/workitem
- 선행 작업: root CPU renderer 후보 READY/hash; 기존 engine/guardian/DownloadRepository

## 사용자 결과

CPU native 후보의 다운로드 성공/정제실패보고서/일시중단재개/닫기CancelWait를실제격리GUI와DB로확인한다. DB완료와UIpaint를구분하고일반사용자세션bus를상속하지않는다.

## 계약과 범위

제품/renderer/Cargo/build수정금지. 기존Glow evidence불변보존. 각case 새ownedXvfb/별도dbus-daemon --session --nofork/0700 임시XDGconfig,data,runtime/cache와GSETTINGS_BACKEND=memory. inheritedDBUS_SESSION_BUS_ADDRESS/AT_SPI_BUS_ADDRESS/Wayland주소제거. GUI/bus로그각64KiB. ownedprocessgroup종료/회수확인. 사용자display/clipboard/screenreader/실제media/network금지,copy는격리Xvfbselection만. CPU READY전실행금지,기존8h/archive유지.

## 인수 조건

정확binary/source/fixturehash와ownedapp/bus/workeridentity. 네GUIcase PASS 및 boundedrawSQLite/report/artifact. Downloaded/Paused DB후0.6초안정화후PNG로실제완료상태독립확인. bus로그/drainer/daemon회수확인,사용자bus접근없음. minimization/zero-size는별도지원조건연구만하며실행/검증완료로쓰지않음.

## 검증과 handoff

준비단계py_compile/check_docs; rootREADY후caseall와UI PNG/SQLite 독립대조. historicalGlow running-state PNG의DB→UIpaint race를정정하고기존증거는삭제/덮어쓰지않는다.

Final actualCPU inspect/all exit0; fourcasesPASS,rawSQLiteintegrity4PASS,actualPNGPaused/최종파일확인완료view확인,binaryhashstart/end동일,4busgroupsabsent/workerreaped. Evidence440447B. [검증과실제명령/한계](../validation/accessible-cpu-download-validation.md). 사용자bus/reader사용없음,최소화/0size미검증.

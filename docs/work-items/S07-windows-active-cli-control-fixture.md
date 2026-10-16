# S07: 설치 선택 CLI의 원 준비신호 취소·기한 fixture

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R11, R08
- 담당/수정 범위: root — Cargo feature·CLI 한정hook·공유signal계약·통합; diagnostic_cache — /tmp originalprivateconsole observer/collector; soak_recovery_audit — runtime후 원control증거독립검토
- 선행 작업: CLI whole/실제14case root·독립PASS. 원단순elapsed/IOcounter에는phasegate가없음.

## 사용자 결과

실제CLI console취소와공동기한만료를 원준비event에서재현하여 원묶음보존을검증한다. 일반제품binary에는준비대기를추가하지않는다.

## 계약과 범위

새default-OFF engine active-selection-control-fixture feature만CLI nativehandler설치후activebranch prepair에서 두원inheritedevent를검증/사용한다. ready Event set은 began/parser/handlerready/precreation의명시ACK, release Event wait은절대deadline/cancel20ms확인하며기한이되면대기종료한다. parent는원console+Job+Processcreation/HandleListexact2를유지하고ready관측뒤 CTRL_BREAK 또는timeout1만료를검증한다. 생산defaultcode에는envswitch/loop가컴파일되지않는다. inheritedevent2는fixturecaps이며파일삭제/실사용계정0. lock/deps변경0, featurebinary크기는실제기록한다.

## 인수 조건

defaultOFF/feature각검사및별도binary pin, 원ready ACK후 actualInterrupted/TimedOut exit1·mutationfalse/caps0·freshactiveabsence·원12/source/notice/sentinel보존. Jobnatural0/privateconsoleclose/input불변·원processPID/creationidentity·원eventhandlesclose. no visiblefallback/globalinput/가짜phase추론. old14success와defaultbinary인수의snapshot을구분한다.

## 검증과 handoff

root 계약결정: 환경키 DLPAD_ACTIVE_CONTROL_READY_HANDLE / DLPAD_ACTIVE_CONTROL_RELEASE_HANDLE는feature에서만읽고decimaluintptr/nonnull/inheritflag를검사한다. ownedparent가CreateEvent(manualReset true,initialfalse)로생성·정확handlelist4(원event2+stdout/NUL2)와환경을구성한다. missing/duplicate/invalidready/release는featurefailclosed. 아직구현·실행전이다.

rootobserver검토: CTRL_BREAK 반환은handler실행완료ACK가아니므로cancel에서도releaseevent를열지않는다. 원gate가cancelAtomic를관측하고자연Interrupted로나갈때까지대기한다. 기본제품source에feature환경조회가남는것은아니며Rust cfg featuredefaultOFF다.

실제defaultOFF feature clippy/test/release와 원ACK취소/기한은root검토PASS다. [원control](../validation/evidence/windows-active-cli-control-real-20261006-continue/root-review.json). featureCLI b55243bb…/724,480B(+1,024B vsdefault723,456B), source/caps/readylifetime/naturalJob0/4handlelist/eventsclose/원14files보존. 독립검토와default4복귀는진행중이다. 원수집기동일bytes를 scripts/windows_active_cli_control_probe.py/.cs에보존해다음세션재현가능하게했다. 현재실제인수는 select-active prepair/creation전이며inspect-active/midwrite취소/기한까지확대하지않는다. C#unusedwarnings2도원로그에보존했다.


2026-10-06 최종 범위 인수: [현재 기본 CLI14 주 담당 검토](../validation/evidence/windows-active-cli-current-runtime-real-20261006-continue/root-completion-review.json)와 [독립 검토](../validation/evidence/windows-active-cli-current-runtime-real-20261006-continue/independent-review.json), [기본 전체4검사](../validation/evidence/windows-selected-process-final-workspace-20261006-continue/root-completion-review.json)를 대조했다. 현재 source575는 항목 수이며 CPU bb89e182…/7,917,568B, 기본 CLI5dc933c…/723,456B, 전체755passed/13ignored·설치89회귀다. 원control의 별도 featureCLI b55243bb…/724,480B는 [실제 준비신호 취소·기한](../validation/evidence/windows-active-cli-control-real-20261006-continue/root-review.json) 및 같은 디렉터리 independent-review.json의 좁은 select-active prepair 인수다. 기본 CLI14에 signal 인수를 소급하지 않는다. 현재14의 새 nativeID 조회는 없고 원 engine ID 증거와 원 파일 SHA 보존을 구분한다. deps/lock 불변·자동 실행/installer/upgrade/전체P0 완료는 아니다.

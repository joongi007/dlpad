# CPU 이력 패널 구현·검증 상태

2026-10-03. Linux CPU 후보의 scoped 이력 구현과 실제 GUI 검증을 완료했다. 최초 실패와 후속 성공을 분리해 보존한다. 현재 전용 source는 `crates/app/src/accessible_history.rs`, 실제GUI collector는 `scripts/cpu_history_probe.py`이다.

## 구현 계약

UI는 최대128개 현재page만 보유한다. 긴DB I/O는 단일owned worker가 수행하고 command/result channel각capacity1이다. next는마지막id의keyset page, refresh는처음page. 오류에는기존rows유지. UI현재옵션을저장된작업의옵션으로대체하지않고, worker가engine의bounded `validated_resume(id)`로최신상태/typed snapshot을검증한뒤같은id한개만root에전달한다. 실제download spawn은root소유이다.

catalog의existing label로state/attempt/tool-version을표시한다. 최신tool-version은root engine에서최대48byte/finite token 정책으로읽는다. rawfailure상세는현재journal에저장된계약이없으므로추정하거나새schema를만들지않았다. Failed 상태를상세원인확인완료라고표현하지않는다.

`request_stop` atomic은대기중송신도풀어주며 StopAck는DB자원unwind뒤에stopped를게시한다. UI/Drop는미완료worker를join하지않는다. context는show에서등록해worker결과가UI repaint를요청한다. new는첫page를요청하므로root는startuprecovery Ready후에생성한다.

## 준비된 회귀 시험

missing/nonregular/corrupt/symlink journal보존,invalidID failclosed,capacity1resultfull중stop,stop시resume전달제거,실패page기존rows보존,panic자원해제후stopack의8개dedicatedtests를작성했다. 초기 작성 시에는 shared engine API/module선언을 기다렸으며 아래 통합 결과에서 실제 검증을 기록한다. test코드를작성했다고통과로보고하지않는다.

단독ownedfile rustfmt exit0. Python py_compile/--validate-only exit0(프로세스없음). Cargo/제품build는root소유로실행하지않았다.

## 실제 GUI 계획과 상태

초기 준비 시점에는 새CPU READY전GUI를 실행하지 않았다. 이후 실제 실행은 아래 완료 절에 기록한다. harness는owned Xvfb/privatebus/temp data/media/합성ytfixture만사용한다. 기존guardian 옵션fixture/harness는read-only재사용하며수정하지않는다. userdesktop/audio/clipboard/실제영상/외부account사용없음.

- missing journal이력 empty표시와불필요DB생성없음.
- 실제GUI Height720job Pause후currenteditor Audio변경,History원snapshot재개로같은id/attempt2/argv720확인.
- 이작업의owned journal에260syntheticterminalrows를seed해first128/next128/thirdpage를확인.260실제다운로드로주장하지않음.
- terminal과corruptoptions의resumeaction이새attempt/worker를만들지않음.
- ownedSQLiteEXCLUSIVE lock/Refresh後namedHISTORY_LOADING표시로pending를확인,ClosePromptStopExit後lock보유생존/release후exit0.

button ENABLED bit는 pinned AT-SPI mapping 때문에일부disabled를반영하지않으므로busy proxy로사용하지않는다. 원시실패보존/변경후freshprefix/SHA검증/모든ownedchildcleanup필수. 전체reader/모든OS/전체P0완료로확대하지않는다.


## 통합 검증과 actual GUI 완료

주 담당이 shared bounded engine API/entry integration을 수행해 CPU59tests/alltargetclippy와 boundedAPI3tests/release PASS를 보고했다. 본인이 실행한 Cargo 결과로 바꾸어 쓰지 않는다. 초기 실제 clippy의 NEXT nested-if 실패는 해당 owned source를 let-chain으로 수정 후 root가 재검증했다. 신규 history 모듈8tests는 통합 대상에 포함됐다.

실제 후보 SHA256 `522e2304fc38618df696cfca1f126e08bc77634f48ac1db631d3c797fe8f8cbd`에서 다음 명령을 실행했다.

```text
python3 scripts/cpu_history_probe.py --binary target/release/dlpad-cpu --output docs/validation/cpu-history/first --font /mnt/c/Windows/Fonts/malgun.ttf --ready-sha 522e2304fc38618df696cfca1f126e08bc77634f48ac1db631d3c797fe8f8cbd --case all
python3 scripts/cpu_history_probe.py --binary target/release/dlpad-cpu --output docs/validation/cpu-history/pause-action --font /mnt/c/Windows/Fonts/malgun.ttf --ready-sha 522e2304fc38618df696cfca1f126e08bc77634f48ac1db631d3c797fe8f8cbd --case resume-pages-close
```

첫 all exit1: missing와 corrupt는 PASS, resume-pages-close는 physical Pause 이후 Paused 도달 timeout으로 실패했다. 당시 failure.png는Downloading 상태였으며 physical event 수신을 증명하지 못해 제품 결함으로 단정하지 않는다. 이 원시 result는 수정하지 않았다. 후속 대조는 Pause actual AT-SPI DoAction/phase/DBstate/owned worker identity를 계측하여 exit0. 같은 binary이며 제품 source 수정 없이 전체 resume/page/CloseWait 조건을 통과했다.

- fresh journal 없는 History에서 `저장된 작업이 없습니다` actual node/PNG를 확인하고 jobs.sqlite를 불필요하게 생성하지 않았다.
- 실제 GUI Height720 옵션으로 합성 tool worker를 실행하고 Pause했다. 현재 editor를 Audio로 바꾸고 History `재개` action을 수행했다. 같은id `job-1790995643723026931`,attempt2,저장 typed Height720 및 실행 argv2개의 `bv*[height<=720]+ba/b[height<=720]`를 확인했다. 이 합성 fixture의 Artifact 완료이며 실제 사이트/영상 decode 검증은 아니다.
- 이후 owned journal에260synthetic Cancelled rows를 seed했다. 첫 page00000,다음page00128,세번째00256을 actual Text/PNG로 확인했다. 세번째page는fixture00256–259 및 실제 합성job 한 개,`시도2`와 fixture 도구버전`2026.10.02`를 표시한다. 260실제 다운로드 이력으로 주장하지 않는다. terminal Resume action은 새attempt를 만들지 않았다.
- corrupt case는 별도 실제Paused job의 options_json만owned journal에서`{}`로 변경했다. Resume action 후 Paused/attempt1/argv1이 유지됐다. 유효하지 않은 snapshot의 무단 새attempt가 없다.
- owned SQLite BEGIN EXCLUSIVE→HistoryRefresh 뒤 named `작업 이력을 읽는 중` 실제 node로busy를 확인했다. WM_DELETE_WINDOW→actual StopExit action 뒤 lock보유 중 앱생존=true,해제 후15.482ms exit0. 내부 StopAck atomic을 직접 읽는 대신 외부 응답/종료를 관측했다.

모든 prefix의 direct owned 앱/bus/Xvfb 및 임시 fixture 회수, 시작/끝binary hash 동일. 후속 resume trial은 pidfd로 확인한 첫 합성 worker identity도 기록했다. 실제 system desktop/audio/clipboard/기존 사용자 DB를 사용하지 않았다. 제품source/Cargo/shared harness를 추가 수정하지 않았다.

요약 `cpu-history/summary.json`, root 독립 검토용 `pause-action/resume-pages-close/{page2.png,page3.png,result.json}`, 원 실패 `first/resume-pages-close/{failure.png,result.json}`, 실제 journal snapshot 각 prefix의jobs.sqlite를 보존한다. source-final.json은 수집 뒤 source 관측이며 build provenance를 대체하지 않는다. 전체 reader/다른OS/장시간 이력 성능/rawfailure 상세는 별도 미검증 범위다.

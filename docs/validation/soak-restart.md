# 2026-10-02 synthetic soak 재실행

## 이전 결과를 구분한 이유

재개 시 이전 두 /tmp status와 PID29575/29577/103171/103274/103276가 존재하지 않았다. 현재 boot ID는794988eb-539b-437e-8396-330738b565bb다. 이전 session57344/76382는 당시 중간running관측만 남았고 8시간완료 결과는 미검증이다. 세션 손실 원인을 WSL 재시작으로 확정하거나 완료로 간주하지 않는다. 읽기전용관측은 evidence/soak-restart-20261002/previous-run-unverified.json에 보관했다.

## 새 실행과 actual baseline

현재소스·Cargo.lock/toolchain/scripts46개 SHA256을 launch-source-manifest.json에 기록했다. source/Cargo를수정하지않았다. 정상Git저장소가없어기존.git을보존한다. `cargo test -p dlpad-engine --test download_soak --locked -- --nocapture`는1passed/1longignored,7.15초였다. 개발guardianhelper를현재snapshot으로빌드했다(exit0,cachelock대기포함1m30초). 새시험명령과binaryhash·sessions·정확PIDstart는 launch.json에 있다.

```sh
python3 scripts/download_soak.py target/debug/deps/download_soak-eb1ebcee84bd48b1 /tmp/dlpad-eight-hour-download-20261002-794988eb --seconds 28800
python3 scripts/long_job_soak.py target/release/dlpad /tmp/dlpad-eight-hour-guardian-20261002-794988eb --seconds 28800 --interval 60
```

새execsession은DownloadRepository13723,releaseguardian64092다. 두실험은현재running이며**아직8시간완료가아니다**. DownloadRepository는engine+SQLite+합성tool의prepare/download/partial/pause/resume/re-extraction/publication/reopen범위이며actualvideo/transport/site/GUI를검증하지않는다. releaseguardian은중첩·분리합성자손의프로세스수명관측이며DB/UI/media시험을대체하지않는다. 사용자Documents/desktop/영상/네트워크는사용하지않는다.

## /tmp 손실 대비 영구 증거

별도archive session21052가60초마다이번합성시험의status/sample/result/ownedlog만project evidence/soak-restart-20261002/에복사한다. 파일당1MiB,전체8MiB,progress-history512행상한이다. source와runningguardian/proc/exe의실제SHA를기록하므로후속빌드가경로를바꿔도실행당binaryidentity를구분한다. SQLite/userdata/credentials/실제미디어는복사하지않는다. archive-status.json의observedUTC와bootID는historicalsnapshot의신선도를표시한다. 영구runningJSON만으로프로세스가현재살아있다고판정하지않으며host/proc정확startticks대조가필요하다.

최종download-result/runner-result와guardian-result가실제로생성되어야archive-result를쓴다. result파일이없거나stopped/failed이면8시간pass가아니다. 현재관측·용량과PID는launch.json 및archive-status.json에기록한다. 장시간최종assertion과cleanreap·DBreopen검증은종료후통합담당이직접확인한다.

## 최신 authoritative host 관측

2026-10-02T16:46:13.050536+00:00 관측: boot794988eb-539b-437e-8396-330738b565bb 일치. DownloadRepository 14318.96초, guardian 14303.76초 모두running; host9 PID의정확startticks 모두일치. archivePID6202/start48872 watching, 최근복사 2026-10-02T16:46:10.604223+00:00. exec13723/64092/21052 모두running응답. terminalresult미생성으로8시간완료미검증. 재시작·중단·제품소스변경없음. authoritative-observation.json은한개bounded최신관측으로덮어쓰며별도historyarchive는기존512행상한유지.

## 현재 작업 종료·후속 재개 인계

2026-10-02T17:34:32.866438+00:00 host관측: Download 17464.96초/guardian 17429.73초running,9PIDstart모두MATCH/boot변경없음. archive6202/start48872watching,최신 2026-10-02T17:34:15.361670+00:00. GUI옵션4execsession은모두terminal이며추가GUI/새작업없음.

기존장시간runner는28800초fixture자연종료 후assertion/DB재개·reopen/descendantcleanup/result파일작성/프로세스자동종료 경로다. 현재monotonic관측기준28800초까지Download11335초/guardian11370초(약3시간9분)가남았다. 시작UTC만으로계산한과거06:05:45KST 예상과현재wallclock/elapsed관측이차이나므로정확한wallclock종료시각은확정하지않는다. 후처리/assertion시간이추가될수있다. 완료예정은PASS확정이아니다. archive는현재watching/최종result복사계약이며자동종료는terminalproof를읽고확인해야한다. 다음재개시boot/PIDstart및실제result+exitcode를조회하고관측timeout만으로재시작하지않는다. pause-handoff-observation.json과launch.json에정확한session/PID/start가있다. 사용자지시에따라현재실행을중단·재시작하지않았다.

## 2026-10-03 재개 실제 검토

2026-10-03T02:23:16.416425+00:00 host boot/PIDstart와 exec13723/64092/21052 단기poll 모두running 확인. Download 18483.13초/guardian 18451.57초이며28800초까지약2시간53분남음. 9runPIDstart와archive6202/start48872 모두MATCH. terminalresult파일없음. wallclock는 overnight진행했지만monotonicelapsed는아직8h미만이므로시간경과만으로완료판정하지않는다. 실행중지/재시작/GUI/제품변경없음. 정확observation은resume-review-20261003.json.

# 기존 8시간 soak 읽기전용 재개 감사

2026-10-03. 기존 `soak-restart-20261002` 두 합성 headless 시험과 archive를 **변경하지 않고** host에서 조회했다. 프로세스 종료/재시작, 원 archive 수정, 제품 source 변경은 수행하지 않았다. 새 원시 증거: `evidence/soak-resume-audit-20261003/observation.json`.

## 현재 상태

boot ID `794988eb-539b-437e-8396-330738b565bb`는 launch 기록과 일치한다. 기존 known9개 soak 프로세스와 archive PID6202의 `/proc/PID/stat` startticks가 모두 일치했다. 따라서 알려진 프로세스의 live identity가 유지되는 것을 확인했다. PID 번호만으로 살아 있다고 판정하지 않았다.

Download runner는 running/21190.944초, worker sample elapsed21180.971초. Guardian은 running/21156.358초. planned28800초까지 당시 단조시계 기준 약7609/7644초가 남았다. 정확한 벽시계 완료 시각을 보장하지 않는다. WSL 종료/재시작 여부를 추정하지 않는다. terminal result.json은 runner/download/guardian 출력에 없으며 **8시간 완료·성공이 아니다**.

## SQLite와 archive

owned long-job/jobs.sqlite를 `mode=ro&immutable=1`로 열어 최대4개 row/각 job_json131072byte 상한으로조회했다. journal_mode=delete,job state=Downloading,읽기전후파일size/mtime동일. lock/sidecar쓰기/복구를요청하지않았다. 활성SQLite의immutable읽기는일반동시성무결성검증이나quick_check가아니며이관측범위의snapshot만보고한다.

archive는watching,관측시마지막복사약20.48초전,recorded8개파일의size/SHA가모두일치했다. aggregate600KiB수준으로8MiB상한이내이며per-file1MiB/history512행정책이기록돼있다. archive가현재프로세스생존을대체한다고보지않고위hostPID/start대조를별도로수행했다. 모든읽기byte는상한있는status/journal/실행파일과knownprocessmetadata만이며환경/credentials/실제영상은열람하지않았다.

## 실행파일 증거의 중요한 한계

Launch의download testSHA는f612b72a9476b5e59b4ff5050c42b05a6b12044ff1ea76693b38f17a4f2c8c0c,guardianSHA는e9fa3ce36b3e4a1285ef0ee5930179ee4fa126451730fd0cbdcb2bae020bcc7f였다. 기존 running-guardian-binary-proof도e9fa를기록한다.

이번collector는 **resolve()를 호출하지 않고** `Path('/proc/PID/exe').open('rb')`의직접FD로64KiB씩읽어SHA를계산했다. 그럼에도현재읽힌downloadPID5770SHA는2ca709688a2540f8c2454f6b13b1b645a14f8e8312f0a253b83b78c6a1677df4,guardianPID5888SHA는8aa9b235a357c9c6c43f77194261cf522a51bf4aa4158ec1ccf81c455d76f7d5로launch와다르다. 반복directFD읽기도같은값이다.

두 `/proc/PID/exe` 링크는 `(deleted)`다. metadata도다음처럼불일치한다.

| 대상 | proc.stat old inode/size/nlink | direct FD fstat size/nlink | 현재 pathname inode |
|---|---|---|---|
| download | 3659174697570963 /44401376 /0 | 44481824 /1 | 3377699720614266 |
| guardian | 1407374883945582 /9882048 /0 | 9894496 /2 | 2251799814146377 |

Device는71이며directFD가보고한inode는proc oldinode와같다. 현재pathname inode는다르다. `/proc/PID/maps`의executable-only행은oldinode와deleted경로를유지한다. directFD fstat의size/mtime은현재pathname쪽값과같다. 원수치는observation.json의executable_identity_details에있다.

따라서 **이번FD bytes만으로현재loaded code가최신디스크binary로바뀌었다고확정하거나launch hash 불일치를재시작증거로해석하지않는다**. 반대로launch SHA가현재runtime code를완전히증명한다고확대하지않는다. 파일시스템/proc 열람증거의동작차이는주담당검토가필요하다. loadedprocess메모리를덤프하거나추가system변경을하지않았다. soak는live identity를유지하며진행중이고executable provenance는검토가필요하다.

## 실행 명령과 handoff

읽기전용 Python inlinecollector로known `/proc/PID/stat`,boot ID,statusJSON,archive SHA,SQLite URI SELECT를조회했고두번째collector로직접 `/proc/PID/exe` FD hash/fstat/readlink/currentpathname stat/executable-onlymaps를조회했다. 모두exit0이며실행한process action목록은빈배열이다. 원래장시간시험및archive프로세스는그대로유지했다.

이번감사는terminal완료판정이나최신product전체기능검증이아니다. 다음은주담당이종료결과와실행당source/binary 증거의한계를함께검토하는일이다. 잔여시간만보고추가8시간시험을새로시작하지않았다.

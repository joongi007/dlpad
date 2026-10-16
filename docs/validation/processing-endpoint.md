# Linux processing endpoint 구현 검증

## 구현과 사용자 범위

신규 `crates/engine/src/processing_endpoint.rs`는 job-owned staging directory 아래 private childdirectory0700/FIFO0600을 만든다. 기존 libc만 사용하며 dependency/schema를 추가하지 않았다. `ProcessingEndpoint::new(root,feed)`, `argument()`, `postprocessor_argument()`, `finish()`와 Drop으로 수명을 관리한다. 생성/finish는 worker에서 호출한다. endpoint는 도구를 실행/종료하지 않고 media/artifact를 소유하지 않는다.

단일 dedicated machine stream은 O_RDWR|O_NONBLOCK|O_CLOEXEC|O_NOFOLLOW와 poll20ms,4KiBread로 진행률을 받아 최신 sample1만 유지한다. 기존 line64KiB/record64lines parser를 사용하며 오디오-only는 frame 없이 bitrate/out_time/speed/progress로 수용한다. dedicated active 중 human stderr frame observer는 숫자 상태에 개입하지 않으며 다운로드 B/s feed도 독립이다. end 후 다음 FFmpeg record는 새 parser epoch로 시작한다. unknown/negative/malformed/incomplete/stale/end ETA 정책은 기존검증을 유지한다.

## 정리·실패·argv 계약

`finish`는 stop 후 최대256KiB/200ms pending drain을 수행하고 reader를 join한다. FIFO가 writer를 자체 유지하므로 불가능한 EOF를 기다리지 않는다. reader panic/오류/cleanup오류는 processingunknown 및 dedicated ownership종료로 처리한다. constructor 생성 후 실패는 provisional RAII owner가matchingdev/ino검사로정리한다. FIFO/childdir identity가 바뀌면 foreignreplacement를삭제하지않고 ioError를반환한다. 기본directory는삭제하지않는다. 같은계정의동시악의적path교체에대한check→unlink rawpathrace까지atomic보호한다고주장하지않는다. 이endpoint는privilegedsecurityboundary가아니다.

new 실패는 optional telemetry 실패다. 현재 integration은 계측 준비 실패 시 processingunknown으로 되돌리고 정상 download를 계속한다. 계측 준비 실패의 별도 진단 항목은 후속이다. 이외 download/process/artifact 실패를 무시한다는 뜻은 아니다. 기본fallback humanstderr framegate를완화하지않는다.

yt-dlp는 postprocessor argv value 내부를 shlex토큰화하므로 `postprocessor_argument`가 owned Unixrawpath를singlequote로직렬화하고apostrophe를escape한다. `--postprocessor-args`와구간의 `--external-downloader-args`에 이finitevalue를사용한다. 외부shell을실행하지않고rawuserpostprocessor입력은추가하지않는다. outputroot에한글/공백/apostrophe가있어도path가한토큰으로전달된다.

## 실제 회귀

`cargo test -p dlpad-engine --lib processing_endpoint --locked -- --nocapture`:5회귀passed(초기최종fixture3.88s). 성공목록:

- frame없는audio3bytefragment에known4초duration/1.5초timestamp/2.5x에서ETA1초, pendingfinalrecorddrain/end및privatepermissions/cleanup.
- writer없는FIFOstop1초미만, incompletefragmentEOFunknown.
- FIFOforeignreplacementpreserve,symlink/fileparent거절.
- 실제Pythonshlex로한글/공백/apostrophepathroundtrip.
- 실제DownloadRepository+SQLite+ownedprocessrunner에서synthetictool성공/실패/취소각1회. successDownloaded/finalend,invalidspeed+exit7 Failed/unknown,firstprocessingsample후cancel Cancelled. 모두 readerdrop/FIFO+privatechilddir삭제,64KiBpartial보존.

`cargo test -p dlpad-engine --lib dedicated_audio_epochs --locked -- --nocapture`:1passed,0.02s. audio전용pipe/newprocessepoch와stderr숫자격리,download123B/s독립,unfinishedEOFunknown회귀.

실제yt-dlp/FFmpeg와productadapter연결된fast/accurate및GUI시험은integration owner가담당한다. 선행throwaway 실제merge/audioFIFO live증거는 docs/validation/live-postprocess-probe.md에있다. 이문서의synthetictool통과를실제FFmpeg통과로대체하지않는다. Linux전용module/metricsAPIs는cfg로격리하며Windows/macOStransport미구현/미검증이다. namedpipe/FIFOOS시험게이트는유지한다.

## 비용과 계속 실행 중인 시험

endpoint당readerthread1개(기본threadstackvirtualbudget포함),4KiBreadbuffer,parser최대64KiBline,mutex/latest1과privateFIFO하나를추가한다. 큰progress파일/recordqueue/새dependency는없다. 무관한사용자프로세스를조회/종료하지않는다.

기존releaseguardian8h session57344는11840.05초,DownloadRepository8h session76382는5614.11초까지running이었다. host/proc에서9개namedPIDstart가모두기록과일치했다. 두기존job는종료/재시작하지않았으며아직8시간완료가아니다.

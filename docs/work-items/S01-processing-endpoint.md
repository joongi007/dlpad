# S01: Linux job-owned dedicated processing endpoint

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R01, R05, R08
- 담당/수정 범위: guardian_fault; 신규 engine/src/processing_endpoint.rs, metrics.rs, 전용 validation
- 선행 작업: 실제 FIFO merge/audio live 실험, bounded FFmpeg parser

## 사용자 결과

일반 merge/audio FFmpeg의 전용 progress를 bounded latest processing feed로 전달한다. 계측 실패는 다운로드 자체를 완료/실패로 추정하지 않으며 unknown으로 표시한다.

## 계약과 범위

AGENTS/README→제품/CONTEXT→R01/ADR-0002를 따름. root가 lib.rs/Cargo/download/process/app을 담당하며 동시수정하지 않는다. 신규dependency 없음(libc기존). ProcessingEndpoint::new(job-owned staging root,feed), argument/raw path, postprocessor_argument/typed reserved shlex value, finish/stop+join+cleanup, Drop최선정리 계약합의. Linux전용;Windows/macOS미검증. userDocuments/actualmedia/desktop금지. 기존2개8h job유지.

## 인수 조건

- userprivate childdirectory0700/FIFO0600, nonblocking poll20ms와4KiBchunks, bounded parser/latest로frame없는audio도수용.
- active dedicated stream과humanstderr framegate분리, 다운로드B/s보존, unknown/stale/end계약유지.
- FIFOwriter없음/중간fragment/잘못된parent/취소/소유node변경에대한fail-closed cleanup,무관replacement삭제금지.
- finish는pendingrecord drain후boundedjoin,emptyFIFO도무한EOF대기하지않음.
- 공백/한글/quotepath를finite argv value로안전하게직렬화. productconnection/actualGUI는root통합검증.

## 검증과 handoff

구현중,완료아님. 기존source/Cargo/sharedschema변경없음.

## 구현/검증handoff

endpoint/metrics production소스고정. 5endpoint회귀와1dedicatedaudioepoch회귀통과, 실제Repository성공/실패/취소Drop정리포함. root가lib/download/process/app연결및actualFFmpeg/GUI통합검증담당. 신규dependency없음,Windows/macOS미검증. 상세계약·오류·한계는 docs/validation/processing-endpoint.md. 두기존8h jobrunning유지.

통합 담당 검증: 최종 Linux workspace fmt/clippy/tests/release 모두 통과. 실제 합성 yt-dlp/FFmpeg 전체/빠른·정확 구간/자막/DB reopen1/1 최종36.67초 통과. [최종 통합 기록](../validation/background-processing-integration.md)을 따른다.

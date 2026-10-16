# S02: 실제 다운로드 속도와 ETA

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R01, R04, R08
- 담당/수정 범위: 주 담당; crates/engine/src/metrics.rs, process.rs, download.rs, crates/app/src/main.rs, catalog.rs, crates/engine/tests/local_media.rs, 통합 문서
- 선행 작업: S02 프로세스·다운로드 adapter, S06 metrics parser

## 사용자 결과

실행 중 실제 yt-dlp 다운로드 B/s와 알려진 ETA를 보여준다. 측정 없음·지연·작업 종료 시 오래된 숫자를 유지하지 않는다.

## 계약과 범위

고정 JSON progress template, 줄 64 KiB, 최신 sample 하나, UI channel 1개 및 약 1초 갱신. 완료/실패 상태는 progress와 독립. 공개 execute API 유지, 추가 observed API만 도입. DB/Cargo schema 변경 없음. FFmpeg 및 OS interface 지표는 후속 P0 작업.

## 인수 조건

출력 분할·양쪽 pipe·큰 줄·malformed/stale progress·취소 경로 검증. 합성 실제 도구 시험에서 progress 관측. fmt/clippy/test/release와 가상 화면 GUI 검증. 사용자 데스크톱 사용 금지.

## 검증과 handoff

분할 출력·양쪽 pipe·EOF·잘못된/오래된 sample·출력 상한·취소 테스트 통과. 실제 yt-dlp/FFmpeg 합성 다운로드에서 progress 관측 및 전체·빠른/정확 구간 decode 확인. 누락 필드는 JSON null로 출력하도록 고정했다. 실제 사이트/다른 OS는 미검증. 공통 fmt/clippy/test/release 통과. 전체 74 tests 및 실행 명령/측정은 [통합 검증](../validation/followup-validation.md)에 기록한다. 작업 범위 완료이며 S00 최종 스택 또는 전체 P0 완료를 뜻하지 않는다.

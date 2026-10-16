# S02: 실제 외부 도구와 합성 미디어 통합 시험

- 상태: done
- 분류: test
- 우선순위: P0
- 요구사항: R04, R05, R08
- 담당/수정 범위: 주 담당; crates/engine/tests/local_media.rs
- 선행 작업: S02 download adapter

## 사용자 결과

실제 yt-dlp가 합성 MP4를 받아 한글 경로에 공개하고 재시작 후 완료 기록을 유지하는지 검증한다.

## 계약과 범위

명시적으로 지정한 기존 도구만 실행. 임시 폴더 합성 영상과 127.0.0.1 임시 서버 사용. Drop으로 소유 서버 종료/reap. 사이트·실제 영상·인증·외부 전송 없음.

## 인수 조건

FFmpeg 합성 생성, yt-dlp metadata/download, 최종 파일 byte 비교, SQLite 재열기 완료 유지, 빠른/정확한 구간 저장과 FFmpeg decode 성공. opt-in ignored 테스트로 도구 미설치 CI와 구분.

## 검증과 handoff

DLPAD_SMOKE_YTDLP=<local-tool> DLPAD_SMOKE_FFMPEG=<local-tool> cargo test -p dlpad-engine --locked --test local_media -- --ignored --nocapture: 1 passed (18.97s). yt-dlp 2026.09.27.232945, FFmpeg 4.4.2. 전체 byte 비교/DB 재열기/빠른·정확한 구간 저장/decode 통과. 임시 서버·영상 정리. 실제 사이트·8시간·프레임 정확도 시험을 대체하지 않는다.

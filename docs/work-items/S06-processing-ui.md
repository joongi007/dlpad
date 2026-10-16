# S06: 구간 미디어 처리 배속 연결

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R01, R08
- 담당/수정 범위: 주 담당; engine/process.rs·download.rs, app/main.rs·catalog.rs; guardian_fault metrics API/실증 완료
- 선행 작업: processing-feed 실증

## 사용자 결과

구간 저장의 FFmpeg 처리 배속과 가능한 ETA를 다운로드 B/s와 구분한다. stale·unknown·종료 관측은 완료 증거가 아니다.

## 계약과 범위

stderr-only processing observer, stdout download observer를 분리한다. finite external-downloader args를 구간 실행에만 추가하고 source/range duration을 microseconds로 설정한다. UI는 최신 처리 sample만 표시하며 ended/stale은 미측정으로 되돌린다. Linux에서는 job 소유 FIFO를 일반 merge/audio와 구간 external downloader에 연결한다. Windows/macOS 전용 endpoint는 미지원이며 R01 전체 완료가 아니다.

## 인수 조건

다른 pipe의 frame을 처리 지표로 오인하지 않고 download speed 독립 유지. long I/O UI thread 없음, sample 상한 및 cancellation 기존 수명 유지. 실제 도구·격리 UI 검증 이후 완료로 표시한다.

## 검증과 handoff

Linux 실제 yt-dlp/FFmpeg 합성 영상·자막·전체/빠른·정확 구간 다운로드 통합 1/1 통과(42.21초). 구간 작업은 dedicated processing end record/시간 관측과 owned FIFO cleanup을 확인했다. 최초 실행은 필수 tool env 미설정으로 실패했으며 정확 로컬 경로를 명시한 재실행 결과만 성공으로 기록한다. 일반 merge/audio live 실험과 endpoint unit/GUI 결과는 전용 검증 문서를 따른다. Windows/macOS의 일반 postprocess 표시는 미측정이다.

최종 주 담당 통합검증: fmt/clippy/workspace tests/release 성공, 실제 합성미디어1/1(36.67초), Xvfb 숨김·복귀/완료후숨김/처리표시 성공. [최종 통합 기록](../validation/background-processing-integration.md)을 따른다. Windows/macOS 복귀·전용처리transport와 전체P0는 이 완료 범위에 포함하지 않는다.

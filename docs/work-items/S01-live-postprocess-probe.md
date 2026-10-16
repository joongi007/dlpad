# S01: 일반 후처리 live progress 전용 sink 실험

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R01, R05, R08
- 담당/수정 범위: guardian_fault; 신규 scripts/ffmpeg_live_proxy_probe.py, docs/validation/live-postprocess-probe.md
- 선행 작업: processing feed, yt-dlp 일반 PP stderr 흡수 실증

## 사용자 결과

yt-dlp 일반 merge/audio PP 중 처리 배속을 부모의 전용 bounded progress 경로로 관측 가능한지 확인하고 cross-platform production 설계에 근거를 제공한다.

## 계약과 범위

기존 production/Cargo/metrics 파일은 freeze, 신규 throwaway 실험 파일만 수정한다. AGENTS/README 읽기 순서와 R01/ADR-0002를 따른다. 소유 임시 FIFO 또는 native proxy를 검토하며 전체 progress 파일 누적은 금지한다. 사용자 미디어/Documents/desktop/외부 네트워크 금지. 합성fixture와 owned loopback file transport만 허용, 웹 UI 없음. 기존2개8h job보존.

## 인수 조건

- 실제 yt-dlp merge/audio-only PP에서 -progress owned sink의 live record를 bounded latest로 관측한다.
- stdout/stderr human로그와 구분하고 unknown/ETA/frame 없는 audio-only 계약을 판단한다.
- 취소/실패 시 소유group과FIFO만 정리하며 무관한process를 건드리지 않는다.
- Linux전용FIFO 성립과 Windows/macOS 미검증을 구분하고 platform대안·비용·한계를 제안한다.

## 검증과 handoff

현재 제안 실험, 성공아님. product source와 dependency 변경 없음.

## 실제 결과

최종script exit0. 실제merge13record/6.113초, audio-only13record/6.739초 live성공. ownerEOF취소1.411초 exit-9/endfalse,invalidsink exit1/record0. 모두owneddescgone/FIFOremoved,outsiderpreserved. dedicatedsink만audioframegate불필요. production은미수정,Windows/macOS transport gate 유지. 전용 docs/validation/live-postprocess-probe.md에교정실험·bounded계약·platform후속제안기록.

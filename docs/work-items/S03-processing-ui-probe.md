# S03: 합성 미디어 처리 배속 GUI 검증

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R08
- 담당/수정 범위: diagnostic_cache; scripts/background_ui_probe.py 및 전용 검증문서/evidence
- 선행 작업: 주 담당 FIFO progress/app footer 구현과 release binary

## 사용자 결과

다운로드 B/s와 미디어 처리 배속을 별개로 표시하고 완료 후 배속을 지운다.

## 계약과 범위

주 담당 승인 후 synthetic yt 도구만 postprocessor argv에서 owned FIFO를 찾아 machine records를 보낸다. 실제 FFmpeg/영상/계정 및 제품 소스/Cargo는 수정하지 않는다. Xvfb만 사용한다.

## 인수 조건

native footer2.50×와 다운로드 B/s 분리 표시, Downloaded 후 배속 제거, owned temp/process 정리, 정확 binary hash 기록.

## 검증과 handoff

`python3 scripts/background_ui_probe.py target/release/dlpad <evidence.json> --mode processing`37.197초 통과. [실제 JSON/이미지 및 한계](../validation/background-ui-validation.md). 실제 FFmpeg 배속 정확도/다른 OS는 미검증이다.

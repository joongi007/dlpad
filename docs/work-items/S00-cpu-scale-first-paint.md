# S00: CPU 첫 native paint·200% 접근성 검증

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R11, S00
- 담당/수정 범위: diagnostic_cache; 신규 scripts/cpu_scale_paint_probe.py, docs/validation/cpu-scale-first-paint.md 및 cpu-scale-paint evidence
- 선행 작업: CPU SHA28b412773a5fc1434f86d396667a30e7b6d671a8e30affbe1a41cb1ecea7d658, root bounded probe_display(screen) helper 및 안전 테스트

## 사용자 결과

3회 warm launch의 실제 native 픽셀 게시 시점과 200%/800x600 논리 viewport에서 핵심 입력·버튼·포커스·실패 보고서 접근을 관측한다.

## 계약과 범위

제품/Cargo/기존 probe 수정 금지. owned Xvfb/private bus/temp XDG/app-data/media만 사용하고 사용자 desktop/clipboard/audio 금지. 첫 paint는 실제 비균일 glyph 픽셀을 XGetImage로 확인하며 창 생성만으로 성공 처리하지 않는다. 200%는 WINIT_X11_SCALE_FACTOR=2 및 owned window XResizeWindow(1600,1200), screen2200x1400 시험이다. 실제 multimonitor DPI로 확대하지 않는다. 첫화면 1초 기준은 그대로 적용한다.

## 인수 조건

3회 시간·픽셀 predicate·CPU/해상도/폰트 hash·임시 데이터 조건과 200% 실제 PNG/AT-SPI/Tab/정제 보고서를 보존한다. 합성 도구 외 실제미디어를 사용하지 않는다. 소유 직접 process 회수와 끝 hash를 확인하고 애매한 항목은 미검증으로 남긴다.

## 검증과 handoff

같은28b4 hash에서3warm 실제native glyph 표시0.241526/0.232286/0.233346초(1초이내), 200%/800x600logical 핵심field/button bounds·Tab focus·합성실패report/AT-SPI·PNG 확인. collector XIO/helper인자 실패원본보존. 마지막보고서줄scroll/실제multimonitor/새hostbinary는미검증. 전용문서 docs/validation/cpu-scale-first-paint.md.

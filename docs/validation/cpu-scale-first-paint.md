# S00: CPU 첫 native paint·200% 제한 시험

관측 2026-10-03, Linux/WSL의 owned Xvfb. 바이너리 SHA256 `28b412773a5fc1434f86d396667a30e7b6d671a8e30affbe1a41cb1ecea7d658`, 폰트 SHA256 `7a183cf1c6c56b9609fcc16eda8b5229fbc11758a21e669ec00343239b02192f`(기존 malgun.ttf 읽기만), AMD Ryzen 7 9700X 8-Core Processor/16 logical CPU.

## 실제 첫 화면 픽셀

[3회 원 결과](cpu-scale-paint-corrected/result.json): 100%/1280x900 Xvfb, native 창 960x640에서 **0.241526 / 0.232286 / 0.233346초**. 1초 기준을 그대로 적용하여 모두 이내다. cold launch·일반 사용자 환경 성능의 완료 판정으로 확대하지 않는다.

시간 기준은 private bus/AT-SPI registry 준비 이후 app Popen 직전 perf_counter부터 첫 XGetImage 두 영역의 실제 glyph 픽셀 조건 충족까지다. 창 존재·MapState만으로 paint를 세지 않는다. header [0,0,220,60]와 URL control [0,66,400,70] 영역 모두 고유 BGR 색≥8 및 밝은 픽셀≥20을 요구했다. 실제 header 고유색224/밝은512, control 고유색111/밝은536이었다. 10ms poll 및 XGetImage 관측 비용이 포함된 관측 상한이며 사진 PNG 변환 시간은 제외된다. 실행 파일·폰트의 warm OS filesystem cache 조건이며 각 launch의 app-data/media/XDG는 새 empty temp다. 가상 화면 및 bus 시작 비용은 timer 밖이다.

각 warm-N의 first-visible.png를 보존했다. 첫 collector는 XCloseDisplay를 Xvfb teardown 후 호출하여 XIO exit1이었다. [원 실패 기록](cpu-scale-paint/collection-error.txt)과 partial PNG/logs를 보존했다. 수정 후 정상 3회 수집은 위 fresh prefix다.

## 200% 화면과 접근성

WINIT_X11_SCALE_FACTOR=2, 새 2200x1400 Xvfb. 검증된 소유 window만 positive XResizeWindow(1600,1200)로 바꾸어 800x600 logical viewport를 만들었다. [최종 원 결과](cpu-scale-paint-200-confirm/result.json), [노드·GetState·GetExtents](cpu-scale-paint-200-confirm/scale-200/result.json), [독립 검토](cpu-scale-paint-200-confirm/review.json).

4개 입력 및 Start/Resume/Recovery를 포함한 관측 named widget 15개의 Component.GetExtents가 모두 physical [0,1600]×[0,1200] 내부였다. URL을 실제 입력하고 Tab으로 yt-dlp tool entry에 이동했으며 GetState의 FOCUSED bit12가 해당 entry에만 있었다. 실제 합성 외부 tool을 engine으로 실행해 exit7 실패를 만들고 failure report를 열었다. [report.png](cpu-scale-paint-200-confirm/scale-200/report.png)를 직접 검토하여 입력·버튼·보고서 열기·review/copy 버튼·preview·footer 접근을 확인했다. AT-SPI GetText에 정제 보고서 전체가 있으며 fixture-secret 원문은 없다. clipboard 버튼을 누르거나 clipboard 데이터를 읽지 않았다.

보고서는 내부 ScrollArea이므로 긴 본문 전부가 한 번에 화면에 보이지 않는다. report 마지막 줄까지 scroll 시험은 미검증이다. 200% resize 이전 최초 native 픽셀 관측은 0.246292초이며 resize 이후 800x600 최초 paint를 별도로 재지 않았다. 실제 multi-monitor DPI/OS compositor·다른 OS/전체 reader usability로 확대하지 않는다.

처음 200% collector는 key_input helper 인자누락으로 실패했다. [원 실패 JSON](cpu-scale-paint-corrected/scale-200/result.json)을 유지하고 signature 수정 후 200%만 새 prefix로 재수집했다. 정상 collector 종료0/소유 직접 child 회수/임시 fixture 삭제/끝 SHA 동일을 확인했다. 3 warm 결과와 200% 재확인 결과는 같은 바이너리다.

## 실행과 검증

`python3 scripts/cpu_scale_paint_probe.py target/release/dlpad-cpu docs/validation/cpu-scale-paint-corrected --font /mnt/c/Windows/Fonts/malgun.ttf`

`python3 scripts/cpu_scale_paint_probe.py target/release/dlpad-cpu docs/validation/cpu-scale-paint-200-confirm --font /mnt/c/Windows/Fonts/malgun.ttf --scale-only`

신규 script/전용 문서 외 수정 없음. source/Cargo/build/기존 probe를 변경하지 않았다. owned Xvfb/private D-Bus/AT-SPI/temp XDG/GSettings memory 및 synthetic tool/media만 사용했다. 사용자 desktop/audio/clipboard·실제 미디어를 사용하지 않았다. 첫 paint deadline5초, bus readiness10초, 재사용 gdbus call별5초·출력/48 node 상한이다. 정상 X 연결은 server teardown 전에 닫고 child process group을 회수한다. 초기 collector 예외 때 X 연결은 process 종료로 회수됐다.

이 증거는 warm 첫 glyph 게시와 200% 핵심 화면의 제한 시험이다. GL 미사용/메모리 측정, Windows host adapter 초기화 수정 후 바이너리, 제품 전체 성능/keyboard/reader 게이트는 통합 담당의 별도 검증 범위다.

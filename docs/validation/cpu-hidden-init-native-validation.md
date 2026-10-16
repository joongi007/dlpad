# S00: CPU hidden-init Linux native 재검증

관측 2026-10-03. 후보 SHA256 `1d4757de07cb04d8cfcc08d1f039d7f3521b38de369fb1c4fc2b3cd8762b15b5`, 10172280B. [수집 전 source/binary hash](cpu-hidden-init-native/source-before.json), [종료 후 동일 hash](cpu-hidden-init-native/source-after.json). accessible.rs/accessible_cpu_main.rs/software_host.rs 및 바이너리 모두 동일했다. 기존28b4 원시 증거는 수정하지 않았다.

## 실제 입력·reader

[inspect](cpu-hidden-init-native/inspect/result.json)의 actual initial.png를 직접 확인해 설정 입력 y=102를 확인했다. [IME 원 관측](cpu-hidden-init-native/ime/result.json) 및 [별도 시각 판정](cpu-hidden-init-native/ime/review.json): ASCII abc, 한글 Space 확정 `한글 `, 조합 중 BackSpace 두 번 취소 후 `한글 `, Latin `한글 abc`, 선택교체 xy를 AT-SPI GetText로 확인했다. [preedit.png](cpu-hidden-init-native/ime/preedit.png)를 직접 보니 입력 칸 내부의 한글 및 마지막 글 조합 선택 표시·caret이 있다. preedit GetText도 한글이다. 원 collector exit1/시각 미판정 false를 유지하고 실제 시각검토를 review로 구분했다. raw IME 이벤트 trace 기능은 없으므로 event trace 도착을 주장하지 않는다.

[reader 원 결과](cpu-hidden-init-native/reader/comparison.json), [전체 노드](cpu-hidden-init-native/reader/candidate/result.json), [실제 무음 Orca 로그](cpu-hidden-init-native/reader/candidate/reader.log): 한글 label/entry/GetText readerfixture, registry 초기화, text-changed/caret 처리, Tab 후 entry→다운로드 button locusOfFocus 처리를 확인했다. reader_event_processing_verified=true, 전체 screen_reader_verified=false. collector0/Orca0, 로그81802B. Clipboard/음성 출력은 사용하지 않았다.

## 실제 첫 native glyph와 200%

[첫 paint·scale 원 결과](cpu-hidden-init-native/paint-scale/result.json): 3회 warm 첫 glyph 게시 관측 **0.228988 / 0.186718 / 0.160565초**, 각1초 기준 이내. Popen 직전 perf_counter→header/control 두 XGetImage 영역의 실제 비균일 밝은 glyph 픽셀 조건까지다. Window 생성/MapState만으로 paint를 세지 않았다. 영역·10ms poll·warm filesystem cache·매회 fresh empty app-data/media/XDG·private bus 준비를 timer 밖에 두는 조건은 원 JSON에 기록했다. 각 warm-N/first-visible.png를 보존하며 warm-2 PNG도 직접 확인했다. IME/reader/paint 수집은 별도 화면·fixture에서 병렬 수행했지만 같은 CPU/호스트를 공유했으므로 전용 성능 호스트가 아니다.

AMD Ryzen7 9700X/16 logicalCPU, 폰트 SHA256 `7a183cf1c6c56b9609fcc16eda8b5229fbc11758a21e669ec00343239b02192f`. 100% Xvfb1280x900/창960x640. 200%는 WINIT_X11_SCALE_FACTOR=2 및 Xvfb2200x1400/소유 창의 positive resize1600x1200→800x600logical 조건이다.

[200% 노드 원 관측](cpu-hidden-init-native/paint-scale/scale-200/result.json), [bounds/focus/정제 보고서 판정](cpu-hidden-init-native/paint-scale/review.json): 관측 named widget15개의 GetExtents가 모두 viewport 내부, URL 입력 뒤 Tab의 tool entry FOCUSED 확인. 실제 합성 외부 도구 exit7 실패를 engine으로 만들고 failure header/review/copy/preview와 footer가 보이는 [report.png](cpu-hidden-init-native/paint-scale/scale-200/report.png)를 직접 확인했다. GetText 보고서에서 fixture-secret 원문은 없다. 보고서 마지막줄까지 scroll은 미검증이다. resize 이전 200% 첫 native glyph 관측은0.235637초이고 resize 이후 800x600 최초paint를 별도로 재지 않았다.

## 실행·격리·한계

기존 scripts/accessible_ime_probe.py, scripts/accessible_reader_probe.py, scripts/cpu_scale_paint_probe.py에 동일 바이너리와 fresh output prefix `docs/validation/cpu-hidden-init-native/{inspect,ime,reader,paint-scale}` 및 기존 읽기 전용 font를 지정했다. inspect에만 --inspect, 다른 시험은 기본옵션이다. source/Cargo/build/기존 harness 수정 없음. 소유 Xvfb/private D-Bus/AT-SPI/temp XDG/GSettings memory 및 app-data/media sibling, 기존 프로젝트 전용 Fcitx/무음 Orca/synthetic tool만 사용했다. 모든 소유 직접 child 회수 및 임시 fixture 제거는 원 결과 true다. 기존 장시간 soak·사용자 desktop/audio/clipboard/실제 영상/문서 폴더에 손대지 않았다.

첫paint1초 기준은 유지했다. 3 OS/실제multimonitor DPI/전체reader 사용자 경험/음성/보고서 전체scroll·keyboard/전체제품parity/전체P0 완료는 미검증이다. GL 부재·RSS/CPU 성능 측정은 root의 별도 evidence를 따른다. 이번 범위는 Linux hidden-init 바이너리의 핵심 native 재검증이다.

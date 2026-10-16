# 자막 선택 native GUI 격리 검증

2026-10-01. [작업 계약](../work-items/S04-subtitle-ui-probe.md), [harness](../../scripts/subtitle_ui_probe.py).

실행한 Linux release binary SHA256는 `7e3afa82764e4bc52b254f337db39c81431efede22c7dce9b3bb2aa8aafdfa3d`다. 주 담당이 빌드했고 harness는 빌드하지 않았다. Xvfb960x640/100%, 명시적으로 읽은 기존 Malgun.ttf, 임시 sibling app-data/synthetic-media와 합성 yt-dlp만 사용했다. 실제 영상·계정·사이트 요청과 visible desktop fallback은 없다.

```sh
DLPAD_TEST_FONT=<fixture-font> python3 scripts/subtitle_ui_probe.py target/release/dlpad /tmp/new-subtitle-report.json --case all --layout docs/validation/subtitle-ui/layout.json
DLPAD_TEST_FONT=<fixture-font> python3 scripts/subtitle_ui_probe.py target/release/dlpad /tmp/new-subtitle-auto-off.json --case ko --auto off --layout docs/validation/subtitle-ui/layout.json
```

[Layout](subtitle-ui/layout.json)은 실제 screenshot로 확인한 해당 font/viewport 범위다. 다른 font/layout은 `--inspect`의 top/controls/range screenshots로 먼저 확인한다. 양쪽 구간 입력은 controls-end에서 scroll-up3 후의 좌표다. 인자 검증은 GUI 성공 증거가 아니다.

| 실제 case | 확인 | 시간/증거 |
|---|---|---|
| 저장 안 함 | 자막 옵션/argv 없음, 합성 영상1 artifact |8.272초·[JSON](subtitle-ui/none.json)·[계획](subtitle-ui/none.png)|
| 한국어+auto | options Korean/true, --sub-langs ko/--write-auto-subs, 영상+VTT2 artifacts |8.239초·[JSON](subtitle-ui/ko.json)·[계획](subtitle-ui/ko.png)|
| 영어+auto | options English/true, --sub-langs en/--write-auto-subs,2 artifacts |8.242초·[JSON](subtitle-ui/en.json)·[계획](subtitle-ui/en.png)|
| 한국어와 영어+auto | options KoreanAndEnglish/true, ko,en/auto,3 artifacts |8.324초·[JSON](subtitle-ui/both.json)·[계획](subtitle-ui/both.png)|
| 한국어/auto 꺼짐 | --write-subs/ko, --write-auto-subs 없음, options false |[JSON](subtitle-ui/ko-auto-off.json)·[계획](subtitle-ui/ko-auto-off.png)|
| 구간 충돌 | GUI0→1초 입력+both 선택 후 preview/start, nojob/noexecute |9.970초·[JSON](subtitle-ui/conflict.json)·[입력](subtitle-ui/conflict-range.png)·[오류](subtitle-ui/conflict.png)|
| 실행 중 freeze | none/auto 클릭에도 원래 snapshot/options 보존→Cancelled; 다음 GUI Start도 both/true로 Downloaded/3 artifacts |9.271초·[JSON](subtitle-ui/freeze.json)·[실행 중](subtitle-ui/freeze-running.png)·[계획](subtitle-ui/freeze.png)|

실제 실행 argv를 합성 executable이 기록하고 DB snapshot/options와 선택을 자동 대조했다. 최종6case는 publication artifact 개수·임시 media 내부 경로·정확 size와 합성 VTT bytes도 대조했다. preview 화면을 직접 검토해 both의 `--write-subs`, `--sub-langs ko,en`, `--write-auto-subs` 및 URL 생략을 확인했다. 구간 오류 화면에 “자막 저장은 현재 전체 영상에서만 사용할 수 있습니다”가 표시되고 running 화면에는 모든 자막 버튼 비활성/선택 both+auto 보존을 확인했다. 자동 OCR은 없으므로 conflict JSON의 native_gui_verified=false는 유지한다. nojob/noexecute는 자동 확인이며 오류 문구는 이미지 직접 검토 결과다.

첫6case도 성공했고 이후 freeze를 강화했다. immutable journal만으로 latent UI 상태 보존을 주장하지 않고 취소 후 실제 두 번째 요청까지 대조했다. Ko auto-off를 추가했다. 최종6case는 동일 binary에서 재시험했고 오류가 없었다. AST/인자10건(정상 all/off/inspect와 layout누락/NaN/초과/invalid case/desktop/inspect-all 거절), 합성 tool version/metadata/ko/en/both framing5회도 통과했다. 인자 시험은 GUI나 사이트 요청을 하지 않았다.

가상 화면 외 OS GUI, IME/screen reader, 실제 사이트 자막/번역/언어 부재/auto source 구분은 이 harness의 검증 범위 밖이다. Synthetic media는15 byte fixture이며 실제 영상 decode 성공을 뜻하지 않는다. engine 실제 yt-dlp/FFmpeg loopback 자막 시험은 주 담당의 별도 기록을 따른다. Binary가 바뀌면 해당 SHA의 GUI 결과를 새 build에 자동 적용하지 않는다. 전체 P0/3OS 출시 완료를 의미하지 않는다. 주 담당의 통합 독립 검토가 남는다.

# CPU 파일 목록 실제 GUI 검증

관측일: 2026-10-03. 이 검증은 Linux CPU 후보의 파일 목록 범위이며 전체 P0/다른 OS/전체 reader 사용성을 완료로 판정하지 않는다.

## 실행 계약과 환경

제품 바이너리 `target/release/dlpad-cpu`: SHA256 `185442b2dd209aa8e2ce43251c5ef695c835fe1c60c33c4de3bfb0d15d7971ad`, 10,359,544 bytes. 시작/끝 hash 동일. 제품/source/Cargo/공유 probe를 변경하지 않았다. Xvfb 960×640, scale 1, 별도 D-Bus/AT-SPI, temp XDG/data/media를 사용했다. 읽기 전용 Malgun font SHA256 `7a183cf1c6c56b9609fcc16eda8b5229fbc11758a21e669ec00343239b02192f`를 사용했다. 합성 `.mp4` 10,000개는 각각 `dummy` 5 bytes이며 실제 영상이 아니다. 사용자 desktop/clipboard/audio/Documents를 사용하지 않았다.

실행 명령은 다음 형태로 세 번 수행했다.

```text
python3 scripts/cpu_library_probe.py target/release/dlpad-cpu docs/validation/cpu-library-gui/<prefix> --font /mnt/c/Windows/Fonts/malgun.ttf
```

`first` exit 0, `corrected` exit 1, `close-instrumented` exit 1. 최초 collector exit 0을 모든 인수 조건의 성공으로 해석하지 않는다. 이후 collector는 CloseWait 미확인도 실패 exit로 반환하도록 보강했다. `--validate-only` exit 0이며 프로세스를 생성하지 않았다. 모든 실제 실행의 직접 소유 앱/bus는 회수됐고 Xvfb와 임시 fixture도 종료/제거됐다.

## 관측 결과

- 실제 파일 목록 탭 클릭 후 DB의 present rows 10,000개를 확인했다. `corrected`에서 click→DB commit 0.125129초, CPU 8 ticks, RSS 51,000→56,332 KiB. 이는 첫 paint나 peak RSS 측정이 아니다.
- 첫 페이지 top 00000–00017/end 00112–00127, Next 후 top 00128–00145/end 00240–00255를 AT-SPI Text와 PNG로 확인했다. 화면에는 virtualized 일부 행만 노출된다. 전체 page를 AT-SPI에 동시에 128행 노출해야 한다고 요구하지 않는다.
- 소유 fixture의 add/rename/delete 후 present count 10,000과 해당 새 경로/삭제 경로 부재가 수렴했다. explicit Refresh도 클릭했으나 watcher와 함께 동작하므로 수렴을 Refresh 단독 효과로 분리하지 않았다.
- 정상 root 적용 후 data와 중첩되는 invalid root Apply는 기존 inventory와 persisted `library_root`를 보존했다. 실패 안내와 기존 파일 행을 실제 PNG/Text에서 확인했다. root 입력값은 사용자가 시도한 invalid 값으로 남는다.
- 저장 폴더 entry에서 물리 Tab 후 `저장 위치 적용`의 AT-SPI FOCUSED bit 12를 확인했다. label/한글 파일명이 tree에 존재한다. 이번 시험은 Orca 전체 사용성 시험이 아니다.

## CloseWait 미충족과 원인분리

`first`는 owned SQLite `BEGIN IMMEDIATE` 후 Refresh, WM_DELETE_WINDOW, 고정 좌표 StopExit를 보냈다. 앱은 lock release 전 exit 0이므로 scan busy/CloseWait 증거가 아니다. 실제 busy 진입과 고정 좌표 버튼의 event 수신을 입증하지 못했다.

`corrected`는 `BEGIN EXCLUSIVE`와 Next ENABLED bit 8 제거를 busy proxy로 사용했다. Next는 enabled 상태를 유지하여 1.5초 관측 deadline에서 timeout됐다. `close-instrumented`는 Refresh 재클릭 및 fresh AT-SPI tree를 추가해 stale-node 가능성을 줄였으나 같은 결과였다. 두 경우 busy assertion 이후 종료 이벤트 단계로 진입하지 않았다. 따라서 이 결과로 제품 CloseWait 성공 또는 실패를 단정하지 않는다. raw timeout과 최초 close=false는 모두 보존한다.

다음 안전한 실험은 Refresh의 실제 command serial/pending 진입을 확인할 bounded 관측 수단 또는 press/release가 별도 UI frame에 처리되는 키보드 activation을 사용해 scan busy를 먼저 확정하는 것이다. 이후 실제 닫기 안내의 AT-SPI button을 확인해 StopExit를 보내고, lock 보유 중 생존/해제 후 종료를 비교해야 한다. 현 문서 작성 시 제품 관측 API를 추가하지 않았다.

첫 목록 scroll 미관측은 collector 원인이었다. 좁게 자동 축소된 목록 밖 x=500에 wheel을 보냈다. owned 목록 안 x=150으로 고친 후 페이지 end가 확인됐다. 첫 raw를 수정하거나 제품 수정으로 해결했다고 표현하지 않는다.

## 보존 증거

- 간단 요약: `cpu-library-gui/summary.json`.
- 최초 raw: `cpu-library-gui/first/result.json`, PNG와 inventory.json.
- 수정 scroll/주 결과: `cpu-library-gui/corrected/result.json`, `page1-top.png`, `page1-end.png`, `page2-top.png`, `page2-end.png`, `external-refresh.png`, `invalid-root.png`.
- fresh busy tree/timeout: `cpu-library-gui/close-instrumented/result.json`의 `tree_scan_busy`, lock mode, Refresh timestamp, 원 오류.

작업 수집은 종료됐으나 CloseWait GUI 인수 조건은 미충족이다. 주 담당에게 원시 경로와 한계를 전달했으며 최종 제품 판정은 통합 검증 담당이 수행한다.

## 후속 CloseWait gate 결과

새 CPU SHA8fb4a67c27ff89fd883ff6fddaa9136a5fc36c62d74c049665543a513ab06986에서 독립 관측 대조를 수행해 CloseWait 외부 동작을 확인했다. button ENABLED bit가 실제 disabled를 반영하지 않는 pinned adapter mapping을 확인했고 text entry 상태로 busy를 확인했다. 원 미충족 raw는 그대로 보존한다. 상세: `cpu-library-closewait-validation.md`.

# CPU 목록 CloseWait 관측 원인분리

2026-10-03 Linux actual GUI. CPU binary SHA256 `8fb4a67c27ff89fd883ff6fddaa9136a5fc36c62d74c049665543a513ab06986`, 10,437,144 bytes. 이전185442 결과를 새 binary 검증으로 확대하지 않았다. 제품/Cargo/shared probe 변경 없이 신규 narrow harness만 추가했다. owned Xvfb/private D-Bus/AT-SPI/temp XDG, 10,000개5-byte 합성파일, 읽기 전용 기존 Korean font를 사용했다. 사용자 desktop/audio/clipboard/실제영상 사용 없음.

## 원인

Pinned `accesskit_atspi_common-0.14.2/src/node.rs` 348–352는 `is_read_only_supported && is_read_only_or_disabled` 조건에서만 ReadOnly를 설정하고 그 밖에는 Enabled|Sensitive를 설정한다. 일반 button은 read-only 지원 대상이 아니므로 disabled여도 이 mapping의 ENABLED bit가 남는다. egui 자체는 response.rs에서 disabled를 AccessKit에 전달한다. 따라서 **Next button ENABLED 제거를 busy의 필수 proxy로 사용한 collector가 잘못됐다**. 제품 pending이 없었다고 해석하지 않는다.

실제 `action` 대조에서 Refresh의 AT-SPI GetActions는 click, DoAction(0)은 true였으며 fresh GetState 각각4–6ms로1.5초간 직접 조회했다. full-tree 수집은1.571초였다. 이번 관측에서는5초 SQLite timeout보다 먼저 실패했으므로 full-tree가5초를 모두 소진했다는 가설은 원인으로 채택하지 않았다. button proxy timeout raw와 기존185442 raw를 보존했다.

## 수정된 관측과 실제 결과

`entry-state` 대조는 lock 전에 named Refresh/저장 폴더 Text entry를 찾았다. 이후 full-tree를 사용하지 않고 cached node 경로에 **fresh GetState 요청**을 보냈다. AT-SPI DoAction은 실제 accessible GUI action이며 제품 worker/DB API 직접 호출이 아니다.

1. SQLite BEGIN EXCLUSIVE 획득(owned fixture only).
2. Refresh DoAction(true),4.42ms.
3. 약35ms 후 저장 폴더 entry 상태 `[1191184768,64]`→`[1174407296,2112]`: ENABLED bit8 제거, READ_ONLY bit43 추가. 실제 busy에서 root editable이 false가 됐다는 UI 증거.
4. WM_DELETE_WINDOW 요청. 최소 properties/children 조회로 실제 named `중단 후 종료` 노드를297ms에 발견했다. 고정좌표나 잘못된 `중지 후 종료` 문자열을 사용하지 않았다.
5. StopExit DoAction(true),3.89ms.150ms 뒤 lock 보유 중 앱 생존=true.
6. lock 해제 후116ms에 앱 정상 exit0. 그때까지 SQLite의5초 busy timeout은 도달하지 않았다.

즉 실제 busy prompt와 StopExit를 거쳐 worker가 blocking SQLite 작업에서 빠져나올 때까지 앱이 종료되지 않는 외부 동작을 확인했다. 내부 StopAck atomic을 직접 읽지는 않았다. 강제 kill/다른OS/모든 timing의 종료를 검증했다고 주장하지 않는다. 이 narrow 범위의 CloseWait gate는 충족한다.

## 명령과 증거

```text
python3 scripts/cpu_library_close_probe.py target/release/dlpad-cpu docs/validation/cpu-library-closewait/action --font /mnt/c/Windows/Fonts/malgun.ttf
python3 scripts/cpu_library_close_probe.py target/release/dlpad-cpu docs/validation/cpu-library-closewait/entry-state --font /mnt/c/Windows/Fonts/malgun.ttf
```

첫 action 대조 exit1(proxy timeout), entry-state 대조 exit0. 증거는 `cpu-library-closewait/{action,entry-state}/result.json`: 각 호출 시작·duration·원 reply, busy/close/lock timestamps, 정상종료와 SHA. 각 prefix의 downloads.png/isolated.json/app.log/bus.log도 보존했다. busy prompt는 AT-SPI named node와 실제 DoAction으로 확인했으며 PNG를 수집하지 않았다. full tree를 lock 중 수집하지 않는 설계다.

`source-before.json`/`source-after.json`은 최초 대조 시 snapshot이며 요청된 diagnostics.rs 경로는 실제 존재하지 않아 null, 실제 파일 `accessible_diagnostics.rs` hash를 별도로 기록했다. `source-after-entry.json`은 마지막 수집 뒤 snapshot이다. parent에게 최초 대조 종료 뒤 test-only Windows fixture collision fix를 허용했으므로 향후 source 변화와 frozen 실행 binary를 혼동하지 않는다.

모든 시험 앱/bus 직접 child와 Xvfb를 종료하고 임시 fixture를 제거했다. 각 실행의 시작/끝 binary SHA 동일. 최종 제품 source/통합 test 판정은 root 담당이다.

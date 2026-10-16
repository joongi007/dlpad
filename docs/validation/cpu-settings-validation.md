# CPU 설정 실제 GUI 검증

관측일2026-10-03, Linux CPU 후보. 최종 SHA256 `330096e38bedd88fede1751dd84ec11ac518c00c9a2a47ac1a84b201617aaa1b`, 10,489,368 bytes. 제품/Cargo/공유 probe를 변경하지 않고 독립 `scripts/cpu_settings_probe.py`로 시험했다. root의38 tests/clippy/release READY와 별도로 actual GUI를 수행했다.

## 환경과 실제 명령

owned Xvfb2200×1400, 기본 native window960×640, private D-Bus/AT-SPI, temp data/media/XDG. 이후 앱의 화면 배율200%를 실제 선택하고 창을1600×1200으로 positive resize했다. 이는800×600logical equivalent 조건이며 실제 multi-monitor DPI 시험이 아니다. 기존 Malgun font SHA256 `7a183cf1c6c56b9609fcc16eda8b5229fbc11758a21e669ec00343239b02192f`. 사용자 desktop/audio/clipboard/Documents/실제영상 사용 없음.

```text
python3 scripts/cpu_settings_probe.py target/release/dlpad-cpu docs/validation/cpu-settings/popup-fixed --font /mnt/c/Windows/Fonts/malgun.ttf --ready-sha 330096e38bedd88fede1751dd84ec11ac518c00c9a2a47ac1a84b201617aaa1b
```

실제 exit0. 직접 소유 앱/bus 및 Xvfb 종료, 임시 fixture 제거. 시작/끝 binary hash 동일. READY hash 불일치는 실행 전 거절한다. help/validate-only/py_compile도 수행했으며 validate-only는 GUI를 시작하지 않는다.

## 확인한 동작

- 설정 탭의 실제 ComboBox(role11) 좌표를 접근성 extents로 찾아 물리 click, popup option의 실제 AT-SPI action으로 어둡게/밝게/시스템을 선택했다. 각 선택 뒤 다른 option node가 사라져 **Escape 없이 popup 닫힘**을 확인했다. 화면 배율200%의 option 선택 뒤에도175% 노드가 사라졌다.
- SQLite `settings.theme`가 각각 JSON `"Dark"`, `"Light"`, `"System"`이 될 때까지 관측했다. 이후 정상 닫기 exit0, 동일 data로 재실행했다.
- 재실행 **first observed painted frame**의 native glyph predicate와 blank background BGR은 Dark27/Light248/System27이다. 시간은 각각0.224979/0.216169/0.217881초이며50ms polling 기반이다. 이것이 compositor의 첫 presentation이었다고 증명하지 않는다. 시작 window 생성만으로 paint를 판정하지 않았으며 실제 XGetImage glyph/color를 읽었다. System은 이 격리 환경의 dark 관측이며 실제 OS theme-change event를 시험하지 않았다.
- 200% PNG에서 한글 entry physical bounds `(16,405,1568,43)`는1600×1200 안에 있고 아래 설명도 보인다. 하단 clipping 없음. 실제 entry click 후 `abc` GetText와 FOCUSED bit12, ShiftTab 후 `화면 배율` FOCUSED, Tab 후 같은 entry FOCUSED를 확인했다. 단순 임의 focus 발견으로 대체하지 않았다. `abc`는 ASCII 입력으로 실제 한글 preedit/commit/IME 검증이 아니다.
- 앱 배율은 현재 session-only 계약이며 영구 저장을 검증했다고 주장하지 않는다.

## 저장 중 닫기

기존theme.sqlite에 owned BEGIN EXCLUSIVE lock을 획득하고 System→Light UI 선택을 보냈다. 아직 DB에 저장될 수 없는 상태에서 WM_DELETE_WINDOW와 실제 `중단 후 종료` action을 수행했다.150ms 뒤 lock 보유 중 앱은 살아 있었고, lock 해제 후169ms에 exit0이었다. 최종 SQLite theme는`"Light"`. 저장을 완료한 뒤 닫는 앞선 세 시험과 별도로 **save-in-flight drain/저장 유지**를 확인했다. 내부worker ack를 직접 읽지 않았고 blocking SQLite의5초 timeout 전 해제한 범위다.

## 원 실패 보존과 수정 원인

이전 SHA52b55의 `first`, `actions`, `physical-combo`, `physical-fixed`, `role-filtered`, `final`, `keyboard-save-close` raw를 모두 보존했다. 최초 label과 ComboBox는 같은 Name이었고 label도 GetActions(click)이어서 role29를 명시 제외해야 했다. origin tuple→dictionary 호출 계약 오류도 collector 오류로 보존했다. 한글 field는 새 후보에서 Settings로 이동하여 이전 download-tab 가정이 실패했다.

role-filtered/final의 접근성 option 선택은 저장에는 반영됐지만 popup이 남았다. 실제 `final/scale200.png`에는 menu가 field와 겹친다. root가 pointer.any_click에 의존한 popup close 경로를 확인하고 제품에 명시 close를 구현했다. 이 원시 현상을 최종 성공으로 덮어쓰지 않는다.

keyboard-save-close의 abc 실패는 owned window 재실행 뒤 WM 없는 환경에서 keyboard focus를 확보하지 않은 collector 누락도 포함했다. 새 최종 harness는 매 launch 후 focus_owned_window를 호출하며 실제 entry GetText/양방향 focus를 확인한다.

## 증거와 남은 게이트

간단 요약 `cpu-settings/summary.json`; 최종 raw `cpu-settings/popup-fixed/result.json`; 실제 화면 `Dark-first.png`, `Light-first.png`, `System-first.png`, `scale200.png`, `scale200-keyboard.png` 및 theme popup PNG. source-final.json은 관측 뒤 source hash이며 build provenance 자체를 대체하지 않는다.

이 설정 화면의 scoped Linux 검증만 통과했다. 실제 한글 IME를 이동된 Settings field에서 다시 검증하는 일, 전체 reader 사용성, System OS 변경 실시간 반영, 다른 OS, 정확한 첫 presentation은 남는다. 최종 제품/P0 판정은 root 통합 검증 담당이 수행한다.


## styles32 후속 최종 후보와 이동 필드 IME

새 SHA256 `f052fa5b3803c06b97f82ef360b1ad11776c6c3b0854e9fc20e00381faa75f19`,10,486,680 bytes에서 settingsfull을 다시 수행했다. `cpu-settings/styles32/result.json` exit0. Light 재실행의 낮은19px ComboBox를 root가 발견해 theme별 all_styles_mut 초기화를 수정한 후, 실제 theme ComboBox extents는 `(8,91,100,32)` 세 번,100%scale ComboBox `(8,146,100,32)`,200%theme ComboBox `(16,183,200,64)`로 모든>=32 assert를 통과했다. 저장/first observed painted theme/200%focus/save-in-flight 인수 조건도 재통과했다. lock 해제 후175ms 정상 종료/Light 저장. 이전330096 및52b55 raw는 그대로 보존한다.

실제 이동 Settings field IME는 다음 명령으로 같은 binary에서 수행했다.

```text
python3 scripts/accessible_ime_probe.py target/release/dlpad-cpu docs/validation/cpu-settings/ime-settings --font /mnt/c/Windows/Fonts/malgun.ttf
```

private Fcitx5/libhangul/XIM server, 별도bus/Xvfb/temp만 사용했고 사용자IMEdaemon/audio/desktop/clipboard를 변경하지 않았다. field는 named role79 GetExtents `(8,201,944,21)`로 발견해 실제point `(480,211)`을 사용했다. ASCII focus `abc`, physical `gksrmf` Korean composition→space commit `한글 `,Backspace composition cancel 후`한글 `,Latin전환`한글 abc`,전체선택replace`xy`를 GetText로 확인했다. preedit.png 직접 검사에서 entry 안의 `한글`,마지막`글`selection/caret를 확인했다. root에도 실제PNG리뷰를 요청했다.

이 IME collector는 화면 판정을 자동화하지 않아 raw exit1 및 preedit_verified=false/ime_verified=false를 반환했다. raw result.json을 성공값으로 고치지 않았다. 별도 `ime-settings/preedit-review.json`에 실제 시각 판정과 raw 이벤트 부재를 구분했다. 제품에 bounded raw IME trace가 없으므로 raw_preedit_event_verified=false를 유지한다. 성공한 commit/cancel/Latin/selection과 inline 시각 증거 범위의 실제IME 결과이며 전체reader/모든OS/200%KoreanIME로 확대하지 않는다.

두 GUI 모두 종료하고 소유직접child/temp를 회수했으며 두 시험 시작/끝SHA가 동일하다. 새 요약은 `styles32-summary.json`, source 관측은 `source-styles32.json`이다. 기존330096 summary/source 증거를 덮어쓰지 않았다.

## root 독립 검토 및 후속 gate

root가 preedit.png를 직접 열어 Settings100%의 한[글]과 마지막 글의 파란 조합선택/caret를 독립 확인했다. raw exit1/preeditfalse/imefalse는 유지한다. 조건부 Linux 실제IME 관측으로 기록한다. 현재 input 높이는21logicalpx로 DESIGN 기본32px보다 작다. ComboBox32px 수정 범위는 통과했으나 typed options 통합에서 일반/IME field min_size32를 개선하고 실제GUI를 다시 확인해야 한다. startup settings busytimeout 때문에 첫 표시가5초 지연되는 경로와 실제 System OS theme-change event는 이번GUI로 측정하지 않았다.


## input32 후속 최종 후보

새 SHA256 `2459ba126c6bcc13470b470c8a384d9d8ecef779893fd832411db0a78932f559`,10,508,440 bytes에서 `input32` settings 전체 검증 exit0. 실제 Settings fixture input100% 높이32px를 각 theme에서 확인하고200% `(16,405,1568,64)`를 확인했다. 별도 실제 Library tab 단계의 저장 폴더 role79 bounds `(8,91,288,32)`를 확인했다. 이전21px 입력 높이 개선 gate는 이 범위에서 해결됐다. theme/popup/재실행 first observed painted frame/양방향 keyboard focus/save-in-flight도 재통과했으며 lock 해제 후222ms exit0/Light 저장. 새로운 요약은 `input32-summary.json`, LibraryPNG는 `input32/library-root32.png`이다.

같은 binary에서 기존 actual private Fcitx IME harness를 `ime-input32` 새prefix로 수행했다. named role79 field `(8,201,944,32)`, actualpoint `(480,217)`에서 ASCIIabc/Koreancommit`한글 `/cancel`한글 `/Latin`한글 abc`/selectionreplace`xy`를 확인했다. preedit.png 직접 시각 검토에서32px 입력란 내 한[글]과 마지막 글의 파란 조합선택/caret를 확인해 root 독립 리뷰를 요청했다. raw exit1/preeditfalse/imefalse/raw-eventfalse를 그대로 보존하고 `preedit-review.json`에 조건부 Linux100% 시각 검토를 분리했다. 다른OS/200% 실제KoreanIME/전체reader/SystemOSevent/startupbusytimeout5초firstdisplay의 미측정 범위는 유지한다.

두 실행의 binary hash 불변, 모든 소유 앱/Fcitx/bus/Xvfb 및 임시fixture 종료/제거. 새GUI를 추가 실행하지 않았고 root 성능측정 슬롯을 해제했다. 기존 모든 prefix 증거는 보존했다. source 관측은 `source-input32.json`이다.

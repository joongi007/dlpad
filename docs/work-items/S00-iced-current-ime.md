# S00: 최신 제품 Iced 한글 IME 실증

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R11, S00
- 담당/수정 범위: diagnostic_cache; scripts/ime_probe.py, 이 신규 계약·docs/validation/iced-current-ime-validation.md 및 전용 증거
- 선행 작업: root winit Start없는Update patch와제품release SHA7e5fe814…cd469

## 사용자 결과

실제제품 Iced 입력에서inline한글조합·정확commit·취소·Latin·선택대체를검증한다. egui의통과를제품으로확대하지않는다.

## 계약과 범위

README/PRODUCT/CONTEXT/요구사항/ADR-0001과기존S00 evidence를따른다. ownedXvfb/privatebus/tempXDG/Fcitx와읽기전용폰트만사용한다. 제품/Cargo/vendor/공통문서수정·desktopfallback은금지한다. 현재binaryhash를고정해새prefix로실패도보존한다.

## 인수 조건

actualinline조합PNG, 정확한글commit/BackSpace취소/Latin/키보드selection replacement, 외부IMwindow분리와도구/GUI회수, binaryhash·미검증OS/reader를기록한다. pixelheuristic를시각검토로대체하지않는다.

## 검증과 handoff

현재제품hash일치확인. 기존probe좌표/heuristic를최신화하기전inspect화면부터수집한다.

[실제 제품 검증](../validation/iced-current-ime-validation.md) 완료: SHA7e5fe814…cd469/9873856B, inline한글/underline·정확commit·BackSpace취소·Latin·selection·GTK동일Escape통과. rawcollector exit1/pixel45·126보존과수동PNG판정을분리했다. ASTCLI4/PNG상한거절통과,ownedteardown/temp삭제확인. actualreader/privatebus가능성은설치없는read-only조사로보고했으며Orca/pyatspi가없고제품tree/실제reader/다른OS는남는다. 제품/Cargo/vendor수정없음.

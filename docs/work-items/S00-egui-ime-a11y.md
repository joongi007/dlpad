# S00: egui alternative 격리 IME·접근성 실험

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R06, R08, R11, S00
- 담당/수정 범위: diagnostic_cache; 신규 scripts/egui_ime_probe.py와 전용 validation/evidence/이 계약. egui_spike.rs/Cargo/build는 root.
- 선행 작업: S00-current-audit, 기존 alternative feature 및 Fcitx/Xvfb 도구

## 사용자 결과

Iced inline preedit/접근성 게이트에 대해 다른 Rust native toolkit의 실제 동작과 비용을 비교한다.

## 계약과 범위

기존 native_probe/ime_probe read-only helper를 재사용한다. owned Xvfb/private D-Bus/temp XDG/Fcitx만 사용하며 사용자 desktop·IME설정·폰트설치 변경은 금지한다. 실제 reader가 없으면 AT-SPI tree 공개까지만 보고한다. 후보 스택을 확정하지 않는다.

## 인수 조건

candidate binary hash/bytes, 실제 key compose/commit/cancel/Latin과 inline preedit 이미지, 외부IM창 inventory, bounded AT-SPI role/name/text 증거를 기록한다. 초기 오류와 미검증 OS/reader를 보존한다. 소유 프로세스/화면만 회수한다.

## 검증과 handoff

[실제 실험](../validation/egui-ime-validation.md) 완료: 2candidate hash/bytes 고정, ownedXvfb 실제Hangul 입력과실패exit1, AT-SPI29node/role79/name/text 공개, boundedtrace에서Commit한/글도착·Enabled/Preedit부재를확인했다. 실험완료이지S00/stack acceptance가아니다. root가후속cursor/XIM경계를조사하며다른OS/실제reader는미검증. AST/CLI6건통과, 제품소스/Cargo/기존probe변경없음.

후속가설검증: root가fixtureonly/defaultoff inactiveCommit직전Enabled주입옵션을소유한다. agent는--commit-fallback으로원동일키입력/취소/Latin 및Ctrl+A selection replacement를새prefix에비교한다. inlinepreedit없는gate는완화하지않는다.

fixture fallback대조완료: 같은69dd5adf…831f binary에서defaultoff한만commit/on은한글commit·취소·Latin·selection통과. 양쪽inlinepreedit실패로전체IMEfalse/exit1 유지. actualAT-SPI29node/rawCommit글도착증거와스크린샷을전용validation에보존했다. 7CLI검증통과. 구현/실험범위완료이며S00채택gate아님.

root승인후속: vendorwinit의Start없는nonemptyUpdate처리를defaultfallbackoff후보에서실증한다. precommitAT-SPI rawPreedit와actualPNG의마지막글/selection/caret를검토하고mirror동일조건은인수조건으로사용하지않는다. exactcommit/취소/Latin/selection을유지하며제품스택확정은별도다.

승인된vendor-startless후보시험완료: e99af18a…aeaa/8806208B, fallbackOFF/actualPreedit와inline글·selection/caret를직접검토했고정확commit/취소/Latin/selection/AT-SPI통과. 원collectionexit1은수동visualgate전결과로보존, 검토후인수assertion통과. 전용validation에수동판정출처를기록했다. 제품Iced/3OS/reader/stack acceptance는미검증. 최신범위에따라새S00확장없이결과보고후멈춘다.

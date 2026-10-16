# S00 egui alternative 실제 격리 IME·AT-SPI 검증

2026-10-02. [계약](../work-items/S00-egui-ime-a11y.md). 제품 Iced/Cargo/기존 probe는 수정하지 않았으며 신규 `scripts/egui_ime_probe.py`만 구현했다. root가 기존 alternative feature와 `--settings` 우선선택을 빌드했다. 이 실험으로 제품 스택을 채택하지 않는다.

## 현재 판정

candidate egui/eframe0.33.3 Glow + AccessKit, winit0.30.13, SHA256 `8de24f4a1d526538698dee0482d71e48af7b2450a9a1c5da80febaca507e6769`,8,809,792B. owned Xvfb/private D-Bus/Fcitx5 Hangul/XIM/960×640/100%와 읽기 전용 Malgun fixture font. 실제 window settings/glyph/ASCIIfocus는 통과했다. **IME 필수 인수 조건은 실패한다.** AT-SPI tree 공개는 실제 관측했지만 실제 screen reader는 사용하지 않았다.

[inspect](egui-ime/inspect-wait/result.json)와 [초기 화면](egui-ime/inspect-wait/initial.png)은 Window/한글 글리프를 보여준다. 이는 IME 성공이 아니다. [bounded-final](egui-ime/bounded-final/result.json)의 `ascii_focus_text=abc`는 keyboard focus를 확인했다. 실제 Hangul 활성 상태에서 물리 `gksrmf` 뒤 Space의 결과는 기대 `한글 ` 대신 `한 `였다. key 사이0.12초에서 추가0.25초로 느리게 보내도 동일했다. [preedit 화면](egui-ime/bounded-final/preedit.png)을 직접 검토해 input과 committed mirror 둘 다 `한`이며 마지막 `글`이 inline에 없음을 확인했다. window inventory의 별도 Fcitx Input Window는 (0,650,27,31)로 앱640px 밖이다. 외부 IME 창을 inline 성공으로 세지 않는다.

BackSpace 취소와 Latin 전환은 현재 commit 기대값 기준으로 모두false다(`한 `, `한 abc`). 이 결과를 취소/전환 독립 제품결함으로 단정하지 않는다. 첫 commit 실패 때문에 전체 정확문자열 gate가 미충족이다. clipboard는 key 입력을 검토하기 위해 읽었으며 Unicode paste로 시험하지 않았다.

## 접근성 실제 관측

[focus-a11y](egui-ime/focus-a11y/result.json), [bounded-final](egui-ime/bounded-final/result.json)의 private org.a11y.Bus 및 AT-SPI registry가 앱19node를 공개했다. application name `dlpad-egui-spike`, 설정/화면테마/배율/한글 입력 등 한국어 Accessible.Name과 Text.GetText를 실제 읽었다. labelled_by 입력 node 이름 `한글 입력 검증`, 마지막 Text.GetText `한 abc`를 확인했다.

처음 GetRoleName은 UnknownMethod였고 AccessKit 소스의 GetLocalizedRoleName으로 바꿨으나 기본 localized role name은 빈 문자열이었다. 이것을 tree 부재로 오해하지 않는다. GetRole 숫자 호출로 후속 검사하며 Text.GetText(0,-1)는 gdbus positional `--`를 넣어 실제 호출했다. 첫 잘못된 gdbus call 결과를 보존한다. `ScreenReaderEnabled=true`는 private adapter 활성화 flag이며 실제 읽기 프로그램/음성/키보드 탐색 검증이 아니다. screen_reader_verified=false를 유지한다.

## 원인 범위와 남은 검사

로컬 pinned egui-winit0.33.3/src/lib.rs는 Linux Ime::Enabled를 무시하고 Preedit(_,None)를 Disabled로 변환하며 Commit 뒤 Disabled를 보낸다. egui0.33.3 TextEdit는 IME 이벤트를 먼저 정렬/처리하고 preedit 선택 영역을 text buffer에 넣는다. event 전달/clear ordering은 조사 가설이며 이번 evidence로 특정 라이브러리 결함을 확정하지 않는다. root는 최대16 UI-only event trace 실험을 준비한다.

실제 reader의 role/state/name/키보드 focus/readback, Windows/macOS/Wayland IME와 접근성, 배율·테마·장시간 GUI 비용은 미검증이다. eframe은 초기software보다RSS가 높은 이전 비교 결과가 있으므로 bridge가 있다는 이유로최종 대안 채택을 선언하지 않는다.

## 실행과 실패 보존

```sh
DLPAD_TEST_FONT=/mnt/c/Windows/Fonts/malgun.ttf python3 scripts/egui_ime_probe.py target/release/dlpad-egui-spike docs/validation/egui-ime/inspect-wait --inspect
DLPAD_TEST_FONT=/mnt/c/Windows/Fonts/malgun.ttf python3 scripts/egui_ime_probe.py target/release/dlpad-egui-spike docs/validation/egui-ime/bounded-final --input-point 300,268
```

AST/CLI6건 통과(desktop인자와범위밖좌표거절, no GUI). 첫실행은 streaminghash helper가file객체를요구하는데Path를넘겨시작전실패했다. helper호출수정. 다음2초startup은창미준비로실패했고5초ownedstartup뒤창을확인했다. composition/composition-ready는 IMEcontext미활성으로실패했고실제ASCIIfocus를먼저확인하자Hangulcontext에접속했다. 모두새prefix를써서기존증거를덮어쓰지않았다.

bounded-final 당시harness는오류없는수집을exit0로처리했으나IMEflag는false였다. 이후IMEacceptance가false면exit1이되도록수정했다. 성공으로승격하지않는다. 모든시험은소유Xvfb/privatebus/IME/app만실행하고tempXDG를정리했다. 사용자desktop/실제영상/계정/사용자설정을사용하지않았고fallback은없다. 기존사용자AT-SPI helper를종료하지않았다. 제품의공통cargo검증은root통합담당이며새Pythonharness시험과구분한다.

## 최종 bounded trace 재현

root가 fixture 전용 최대16/latest event UI를 추가한 별도 candidate SHA256 `67247712c8eab8de6f2731c86b5580d0685818271628f3709322db72672fef3b`,8,814,272B로 `--ime-events` 시험을 실행했다. [trace JSON](egui-ime/trace/result.json), [preedit](egui-ime/trace/preedit.png), [committed](egui-ime/trace/committed.png), [Latin](egui-ime/trace/latin.png). **probe exit1/IMEfalse**이며 정확 commit은 여전히 `한 `이다. 화면을 직접 검토해 inline 마지막 음절이 없음을 확인했다.

AT-SPI로 UI trace의 정확한 텍스트를 읽었다: Disabled, Disabled, Commit "한", Disabled, Disabled, Commit "글", Disabled, Disabled, Disabled. 이 bounded 관측에 Enabled/Preedit는 없지만 **Commit "글"은 egui ctx.input까지 실제 도착**했다. 화면 trace는 mono font의 한글 glyph가 없어 tofu지만 AT-SPI Text.GetText에서 한/글을 정확히 확인했다. 따라서 마지막 commit이 winit→egui 사이에서 사라졌다는 가설은 해당 시험과 맞지 않는다. local TextEdit Commit은 current cursor.secondary와 ime_cursor_range.secondary가 같아야 삽입하므로 Enabled/Preedit 부재의 stale cursor가 다음 조사 가설이다. runtime 내부 cursor를 직접 관측하지 않았으므로 원인 확정/수정 완료를 주장하지 않는다.

최종 접근성 tree29nodes가 실제 공개됐고 role GetRole 호출도 성공했다. 한글 입력 node role uint32 79/name `한글 입력 검증`/Text.GetText `한 abc`, 별도label role29도 확인했다. tree_verified=true, screen_reader_verified=false. private reader flag는 adapter를 켠 상태만 뜻한다. root의 [별도10초 성능표본](evidence/S00-egui-library-baseline.json)은 초기candidate 8,809,792B/RSS133636KiB/idle0.0%이며 tracecandidate와 별개다. 표본은 battery/장시간예산이나 IME 성공을 입증하지 않는다.

probe 마지막 AST/CLI6건 통과. 소유app/IME/privatebus child handles 전부회수와tmp fixture삭제를확인했고 Xvfb가종료됐다. 일반제품 Iced binary는교체하지않았다. 현재대안은접근성 tree 가능성이있지만IME 필수인수실패로채택gate가충족되지않는다. 다른OS/실제reader/전체성능비교는남는다.

## fixture-only commit fallback 대조

root가 defaultoff/fixtureonly `--ime-commit-fallback`을 추가한 candidate SHA256 `69dd5adf23f019c0b94c6b96d641a03390180549ba270e106b6792831c0d831f`,8,816,192B에서 같은 물리 키와 selection을 비교했다. 신규harness 인자 `--commit-fallback`만 root옵션으로전달한다. 앱 일반동작과제품 Iced에는적용하지않았다.

| 조건 | commit | BackSpace 취소 | Latin | Ctrl+A→xy 대체 | inline preedit / 전체IME |
|---|---|---|---|---|---|
| [defaultoff](egui-ime/fallback-default-off/result.json) | `한 `,false | false(정확기대값미충족) | `한 abc`,false | `xy`,true | 없음,false/exit1 |
| [fallbackon](egui-ime/fallback-enabled/result.json) | `한글 `,true | `한글 `,true | `한글 abc`,true | `xy`,true | 없음,false/exit1 |

[on preedit](egui-ime/fallback-enabled/preedit.png)와 [on committed](egui-ime/fallback-enabled/committed.png)을 직접 검토했다. Space전 마지막`글`은 input에 없고 mirror도`한`이다. Space뒤에는`한글`이정확히들어왔다. 양쪽 rawtrace는동일Disabled/Commit한/Disabled/Commit글순서이고Enabled/Preedit는없으며AT-SPI29node tree true다. inactiveCommit직전에만Enabled를주입하자텍스트삽입이복구된대조는widget IMEcursor상태가원인이라는가설을강하게지지한다. 내부cursor값/모든OS/다양한selection과composition생명주기를검사한수정완료는아니다. inlinepreedit실패는별도필수gate로남으며스택을채택하지않는다.

AST/CLI최종7건통과,desktop/범위밖좌표거절. 모든ownedapp/IME/privatebus handles회수와temp fixture삭제확인. 문서검사결과는작업계약에기록한다.

## 승인된 Start 없는 Update 후보 검증

root소유 vendor winit0.30.13이 Start없는 nonemptyUpdate도composition으로전달하는후보를빌드했다. candidate SHA256 `e99af18a079211c2267336ca9dd228da7847dd9bfd5be7ec583a831e49e4aeaa`,8,806,208B. `python3 scripts/egui_ime_probe.py target/release/dlpad-egui-spike docs/validation/egui-ime/vendor-startless --ime-events`를기존ownedXvfb/privateFcitx/동일키로실행했고 commitfallback은OFF다.

[후보JSON](egui-ime/vendor-startless/result.json)의 precommitAT-SPI에는 실제 Enabled/Preeditㅎ/하/한/Commit한/Enabled/Preeditㄱ/그/글이있다. [preedit화면](egui-ime/vendor-startless/preedit.png)을직접검토하여입력안의`한글`과마지막`글`의읽을수있는glyph/파란selection/보이는caret를확인했다. egui는조합을self.ime buffer에직접넣으므로mirror에`한글`이나오는것을실패조건으로삼지않았다. [commit화면](egui-ime/vendor-startless/committed.png)은Space후정확`한글 `을보인다. BackSpace조합취소`한글 `, Latin`한글 abc`, [Ctrl+A대체](egui-ime/vendor-startless/selection.png)`xy`가모두true다. precommit/후속AT-SPI32/36node tree가공개됐다.

원harness는preedit수동검토전false를유지하므로collection exit1이었다. 이exit를JSON에보존하고actualPNG직접검토후preedit_verified/ime_verified를true로판정했다. Python으로rawPreedit·commit·cancel·Latin·selection·tree·fallbackoff와3PNG존재를대조한후보인수assertion은통과했다. CLIexit1을exit0로고쳐쓰지않는다. root의독립이미지/JSON검토는통합담당범위다.

이변경전후는Start없는callbackUpdate가버려진경계를해결하는가설을지지한다. 같은Linuxegui후보의시험결과일뿐제품Iced·전체IME/3OS/reader·스택채택으로확대하지않는다. screen_reader_verified=false를유지하고ownedapp/IME/privatebus handles회수/temp삭제를확인했다. 최신사용자범위에따라승인된이시험만마치고새S00실험을확장하지않는다. 기존실패증거는유지했다.

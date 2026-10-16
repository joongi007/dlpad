# S00 최신 제품 Iced 한글 IME 실증

2026-10-02. [작업 계약](../work-items/S00-iced-current-ime.md). 제품release SHA256 `7e5fe81444a31806189e3a84d51d1ccf809f48bd3c83df1396c9d2fbabacd469`,9,873,856B를실행직전hash로대조했다. root의winit Start없는Update patch가포함된제품 Iced/tiny-skia이며 egui spike 결과를제품에적용하지않았다.

**제품Linux X11/ownedXvfb의inline한글조합·정확commit·BackSpace취소·Latin·selection대체는실제시각검토와인수대조를통과했다.** OS별IME/접근성·S00최종스택채택은여전히별도게이트다.

## 실행과 실제 결과

```sh
python3 scripts/ime_probe.py target/release/dlpad /tmp/iced-current-ime-inspect --inspect
python3 scripts/ime_probe.py target/release/dlpad /tmp/iced-current-ime-full
```

기존WindowsMalgun.ttf를읽기전용fixture로사용하고ownedXvfb/privateD-Bus/Fcitx5Hangul/XIM/C.UTF-8/100%/960×640/tempXDG를사용했다. 사용자IME설정·Documents·실제영상·계정은사용하지않았고visible fallback은없다. [inspect화면](iced-current-ime/inspect/initial.png)으로현재입력좌표를검토한뒤실제물리ASCII키로Hangul을조합했다. Unicode clipboard paste가아니며clipboard는commit문자열확인에만읽었다.

[원JSON](iced-current-ime/full/raw-result.json)과[검토판정JSON](iced-current-ime/full/result.json)을분리했다.

- [preedit화면](iced-current-ime/full/preedit.png): gksrmf후input안`한글`의마지막`글`이읽히고preedit밑줄이있다. 아래committed mirror는`한`만보인다. 외부FcitxInputWindow는root(160,780,27,31)로app원점(160,130)/높이640바깥이며inline성공증거로사용하지않았다.
- [commit화면](iced-current-ime/full/committed.png): Space뒤clipboard exact`한글 `,mirror`한글`. commit_verified=true.
- BackSpace2회조합취소: 이전commit`한글 하`를그대로보존,cancel_verified=true. Escape는pending`하`를commit하므로취소로주장하지않는다. 독립GTK XIMcontrol도같은`한글 하`를확인했다.
- Latin전환: exact`한글 하abc`,latin_transition_verified=true.
- [selection화면](iced-current-ime/full/selection.png): Ctrl+A→inactiveIME xy→Ctrl+A/C로exact`xy`,selection_replace_verified=true.

원collector exit1은legacy fixedpixel proxy의dark>64조건때문이다. 실제crop[48,308,60,324]의dark45/light126은blackglyph/whitebackground를읽을수있는데도oldblackselection조건에미달했다. 수치와원exit/imefalse를raw-result.json에보존했다. 시각검토후result.json에reviewer/underline/committedmirror와preedit_verified/ime_verified=true를기록하고commit·cancel·Latin·selection·GTK일치/ownedteardown을Pythonassertion으로대조해통과했다. 원exit1을exit0로고쳐쓰지않는다. root의PNG/JSON독립검토는통합담당범위다.

## probe 변경과 안전성

scripts/ime_probe.py에inspect/validate-only·binaryhash/bytes·selection대체·ownedteardown기록을추가했다. privateXDG를짧은/tmp TemporaryDirectory로만들고Fcitx준비시간3초를적용했다. XKeysymToKeycode는실제KeyCode ABI인c_ubyte로정정했다. PNG입력4MiB상한을추가하고기존pixelcontrast수치는visualacceptance가아닌보조diagnostic으로명시했다. preedit_requires_screenshot_review를유지하며자동glyph뜻판정을주장하지않는다.

AST/CLI4건(inspect/legacy허용,desktop거절),PNG크기상한거절통과. 실제소유app/IME/privatebus회수와temp삭제확인. 원/tmp산출물을새프로젝트전용evidence폴더로복사했으며기존증거를덮어쓰지않았다. 제품코드/Cargo/vendor/공통문서는수정하지않았고빌드하지않았다.

## 실제 reader 다음 게이트의 읽기 전용 조사

현재시스템에gdbus/dbus-daemon,at-spi2-core2.44.0-3/libatspi2.0-0,python3-gi3.42.1과python dbus는있다. org.a11y.Bus.service의Exec는/usr/libexec/at-spi-bus-launcher다. Orca/speech-dispatcher/espeak/espeak-ng/pyatspi는없다(command-v/dpkg-query/importlib조사). 설치나제품변경은수행하지않았다.

기존egui evidence가ownedprivatebus의org.a11y.Bus.GetAddress와adapterflag/AT-SPIregistry/한국어role-name-text를실제로공개했으므로격리bus의기술적가능성은이미관측됐다. 이것은읽기프로그램이아니다. 제품Iced/iced_winit pinnedmanifest에는accesskit/accessibilityfeature일치가없고alternativeeframe에만accesskit이있다. 검색부재를모든OS에서절대불가능하다는증명으로쓰지않지만실제제품의tree공개부터검증해야한다.

안전한후속계약은ownedXvfb/privateDBUS_SESSION_BUS_ADDRESS/임시XDG·AT_SPI_BUS_ADDRESS를고정하고제품의role/name/state/focus/값tree를읽은뒤실제reader를그bus에만연결하는순서다. userbus/기존reader/전역IME·audio설정에접속하거나visible desktop으로전환하지않는다. 실제reader검증에는현재없는reader도구준비와출력/로그상한·소유프로세스회수가필요하다. 사용자스피커를방해하지않는private/no-audio출력경로를먼저확정해야한다. 이는후속제안이며이번에설치/reader실행/제품bridge구현을완료한것은아니다.

미검증: Windows TSF/macOS/Wayland와다른IME·200%·reader실제읽기·키보드전체탐색·System첫프레임/전환·최종UI스택. Linux제품IME라는현재검증범위를확대하지않는다.

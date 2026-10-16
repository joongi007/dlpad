# S00 명시 XIM callback reference 원인 분리

2026-10-02. [작업 계약](../work-items/S00-xim-reference.md). 신규 C reference와 Python harness만 구현했다. 기존 제품 UI/registry/vendor/Cargo/probe는 수정하지 않았다. **목표 통과: 같은 Fcitx가 명시 style1026 reference에 실제 한글 preedit draw를 제공한다.** 이는 제품inline가시성/S00채택 통과가 아니다.

## 최종 실제 관측

기존 cc11.4.0/Xlib runtime2:1.7.5-1ubuntu0.3, owned Xvfb/private D-Bus/Fcitx5 Hangul/XIM/C.UTF-8/UseOnTheSpot=True. 실제 keyboard gksrmf→Space, gk→BackSpace2회, IMEoff→abc를 보냈다. reference는 server advertisement에1026이 있음을 확인하고 XCreateIC(XIMPreeditCallbacks|XIMStatusNothing)와 start/draw/done/caret callback을 명시해 성공했다.

동일 최종 C source SHA256 `f9c66f1f9f69a09d4f0b06af3ee1dfd377eea1326856895a7defb347ab948931`, executable26,296B/SHA256 `db3b2146c2f4a9cd6f18bc7c2c05525bccafc525bd4af34fb7400c8784b654fa`로 필터 변인만 비교했다.

| 필터 | 시간/결과 | 실제 기록 |
|---|---|---|
| XFilterEvent(event,None) | 9.795초/exit0 | [JSON](xim-reference/final-none-filter/result.json), [raw trace](xim-reference/final-none-filter/reference.jsonl) |
| XFilterEvent(event,event.xany.window) | 9.792초/exit0 | [JSON](xim-reference/event-window-filter/result.json), [raw trace](xim-reference/event-window-filter/reference.jsonl) |

양쪽 draw는 `ㅎ→하→한`, 다음 `ㄱ→그→글`을 전달했다. commit 정확`한글 `, BackSpace 취소 후 동일`한글 `, Latin최종`한글 abc`가 모두true. done callback은 관측됐고 start/caret는 관측되지 않았다. nonempty draw9개 모두encoding_is_wchar=0/length1/chg_first0이다. readback XGetICValues(XNInputStyle)는unsupported error`inputStyle`을 반환해 actualstyle 관측통과로 표현하지 않는다. explicit1026 CreateIC 성공과 callback 수신이 목표의 근거다.

`callbacks_verified`와`server_preedit_delivery_verified`는 한글draw/정확commit/취소/Latin/오류없음의 이 작업 목표다. `all_callback_kinds_observed=false`, style_readback_supported=false를 별도 유지한다. 초기harness가Start필수로둔시험은exit1이었으나 실제서버시나리오에Start가없어도Draw는온다. 주 담당과 인수목표를 조정한 뒤 최종2시험은제공여부에한정exit0다. callback protocol 모든종류를보장하거나실패를숨긴것이아니다.

raw trace 파일SHA256은 event-window `3fa81269b393e949ef0c5578f38051fbb8c61c41c2c651816c92ac567d6b8e54`, None `3718a31ff7d2ab8f860cb0d322ebdf9086b4af75a78fab3709922da6497c1be3`이다. dynamic raw6,046B/IMElog10,679B, 소유 app/IME/privatebus handles회수·Xvfb종료·tempXDG삭제 확인. 실제 영상/계정/사용자설정/visible desktop은사용하지않았다.

## winit 경계의 의미

[이전 egui 시험](egui-ime-validation.md)은 raw egui에Commit한/글만오고Enabled/Preedit가없었다. reference의직접draw수신은IME가조합텍스트를제공못한다는가설을배제한다. 실제encoding0이므로 pinned winit0.30.13 context.rs의widechar무시branch는이번실험원인을지지하지않는다.

로컬 winit event_processor.rs는ImeEvent::Start에서만is_composing=true를설정하고Update는is_composing조건에서만WindowEvent::Ime로보낸다. reference에서Start없이Draw가오는관측은이guard가Update를버리는가설과맞는다. 다음은root소유winit실험에서실제callback/Update/state를관측하고 Start없는 nonempty Update 처리의preedit/commit/clear/취소 회귀를검증해야한다. 아직제품수정이나특정실행중winit내부state를직접증명한것은아니다.

초기reference는XFilterEvent에고정client window를넘겨IM transport window메시지를잘못거르고CtrlSpace활성화가실패했다. None으로고치면통과했고최종dynamiceventwindow도통과했다. winit는고정clientwindow가아닌event.window를사용하므로이reference실수를winit결함으로확대하지않는다.

## 도구 공급·비용

관리자/system 설치없음. root승인한새 `.local-tools/xim-reference`에공식Ubuntu패키지를다운로드/추출했다. apt-cache Package metadata의SHA256/Size를실제deb hash/size와대조했다. 패키지동봉copyright/licenses와headers를보존했다. Source는libx11/xorgproto이며단일라이선스재배포를주장하지않는다.

| package | bytes / SHA256 | 공식경로 |
|---|---|---|
| libx11-dev2:1.7.5-1ubuntu0.3 amd64 |744438 / `0e3c01edea67b9202c8f0857a8976c43b32ce8e65158e09c86acccb49bad2109`|[Ubuntu libx11-dev](https://archive.ubuntu.com/ubuntu/pool/main/libx/libx11/libx11-dev_1.7.5-1ubuntu0.3_amd64.deb)|
| x11proto-dev2021.5-1 all |604434 / `7cb2471a23ea50e197cec019d0596840b6ddd714e8deb75e427472e820e83548`|[Ubuntu x11proto-dev](https://archive.ubuntu.com/ubuntu/pool/main/x/xorgproto/x11proto-dev_2021.5-1_all.deb)|

총 다운로드1,348,872B/추출 apparent4,198,174B/reference26,296B이며 기존compiler/runtime을사용했다. 개발 시험 비용이고제품bundle의존성이아니다. 새Release서명을독립획득/검증했다고주장하지않고기존APT index metadata/hash대조와apt-get download를사용했다.

## 실행 검사

```sh
cc -std=c11 -D_POSIX_C_SOURCE=200809L -Wall -Wextra -Werror -Wno-cast-function-type -I.local-tools/xim-reference/root/usr/include scripts/xim_reference_probe.c /usr/lib/x86_64-linux-gnu/libX11.so.6 -o .local-tools/xim-reference/xim-reference
DLPAD_TEST_FONT=/mnt/c/Windows/Fonts/malgun.ttf python3 scripts/xim_reference_probe.py .local-tools/xim-reference/xim-reference <fresh-evidence-directory>
DLPAD_TEST_FONT=/mnt/c/Windows/Fonts/malgun.ttf python3 scripts/xim_reference_probe.py .local-tools/xim-reference/xim-reference <fresh-evidence-directory> --event-window-filter
```

start callback은XlibgenericXIMProc에서int반환실제ABI를cast하는표준callback형태라-Wno-cast-function-type만사용하며다른Wall/Wextra는Werror다. C는128events/128codepoints/commit512bytes/30초상한, Python은trace192KiB/128records를거절한다. AST/CLI3·Cinvalidarg(exit2)·trace크기/record초과거절검증통과. 최초strictreadback실패2회와reference필터실패를전용디렉터리에보존했다. 사용자window에는접속하지않고 fallback하지않았다. source/binary변경을각시험hash로분리했다.

미검증: 실제제품inline preedit/renderer/IME완성, Wayland/Windows/macOS, 실제reader, caret/start가필요한다른IME시나리오, 악의적인XIM데이터의완전한안전보장. 이reference는제품UI나범용XIMlibrary가아니다.

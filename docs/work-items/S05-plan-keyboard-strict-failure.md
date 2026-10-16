# S05: Strict plan keyboard 최초 실패 읽기 진단

- 상태: completed (읽기 진단; 원인 미확정)
- 분류: diagnosis
- 우선순위: P0
- 요구사항: R07, R01
- 담당/수정 파일: soak_recovery_audit — 이 문서만
- 의존성: root strictfirst 실제 rawresult/PNG 및 frozen keyboard harness

## 범위와 인수 조건

JSONDecodeError실패의plan text/native focus/expanded/관측stage를원시자료로구분하고최소instrumentation을제안한다. source/scriptfreeze를유지하며빈text를무조건None으로바꿔PASS하지않는다. GUI/Cargo/clipboard/desktop실행없음.

## 원시 관찰

[strictfirst result](../validation/evidence/p0-plan-keyboard-strict-first/result.json)은keyboard_verified=false,Return GrabFocus exit0/true 및FOCUSED bit12가있는state1124080384를기록한다. 직전초기/pointeropen/close/action0/reset0은정합한expanded state이며action0 activation_verified=true다. `key-Return` observation은없다. observe는dict를append하기전plan()을평가하므로그안JSONDecodeError가발생하면당시state/label도기록되지않는다.

[failure PNG](../validation/evidence/p0-plan-keyboard-strict-first/failure.png)은header가열려있고실행계획JSON텍스트가화면에실제로보인다. 따라서키가확실히실패했다거나productplan이실제로빈문자열이었다고현재자료로단정할수없다. Conversely화면에열려있다는것만으로nativeGetText/focus/close까지키보드인수가성공한것도아니다.

any_node는Name일치후Text.GetText exit0이면role/bounds/state검증없이첫node를선택한다. plan111은reply tuple첫값을바로json.loads한다. Error가column1이므로빈text또는JSON이아닌leadingtext가가능하지만실제rawGetText는result에남지않아정확히empty라고확정할수없다. stale/defunctnode·동일Name다른role·accessibilityupdate직후빈read·비가시clip과nativeText의차이를분리할정보가없다. 기존pointer/action에서성공했으므로고정적인전체planjson생성오류보다는관측stage차이를먼저계측할필요가있다.

## 최소 후속 instrumentation 제안

Root수정승인후script소유범위에서plan/observe만보강한다. observe는label/stage/headerstate/focused/expanded를먼저append하고그후plantext를읽어실패직전자료가유실되지않게한다. 마지막64관측·각reply64KiB등상한을두고sourceURL/query/argv비밀값을신규raw로확장하지않는다. 현재URL은합성token이지만fixture구분을계속유지한다.

선택된plan node의destination/path/role/state/extents/Text.GetText exit/rawreply및decodedtextlength/hash/parse-status를보존한다. 동일Name후보는최대8개로bounded기록하고editable/read-onlytextrole및유효Textinterface대상인지분리한다. Preview는256KiB일수있으므로전체원문을새debug로그에무제한복사하지않고이시험의고정합성plan만기존상한내에보존한다.

키open 직후headerEXPANDED=true와별도로expectedvalidplanjson을boundedpoll한다. 잠깐empty읽기라면candidate는`pending-empty`이고collapsedNone으로변환하지않는다. Polldeadline끝까지empty/잘못된JSON이면명확한FAIL과raw상태를남긴다. 닫기에서는headerEXPANDED=false+plan부재semantic을기준으로확인하며staleempty를자동닫힘성공으로취급하지않는다. Emptyexpandedtext를FAIL없이None처리해키실패를PASS하는수정은금지다.

Root의nativefresh재실행에서rawnode선택/role과readreply를대조한후수집기버그인지제품접근성타이밍문제인지판단한다. 제품accesskit/read-only/focus정책변경이필요하면rootRustowner에게반환하고harness로성공상태를제조하지않는다.

## 실제 실행과 잔여

### 최소 계측 구현 후속

Root승인후담당script만보강했다. Observe는stage/headerstate를먼저append한다. Plan조회는기존최대256nodeBFS로exactName후보최대8개의actualrole/state/bounds/GetTextraw를수집하며readhistory최대64를넘으면FAIL이다. Decoded text의len/SHA와valid-json/pending-empty/invalid/unavailable을구분한다. Rawreply는기존call의stdout16KiB상한과추가64KiB검사를유지하며합성preview의URL전체생략정책은변경하지않았다.

Observe는3초boundedpoll에서actualexpanded+유효nonemptyargvJSON또는actualcollapsed+previewnode부재의정합성을요구한다. Pendingempty를None으로닫힘PASS하지않는다. Timeout은FAIL이며마지막headerstate/plan_error/rawcandidate가보존된다. 기존Return/spaceFOCUSED/open/closeassert는유지했다. 기존D-Buscall별5초deadline이있으므로poll3초는call실행시간을단축하는APIdeadline이아니며외부nativeharness전체상한은root가유지한다.

Python `compile(Path(...).read_text(), ..., 'exec')` actualexit0/PASS. Sourcefreeze SHA256 `feb679df8d55e5c6c54557dbddfc3773a9e98fe9945c79e1272c6ab67eba98cb`. GUI/native/Cargo는실행하지않았고root의freshretry결과대기다. 첫JSONDecodeError실패는그대로보존하며이번계측성공을실제키보드PASS로표현하지않는다.

### Root 실제 second 결과 대조

[second result](../validation/evidence/p0-plan-keyboard-strict-second/result.json)는동일CPU SHA `ca8e969f028fdd50f5c7f9ae19c3c71928d7740daa5c93189ec100ee9f5fc77e` 및위계측harness로passed=true/keyboard_verified=true/close_exit0를기록했다. Return/space각각GrabFocus true,FOCUSED/EXPANDED open-close와13개bounded planread absent/valid-json을원시JSON에서독립확인했다. 이번second에는pending-empty/invalid가없어최초GetText빈응답/중복node/staleupdate중어느것이원인이었는지는계속미확정이다.

이결과는planheaderGrabFocus후Return/spaceactivation의실제scopedPASS다. Tab순서·모든5help·screenreader발화·Windows인수는아니다. First failure와PNG는그대로보존했고productRust가변경되어원인을해결했다고주장하지않는다. 초기pending문장은그당시기록이며현재native결과는이후속절을기준으로읽는다.

원시JSON/PNG와frozenharness를읽었다. Source/Rust/scripts/Cargo/GUI입력수정·실행없음. Return실제화면open은관찰됐지만keyboard전체인수는FAIL/미완료이며space/close는도달하지않았다. 원인은미확정이고추가계측/재실행은root승인뒤만수행한다.

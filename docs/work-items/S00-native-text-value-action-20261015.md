# S00: native TextInput Value action

- 상태: in-progress
- 분류: bugfix
- 우선순위: P0
- 요구사항: R08 native 접근성·편집·입력보존
- 담당/수정 범위: windows_auto 새 app/accesskit_value_tests.rs(시험전용), Root vendor TextEdit builder.rs 및main두cfgtest등록; B/accesskit-value-action-tests-v1 및native검색futurev3 helper
- 선행 작업: Root native작업terminal/overlay ACK. 기존whole-v2/source/current3/Evidence 보존.

## 사용자 결과

actual Windows UIA ValuePattern이emit하는string SetValue를실제현재TextInput노드에연결한다. 공통입력은unfocused에서도한글/정확한문자상한/단일행규칙을적용하되다른입력focus/caret를뺏지않는다. 비활성/interactivefalse/readonly는수정을거절하며비밀번호내용을접근성/paint로노출하지않는다.

## 계약과 범위

vendor/accesskit_windows Action::SetValue(Value string)와현재eguiTextEdit 소비누락을Root가진단했다. schema/DB/deps/Cargo/lock/clipboard/globalinput/사용자desktop는변경하지않는다. 이시험파일은실제Context·현재AccessKit TextInputNodeID target ActionRequest를주입하며helper값만비교하지않는다. publicUIcommontext_input_with_id와rawreadonly/interactivefalse/password을검사한다. IME외부변경후늦은Commit과undo/typing/paste/directedselection을보존한다.

## 인수 조건

SetValue한글반영/다른입력포커스·caret불변/samevalueChangedfalse, CR/LF단일행정리(기존Paste replaceCR/LFspace정책)/char_limit은UTF8byte절단아닌Unicode문자수, disabled/readonly/interactivefalse거절을실제렌더응답/노드값으로확인한다. undo와실제keyboard/paste 선택동작,활성IME대체후latecommit의보존조건을정확히구분한다. 비밀번호node/galley에원문을남기지않고실패메시지에도secret을출력하지않는다. 합성Context는actualOS/UIA/물리IME/reader인수를대체하지않는다.

## 검증과 handoff

시험 후보 6개 준비 및 Root ACK 후 새 test 파일 한 개 overlay 완료. rustfmt exit0, Cargo/native 0이며 실제 RED/GREEN은 아직 미검증. Root가원vendorbaseline에서실제RED→vendorGREEN후같은필터/기존입력회귀/전체strict·freshwhole-v3 및privateValueSetter helper를검증한다. Native검색후보는원v2helper/pins보존후별도v3exactproof를사용한다. 준비를플랫폼검증완료로표현하지않는다. 새시험I/O0/추가제품deps0,Rootvendor비용은Root보고범위다.

## v2 시험틀 수정 및 추가 회귀

최초 RED 시도는 direct egui import 누락(E0433)으로 compile 실패했으며 semantic RED가 아니다. 원 v1 후보/증거는 보존했다. v2는 eframe::egui import를 추가하고 기존 6개 조건을 유지한 9개 실제 Context 시험을 제공한다. mutable capability/readonly·disabled capability, NumericValue·비입력 target·2MiB 초과 거부 및 focused keyboard와 별도 SetValue target의 1회 dispatch를 추가했다. Root ACK 후 새 시험 파일만 overlay·동결했으며 실제 RED/GREEN·OS ValuePattern 검증은 Root 담당이다. 파일 rustfmt exit0, native/Cargo 실행 0.


Root 실제 RED: managed-tool-isolated-native-value-red-v2-20261015. import 수정 후 compile 성공, 실제 Context 9개 중 2 통과/7 실패, test exit101. 원 로그와 후보·Live 불변을 독립 확인했다. 최초 red-v1은 compile 실패여서 유효 재현으로 취급하지 않는다. Root vendor 수정은 unfocused 대상 요청만 처리하고 다른 필드의 keyboard를 복제하지 않는다. 문자열은 기존2MiB 상한과 문자수 제한, 단일행 newline 규칙을 적용하며 같은 값은 changed를 발생시키지 않는다. mutable enabled 입력만 SetValue를 광고하고 readonly/disabled를 거절한다. 새로운 dependency/schema/lock은 없다. GREEN 실제 실행 중이며 성공 여부는 아직 확정하지 않았다.

Root 선택 GREEN 실제 검수: managed-tool-isolated-native-value-green-v1-20261015에서 입력 9, 엔진 전송 4, 진행률 Context 1(64조합), 도구 compact Context 1(6장면)이 모두 통과했다. 각 명령 exit0/소유 Job0·private desktop 종료·input desktop 불변 및 Live/후보 source 불변을 Root가 원로그와 receipt로 대조했다. 전체 Windows 네 명령은 새 whole-v3에서 실행 중이며 실제 GUI/새 설치본은 아직 미완료다.

whole-v3는 Root가 등록한 main 두 파일의 module 순서 rustfmt 차이로 첫 fmt exit1, clippy/test/release 미실행이다. 이 실패는 보존했다. main 두 파일을 exact toolchain rustfmt로 정리한 뒤 새 whole-v4-20261015로 재실행한다. 이전 whole-v3 준비 helper를 새 증거로 승격하지 않고 별도 future proof helper를 사용한다.

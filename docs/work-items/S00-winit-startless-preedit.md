# S00: 시작 콜백 없는 XIM 조합 표시

- 상태: in-progress
- 분류: bug
- 우선순위: P0
- 요구사항: R06, R08, R11, S00
- 담당/수정 범위: 주 담당; vendor/winit, root Cargo.toml/Cargo.lock, 공유 검증/ADR. diagnostic_cache는 전용IMEharness와증거만.
- 선행 작업: Xlib reference에서한글Draw/Done/commit관측, Start미관측 및 wchar0확인

## 사용자 결과

Start없이Draw를전달하는XIM서버에서도한글조합중글자를native입력안에표시한다.

## 계약과 범위

기존정확한winit0.30.13 소스를프로젝트vendor에보존한다(원LICENSE/README/metadata유지). X11 Update는nonempty면composition을시작하고emptyinactive는무시한다. wchar/XFilterEvent경로는실제대조가설이배제돼바꾸지않는다. workspace에서vendor를제외하고제품/비교native모두동일patch를사용한다. 추가crate/daemon/web UI없음. source약3MiB이며runtime추가상태는기존bool설정뿐; 실제binary/RSS는새hash로측정한다.

## 인수 조건

격리Xvfb에서defaultfallbackoff의한글preedit/commit/취소/Latin/선택/AT-SPI를다시검증한다. 기존ASCII/IME비활성/emptyupdate와nativewindow/input게이트를유지한다. 공통fmt/clippy/tests/release및exactlock을확인하며OS미검증은완료로쓰지않는다.

## 검증과 handoff

수정및실증진행중. 원cratechecksum과단일sourcepatch근거를 [vendor기록](../validation/winit-vendor.md)에남긴다. 기존nativeWindows/GUIbinary증거는이patch이전snapshot이다.

## 현재 실증

최신 Linux 제품4공통검증 통과 및 egui/Iced 양쪽 private Xvfb inlinepreedit/commit/cancel/Latin/selection 화면을 확인했다. root가 Iced PNG와JSON을 독립검토했다. Windows GNU vendor전체입력278개SHA/4공통검증/release도통과했다. 실제 reader·3OS입력은 미검증이며 stack 채택은 Proposed로 유지한다. [제품 증거](../validation/iced-current-ime/full/result.json), [Windows 기록](../validation/windows-winit-validation.md). 원 수집 exit와시각검토판정을분리했다.

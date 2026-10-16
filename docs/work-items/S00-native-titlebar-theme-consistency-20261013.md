# S00: 앱 제목 표시줄과 화면 테마 일관성

- 상태: in-progress
- 분류: UX
- 우선순위: P2
- 요구사항: R06, R08
- 담당/수정 범위: advanced_units — /tmp 소스 감사/후보만; root — software_host.rs 채택·통합, 본 문서/검증
- 선행 작업: 현재source655 원검토와native작업종료, 기존 Win32_Graphics_Dwm feature

## 사용자 결과

지원되는 Windows에서 앱의 밝은·어두운 테마에 맞춰 제목 표시줄을 표시한다.

## 계약과 범위

격리 full-window 캡처의 고전적인 OS 제목 표시줄은 가상 desktop/DWM 환경 영향일 수 있다. 실제 현재software_host의supported per-window 테마설정 여부를 먼저 조사한다. 채택시 ownedHWND에만 documented DwmSetWindowAttribute를 테마변경시 호출하며 시스템테마/설정/전역후크/새의존성을 추가하지 않는다. unsupported HRESULT는OS기본표시로남기고제품오류로부풀리지않는다. UIthread 긴IO, 매frame 반복native호출을피한다. root가공유Cargo/host계약을조정한다.

## 인수 조건

현재host의실제동작/후보근거, once-per-change/실패시bounded정책과fmt/clippy/test/release, 실제private nativeproperty/HRESULT/목표HWND보존을확인한다. privateclassicchrome를physicalDWM시각인수로확대하지않는다.

## 검증과 handoff

현재는감사/후보단계다. full-window source655 PNG는앱아이콘/단일dlpad창제목을보이며 bodyDark와nativeclassicchrome가다르다. 이관측만으로physicalWindows색상버그를확정하지않는다.

root가후보를현재소스에통합했다. 이전v8의source2/기본binary3파일은 /tmp/dlpad-ui-v8-before-label-titlebar-20261013에원SHA/bytes와함께보존했다. 새Windows whole fmt/clippy/test/release는진행중이며아직통과보고가아니다. 공유Cargo/lock/schema변경없음.

채택범위는documented20/35/36이다. manual Light/Dark는현재ctx panel/textCOLORREF, System은COLOR_DEFAULT를사용한다. cache는실패/owner없음도같은request의반복native호출을막고, 바뀐theme/palette에서만새시도를한다. 실제SET반환값은일반paint호출에서기록하지않으므로원SET HRESULT 성공을주장하지않는다. 다음privategetter관측은GET HRESULT/실제속성값과지원여부만확인한다. classicprivatepixel/physicalDWM/HighContrast인수는미검증이다. [Microsoft API계약](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/ne-dwmapi-dwmwindowattribute).

### 최신 source 후속 실제 검증

Windows whole-v2 fmt/clippy/test/release 실제 성공: 1,284/0/30. 테마 cache 회귀6개 포함. 현재 CPU 9,836,544바이트/f15a830dfcba928bdfff4494cbc834fab8ccaa4876a056a1fe19610b315e1a6b. 새 의존성 없이 직전 기본 빌드보다512바이트 증가했다. 소유 HWND/PID/원 UI thread만 대상으로 변경시에20·35·36을 시도하고 실패도 cache하여 매 프레임 반복하지 않는다.

주 담당이 current-default-titlebar-theme-private-20261013 collector를 실제 실행하고 source/4raw log/artifact/owned lifecycle 독립 검토 및5PNG를 직접 보았다. Light→Dark→System 실제 선택과 native title/icon/geometry 유지, 자연exit0/Job0/입력desktop 유지. Get20은0/1/0 정확 일치. Get35/36은6회 E_INVALIDARG(-2147024809)로 지원 또는 조회 실패이며 raw0은 유효 색상이 아니다. setter HRESULT는 관측하지 않았다. private 캡처의 고전적인 제목 표시줄을 물리 DWM 색상 검증으로 대체하지 않는다. 전체 UI/UX·High Contrast·OS DPI·사용자 최종 인수는 미완료이다.

[실제 결과](../validation/evidence/current-default-titlebar-theme-private-20261013/result.json), [주 담당 검토](../validation/evidence/current-default-titlebar-theme-private-20261013/root-review.json). review.py 첫 상대경로 호출은 plain absolute path guard로 실패했으며 /tmp 원 실패를 유지했다. 절대경로로 다시 실행한 receipt 검토가 통과했다.
